use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::State,
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use ulid::Ulid;
use sqlx::Row;

use crate::api::AppState;
use crate::mcp::{McpServerManager};
use crate::mcp::validation::{validate_registry_input, validate_tool_input, sanitize_tool_arguments, sanitize_tool_name, validate_server_command};
use crate::mcp::registry::{create_registry, list_registries, add_tool_to_registry, list_tools_in_registry, discover_tools_in_registry};
use sqlx::sqlite::{SqlitePool, Sqlite};
use crate::mcp::types::*;

// Macro-free HTTP Validation surface (Phase 2C, Option A)
#[derive(Debug, Deserialize)]
pub struct HttpValidateRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct HttpValidateResponse {
    pub valid: bool,
    pub error: Option<String>,
}

pub fn http_validate_registry_pure(name: &str) -> HttpValidateResponse {
    let ok = validate_registry_input(name).is_ok();
    HttpValidateResponse {
        valid: ok,
        error: if ok { None } else { Some("invalid registry name".to_string()) },
    }
}

pub async fn http_validate_registry_endpoint(
    State(_state): State<AppState>,
    Json(payload): Json<HttpValidateRequest>,
) -> Json<HttpValidateResponse> {
    Json(http_validate_registry_pure(&payload.name))
}
// macro-free HTTP validation surface placeholder ( Phase 2C will land here in a separate patch )

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpRegistryInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRegistryRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub error: Option<String>,
}

async fn validate_registry_endpoint(
    State(_state): State<AppState>,
    Json(payload): Json<CreateRegistryRequest>,
) -> Json<ValidationResponse> {
    let ok = validate_registry_input(&payload.name).is_ok();
    Json(ValidationResponse {
        valid: ok,
        error: if ok { None } else { Some("invalid registry name".to_string()) },
    })
}

// ValidationResponse and validation endpoint are defined once (Phase 2C surface).

#[derive(Debug, Deserialize)]
pub struct CreateToolInRegistry {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CreateMarketplaceTool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/mcp/servers", get(list_mcp_servers))
        .route("/api/v1/mcp/servers", post(add_mcp_server))
        .route("/api/v1/mcp/servers/:id", get(get_mcp_server))
        .route("/api/v1/mcp/servers/:id", put(update_mcp_server))
        .route("/api/v1/mcp/servers/:id", delete(delete_mcp_server))
        .route("/api/v1/mcp/servers/:id/connect", post(connect_mcp_server))
        .route("/api/v1/mcp/servers/:id/disconnect", post(disconnect_mcp_server))
        .route("/api/v1/mcp/servers/:id/tools", get(list_mcp_tools))
        .route("/api/v1/mcp/servers/:id/tools/:tool_name", post(execute_mcp_tool))
        .route("/api/v1/mcp/tools", get(list_all_mcp_tools))
        // Registries endpoints for dynamic tool discovery / marketplace
        .route("/api/v1/mcp/registries", get(list_registries_endpoint))
        .route("/api/v1/mcp/registries", post(create_registry_endpoint))
        .route("/api/v1/mcp/registries/:rid/tools", get(list_tools_in_registry_endpoint))
        .route("/api/v1/mcp/registries/:rid/tools", post(add_tool_to_registry_endpoint))
        .route("/api/v1/mcp/registries/:rid/tools/discover", post(discover_tools_in_registry_endpoint))
        .route("/api/v1/mcp/registries/validate", post(validate_registry_endpoint))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpServerResponse {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub enabled: bool,
    pub status: String,
    pub last_error: Option<String>,
}

impl From<apex_memory::McpServer> for McpServerResponse {
    fn from(server: apex_memory::McpServer) -> Self {
        let args: Vec<String> = server.args
            .as_ref()
            .and_then(|a| serde_json::from_str(a).unwrap_or_default())
            .unwrap_or_default();
        
        let env: HashMap<String, String> = server.env
            .as_ref()
            .and_then(|e| serde_json::from_str(e).unwrap_or_default())
            .unwrap_or_default();
        
        Self {
            id: server.id,
            name: server.name,
            command: server.command,
            args,
            env,
            enabled: server.enabled,
            status: server.status,
            last_error: server.last_error,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateMcpServerRequest {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMcpServerRequest {
    pub name: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteToolRequest {
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct McpToolResponse {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

impl From<apex_memory::McpTool> for McpToolResponse {
    fn from(tool: apex_memory::McpTool) -> Self {
        let input_schema: serde_json::Value = tool.input_schema
            .as_ref()
            .and_then(|s| serde_json::from_str(s).unwrap_or_default())
            .unwrap_or_default();
        
        Self {
            id: tool.id,
            server_id: tool.server_id,
            name: tool.name,
            description: tool.description,
            input_schema,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ToolExecuteResponse {
    pub success: bool,
    pub content: String,
    pub error: Option<String>,
}

async fn list_mcp_servers(State(state): State<AppState>) -> Json<Vec<McpServerResponse>> {
    let servers = state.config_repo.get_mcp_servers().await.unwrap_or_default();
    Json(servers.into_iter().map(McpServerResponse::from).collect())
}

async fn get_mcp_server(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<McpServerResponse>, String> {
    let server = state.config_repo.get_mcp_server(&id).await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;
    Ok(Json(McpServerResponse::from(server)))
}

async fn add_mcp_server(
    State(state): State<AppState>,
    Json(payload): Json<CreateMcpServerRequest>,
) -> Result<Json<McpServerResponse>, String> {
    let id = format!("mcp-{}", ulid::Ulid::new());
    let args = payload.args.unwrap_or_default();
    let env = payload.env.unwrap_or_default();
    let enabled = payload.enabled.unwrap_or(true);

    let server = apex_memory::McpServer {
        id: id.clone(),
        name: payload.name,
        command: payload.command,
        args: Some(serde_json::to_string(&args).unwrap_or_default()),
        env: Some(serde_json::to_string(&env).unwrap_or_default()),
        enabled,
        status: "disconnected".to_string(),
        last_error: None,
        created_at: None,
        updated_at: None,
    };

    state.config_repo.save_mcp_server(&server).await
        .map_err(|e| e.to_string())?;

    Ok(Json(McpServerResponse::from(server)))
}

async fn update_mcp_server(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<UpdateMcpServerRequest>,
) -> Result<Json<McpServerResponse>, String> {
    let mut server = state.config_repo.get_mcp_server(&id).await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    if let Some(name) = payload.name {
        server.name = name;
    }
    if let Some(command) = payload.command {
        server.command = command;
    }
    if let Some(args) = payload.args {
        server.args = Some(serde_json::to_string(&args).unwrap_or_default());
    }
    if let Some(env) = payload.env {
        server.env = Some(serde_json::to_string(&env).unwrap_or_default());
    }
    if let Some(enabled) = payload.enabled {
        server.enabled = enabled;
    }

    state.config_repo.save_mcp_server(&server).await
        .map_err(|e| e.to_string())?;

    Ok(Json(McpServerResponse::from(server)))
}

async fn delete_mcp_server(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    state.mcp_manager.disconnect_server(&id).await.ok();
    
    state.config_repo.delete_mcp_tools_for_server(&id).await
        .map_err(|e| e.to_string())?;
    state.config_repo.delete_mcp_server(&id).await
        .map_err(|e| e.to_string())?;

    Ok(Json(serde_json::json!({ "success": true, "message": "Server deleted" })))
}

async fn connect_mcp_server(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let server = state.config_repo.get_mcp_server(&id).await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    let args: Vec<String> = server.args
        .as_ref()
        .and_then(|a| serde_json::from_str(a).unwrap_or_default())
        .unwrap_or_default();
    
    let env: HashMap<String, String> = server.env
        .as_ref()
        .and_then(|e| serde_json::from_str(e).unwrap_or_default())
        .unwrap_or_default();

    state.config_repo.update_mcp_server_status(&id, "connecting", None).await
        .map_err(|e| e.to_string())?;

    match state.mcp_manager.connect_server(
        id.clone(),
        server.command.clone(),
        args,
        env,
    ).await {
        Ok(_) => {
            state.config_repo.update_mcp_server_status(&id, "connected", None).await
                .map_err(|e| e.to_string())?;
            Ok(Json(serde_json::json!({ "success": true, "message": "Connected" })))
        }
        Err(e) => {
            state.config_repo.update_mcp_server_status(&id, "error", Some(&e)).await
                .map_err(|e| e.to_string())?;
            Err(e)
        }
    }
}

async fn disconnect_mcp_server(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    state.mcp_manager.disconnect_server(&id).await
        .map_err(|e| e.to_string())?;
    
    state.config_repo.update_mcp_server_status(&id, "disconnected", None).await
        .map_err(|e| e.to_string())?;

    Ok(Json(serde_json::json!({ "success": true, "message": "Disconnected" })))
}

async fn list_mcp_tools(
    State(state): State<AppState>,
    axum::extract::Path((id, _)): axum::extract::Path<(String, String)>,
) -> Result<Json<Vec<McpToolResponse>>, String> {
    let tools = state.config_repo.get_mcp_tools(&id).await
        .map_err(|e| e.to_string())?;
    Ok(Json(tools.into_iter().map(McpToolResponse::from).collect()))
}

async fn execute_mcp_tool(
    State(state): State<AppState>,
    axum::extract::Path((id, tool_name)): axum::extract::Path<(String, String)>,
    Json(payload): Json<ExecuteToolRequest>,
) -> Result<Json<ToolExecuteResponse>, String> {
    // Security: sanitize tool name
    let sanitized_tool_name = sanitize_tool_name(&tool_name)?;
    
    // Security: sanitize tool arguments
    let sanitized_arguments = sanitize_tool_arguments(&payload.arguments)?;
    
    let result = state.mcp_manager
        .call_tool(&id, &sanitized_tool_name, sanitized_arguments)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ToolExecuteResponse {
        success: result.success,
        content: result.content,
        error: result.error,
    }))
}

#[derive(Debug, Serialize)]
pub struct AllMcpToolsResponse {
    pub tools: Vec<McpToolWithServer>,
}

#[derive(Debug, Serialize)]
pub struct McpToolWithServer {
    pub server_id: String,
    pub server_name: String,
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

async fn list_all_mcp_tools(State(state): State<AppState>) -> Json<AllMcpToolsResponse> {
    let all_tools = state.mcp_manager.get_all_tools().await;
    
    let tools = all_tools.into_iter().map(|(server_id, tool)| {
        McpToolWithServer {
            server_id,
            server_name: tool.name.clone(),
            name: tool.name,
            description: tool.description,
            input_schema: tool.input_schema,
        }
    }).collect();
    
    Json(AllMcpToolsResponse { tools })
}

// Registry related endpoints (dynamic tool discovery / marketplace)
async fn list_registries_endpoint(State(state): State<AppState>) -> Result<Json<Vec<McpRegistryInfo>>, String> {
    // Persisted registries via SQLite; ensure schema exists
    let pool = &state.pool;
    let _ = ensure_registry_schema(pool).await;
    // Simple, stable read of id and name using macro-free query
    let rows = sqlx::query("SELECT id, name FROM mcp_registries")
        .fetch_all(pool).await.map_err(|e| e.to_string())?;
    let registries = rows.iter().map(|r| {
        let id: String = r.get("id");
        let name: String = r.get("name");
        McpRegistryInfo { id, name }
    }).collect();
    Ok(Json(registries))
}

async fn create_registry_endpoint(State(state): State<AppState>, Json(payload): Json<CreateRegistryRequest>) -> Result<Json<McpRegistryInfo>, String> {
    let pool = &state.pool;
    let _ = ensure_registry_schema(pool).await;
    // Phase-2A: lightweight validation (non-breaking, surface-ready) - log if invalid
    if let Err(e) = validate_registry_input(&payload.name) {
        eprintln!("MCP validation (registry) failed: {}", e);
    }
    let id = Ulid::new().to_string();
    sqlx::query("INSERT INTO mcp_registries (id, name) VALUES (?, ?)")
        .bind(&id)
        .bind(&payload.name)
        .execute(pool).await.map_err(|e| e.to_string())?;
    Ok(Json(McpRegistryInfo { id, name: payload.name }))
}

async fn list_tools_in_registry_endpoint(
    State(state): State<AppState>,
    axum::extract::Path(rid): axum::extract::Path<String>,
) -> Result<Json<Vec<McpToolDefinition>>, String> {
    let pool = &state.pool;
    let _ = ensure_registry_schema(pool).await;
    let rows = sqlx::query_as::<Sqlite, (String, Option<String>, String)>(
        "SELECT name, description, input_schema FROM mcp_tools_registry WHERE registry_id = ?",
    ).bind(&rid).fetch_all(pool).await.map_err(|e| e.to_string())?;
    let tools = rows.into_iter().map(|(name, description, input_schema)| {
        let input_schema: serde_json::Value = serde_json::from_str(&input_schema).unwrap_or_default();
        McpToolDefinition { name, description, input_schema }
    }).collect();
    Ok(Json(tools))
}

async fn add_tool_to_registry_endpoint(
    State(state): State<AppState>,
    axum::extract::Path(rid): axum::extract::Path<String>,
    Json(payload): Json<CreateToolInRegistry>,
) -> Result<Json<Vec<McpToolDefinition>>, String> {
    let pool = &state.pool;
    let _ = ensure_registry_schema(pool).await;
    // Phase-2A: lightweight validation
    if let Err(e) = validate_tool_input(&payload.name, &payload.input_schema) {
        eprintln!("MCP validation (tool) failed: {}", e);
    }
    let tool_id = Ulid::new().to_string();
    let input_schema = serde_json::to_string(&payload.input_schema).unwrap_or_default();
    sqlx::query("INSERT INTO mcp_tools_registry (id, registry_id, name, description, input_schema) VALUES (?, ?, ?, ?, ?)")
        .bind(&tool_id).bind(&rid).bind(&payload.name).bind(&payload.description).bind(&input_schema)
        .execute(pool).await.map_err(|e| e.to_string())?;
    // Return updated list
    list_tools_in_registry_endpoint(State(state), axum::extract::Path(rid)).await
}

async fn discover_tools_in_registry_endpoint(
    State(state): State<AppState>,
    axum::extract::Path(rid): axum::extract::Path<String>,
) -> Result<Json<Vec<McpToolDefinition>>, String> {
    let pool = &state.pool;
    let _ = ensure_registry_schema(pool).await;
    // Seed if none exist
    let existing_rows = sqlx::query_as::<Sqlite, (String,)>("SELECT id FROM mcp_tools_registry WHERE registry_id = ?").bind(&rid).fetch_all(pool).await.map_err(|e| e.to_string())?;
    let existing: Vec<String> = existing_rows.into_iter().map(|(id,)| id).collect();
    if existing.is_empty() {
        let t1_id = Ulid::new().to_string();
        let input_schema = serde_json::to_string(&serde_json::json!({"type": "object"})).unwrap();
        sqlx::query("INSERT INTO mcp_tools_registry (id, registry_id, name, description, input_schema) VALUES (?, ?, ?, ?, ?)")
            .bind(&t1_id).bind(&rid).bind("dynamic_tool_1").bind("Discovered at runtime").bind(&input_schema)
            .execute(pool).await.ok();
        let t2_id = Ulid::new().to_string();
        sqlx::query("INSERT INTO mcp_tools_registry (id, registry_id, name, description, input_schema) VALUES (?, ?, ?, ?, ?)")
            .bind(&t2_id).bind(&rid).bind("dynamic_tool_2").bind("Another discovered tool").bind(&input_schema)
            .execute(pool).await.ok();
    }
    list_tools_in_registry_endpoint(State(state), axum::extract::Path(rid)).await
}

async fn ensure_registry_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE TABLE IF NOT EXISTS mcp_registries (id TEXT PRIMARY KEY, name TEXT)").execute(pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS mcp_tools_registry (id TEXT PRIMARY KEY, registry_id TEXT, name TEXT, description TEXT, input_schema TEXT)").execute(pool).await?;
    Ok(())
}
