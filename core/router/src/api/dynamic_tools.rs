use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::dynamic_tools::{DynamicTool, ToolRegistry};

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/dynamic-tools", get(list_tools))
        .route("/api/v1/dynamic-tools", post(create_tool))
        .route("/api/v1/dynamic-tools/:name", get(get_tool))
        .route("/api/v1/dynamic-tools/:name", delete(delete_tool))
        .route("/api/v1/dynamic-tools/:name/execute", post(execute_tool))
}

#[derive(Debug, Deserialize)]
pub struct CreateToolRequest {
    pub goal: String,
    pub context: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteToolRequest {
    pub parameters: serde_json::Value,
}

async fn list_tools(State(state): State<AppState>) -> Result<Json<Vec<DynamicTool>>, String> {
    let registry = state.dynamic_tools.read().await;
    let tools = registry.list().await;
    Ok(Json(tools))
}

async fn create_tool(
    State(state): State<AppState>,
    Json(payload): Json<CreateToolRequest>,
) -> Result<Json<DynamicTool>, String> {
    // Get LLM config
    let llm_url =
        std::env::var("LLAMA_SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let model = std::env::var("LLAMA_MODEL").unwrap_or_else(|_| "qwen3-4b".to_string());

    let tool =
        crate::dynamic_tools::generate_tool(&payload.goal, &payload.context, &llm_url, &model)
            .await
            .map_err(|e| e.to_string())?;

    // Register the tool
    let registry = state.dynamic_tools.read().await;
    registry.register(tool.clone()).await;

    Ok(Json(tool))
}

async fn get_tool(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DynamicTool>, String> {
    let registry = state.dynamic_tools.read().await;
    registry
        .get(&name)
        .await
        .map(Json)
        .ok_or_else(|| "Tool not found".to_string())
}

async fn delete_tool(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let registry = state.dynamic_tools.read().await;
    let deleted = registry.remove(&name).await;

    if deleted {
        Ok(Json(
            serde_json::json!({ "success": true, "message": "Tool deleted" }),
        ))
    } else {
        Err("Tool not found".to_string())
    }
}

async fn execute_tool(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<ExecuteToolRequest>,
) -> Result<Json<serde_json::Value>, String> {
    let registry = state.dynamic_tools.read().await;

    // Get sandbox config from unified config
    let sandbox_config = crate::dynamic_tools::SandboxConfig {
        memory_limit_mb: state.config.execution.sandbox.memory_limit_mb,
        timeout_secs: state.config.execution.sandbox.timeout_secs,
    };

    let result = registry
        .execute(&name, payload.parameters, Some(sandbox_config))
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(result))
}
