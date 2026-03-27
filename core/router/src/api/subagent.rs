use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::subagent::{SubAgentPool, SubTask, SubTaskStatus};

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/subagent/decompose", post(decompose_task))
        .route("/api/v1/subagent/tasks", get(list_tasks))
        .route("/api/v1/subagent/tasks/:id", get(get_task))
        .route("/api/v1/subagent/tasks/:id/status", put(update_task_status))
        .route("/api/v1/subagent/ready", get(get_ready_tasks))
        .route("/api/v1/subagent/complete", get(check_complete))
}

#[derive(Debug, Deserialize)]
pub struct DecomposeRequest {
    pub goal: String,
    pub context: String,
}

#[derive(Debug, Serialize)]
pub struct DecomposeResponse {
    pub subtasks: Vec<SubTask>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: SubTaskStatus,
    pub result: Option<String>,
}

async fn decompose_task(
    State(state): State<AppState>,
    Json(payload): Json<DecomposeRequest>,
) -> Result<Json<DecomposeResponse>, String> {
    let pool = state.subagent_pool.read().await;

    // Get LLM config from environment or use defaults
    let llm_url =
        std::env::var("LLAMA_SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let model = std::env::var("LLAMA_MODEL").unwrap_or_else(|_| "qwen3-4b".to_string());

    let subtasks = pool
        .split_task(&payload.goal, &payload.context, &llm_url, &model)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(DecomposeResponse { subtasks }))
}

async fn list_tasks(State(state): State<AppState>) -> Result<Json<Vec<SubTask>>, String> {
    let pool = state.subagent_pool.read().await;
    let tasks = pool.get_all_tasks().await;
    Ok(Json(tasks))
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SubTask>, String> {
    let pool = state.subagent_pool.read().await;

    pool.get_task(&id)
        .await
        .map(Json)
        .ok_or_else(|| "Task not found".to_string())
}

async fn update_task_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<Json<serde_json::Value>, String> {
    let pool = state.subagent_pool.read().await;

    let success = pool
        .update_status(&id, payload.status, payload.result)
        .await;

    if success {
        Ok(Json(serde_json::json!({ "success": true })))
    } else {
        Err("Task not found".to_string())
    }
}

async fn get_ready_tasks(State(state): State<AppState>) -> Result<Json<Vec<SubTask>>, String> {
    let pool = state.subagent_pool.read().await;
    let tasks = pool.get_ready_tasks().await;
    Ok(Json(tasks))
}

async fn check_complete(State(state): State<AppState>) -> Result<Json<serde_json::Value>, String> {
    let pool = state.subagent_pool.read().await;
    let is_complete = pool.is_complete().await;
    let failed = pool.has_failed().await;

    Ok(Json(serde_json::json!({
        "complete": is_complete,
        "failed": failed,
    })))
}
