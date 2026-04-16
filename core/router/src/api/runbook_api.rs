//! Runbook API Endpoints
//! 
//! v1.10.0: Automated Runbook Execution API

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;

use crate::api::api_error::ApiError;
use crate::api::AppState;
use crate::runbook::{
    CreateRunbookRequest, ExecuteRunbookRequest, Runbook, RunbookExecution, 
    RunbookManager, RunbookTemplate, UpdateRunbookRequest,
};

/// Create the runbook router
pub fn create_runbook_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/runbooks", get(list_runbooks))
        .route("/api/v1/runbooks", post(create_runbook))
        .route("/api/v1/runbooks/templates", get(list_templates))
        .route("/api/v1/runbooks/:id", get(get_runbook))
        .route("/api/v1/runbooks/:id", put(update_runbook))
        .route("/api/v1/runbooks/:id", delete(delete_runbook))
        .route("/api/v1/runbooks/:id/execute", post(execute_runbook))
        .route("/api/v1/runbooks/:id/executions", get(list_executions))
        .route("/api/v1/runbooks/executions/:exec_id", get(get_execution))
}

/// List all runbooks
async fn list_runbooks(
    State(state): State<AppState>,
) -> Result<Json<Vec<Runbook>>, (axum::http::StatusCode, String)> {
    let manager = state.runbook_manager.read().await;
    let runbooks = manager.list().await;
    Ok(Json(runbooks))
}

/// Get a runbook by ID
async fn get_runbook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Runbook>, (axum::http::StatusCode, String)> {
    let manager = state.runbook_manager.read().await;
    manager
        .get(&id)
        .await
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Runbook {} not found", id)))
}

/// Create a new runbook
async fn create_runbook(
    State(state): State<AppState>,
    Json(payload): Json<CreateRunbookRequest>,
) -> Result<Json<Runbook>, (axum::http::StatusCode, String)> {
    let runbook = Runbook {
        id: ulid::Ulid::new().to_string(),
        name: payload.name,
        description: payload.description,
        trigger_alert_type: payload.trigger_alert_type,
        steps: payload.steps,
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state.runbook_manager.write().await.create(runbook.clone()).await;
    
    tracing::info!("Created runbook: {} ({})", runbook.name, runbook.id);
    
    Ok(Json(runbook))
}

/// Update a runbook
async fn update_runbook(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateRunbookRequest>,
) -> Result<Json<Runbook>, (axum::http::StatusCode, String)> {
    let manager = state.runbook_manager.write().await;
    let mut runbook = manager
        .get(&id)
        .await
        .ok_or_else(|| ApiError::not_found(format!("Runbook {} not found", id)))?;

    if let Some(name) = payload.name {
        runbook.name = name;
    }
    if let Some(desc) = payload.description {
        runbook.description = Some(desc);
    }
    if let Some(trigger) = payload.trigger_alert_type {
        runbook.trigger_alert_type = Some(trigger);
    }
    if let Some(steps) = payload.steps {
        runbook.steps = steps;
    }
    if let Some(enabled) = payload.enabled {
        runbook.enabled = enabled;
    }
    runbook.updated_at = chrono::Utc::now();

    manager
        .update(&id, runbook.clone())
        .await
        .ok_or_else(|| ApiError::not_found(format!("Runbook {} not found", id)))?;

    tracing::info!("Updated runbook: {} ({})", runbook.name, runbook.id);

    Ok(Json(runbook))
}

/// Delete a runbook
async fn delete_runbook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    state.runbook_manager.write().await
        .delete(&id)
        .await
        .ok_or_else(|| ApiError::not_found(format!("Runbook {} not found", id)))?;

    tracing::info!("Deleted runbook: {}", id);

    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// Execute a runbook
async fn execute_runbook(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ExecuteRunbookRequest>,
) -> Result<Json<RunbookExecution>, (axum::http::StatusCode, String)> {
    let execution = state.runbook_manager.write().await
        .execute(&id, payload.alert_id)
        .await
        .ok_or_else(|| ApiError::not_found(format!("Runbook {} not found or disabled", id)))?;

    tracing::info!(
        "Executed runbook {}: {} (execution: {})",
        id,
        execution.status.to_string(),
        execution.id
    );

    Ok(Json(execution))
}

/// List executions for a runbook
async fn list_executions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<RunbookExecution>>, (axum::http::StatusCode, String)> {
    let manager = state.runbook_manager.read().await;
    let executions = manager.get_executions(&id).await;
    Ok(Json(executions))
}

/// Get a specific execution
async fn get_execution(
    State(state): State<AppState>,
    Path(exec_id): Path<String>,
) -> Result<Json<RunbookExecution>, (axum::http::StatusCode, String)> {
    let manager = state.runbook_manager.read().await;
    manager
        .get_execution(&exec_id)
        .await
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Execution {} not found", exec_id)))
}

/// List available runbook templates
async fn list_templates() -> Json<Vec<RunbookTemplate>> {
    let templates = vec![
        RunbookTemplate::restart_failed_task(),
        RunbookTemplate::clear_resource(),
        RunbookTemplate::escalate_notification(),
    ];
    Json(templates)
}

/// Helper to get string from ExecutionStatus
impl ToString for crate::runbook::ExecutionStatus {
    fn to_string(&self) -> String {
        match self {
            crate::runbook::ExecutionStatus::Pending => "pending".to_string(),
            crate::runbook::ExecutionStatus::Running => "running".to_string(),
            crate::runbook::ExecutionStatus::Completed => "completed".to_string(),
            crate::runbook::ExecutionStatus::Failed => "failed".to_string(),
            crate::runbook::ExecutionStatus::Cancelled => "cancelled".to_string(),
        }
    }
}
