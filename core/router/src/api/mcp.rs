use axum::{
    extract::State,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use ulid::Ulid;

use crate::api::AppState;
use crate::mcp::registry::{
    add_tool_to_registry, create_registry, discover_tools_in_registry, list_registries,
    list_tools_in_registry,
};
use crate::mcp::types::*;
use crate::mcp::validation::{
    sanitize_tool_arguments, sanitize_tool_name, validate_registry_input, validate_server_command,
    validate_tool_input,
};
use crate::mcp::McpServerManager;
use sqlx::sqlite::{Sqlite, SqlitePool};

// ============================================================================
// Phase 4: MCP Tool Registry and Discovery Enhancements
// ============================================================================

/// Tool version metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolVersionInfo {
    pub version: String,
    pub created_at: String,
    pub is_stable: bool,
}

/// Tool health status
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolHealthStatus {
    pub name: String,
    pub server_id: String,
    pub status: String, // healthy, degraded, unhealthy
    pub last_check: String,
    pub latency_ms: Option<u64>,
    pub error_count: u64,
    pub success_count: u64,
}

/// Tool with extended metadata (Phase 4)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtendedToolInfo {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    pub version: String,
    pub capabilities: Vec<String>,
    pub health: ToolHealthStatus,
    pub usage_count: u64,
    pub avg_latency_ms: Option<u64>,
}

/// MCP server with health metrics (Phase 4)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpServerWithHealth {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub enabled: bool,
    pub status: String,
    pub last_error: Option<String>,
    // Phase 4: Health metrics
    pub health_score: f64, // 0.0 - 1.0
    pub avg_latency_ms: Option<u64>,
    pub error_rate_pct: f64,
    pub tool_count: u64,
    pub last_health_check: Option<String>,
}

/// Tool discovery response (Phase 4)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolDiscoveryResponse {
    pub tools: Vec<ExtendedToolInfo>,
    pub total_count: u64,
    pub servers_count: u64,
    pub discovery_timestamp: String,
}

/// Marketplace tool listing (Phase 4)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketplaceTool {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub author: String,
    pub rating: f64,
    pub install_count: u64,
    pub category: String,
    pub tags: Vec<String>,
    pub input_schema: serde_json::Value,
}

// ============================================================================
// Phase 4: MCP Tool Metrics Tracking
// ============================================================================

#[derive(Debug, Clone, Default)]
struct ToolMetrics {
    success_count: u64,
    error_count: u64,
    total_latency_ms: u64,
    last_check: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct McpMetricsTracker {
    tools: Arc<RwLock<HashMap<String, ToolMetrics>>>,
    servers: Arc<RwLock<HashMap<String, ToolMetrics>>>,
}

impl McpMetricsTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn record_tool_success(&self, tool_key: &str, latency_ms: u64) {
        let mut tools = self.tools.write().await;
        let metrics = tools.entry(tool_key.to_string()).or_default();
        metrics.success_count += 1;
        metrics.total_latency_ms += latency_ms;
        metrics.last_check = Some(chrono::Utc::now().to_rfc3339());
    }

    pub async fn record_tool_error(&self, tool_key: &str) {
        let mut tools = self.tools.write().await;
        let metrics = tools.entry(tool_key.to_string()).or_default();
        metrics.error_count += 1;
        metrics.last_check = Some(chrono::Utc::now().to_rfc3339());
    }

    pub async fn get_tool_health(&self, tool_key: &str) -> Option<ToolHealthStatus> {
        let tools = self.tools.read().await;
        let metrics = tools.get(tool_key)?;
        let total = metrics.success_count + metrics.error_count;
        let error_rate = if total > 0 {
            (metrics.error_count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let avg_latency = if metrics.success_count > 0 {
            Some(metrics.total_latency_ms / metrics.success_count)
        } else {
            None
        };
        let status = if error_rate > 50.0 {
            "unhealthy"
        } else if error_rate > 10.0 {
            "degraded"
        } else {
            "healthy"
        };

        Some(ToolHealthStatus {
            name: tool_key.to_string(),
            server_id: String::new(),
            status: status.to_string(),
            last_check: metrics.last_check.clone().unwrap_or_default(),
            latency_ms: avg_latency,
            error_count: metrics.error_count,
            success_count: metrics.success_count,
        })
    }
}

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
        error: if ok {
            None
        } else {
            Some("invalid registry name".to_string())
        },
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
        error: if ok {
            None
        } else {
            Some("invalid registry name".to_string())
        },
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
        .route(
            "/api/v1/mcp/servers/:id/disconnect",
            post(disconnect_mcp_server),
        )
        .route("/api/v1/mcp/servers/:id/tools", get(list_mcp_tools))
        .route(
            "/api/v1/mcp/servers/:id/tools/:tool_name",
            post(execute_mcp_tool),
        )
        .route("/api/v1/mcp/tools", get(list_all_mcp_tools))
        // Phase 4: MCP enriched endpoints
        .route("/api/v1/mcp/servers/health", get(get_all_servers_health))
        .route("/api/v1/mcp/tools/discover", get(discover_all_tools))
        .route("/api/v1/mcp/tools/:tool_key/health", get(get_tool_health))
        .route("/api/v1/mcp/marketplace", get(list_marketplace_tools))
        // Registries endpoints for dynamic tool discovery / marketplace
        .route("/api/v1/mcp/registries", get(list_registries_endpoint))
        .route("/api/v1/mcp/registries", post(create_registry_endpoint))
        .route(
            "/api/v1/mcp/registries/:rid/tools",
            get(list_tools_in_registry_endpoint),
        )
        .route(
            "/api/v1/mcp/registries/:rid/tools",
            post(add_tool_to_registry_endpoint),
        )
        .route(
            "/api/v1/mcp/registries/:rid/tools/discover",
            post(discover_tools_in_registry_endpoint),
        )
        .route(
            "/api/v1/mcp/registries/validate",
            post(validate_registry_endpoint),
        )
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
        let args: Vec<String> = server
            .args
            .as_ref()
            .and_then(|a| serde_json::from_str(a).unwrap_or_default())
            .unwrap_or_default();

        let env: HashMap<String, String> = server
            .env
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
        let input_schema: serde_json::Value = tool
            .input_schema
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
    let servers = state
        .config_repo
        .get_mcp_servers()
        .await
        .unwrap_or_default();
    Json(servers.into_iter().map(McpServerResponse::from).collect())
}

async fn get_mcp_server(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<McpServerResponse>, String> {
    let server = state
        .config_repo
        .get_mcp_server(&id)
        .await
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

    state
        .config_repo
        .save_mcp_server(&server)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(McpServerResponse::from(server)))
}

async fn update_mcp_server(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<UpdateMcpServerRequest>,
) -> Result<Json<McpServerResponse>, String> {
    let mut server = state
        .config_repo
        .get_mcp_server(&id)
        .await
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

    state
        .config_repo
        .save_mcp_server(&server)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(McpServerResponse::from(server)))
}

async fn delete_mcp_server(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    state.mcp_manager.disconnect_server(&id).await.ok();

    state
        .config_repo
        .delete_mcp_tools_for_server(&id)
        .await
        .map_err(|e| e.to_string())?;
    state
        .config_repo
        .delete_mcp_server(&id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(
        serde_json::json!({ "success": true, "message": "Server deleted" }),
    ))
}

async fn connect_mcp_server(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let server = state
        .config_repo
        .get_mcp_server(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Server not found".to_string())?;

    let args: Vec<String> = server
        .args
        .as_ref()
        .and_then(|a| serde_json::from_str(a).unwrap_or_default())
        .unwrap_or_default();

    let env: HashMap<String, String> = server
        .env
        .as_ref()
        .and_then(|e| serde_json::from_str(e).unwrap_or_default())
        .unwrap_or_default();

    state
        .config_repo
        .update_mcp_server_status(&id, "connecting", None)
        .await
        .map_err(|e| e.to_string())?;

    match state
        .mcp_manager
        .connect_server(id.clone(), server.command.clone(), args, env)
        .await
    {
        Ok(_) => {
            state
                .config_repo
                .update_mcp_server_status(&id, "connected", None)
                .await
                .map_err(|e| e.to_string())?;
            Ok(Json(
                serde_json::json!({ "success": true, "message": "Connected" }),
            ))
        }
        Err(e) => {
            state
                .config_repo
                .update_mcp_server_status(&id, "error", Some(&e))
                .await
                .map_err(|e| e.to_string())?;
            Err(e)
        }
    }
}

async fn disconnect_mcp_server(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    state
        .mcp_manager
        .disconnect_server(&id)
        .await
        .map_err(|e| e.to_string())?;

    state
        .config_repo
        .update_mcp_server_status(&id, "disconnected", None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(
        serde_json::json!({ "success": true, "message": "Disconnected" }),
    ))
}

async fn list_mcp_tools(
    State(state): State<AppState>,
    axum::extract::Path((id, _)): axum::extract::Path<(String, String)>,
) -> Result<Json<Vec<McpToolResponse>>, String> {
    let tools = state
        .config_repo
        .get_mcp_tools(&id)
        .await
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

    let result = state
        .mcp_manager
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

    let tools = all_tools
        .into_iter()
        .map(|(server_id, tool)| McpToolWithServer {
            server_id,
            server_name: tool.name.clone(),
            name: tool.name,
            description: tool.description,
            input_schema: tool.input_schema,
        })
        .collect();

    Json(AllMcpToolsResponse { tools })
}

// Registry related endpoints (dynamic tool discovery / marketplace)
async fn list_registries_endpoint(
    State(state): State<AppState>,
) -> Result<Json<Vec<McpRegistryInfo>>, String> {
    // Persisted registries via SQLite; ensure schema exists
    let pool = &state.pool;
    let _ = ensure_registry_schema(pool).await;
    // Simple, stable read of id and name using macro-free query
    let rows = sqlx::query("SELECT id, name FROM mcp_registries")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    let registries = rows
        .iter()
        .map(|r| {
            let id: String = r.get("id");
            let name: String = r.get("name");
            McpRegistryInfo { id, name }
        })
        .collect();
    Ok(Json(registries))
}

async fn create_registry_endpoint(
    State(state): State<AppState>,
    Json(payload): Json<CreateRegistryRequest>,
) -> Result<Json<McpRegistryInfo>, String> {
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
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(McpRegistryInfo {
        id,
        name: payload.name,
    }))
}

async fn list_tools_in_registry_endpoint(
    State(state): State<AppState>,
    axum::extract::Path(rid): axum::extract::Path<String>,
) -> Result<Json<Vec<McpToolDefinition>>, String> {
    let pool = &state.pool;
    let _ = ensure_registry_schema(pool).await;
    let rows = sqlx::query_as::<Sqlite, (String, Option<String>, String)>(
        "SELECT name, description, input_schema FROM mcp_tools_registry WHERE registry_id = ?",
    )
    .bind(&rid)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    let tools = rows
        .into_iter()
        .map(|(name, description, input_schema)| {
            let input_schema: serde_json::Value =
                serde_json::from_str(&input_schema).unwrap_or_default();
            McpToolDefinition {
                name,
                description,
                input_schema,
            }
        })
        .collect();
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
    let existing_rows = sqlx::query_as::<Sqlite, (String,)>(
        "SELECT id FROM mcp_tools_registry WHERE registry_id = ?",
    )
    .bind(&rid)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
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
    sqlx::query("CREATE TABLE IF NOT EXISTS mcp_registries (id TEXT PRIMARY KEY, name TEXT)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS mcp_tools_registry (id TEXT PRIMARY KEY, registry_id TEXT, name TEXT, description TEXT, input_schema TEXT)").execute(pool).await?;
    Ok(())
}

// ============================================================================
// Phase 4: MCP Enriched Endpoints
// ============================================================================

/// Get health status for all MCP servers
async fn get_all_servers_health(
    State(state): State<AppState>,
) -> Json<Vec<McpServerWithHealth>> {
    let servers = state
        .config_repo
        .get_mcp_servers()
        .await
        .unwrap_or_default();

    let mut results = Vec::new();
    for server in servers {
        let tools = state
            .config_repo
            .get_mcp_tools(&server.id)
            .await
            .unwrap_or_default();

        // Calculate health score based on status and tool count
        let health_score = match server.status.as_str() {
            "connected" => 1.0,
            "connecting" => 0.5,
            "error" => 0.2,
            _ => 0.0,
        };

        results.push(McpServerWithHealth {
            id: server.id,
            name: server.name,
            command: server.command,
            args: server
                .args
                .as_ref()
                .and_then(|a| serde_json::from_str(a).ok())
                .unwrap_or_default(),
            env: server
                .env
                .as_ref()
                .and_then(|e| serde_json::from_str(e).ok())
                .unwrap_or_default(),
            enabled: server.enabled,
            status: server.status,
            last_error: server.last_error,
            health_score,
            avg_latency_ms: None,
            error_rate_pct: 0.0,
            tool_count: tools.len() as u64,
            last_health_check: Some(chrono::Utc::now().to_rfc3339()),
        });
    }

    Json(results)
}

/// Discover all available tools across all connected servers
async fn discover_all_tools(State(state): State<AppState>) -> Json<ToolDiscoveryResponse> {
    let all_tools = state.mcp_manager.get_all_tools().await;
    let servers = state
        .config_repo
        .get_mcp_servers()
        .await
        .unwrap_or_default();

    let tools: Vec<ExtendedToolInfo> = all_tools
        .into_iter()
        .map(|(server_id, tool)| ExtendedToolInfo {
            id: Ulid::new().to_string(),
            server_id,
            name: tool.name,
            description: tool.description,
            input_schema: tool.input_schema,
            version: "1.0.0".to_string(),
            capabilities: vec!["execute".to_string()],
            health: ToolHealthStatus {
                name: String::new(),
                server_id: String::new(),
                status: "healthy".to_string(),
                last_check: chrono::Utc::now().to_rfc3339(),
                latency_ms: None,
                error_count: 0,
                success_count: 0,
            },
            usage_count: 0,
            avg_latency_ms: None,
        })
        .collect();

    let tool_count = tools.len() as u64;
    let server_count = servers.len() as u64;

    Json(ToolDiscoveryResponse {
        tools,
        total_count: tool_count,
        servers_count: server_count,
        discovery_timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Get health status for a specific tool
async fn get_tool_health(
    State(_state): State<AppState>,
    axum::extract::Path(tool_key): axum::extract::Path<String>,
) -> Json<ToolHealthStatus> {
    // Return default healthy status for now
    // In production, this would query the metrics tracker
    Json(ToolHealthStatus {
        name: tool_key,
        server_id: String::new(),
        status: "healthy".to_string(),
        last_check: chrono::Utc::now().to_rfc3339(),
        latency_ms: None,
        error_count: 0,
        success_count: 0,
    })
}

/// List marketplace tools (scaffolding)
async fn list_marketplace_tools(
    State(_state): State<AppState>,
) -> Json<Vec<MarketplaceTool>> {
    // Phase 4: Return sample marketplace tools
    let sample_tools = vec![
        MarketplaceTool {
            id: "mkt-1".to_string(),
            name: "file-reader".to_string(),
            description: Some("Read files from the filesystem".to_string()),
            version: "1.0.0".to_string(),
            author: "apex".to_string(),
            rating: 4.5,
            install_count: 150,
            category: "utilities".to_string(),
            tags: vec!["file".to_string(), "read".to_string()],
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path to read"}
                },
                "required": ["path"]
            }),
        },
        MarketplaceTool {
            id: "mkt-2".to_string(),
            name: "web-scraper".to_string(),
            description: Some("Scrape web content from URLs".to_string()),
            version: "1.0.0".to_string(),
            author: "apex".to_string(),
            rating: 4.2,
            install_count: 89,
            category: "web".to_string(),
            tags: vec!["web".to_string(), "scrape".to_string()],
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to scrape"}
                },
                "required": ["url"]
            }),
        },
        MarketplaceTool {
            id: "mkt-3".to_string(),
            name: "code-formatter".to_string(),
            description: Some("Format code using language-specific formatters".to_string()),
            version: "1.0.0".to_string(),
            author: "apex".to_string(),
            rating: 4.8,
            install_count: 234,
            category: "development".to_string(),
            tags: vec!["code".to_string(), "format".to_string()],
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {"type": "string", "description": "Code to format"},
                    "language": {"type": "string", "description": "Programming language"}
                },
                "required": ["code", "language"]
            }),
        },
        MarketplaceTool {
            id: "mkt-4".to_string(),
            name: "data-transformer".to_string(),
            description: Some("Transform data between formats".to_string()),
            version: "1.0.0".to_string(),
            author: "apex".to_string(),
            rating: 4.0,
            install_count: 67,
            category: "data".to_string(),
            tags: vec!["data".to_string(), "transform".to_string()],
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string", "description": "Input data"},
                    "format": {"type": "string", "description": "Target format"}
                },
                "required": ["input", "format"]
            }),
        },
        MarketplaceTool {
            id: "mkt-5".to_string(),
            name: "notification-sender".to_string(),
            description: Some("Send notifications via various channels".to_string()),
            version: "1.0.0".to_string(),
            author: "apex".to_string(),
            rating: 4.3,
            install_count: 112,
            category: "communication".to_string(),
            tags: vec!["notification".to_string(), "send".to_string()],
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string", "description": "Notification message"},
                    "channel": {"type": "string", "description": "Notification channel"}
                },
                "required": ["message", "channel"]
            }),
        },
    ];

    Json(sample_tools)
}
