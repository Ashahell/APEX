//! API endpoints for Background Process Monitoring
//! 
//! This module integrates the monitoring module with the API router.

use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::Deserialize;

use crate::api::api_error::ApiError;
use crate::api::AppState;
use crate::monitoring::models::{
    MonitorEvent, NotifyMode, NotifyOn, WatchPattern, WatchScope, MonitorEventRecord,
};
use crate::monitoring::watcher::{WatchPatternUpdate, SharedWatcherRegistry};
use crate::monitoring::hooks::{SharedHookEmitter};
use crate::monitoring::notifier::SharedNotificationDispatcher;

/// Create the monitoring router
pub fn create_monitoring_router(state: crate::api::MonitoringState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/monitor/watchers", get(list_watchers))
        .route("/api/v1/monitor/watchers", post(create_watcher))
        .route("/api/v1/monitor/watchers/:id", get(get_watcher))
        .route("/api/v1/monitor/watchers/:id", put(update_watcher))
        .route("/api/v1/monitor/watchers/:id", delete(delete_watcher))
        .route("/api/v1/monitor/events", get(get_events))
        .route("/api/v1/monitor/events/task/:task_id", get(get_task_events))
        .route("/api/v1/monitor/stats", get(get_stats))
        .route("/api/v1/monitor/emit", post(emit_event))
        .with_state(state)
}

/// List all watch patterns
async fn list_watchers(
    State(state): State<crate::api::MonitoringState>,
) -> Result<Json<Vec<WatchPattern>>, (axum::http::StatusCode, String)> {
    let watchers = state.watcher_registry.list().await;
    Ok(Json(watchers))
}

/// Create a new watch pattern
async fn create_watcher(
    State(state): State<crate::api::MonitoringState>,
    Json(payload): Json<CreateWatcherRequest>,
) -> Result<Json<WatchPattern>, (axum::http::StatusCode, String)> {
    let pattern = WatchPattern::new(
        ulid::Ulid::new().to_string(),
        payload.name,
        payload.pattern,
        payload.watch_scope.unwrap_or_default(),
        payload.notify_on.unwrap_or_default(),
        payload.notification_mode.unwrap_or_default(),
    )
    .map_err(|e| ApiError::bad_request(e.to_string()))?;

    state
        .watcher_registry
        .add(pattern.clone())
        .await
        .map_err(|e: crate::monitoring::MonitoringError| ApiError::internal(e.to_string()))?;

    Ok(Json(pattern))
}

/// Get a specific watch pattern
async fn get_watcher(
    State(state): State<crate::api::MonitoringState>,
    Path(id): Path<String>,
) -> Result<Json<WatchPattern>, (axum::http::StatusCode, String)> {
    state
        .watcher_registry
        .get(&id)
        .await
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Watcher {} not found", id)))
}

/// Update a watch pattern
async fn update_watcher(
    State(state): State<crate::api::MonitoringState>,
    Path(id): Path<String>,
    Json(updates): Json<WatchPatternUpdate>,
) -> Result<Json<WatchPattern>, (axum::http::StatusCode, String)> {
    state
        .watcher_registry
        .update(&id, updates)
        .await
        .map_err(|e: crate::monitoring::MonitoringError| ApiError::internal(e.to_string()))?;

    state
        .watcher_registry
        .get(&id)
        .await
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Watcher {} not found", id)))
}

/// Delete a watch pattern
async fn delete_watcher(
    State(state): State<crate::api::MonitoringState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    state.watcher_registry.remove(&id).await;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// Get recent monitoring events
async fn get_events(
    State(state): State<crate::api::MonitoringState>,
    Query(params): Query<EventsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (axum::http::StatusCode, String)> {
    let limit = params.limit.unwrap_or(100) as usize;
    let events: Vec<MonitorEventRecord> = state.hook_emitter.recent_events(limit).await;
    
    Ok(Json(events.into_iter().map(|e| {
        serde_json::json!({
            "id": e.id,
            "type": e.event_type,
            "task_id": e.task_id,
            "session_id": e.session_id,
            "payload": e.payload,
            "created_at": e.created_at,
        })
    }).collect()))
}

/// Get events for a specific task
async fn get_task_events(
    State(state): State<crate::api::MonitoringState>,
    Path(task_id): Path<String>,
    Query(params): Query<EventsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (axum::http::StatusCode, String)> {
    let limit = params.limit.unwrap_or(100) as usize;
    let events: Vec<MonitorEventRecord> = state.hook_emitter.events_for_task(&task_id, limit).await;
    
    Ok(Json(events.into_iter().map(|e| {
        serde_json::json!({
            "id": e.id,
            "type": e.event_type,
            "task_id": e.task_id,
            "payload": e.payload,
            "created_at": e.created_at,
        })
    }).collect()))
}

/// Get monitoring statistics
async fn get_stats(
    State(state): State<crate::api::MonitoringState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let stats = state.watcher_registry.stats().await;
    let notifier_stats = state.notifier.stats().await;

    Ok(Json(serde_json::json!({
        "watchers": {
            "total": stats.total_watchers,
            "active": stats.active_watchers,
            "patterns_matched": stats.patterns_matched,
        },
        "notifications": {
            "total_sent": notifier_stats.total_sent,
            "by_mode": notifier_stats.by_mode,
        }
    })))
}

/// Emit a monitoring event (for internal/testing use)
async fn emit_event(
    State(state): State<crate::api::MonitoringState>,
    Json(event): Json<EmitEventRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let monitor_event = match event.event_type.as_str() {
        "AgentStart" => MonitorEvent::AgentStart {
            task_id: event.task_id.unwrap_or_else(|| ulid::Ulid::new().to_string()),
            prompt: event.payload.unwrap_or_default(),
        },
        "AgentEnd" => MonitorEvent::AgentEnd {
            task_id: event.task_id.unwrap_or_else(|| ulid::Ulid::new().to_string()),
            result: event.payload.unwrap_or_default(),
            success: true,
        },
        "AgentStep" => MonitorEvent::AgentStep {
            task_id: event.task_id.unwrap_or_else(|| ulid::Ulid::new().to_string()),
            step: 0,
            action: event.payload.clone().unwrap_or_default(),
            output: None,
        },
        _ => {
            return Err(ApiError::bad_request(format!(
                "Unknown event type: {}",
                event.event_type
            )));
        }
    };

    state.hook_emitter.emit(monitor_event.clone()).await;

    let matches: Vec<(String, NotifyMode)> = state
        .watcher_registry
        .check_matches(&monitor_event, event.project.as_deref())
        .await;

    Ok(Json(serde_json::json!({
        "emitted": true,
        "matches": matches.len(),
    })))
}

// Request/Response types

#[derive(Debug, Deserialize)]
pub struct CreateWatcherRequest {
    pub name: String,
    pub pattern: String,
    pub watch_scope: Option<WatchScope>,
    pub notify_on: Option<NotifyOn>,
    pub notification_mode: Option<NotifyMode>,
}

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct EmitEventRequest {
    pub event_type: String,
    pub task_id: Option<String>,
    pub payload: Option<String>,
    pub project: Option<String>,
}
