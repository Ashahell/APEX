//! Webhook Filter API Endpoints
//! 
//! v1.10.0: Webhook Event Filtering API

use tokio::sync::RwLock;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use chrono::Utc;

use crate::api::api_error::ApiError;
use crate::api::AppState;
use crate::webhook_filter::{
    WebhookFilter, FilteredEvent,
    CreateWebhookFilterRequest, UpdateWebhookFilterRequest,
    TestFilterRequest, TestFilterResponse, FilterAction,
};

/// Shared webhook filter state
pub struct WebhookFilterState {
    pub filters: RwLock<Vec<WebhookFilter>>,
    pub events: RwLock<Vec<FilteredEvent>>,
}

impl WebhookFilterState {
    pub fn new() -> Self {
        Self {
            filters: RwLock::new(Vec::new()),
            events: RwLock::new(Vec::new()),
        }
    }

    pub async fn list_filters(&self, webhook_id: Option<&str>) -> Vec<WebhookFilter> {
        let filters = self.filters.read().await;
        match webhook_id {
            Some(id) => filters.iter().filter(|f| f.webhook_id == id).cloned().collect(),
            None => filters.clone(),
        }
    }

    pub async fn get_filter(&self, id: &str) -> Option<WebhookFilter> {
        self.filters.read().await.iter()
            .find(|f| f.id == id)
            .cloned()
    }

    pub async fn create_filter(&self, mut filter: WebhookFilter) -> WebhookFilter {
        filter.id = ulid::Ulid::new().to_string();
        filter.created_at = Utc::now();
        filter.updated_at = Utc::now();
        self.filters.write().await.push(filter.clone());
        filter
    }

    pub async fn update_filter(&self, id: &str, update: UpdateWebhookFilterRequest) -> Option<WebhookFilter> {
        let mut filters = self.filters.write().await;
        if let Some(filter) = filters.iter_mut().find(|f| f.id == id) {
            if let Some(name) = update.name {
                filter.name = name;
            }
            if let Some(events) = update.event_types {
                filter.event_types = events;
            }
            if let Some(conditions) = update.conditions {
                filter.conditions = conditions;
            }
            if let Some(logic) = update.condition_logic {
                filter.condition_logic = logic;
            }
            if let Some(action) = update.action {
                filter.action = action;
            }
            if let Some(enabled) = update.enabled {
                filter.enabled = enabled;
            }
            if let Some(priority) = update.priority {
                filter.priority = priority;
            }
            filter.updated_at = Utc::now();
            return Some(filter.clone());
        }
        None
    }

    pub async fn delete_filter(&self, id: &str) -> bool {
        let mut filters = self.filters.write().await;
        let pos = filters.iter().position(|f| f.id == id);
        if let Some(pos) = pos {
            filters.remove(pos);
            true
        } else {
            false
        }
    }

    pub async fn test_filter(&self, id: &str, req: TestFilterRequest) -> Option<TestFilterResponse> {
        let filter = self.get_filter(id).await?;
        
        if !filter.event_types.is_empty() && !filter.event_types.iter().any(|t| t == &req.event_type) {
            return Some(TestFilterResponse {
                matched: false,
                action: filter.action,
                message: "Event type does not match filter".to_string(),
            });
        }

        let matched = filter.matches(&req.event_type, &req.payload);
        Some(TestFilterResponse {
            matched,
            action: filter.action,
            message: if matched {
                format!("Event matches filter '{}'", filter.name)
            } else {
                "Event does not match filter conditions".to_string()
            }.to_string(),
        })
    }

    pub async fn get_events(&self, filter_id: Option<&str>, limit: usize) -> Vec<FilteredEvent> {
        let events = self.events.read().await;
        let filtered: Vec<_> = match filter_id {
            Some(id) => events.iter().filter(|e| e.filter_id == id).collect(),
            None => events.iter().collect(),
        };
        filtered.into_iter().rev().take(limit).cloned().collect()
    }
}

impl Default for WebhookFilterState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the webhook filter router
pub fn create_webhook_filter_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/webhook-filters", get(list_filters))
        .route("/api/v1/webhook-filters/:id", get(get_filter))
        .route("/api/v1/webhook-filters", post(create_filter))
        .route("/api/v1/webhook-filters/:id", put(update_filter))
        .route("/api/v1/webhook-filters/:id", delete(delete_filter))
        .route("/api/v1/webhook-filters/:id/test", post(test_filter))
        .route("/api/v1/webhook-filter-events", get(list_events))
}

/// GET /api/v1/webhook-filters - List all filters
#[derive(Debug, serde::Deserialize)]
pub struct ListFiltersQuery {
    pub webhook_id: Option<String>,
}

async fn list_filters(
    State(state): State<AppState>,
    Query(query): Query<ListFiltersQuery>,
) -> Result<Json<Vec<WebhookFilter>>, (axum::http::StatusCode, String)> {
    let filters = state.webhook_filter_state.list_filters(query.webhook_id.as_deref()).await;
    Ok(Json(filters))
}

/// GET /api/v1/webhook-filters/:id - Get a specific filter
async fn get_filter(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WebhookFilter>, (axum::http::StatusCode, String)> {
    state.webhook_filter_state.get_filter(&id).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Filter {} not found", id)))
}

/// POST /api/v1/webhook-filters - Create a new filter
async fn create_filter(
    State(state): State<AppState>,
    Json(req): Json<CreateWebhookFilterRequest>,
) -> Result<Json<WebhookFilter>, (axum::http::StatusCode, String)> {
    let mut filter = WebhookFilter::new(req.name, req.webhook_id);
    if let Some(events) = req.event_types {
        filter.event_types = events;
    }
    if let Some(conditions) = req.conditions {
        filter.conditions = conditions;
    }
    if let Some(logic) = req.condition_logic {
        filter.condition_logic = logic;
    }
    if let Some(action) = req.action {
        filter.action = action;
    }
    if let Some(priority) = req.priority {
        filter.priority = priority;
    }

    let created = state.webhook_filter_state.create_filter(filter).await;
    Ok(Json(created))
}

/// PUT /api/v1/webhook-filters/:id - Update a filter
async fn update_filter(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateWebhookFilterRequest>,
) -> Result<Json<WebhookFilter>, (axum::http::StatusCode, String)> {
    state.webhook_filter_state.update_filter(&id, req).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Filter {} not found", id)))
}

/// DELETE /api/v1/webhook-filters/:id - Delete a filter
async fn delete_filter(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    if state.webhook_filter_state.delete_filter(&id).await {
        Ok(Json(serde_json::json!({ "success": true })))
    } else {
        Err(ApiError::not_found(format!("Filter {} not found", id)))
    }
}

/// POST /api/v1/webhook-filters/:id/test - Test a filter
async fn test_filter(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<TestFilterRequest>,
) -> Result<Json<TestFilterResponse>, (axum::http::StatusCode, String)> {
    state.webhook_filter_state.test_filter(&id, req).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Filter {} not found", id)))
}

/// GET /api/v1/webhook-filter-events - List filtered events
#[derive(Debug, serde::Deserialize)]
pub struct ListEventsQuery {
    pub filter_id: Option<String>,
    pub limit: Option<usize>,
}

async fn list_events(
    State(state): State<AppState>,
    Query(query): Query<ListEventsQuery>,
) -> Result<Json<Vec<FilteredEvent>>, (axum::http::StatusCode, String)> {
    let limit = query.limit.unwrap_or(100).min(1000);
    let events = state.webhook_filter_state.get_events(query.filter_id.as_deref(), limit).await;
    Ok(Json(events))
}
