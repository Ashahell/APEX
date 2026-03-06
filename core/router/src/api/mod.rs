#![allow(unused_imports)]

pub mod tasks;
pub mod skills;
pub mod workflows;
pub mod notifications;
pub mod webhooks;
pub mod adapters;
pub mod memory;
pub mod system;

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use ulid::Ulid;
use lazy_static::lazy_static;

use crate::circuit_breaker::CircuitBreakerRegistry;
use crate::governance::GovernanceEngine;
use crate::message_bus::{DeepTaskMessage, MessageBus};
use crate::metrics::RouterMetrics;
use crate::moltbook::MoltbookClient;
use crate::vm_pool::VmPool;
use crate::skill_pool::SkillPool;
use crate::execution_stream::ExecutionStreamManager;
use crate::websocket::WebSocketManager;
use crate::system_health::SystemMonitor;
use crate::response_cache::ResponseCache;
use crate::rate_limiter::RateLimiter;
use apex_memory::{Workflow, WorkflowExecution};
use apex_memory::task_repo::TaskRepository;
use apex_memory::tasks::{CreateTask, TaskTier};
use apex_memory::{embedder::Embedder, background_indexer::BackgroundIndexer, narrative::NarrativeMemory};
use crate::unified_config::AppConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskRequest {
    pub content: String,
    pub channel: Option<String>,
    pub thread_id: Option<String>,
    pub author: Option<String>,
    pub attachments: Option<Vec<Attachment>>,
    pub max_steps: Option<u32>,
    pub budget_usd: Option<f64>,
    pub time_limit_secs: Option<u64>,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
    pub filename: String,
    pub mime_type: String,
    pub url: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskResponse {
    pub task_id: String,
    pub status: String,
    pub tier: String,
    pub capability_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instant_response: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskStatusResponse {
    pub task_id: String,
    pub status: String,
    pub content: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskFilterRequest {
    pub project: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateTaskRequest {
    pub project: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterSkillRequest {
    pub name: String,
    pub version: String,
    pub tier: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateHealthRequest {
    pub health_status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillResponse {
    pub name: String,
    pub version: String,
    pub tier: String,
    pub enabled: bool,
    pub health_status: String,
    pub last_health_check: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecuteSkillRequest {
    pub skill_name: String,
    pub input: serde_json::Value,
    pub task_id: String,
    pub tier: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecuteSkillResponse {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecuteDeepTaskRequest {
    pub content: String,
    pub max_steps: Option<u32>,
    pub budget_usd: Option<f64>,
    pub time_limit_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecuteDeepTaskResponse {
    pub task_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,  // C4 Step 2: Config as first-class field
    pub pool: sqlx::SqlitePool,
    pub metrics: RouterMetrics,
    pub message_bus: MessageBus,
    pub circuit_breakers: CircuitBreakerRegistry,
    pub vm_pool: Option<VmPool>,
    pub skill_pool: Option<Arc<SkillPool>>,
    pub execution_streams: ExecutionStreamManager,
    pub ws_manager: WebSocketManager,
    pub moltbook: Option<MoltbookClient>,
    pub governance: Arc<std::sync::Mutex<GovernanceEngine>>,
    pub system_monitor: SystemMonitor,
    pub cache: ResponseCache,
    pub rate_limiter: RateLimiter,
    pub workflow_repo: apex_memory::WorkflowRepository,
    pub webhook_manager: crate::webhook::WebhookManager,
    pub notification_manager: crate::notification::NotificationManager,
    pub embedder: std::sync::Arc<Embedder>,
    pub background_indexer: std::sync::Arc<BackgroundIndexer>,
    pub narrative_memory: std::sync::Arc<NarrativeMemory>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub definition: String,
    pub category: Option<String>,
    pub version: i32,
    pub is_active: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub last_executed_at_ms: Option<i64>,
    pub execution_count: i32,
    pub avg_duration_secs: Option<f64>,
    pub success_rate: Option<f64>,
}

impl From<Workflow> for WorkflowResponse {
    fn from(w: Workflow) -> Self {
        Self {
            id: w.id,
            name: w.name,
            description: w.description,
            definition: w.definition,
            category: w.category,
            version: w.version,
            is_active: w.is_active == 1,
            created_at_ms: w.created_at_ms,
            updated_at_ms: w.updated_at_ms,
            last_executed_at_ms: w.last_executed_at_ms,
            execution_count: w.execution_count,
            avg_duration_secs: w.avg_duration_secs,
            success_rate: w.success_rate,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    pub definition: String,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub definition: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowExecutionResponse {
    pub id: String,
    pub workflow_id: String,
    pub status: String,
    pub started_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub duration_secs: Option<f64>,
    pub input_data: Option<String>,
    pub output_data: Option<String>,
    pub error_message: Option<String>,
    pub triggered_by: Option<String>,
}

impl From<WorkflowExecution> for WorkflowExecutionResponse {
    fn from(e: WorkflowExecution) -> Self {
        Self {
            id: e.id,
            workflow_id: e.workflow_id,
            status: e.status,
            started_at_ms: e.started_at_ms,
            completed_at_ms: e.completed_at_ms,
            duration_secs: e.duration_secs,
            input_data: e.input_data,
            output_data: e.output_data,
            error_message: e.error_message,
            triggered_by: e.triggered_by,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListWorkflowsQuery {
    pub category: Option<String>,
    pub active_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdapterConfig {
    pub name: String,
    pub adapter_type: String,
    pub enabled: bool,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAdapterRequest {
    pub enabled: Option<bool>,
    pub config: Option<serde_json::Value>,
}

lazy_static! {
    pub static ref ADAPTERS: std::sync::RwLock<HashMap<String, AdapterConfig>> = {
        let mut map = HashMap::new();
        map.insert("slack".to_string(), AdapterConfig {
            name: "slack".to_string(),
            adapter_type: "slack".to_string(),
            enabled: false,
            config: serde_json::json!({
                "bot_token": "",
                "signing_secret": "",
                "default_channel": "#apex"
            }),
        });
        map.insert("telegram".to_string(), AdapterConfig {
            name: "telegram".to_string(),
            adapter_type: "telegram".to_string(),
            enabled: false,
            config: serde_json::json!({
                "bot_token": "",
                "allowed_users": []
            }),
        });
        map.insert("discord".to_string(), AdapterConfig {
            name: "discord".to_string(),
            adapter_type: "discord".to_string(),
            enabled: false,
            config: serde_json::json!({
                "bot_token": "",
                "guild_id": "",
                "channel_id": ""
            }),
        });
        map.insert("email".to_string(), AdapterConfig {
            name: "email".to_string(),
            adapter_type: "email".to_string(),
            enabled: false,
            config: serde_json::json!({
                "smtp_host": "",
                "smtp_port": 587,
                "smtp_user": "",
                "smtp_pass": "",
                "from_address": ""
            }),
        });
        map.insert("whatsapp".to_string(), AdapterConfig {
            name: "whatsapp".to_string(),
            adapter_type: "whatsapp".to_string(),
            enabled: false,
            config: serde_json::json!({
                "phone_number_id": "",
                "access_token": "",
                "verify_token": ""
            }),
        });
        std::sync::RwLock::new(map)
    };
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebhookResponse {
    pub id: String,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub enabled: bool,
    pub created_at_ms: i64,
    pub last_triggered_ms: Option<i64>,
    pub failure_count: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateWebhookRequest {
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationResponse {
    pub id: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub severity: String,
    pub read: bool,
    pub created_at_ms: i64,
    pub data: Option<serde_json::Value>,
}

impl From<crate::notification::Notification> for NotificationResponse {
    fn from(n: crate::notification::Notification) -> Self {
        Self {
            id: n.id,
            notification_type: n.notification_type,
            title: n.title,
            message: n.message,
            severity: n.severity,
            read: n.read,
            created_at_ms: n.created_at_ms,
            data: n.data,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListNotificationsQuery {
    pub include_read: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileItem {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    modified: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileContent {
    path: String,
    content: String,
    encoding: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListFilesQuery {
    path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetFileContentQuery {
    path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryStatsResponse {
    pub total_entities: u32,
    pub total_knowledge: u32,
    pub total_reflections: u32,
    pub recent_reflections: Vec<ReflectionItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReflectionItem {
    pub id: u32,
    pub content: String,
    pub importance: u32,
    pub created_at: String,
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(system::router())
        .merge(tasks::router())
        .merge(skills::router())
        .merge(workflows::router())
        .merge(notifications::router())
        .merge(webhooks::router())
        .merge(adapters::router())
        .merge(memory::router())
        .route("/", axum::routing::get(root))
        .route("/health", axum::routing::get(health))
        .route("/api/v1/deep", post(execute_deep_task))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn root() -> &'static str {
    "APEX Router v1.3.0"
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn execute_deep_task(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteDeepTaskRequest>,
) -> Result<Json<ExecuteDeepTaskResponse>, String> {
    let task_id = Ulid::new().to_string();
    let max_steps = payload.max_steps.unwrap_or(10);
    let budget_usd = payload.budget_usd.unwrap_or(1.0);

    let repo = TaskRepository::new(&state.pool);
    let create_input = CreateTask {
        input_content: payload.content.clone(),
        channel: None,
        thread_id: None,
        author: Some("system".to_string()),
        skill_name: None,
        project: None,
        priority: None,
        category: None,
    };

    repo.create(&task_id, create_input, TaskTier::Deep)
        .await
        .map_err(|e| format!("Failed to create task: {}", e))?;

    let time_limit_secs = payload.time_limit_secs;

    state.message_bus.publish_deep_task(DeepTaskMessage {
        task_id: task_id.clone(),
        content: payload.content,
        max_steps,
        budget_usd,
        time_limit_secs,
        permission_tier: "T2".to_string(),
    });

    Ok(Json(ExecuteDeepTaskResponse {
        task_id,
        status: "running".to_string(),
        message: "Deep task queued for execution".to_string(),
    }))
}
