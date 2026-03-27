#![allow(unused_imports)]

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};

use super::{AppState, ListNotificationsQuery, NotificationResponse};
use crate::notification::ExternalNotificationConfig;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/notifications",
            get(list_notifications).delete(clear_notifications),
        )
        .route("/api/v1/notifications/unread-count", get(get_unread_count))
        .route(
            "/api/v1/notifications/:id",
            get(get_notification).delete(delete_notification),
        )
        .route(
            "/api/v1/notifications/:id/read",
            post(mark_notification_read),
        )
        .route("/api/v1/notifications/read-all", post(mark_all_read))
        .route(
            "/api/v1/notifications/external",
            get(get_external_config).put(set_external_config),
        )
        .route(
            "/api/v1/notifications/external/test",
            post(test_external_notification),
        )
}

async fn list_notifications(
    State(state): State<AppState>,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<Vec<NotificationResponse>>, String> {
    let notifications = state
        .notification_manager
        .list(query.include_read.unwrap_or(false))
        .await;
    Ok(Json(
        notifications
            .into_iter()
            .map(NotificationResponse::from)
            .collect(),
    ))
}

async fn get_unread_count(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, String> {
    let count = state.notification_manager.unread_count().await;
    Ok(Json(serde_json::json!({ "unread_count": count })))
}

async fn get_notification(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<NotificationResponse>, String> {
    let notification = state
        .notification_manager
        .get(&id)
        .await
        .ok_or_else(|| "Notification not found".to_string())?;
    Ok(Json(NotificationResponse::from(notification)))
}

async fn mark_notification_read(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<NotificationResponse>, String> {
    let notification = state
        .notification_manager
        .mark_read(&id)
        .await
        .ok_or_else(|| "Notification not found".to_string())?;
    Ok(Json(NotificationResponse::from(notification)))
}

async fn mark_all_read(State(state): State<AppState>) -> Result<Json<serde_json::Value>, String> {
    state.notification_manager.mark_all_read().await;
    Ok(Json(serde_json::json!({ "success": true })))
}

async fn delete_notification(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let deleted = state.notification_manager.delete(&id).await;
    if deleted {
        Ok(Json(serde_json::json!({ "success": true, "deleted": id })))
    } else {
        Err("Notification not found".to_string())
    }
}

async fn clear_notifications(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, String> {
    state.notification_manager.clear_all().await;
    Ok(Json(
        serde_json::json!({ "success": true, "cleared": true }),
    ))
}

/// Get external notification configuration
async fn get_external_config(
    State(state): State<AppState>,
) -> Result<Json<ExternalNotificationConfig>, String> {
    let config = state.notification_manager.get_external_config().await;
    Ok(Json(config))
}

/// Update external notification configuration
async fn set_external_config(
    State(state): State<AppState>,
    Json(config): Json<ExternalNotificationConfig>,
) -> Result<Json<ExternalNotificationConfig>, String> {
    state
        .notification_manager
        .set_external_config(config.clone())
        .await;
    Ok(Json(config))
}

/// Test external notification (sends a test message)
async fn test_external_notification(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, String> {
    let result = state
        .notification_manager
        .send_external_notification("Test Notification", "This is a test from APEX!")
        .await;

    match result {
        Ok(()) => Ok(Json(
            serde_json::json!({ "success": true, "message": "Test notification sent" }),
        )),
        Err(e) => Err(e),
    }
}
