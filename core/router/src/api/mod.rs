#![allow(unused_imports)]

pub mod adapters;
pub mod audit;
pub mod bounded_memory; // NEW - Hermes-style bounded memory
pub mod channels;
pub mod channels_extended; // NEW - Phase 6 Additional Channels
pub mod context_scope_api;
pub mod continuity_api; // NEW - Continuity Scheduler (Feature 4)
pub mod dashboard; // NEW
pub mod dynamic_tools;
pub mod execution_patterns; // NEW - Phase 9 Death Spiral Detection
pub mod heartbeat;
pub mod hub_api; // NEW - Hermes-style skills hub
pub mod journal;
pub mod llms;
pub mod mcp;
pub mod memory;
pub mod memory_ttl_api; // NEW - Phase 5 Memory TTL and Consolidation
pub mod moltbook;
pub mod notifications;
pub mod pdf; // NEW
pub mod persona_api; // NEW - Persona Assembly (Feature 2)
pub mod privacy_api; // NEW - Privacy Toggle (Feature 6)
pub mod secrets; // NEW - Phase 7 Secrets Expansion
pub mod security;
pub mod session_search_api; // NEW - Hermes-style session search
pub mod sessions; // NEW
pub mod settings;
pub mod signing_api; // NEW - Plugin Signing (Feature 5)
pub mod skill_manager_api; // NEW - Hermes-style auto-created skills
pub mod skills;
pub mod slack_blocks; // NEW - Phase 8 Slack Block Kit
pub mod soul;
pub mod story_api; // NEW - Story Engine (Feature 7)
pub mod subagent;
pub mod system;
pub mod tasks;
pub mod tool_validation; // NEW - Tool Maker Validation (Feature 1)
pub mod totp;
pub mod user_profile_api; // NEW - Hermes-style user profile
pub mod webhooks;
pub mod workflows; // NEW - Context Scope (Feature 3)

/// Helper module for API error handling
pub mod api_error {
    use axum::http::StatusCode;
    use std::fmt;

    /// API Error type for consistent error responses
    pub struct ApiError {
        pub status: StatusCode,
        pub message: String,
    }

    impl ApiError {
        /// Create a new API error with INTERNAL_SERVER_ERROR
        pub fn internal(message: impl fmt::Display) -> (StatusCode, String) {
            (StatusCode::INTERNAL_SERVER_ERROR, message.to_string())
        }

        /// Create a new API error with NOT_FOUND
        pub fn not_found(message: impl fmt::Display) -> (StatusCode, String) {
            (StatusCode::NOT_FOUND, message.to_string())
        }

        /// Create a new API error with BAD_REQUEST
        pub fn bad_request(message: impl fmt::Display) -> (StatusCode, String) {
            (StatusCode::BAD_REQUEST, message.to_string())
        }

        /// Create a new API error with UNAUTHORIZED
        pub fn unauthorized(message: impl fmt::Display) -> (StatusCode, String) {
            (StatusCode::UNAUTHORIZED, message.to_string())
        }

        /// Create a new API error with FORBIDDEN
        pub fn forbidden(message: impl fmt::Display) -> (StatusCode, String) {
            (StatusCode::FORBIDDEN, message.to_string())
        }

        /// Convert a Result to ApiError
        pub fn from_result<T, E: fmt::Display>(
            result: Result<T, E>,
            context: &str,
        ) -> Result<T, (StatusCode, String)> {
            result.map_err(|e| Self::internal(format!("{}: {}", context, e)))
        }
    }

    /// Macro to simplify error handling in API handlers
    /// Usage: `api_try!(repo.operation(), "description of operation")`
    #[macro_export]
    macro_rules! api_try {
        ($expr:expr, $context:literal) => {
            $expr.map_err(|e| {
                use $crate::api::api_error::ApiError;
                ApiError::internal(format!("{}: {}", $context, e))
            })
        };
    }
}

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use ulid::Ulid;

use crate::circuit_breaker::CircuitBreakerRegistry;
use crate::dynamic_tools::ToolRegistry;
use crate::execution_stream::ExecutionStreamManager;
use crate::governance::GovernanceEngine;
use crate::heartbeat::HeartbeatScheduler;
use crate::hub_client::HubClient;
use crate::mcp::McpServerManager;
use crate::message_bus::{DeepTaskMessage, MessageBus};
use crate::metrics::RouterMetrics;
use crate::moltbook::MoltbookClient;
use crate::rate_limiter::RateLimiter;
use crate::response_cache::ResponseCache;
use crate::session_search::SessionSearch;
use crate::skill_manager::SkillManager;
use crate::skill_pool::SkillPool;
use crate::soul::loader::SoulLoader;
use crate::subagent::SubAgentPool;
use crate::system_health::SystemMonitor;
use crate::telemetry_middleware::TelemetryLayer;
use crate::totp::TotpManager;
use crate::unified_config::AppConfig;
use crate::user_profile::UserProfileManager;
use crate::vm_pool::VmPool;
use crate::websocket::WebSocketManager;
use apex_memory::background_indexer::BackgroundIndexer;
use apex_memory::Embedder;
use apex_memory::NarrativeMemory;
use apex_memory::{CreateTask, TaskRepository, TaskTier, Workflow, WorkflowExecution};

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
    pub use_tir: Option<bool>,
    pub enable_subagents: Option<bool>,
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
    pub use_tir: Option<bool>,
    pub enable_subagents: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecuteDeepTaskResponse {
    pub task_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig, // C4 Step 2: Config as first-class field
    pub pool: sqlx::SqlitePool,
    pub metrics: RouterMetrics,
    pub message_bus: MessageBus,
    pub circuit_breakers: CircuitBreakerRegistry,
    pub vm_pool: Option<VmPool>,
    pub skill_pool: Option<Arc<SkillPool>>,
    pub subagent_pool: Arc<tokio::sync::RwLock<SubAgentPool>>,
    pub dynamic_tools: Arc<tokio::sync::RwLock<ToolRegistry>>,
    pub execution_streams: ExecutionStreamManager,
    pub ws_manager: WebSocketManager,
    pub moltbook: Option<MoltbookClient>,
    pub governance: Arc<std::sync::Mutex<GovernanceEngine>>,
    pub system_monitor: SystemMonitor,
    pub cache: ResponseCache,
    pub rate_limiter: RateLimiter,
    pub workflow_repo: apex_memory::WorkflowRepository,
    pub preferences_repo: apex_memory::PreferencesRepository,
    pub config_repo: apex_memory::ConfigRepository,
    pub audit_repo: apex_memory::AuditRepository,
    pub webhook_manager: crate::webhook::WebhookManager,
    pub notification_manager: crate::notification::NotificationManager,
    pub embedder: std::sync::Arc<Embedder>,
    pub background_indexer: std::sync::Arc<BackgroundIndexer>,
    pub narrative_memory: std::sync::Arc<NarrativeMemory>,
    pub bounded_memory: bounded_memory::BoundedMemoryState,
    pub skill_manager: std::sync::Arc<tokio::sync::Mutex<SkillManager>>,
    pub user_profile: std::sync::Arc<UserProfileManager>,
    pub session_search: std::sync::Arc<SessionSearch>,
    pub hub_client: std::sync::Arc<HubClient>,
    pub totp_manager: TotpManager,
    pub soul_loader: SoulLoader,
    pub heartbeat_scheduler: HeartbeatScheduler,
    pub mcp_manager: std::sync::Arc<McpServerManager>,
    pub anomaly_detector: Option<std::sync::Arc<crate::security::AnomalyDetector>>,
    // Feature 5: Plugin Signing
    pub signature_store: std::sync::Arc<std::sync::Mutex<crate::skill_signer::SignatureStore>>,
    // Feature 7: Story Engine
    pub story_engine: std::sync::Arc<std::sync::Mutex<crate::story_engine::StoryEngine>>,
    // Feature 4: Continuity Scheduler
    pub continuity_state:
        std::sync::Arc<std::sync::Mutex<crate::api::continuity_api::ContinuityState>>,
    // Feature 6: Privacy Toggle
    pub privacy_guard: std::sync::Arc<std::sync::Mutex<crate::privacy_guard::PrivacyGuard>>,
    // Feature 3: Context Scope
    pub context_scope_state:
        std::sync::Arc<std::sync::Mutex<crate::api::context_scope_api::ContextScopeState>>,
    // Patch 15: Replay protection backend (trait-injected, configurable)
    pub replay_protection: std::sync::Arc<dyn crate::security::replay_protection::ReplayProtection>,
    // Patch 16: Streaming analytics
    pub streaming_metrics: std::sync::Arc<crate::streaming::StreamingMetrics>,
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
        map.insert(
            "slack".to_string(),
            AdapterConfig {
                name: "slack".to_string(),
                adapter_type: "slack".to_string(),
                enabled: false,
                config: serde_json::json!({
                    "bot_token": "",
                    "signing_secret": "",
                    "default_channel": "#apex"
                }),
            },
        );
        map.insert(
            "telegram".to_string(),
            AdapterConfig {
                name: "telegram".to_string(),
                adapter_type: "telegram".to_string(),
                enabled: false,
                config: serde_json::json!({
                    "bot_token": "",
                    "allowed_users": []
                }),
            },
        );
        map.insert(
            "discord".to_string(),
            AdapterConfig {
                name: "discord".to_string(),
                adapter_type: "discord".to_string(),
                enabled: false,
                config: serde_json::json!({
                    "bot_token": "",
                    "guild_id": "",
                    "channel_id": ""
                }),
            },
        );
        map.insert(
            "email".to_string(),
            AdapterConfig {
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
            },
        );
        map.insert(
            "whatsapp".to_string(),
            AdapterConfig {
                name: "whatsapp".to_string(),
                adapter_type: "whatsapp".to_string(),
                enabled: false,
                config: serde_json::json!({
                    "phone_number_id": "",
                    "access_token": "",
                    "verify_token": ""
                }),
            },
        );
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
        .merge(settings::router())
        .merge(audit::router())
        .merge(channels::create_router())
        .merge(journal::create_router())
        .merge(totp::create_router())
        .merge(soul::create_router())
        .merge(heartbeat::create_router())
        .merge(moltbook::create_router())
        .merge(llms::create_router())
        .merge(mcp::create_router())
        .merge(subagent::router())
        .merge(dynamic_tools::router())
        .merge(security::create_router())
        .merge(dashboard::router()) // NEW
        .merge(sessions::router()) // NEW - sessions_yield & sessions_resume
        .merge(pdf::router()) // NEW - PDF tool
        .merge(channels_extended::router()) // NEW - Additional Channels (Phase 6)
        .merge(secrets::router()) // NEW - Secrets Expansion (Phase 7)
        .merge(slack_blocks::router()) // NEW - Slack Block Kit (Phase 8)
        .merge(execution_patterns::router()) // NEW - Death Spiral Detection (Phase 9)
        .merge(bounded_memory::router()) // NEW - Hermes-style Bounded Memory
        .merge(skill_manager_api::router()) // NEW - Hermes-style Auto-Created Skills
        .merge(user_profile_api::router()) // NEW - Hermes-style User Profile
        .merge(session_search_api::router()) // NEW - Hermes-style Session Search
        .merge(memory_ttl_api::router()) // NEW - Phase 5 Memory TTL and Consolidation
        .merge(hub_api::router()) // NEW - Hermes-style Skills Hub
        .merge(tool_validation::create_tool_validation_router()) // NEW - Tool Validation (Feature 1)
        .merge(persona_api::create_persona_router()) // NEW - Persona Assembly (Feature 2)
        .merge(signing_api::create_signing_router()) // NEW - Plugin Signing (Feature 5)
        .merge(story_api::create_story_router()) // NEW - Story Engine (Feature 7)
        .merge(continuity_api::create_continuity_router()) // NEW - Continuity (Feature 4)
        .merge(privacy_api::create_privacy_router()) // NEW - Privacy (Feature 6)
        .merge(context_scope_api::create_context_scope_router()) // NEW - Context Scope (Feature 3)
        .merge(crate::streaming::create_streaming_router(state.clone())) // NEW - Patch 11: SSE streaming for Hands and MCP
        .merge(crate::streaming_sign::create_stream_sign_router()) // NEW - Signed URL for streaming
        .route("/", axum::routing::get(root))
        .route("/health", axum::routing::get(health))
        .route("/api/v1/deep", post(execute_deep_task))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(TelemetryLayer::new(state.metrics.clone()))
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
        use_tir: payload.use_tir.unwrap_or(false),
        enable_subagents: payload.enable_subagents.unwrap_or(true),
    });

    Ok(Json(ExecuteDeepTaskResponse {
        task_id,
        status: "running".to_string(),
        message: "Deep task queued for execution".to_string(),
    }))
}
