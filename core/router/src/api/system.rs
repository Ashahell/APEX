use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};

use apex_memory::msg_repo::MessageRepository;
use apex_memory::task_repo::TaskRepository;

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/metrics", get(get_metrics))
        .route("/api/v1/vm/stats", get(get_vm_stats))
        .route("/api/v1/skills/pool/stats", get(get_skill_pool_stats))
        .route("/api/v1/messages", get(list_messages))
        .route("/api/v1/messages/task/:task_id", get(get_task_messages))
}

async fn get_metrics(State(state): State<AppState>) -> Json<serde_json::Value> {
    let metrics = state.metrics.get_metrics().await;
    let repo = TaskRepository::new(&state.pool);
    let db_total_cost = repo.get_total_cost().await.unwrap_or(0.0);
    let tasks_completed = metrics.tasks_by_status.get("completed").copied().unwrap_or(0);
    let tasks_failed = metrics.tasks_by_status.get("failed").copied().unwrap_or(0);
    Json(serde_json::json!({
        "tasks": metrics.tasks_total,
        "by_tier": metrics.tasks_by_tier,
        "by_status": metrics.tasks_by_status,
        "total_cost_usd": db_total_cost,
        "tasks_completed": tasks_completed,
        "tasks_failed": tasks_failed,
    }))
}

async fn get_vm_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    if let Some(pool) = &state.vm_pool {
        let stats = pool.get_stats().await;
        Json(serde_json::json!({
            "enabled": stats.enabled,
            "backend": stats.backend,
            "total": stats.total,
            "ready": stats.ready,
            "busy": stats.busy,
            "starting": stats.starting,
            "stopped": stats.stopped,
            "available": stats.available,
        }))
    } else {
        Json(serde_json::json!({
            "enabled": false,
            "message": "VM pool not initialized"
        }))
    }
}

async fn get_skill_pool_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    if let Some(pool) = &state.skill_pool {
        let stats = pool.stats().await;
        Json(serde_json::json!({
            "enabled": stats.enabled,
            "pool_size": stats.pool_size,
            "available_slots": stats.available_slots,
            "total_requests": stats.total_requests,
            "total_errors": stats.total_errors,
            "avg_latency_ms": stats.avg_latency_ms,
        }))
    } else {
        Json(serde_json::json!({
            "enabled": false,
            "message": "Skill pool not initialized"
        }))
    }
}

async fn list_messages(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<apex_memory::msg_repo::Message>>, String> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(100);
    let offset = params
        .get("offset")
        .and_then(|o| o.parse().ok())
        .unwrap_or(0);
    let channel = params.get("channel");

    let repo = MessageRepository::new(&state.pool);

    let messages = if let Some(ch) = channel {
        repo.find_by_channel(ch, limit, offset).await
    } else {
        repo.find_recent(limit).await
    }
    .map_err(|e| format!("Failed to list messages: {}", e))?;

    Ok(Json(messages))
}

async fn get_task_messages(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Vec<apex_memory::msg_repo::Message>>, String> {
    let repo = MessageRepository::new(&state.pool);
    let messages = repo
        .find_by_task(&task_id)
        .await
        .map_err(|e| format!("Failed to get task messages: {}", e))?;

    Ok(Json(messages))
}
