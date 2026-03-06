#![allow(unused_imports)]

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};

use crate::webhook::CreateWebhook;

use super::{AppState, CreateWebhookRequest, WebhookResponse};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/webhooks", get(list_webhooks).post(create_webhook))
        .route("/api/v1/webhooks/:id", get(get_webhook).delete(delete_webhook))
        .route("/api/v1/webhooks/:id/toggle", post(toggle_webhook))
}

async fn list_webhooks(State(state): State<AppState>) -> Result<Json<Vec<WebhookResponse>>, String> {
    let webhooks = state.webhook_manager.list_webhooks().await;
    let responses: Vec<WebhookResponse> = webhooks
        .into_iter()
        .map(|w| WebhookResponse {
            id: w.id,
            name: w.name,
            url: w.url,
            events: w.events,
            enabled: w.enabled,
            created_at_ms: w.created_at_ms,
            last_triggered_ms: w.last_triggered_ms,
            failure_count: w.failure_count,
        })
        .collect();
    Ok(Json(responses))
}

async fn get_webhook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WebhookResponse>, String> {
    let webhook = state
        .webhook_manager
        .get_webhook(&id)
        .await
        .ok_or_else(|| "Webhook not found".to_string())?;
    Ok(Json(WebhookResponse {
        id: webhook.id,
        name: webhook.name,
        url: webhook.url,
        events: webhook.events,
        enabled: webhook.enabled,
        created_at_ms: webhook.created_at_ms,
        last_triggered_ms: webhook.last_triggered_ms,
        failure_count: webhook.failure_count,
    }))
}

async fn create_webhook(
    State(state): State<AppState>,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<Json<WebhookResponse>, String> {
    let create = CreateWebhook {
        name: req.name,
        url: req.url,
        events: req.events,
        secret: req.secret,
    };
    let webhook = state.webhook_manager.create_webhook(create).await;
    Ok(Json(WebhookResponse {
        id: webhook.id,
        name: webhook.name,
        url: webhook.url,
        events: webhook.events,
        enabled: webhook.enabled,
        created_at_ms: webhook.created_at_ms,
        last_triggered_ms: webhook.last_triggered_ms,
        failure_count: webhook.failure_count,
    }))
}

async fn delete_webhook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let deleted = state.webhook_manager.delete_webhook(&id).await;
    if deleted {
        Ok(Json(serde_json::json!({ "success": true, "deleted": id })))
    } else {
        Err("Webhook not found".to_string())
    }
}

async fn toggle_webhook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WebhookResponse>, String> {
    let webhook = state
        .webhook_manager
        .toggle_webhook(&id)
        .await
        .ok_or_else(|| "Webhook not found".to_string())?;
    Ok(Json(WebhookResponse {
        id: webhook.id,
        name: webhook.name,
        url: webhook.url,
        events: webhook.events,
        enabled: webhook.enabled,
        created_at_ms: webhook.created_at_ms,
        last_triggered_ms: webhook.last_triggered_ms,
        failure_count: webhook.failure_count,
    }))
}
