use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use ulid::Ulid;
use lazy_static::lazy_static;

use crate::apex_security::capability::{CapabilityToken, PermissionTier};
use crate::circuit_breaker::CircuitBreakerRegistry;
use crate::classifier::TaskClassifier;
use crate::governance::GovernanceEngine;
use crate::message_bus::DeepTaskMessage;
use crate::message_bus::MessageBus;
use crate::metrics::RouterMetrics;
use crate::moltbook::MoltbookClient;
use crate::vm_pool::VmPool;
use crate::execution_stream::ExecutionStreamManager;
use crate::websocket::WebSocketManager;
use crate::system_health::SystemMonitor;
use crate::response_cache::ResponseCache;
use crate::rate_limiter::RateLimiter;
use apex_memory::{Workflow, WorkflowExecution, CreateWorkflow, UpdateWorkflow};
use apex_memory::msg_repo::MessageRepository;
use apex_memory::skill_registry::{SkillRegistry, SkillRegistryEntry};
use apex_memory::task_repo::TaskRepository;
use apex_memory::tasks::{CreateTask, TaskStatus, TaskTier};

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub mime_type: String,
    pub url: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: String,
    pub status: String,
    pub tier: String,
    pub capability_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instant_response: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatusResponse {
    pub task_id: String,
    pub status: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskFilterRequest {
    pub project: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub project: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterSkillRequest {
    pub name: String,
    pub version: String,
    pub tier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateHealthRequest {
    pub health_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillResponse {
    pub name: String,
    pub version: String,
    pub tier: String,
    pub enabled: bool,
    pub health_status: String,
    pub last_health_check: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteSkillRequest {
    pub skill_name: String,
    pub input: serde_json::Value,
    pub task_id: String,
    pub tier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteSkillResponse {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteDeepTaskRequest {
    pub content: String,
    pub max_steps: Option<u32>,
    pub budget_usd: Option<f64>,
    pub time_limit_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteDeepTaskResponse {
    pub task_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub metrics: RouterMetrics,
    pub message_bus: MessageBus,
    pub circuit_breakers: CircuitBreakerRegistry,
    pub vm_pool: Option<VmPool>,
    pub execution_streams: ExecutionStreamManager,
    pub ws_manager: WebSocketManager,
    pub moltbook: Option<MoltbookClient>,
    pub governance: std::sync::Arc<std::sync::Mutex<GovernanceEngine>>,
    pub system_monitor: SystemMonitor,
    pub cache: ResponseCache,
    pub rate_limiter: RateLimiter,
    pub workflow_repo: apex_memory::WorkflowRepository,
    pub webhook_manager: crate::webhook::WebhookManager,
    pub notification_manager: crate::notification::NotificationManager,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    pub definition: String,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub definition: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct ListWorkflowsQuery {
    pub category: Option<String>,
    pub active_only: Option<bool>,
}

async fn list_workflows(
    State(state): State<AppState>,
    Query(query): Query<ListWorkflowsQuery>,
) -> Result<Json<Vec<WorkflowResponse>>, String> {
    let repo = &state.workflow_repo;
    let workflows = if query.active_only.unwrap_or(false) {
        repo.find_active().await.map_err(|e| e.to_string())?
    } else if let Some(ref cat) = query.category {
        repo.find_by_category(cat).await.map_err(|e| e.to_string())?
    } else {
        repo.find_all().await.map_err(|e| e.to_string())?
    };
    Ok(Json(workflows.into_iter().map(WorkflowResponse::from).collect()))
}

async fn get_workflow_filter_options(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = &state.workflow_repo;
    let categories = repo.get_categories().await.map_err(|e| e.to_string())?;
    Ok(Json(serde_json::json!({ "categories": categories })))
}

async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WorkflowResponse>, String> {
    let repo = &state.workflow_repo;
    let workflow = repo.find_by_id(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Workflow not found".to_string())?;
    Ok(Json(WorkflowResponse::from(workflow)))
}

async fn create_workflow(
    State(state): State<AppState>,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<Json<WorkflowResponse>, String> {
    let repo = &state.workflow_repo;
    let id = Ulid::new().to_string();
    let create = CreateWorkflow {
        name: req.name,
        description: req.description,
        definition: req.definition,
        category: req.category,
    };
    repo.create(&id, &create).await.map_err(|e| e.to_string())?;
    let workflow = repo.find_by_id(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Workflow not found after creation".to_string())?;
    Ok(Json(WorkflowResponse::from(workflow)))
}

async fn update_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateWorkflowRequest>,
) -> Result<Json<WorkflowResponse>, String> {
    let repo = &state.workflow_repo;
    let update = UpdateWorkflow {
        name: req.name,
        description: req.description,
        definition: req.definition,
        category: req.category,
        is_active: req.is_active,
    };
    repo.update(&id, &update).await.map_err(|e| e.to_string())?;
    let workflow = repo.find_by_id(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Workflow not found after update".to_string())?;
    Ok(Json(WorkflowResponse::from(workflow)))
}

async fn delete_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = &state.workflow_repo;
    repo.delete(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(serde_json::json!({ "success": true, "deleted": id })))
}

async fn get_workflow_executions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<WorkflowExecutionResponse>>, String> {
    let repo = &state.workflow_repo;
    let executions = repo.get_executions(&id, 50).await.map_err(|e| e.to_string())?;
    Ok(Json(executions.into_iter().map(WorkflowExecutionResponse::from).collect()))
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/v1/tasks", get(list_tasks))
        .route("/api/v1/tasks", post(create_task))
        .route("/api/v1/tasks/filter-options", get(get_filter_options))
        .route("/api/v1/tasks/:id", get(get_task))
        .route("/api/v1/tasks/:id", put(update_task))
        .route("/api/v1/tasks/:id/cancel", post(cancel_task))
        .route("/api/v1/messages", get(list_messages))
        .route("/api/v1/messages/task/:task_id", get(get_task_messages))
        .route("/api/v1/metrics", get(get_metrics))
        .route("/api/v1/skills", get(list_skills))
        .route("/api/v1/skills", post(register_skill))
        .route("/api/v1/skills/:name", get(get_skill))
        .route("/api/v1/skills/:name", delete(delete_skill))
        .route("/api/v1/skills/:name/health", put(update_skill_health))
        .route("/api/v1/skills/execute", post(execute_skill))
        .route("/api/v1/deep", post(execute_deep_task))
        .route("/api/v1/vm/stats", get(get_vm_stats))
        .route("/api/v1/workflows", get(list_workflows))
        .route("/api/v1/workflows", post(create_workflow))
        .route("/api/v1/workflows/filter-options", get(get_workflow_filter_options))
        .route("/api/v1/workflows/:id", get(get_workflow))
        .route("/api/v1/workflows/:id", put(update_workflow))
        .route("/api/v1/workflows/:id", delete(delete_workflow))
        .route("/api/v1/workflows/:id/executions", get(get_workflow_executions))
        .route("/api/v1/adapters", get(list_adapters))
        .route("/api/v1/adapters/:name", get(get_adapter))
        .route("/api/v1/adapters/:name", put(update_adapter))
        .route("/api/v1/adapters/:name/toggle", post(toggle_adapter))
        .route("/api/v1/webhooks", get(list_webhooks))
        .route("/api/v1/webhooks", post(create_webhook))
        .route("/api/v1/webhooks/:id", get(get_webhook))
        .route("/api/v1/webhooks/:id", delete(delete_webhook))
        .route("/api/v1/webhooks/:id/toggle", post(toggle_webhook))
        .route("/api/v1/notifications", get(list_notifications))
        .route("/api/v1/notifications/unread-count", get(get_unread_count))
        .route("/api/v1/notifications/:id", get(get_notification))
        .route("/api/v1/notifications/:id/read", post(mark_notification_read))
        .route("/api/v1/notifications/read-all", post(mark_all_read))
        .route("/api/v1/notifications/:id", delete(delete_notification))
        .route("/api/v1/notifications", delete(clear_notifications))
        .route("/api/v1/files", get(list_files))
        .route("/api/v1/files/content", get(get_file_content))
        .route("/api/v1/memory/stats", get(get_memory_stats))
        .route("/api/v1/memory/reflections", get(get_reflections))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub name: String,
    pub adapter_type: String,
    pub enabled: bool,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAdapterRequest {
    pub enabled: Option<bool>,
    pub config: Option<serde_json::Value>,
}

lazy_static! {
    static ref ADAPTERS: std::sync::RwLock<HashMap<String, AdapterConfig>> = {
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

async fn list_adapters() -> Result<Json<Vec<AdapterConfig>>, String> {
    let adapters = ADAPTERS.read().map_err(|e| e.to_string())?;
    let list: Vec<AdapterConfig> = adapters.values().cloned().collect();
    Ok(Json(list))
}

async fn get_adapter(
    Path(name): Path<String>,
) -> Result<Json<AdapterConfig>, String> {
    let adapters = ADAPTERS.read().map_err(|e| e.to_string())?;
    let adapter = adapters.get(&name)
        .ok_or_else(|| "Adapter not found".to_string())?
        .clone();
    Ok(Json(adapter))
}

async fn update_adapter(
    Path(name): Path<String>,
    Json(req): Json<UpdateAdapterRequest>,
) -> Result<Json<AdapterConfig>, String> {
    let mut adapters = ADAPTERS.write().map_err(|e| e.to_string())?;
    let adapter = adapters.get_mut(&name)
        .ok_or_else(|| "Adapter not found".to_string())?;
    
    if let Some(enabled) = req.enabled {
        adapter.enabled = enabled;
    }
    if let Some(config) = req.config {
        adapter.config = config;
    }
    
    Ok(Json(adapter.clone()))
}

async fn toggle_adapter(
    Path(name): Path<String>,
) -> Result<Json<AdapterConfig>, String> {
    let mut adapters = ADAPTERS.write().map_err(|e| e.to_string())?;
    let adapter = adapters.get_mut(&name)
        .ok_or_else(|| "Adapter not found".to_string())?;
    
    adapter.enabled = !adapter.enabled;
    Ok(Json(adapter.clone()))
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
}

async fn list_webhooks(
    State(state): State<AppState>,
) -> Result<Json<Vec<WebhookResponse>>, String> {
    let webhooks = state.webhook_manager.list_webhooks().await;
    let responses: Vec<WebhookResponse> = webhooks.into_iter().map(|w| WebhookResponse {
        id: w.id,
        name: w.name,
        url: w.url,
        events: w.events,
        enabled: w.enabled,
        created_at_ms: w.created_at_ms,
        last_triggered_ms: w.last_triggered_ms,
        failure_count: w.failure_count,
    }).collect();
    Ok(Json(responses))
}

async fn get_webhook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WebhookResponse>, String> {
    let webhook = state.webhook_manager.get_webhook(&id)
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
    let create = crate::webhook::CreateWebhook {
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub include_read: Option<bool>,
}

async fn list_notifications(
    State(state): State<AppState>,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<Vec<NotificationResponse>>, String> {
    let notifications = state.notification_manager.list(query.include_read.unwrap_or(false)).await;
    Ok(Json(notifications.into_iter().map(NotificationResponse::from).collect()))
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
    let notification = state.notification_manager.get(&id)
        .await
        .ok_or_else(|| "Notification not found".to_string())?;
    Ok(Json(NotificationResponse::from(notification)))
}

async fn mark_notification_read(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<NotificationResponse>, String> {
    let notification = state.notification_manager.mark_read(&id)
        .await
        .ok_or_else(|| "Notification not found".to_string())?;
    Ok(Json(NotificationResponse::from(notification)))
}

async fn mark_all_read(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, String> {
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
    Ok(Json(serde_json::json!({ "success": true, "cleared": true })))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileItem {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    modified: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContent {
    path: String,
    content: String,
    encoding: String,
}

#[derive(Debug, Deserialize)]
pub struct ListFilesQuery {
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetFileContentQuery {
    path: String,
}

async fn list_files(
    Query(query): Query<ListFilesQuery>,
) -> Result<Json<Vec<FileItem>>, String> {
    let path = query.path.as_deref().unwrap_or("/");
    
    let entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;
    
    let mut files: Vec<FileItem> = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            let metadata = entry.metadata().ok();
            let name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path().to_string_lossy().to_string();
            let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified = metadata
                .and_then(|m| m.modified().ok())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64)
                .unwrap_or(0);
            
            files.push(FileItem {
                name,
                path: file_path,
                is_dir,
                size,
                modified,
            });
        }
    }
    
    files.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    
    Ok(Json(files))
}

async fn get_file_content(
    Query(query): Query<GetFileContentQuery>,
) -> Result<Json<FileContent>, String> {
    let path = &query.path;
    
    if !std::path::Path::new(path).exists() {
        return Err("File not found".to_string());
    }
    
    let content = std::fs::read_to_string(path).unwrap_or_else(|_| "// Binary file or unreadable content".to_string());
    
    Ok(Json(FileContent {
        path: path.clone(),
        content,
        encoding: "utf-8".to_string(),
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryStatsResponse {
    pub total_entities: u32,
    pub total_knowledge: u32,
    pub total_reflections: u32,
    pub recent_reflections: Vec<ReflectionItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReflectionItem {
    pub id: u32,
    pub content: String,
    pub importance: u32,
    pub created_at: String,
}

async fn get_memory_stats() -> Result<Json<MemoryStatsResponse>, String> {
    let base_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".apex")
        .join("memory");

    let reflections_dir = base_path.join("reflections");
    let entities_dir = base_path.join("entities");
    let knowledge_dir = base_path.join("knowledge");

    let total_reflections = count_files_recursive(&reflections_dir).await.unwrap_or(0);
    let total_entities = count_files_recursive(&entities_dir).await.unwrap_or(0);
    let total_knowledge = count_files_recursive(&knowledge_dir).await.unwrap_or(0);

    let mut recent_reflections = Vec::new();
    if reflections_dir.exists() {
        if let Ok(entries) = tokio::fs::read_dir(&reflections_dir).await {
            let mut count = 0u32;
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        let importance = (count % 10) as u32 + 1;
                        recent_reflections.push(ReflectionItem {
                            id: count + 1,
                            content: entry.file_name().to_string_lossy().to_string(),
                            importance,
                            created_at: format!("2026-03-{:02}T10:00:00Z", (count % 28) + 1),
                        });
                        count += 1;
                        if count >= 5 {
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(Json(MemoryStatsResponse {
        total_entities,
        total_knowledge,
        total_reflections,
        recent_reflections,
    }))
}

async fn count_files_recursive(dir: &std::path::Path) -> std::io::Result<u32> {
    use tokio::fs;
    
    let mut count = 0u32;
    
    if !dir.exists() {
        return Ok(0);
    }
    
    let mut stack = vec![dir.to_path_buf()];
    
    while let Some(current_dir) = stack.pop() {
        let mut entries = fs::read_dir(&current_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().map_or(false, |ext| ext == "md") {
                count += 1;
            }
        }
    }
    
    Ok(count)
}

async fn get_reflections() -> Result<Json<Vec<ReflectionItem>>, String> {
    let base_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".apex")
        .join("memory")
        .join("reflections");

    let mut reflections = Vec::new();
    
    if base_path.exists() {
        if let Ok(entries) = tokio::fs::read_dir(&base_path).await {
            let mut count = 0u32;
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        reflections.push(ReflectionItem {
                            id: count + 1,
                            content: entry.file_name().to_string_lossy().to_string(),
                            importance: (count % 10) as u32 + 1,
                            created_at: format!("2026-03-{:02}T10:00:00Z", (count % 28) + 1),
                        });
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(Json(reflections))
}

async fn toggle_webhook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WebhookResponse>, String> {
    let webhook = state.webhook_manager.toggle_webhook(&id)
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

async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<TaskRequest>,
) -> Result<Json<TaskResponse>, String> {
    let task_id = Ulid::new().to_string();
    let tier = TaskClassifier::classify(&payload.content);
    let tier_str = tier.as_str().to_string();

    let permission_tier = match tier {
        TaskTier::Instant => PermissionTier::T0,
        TaskTier::Shallow => PermissionTier::T1,
        TaskTier::Deep => PermissionTier::T2,
    };

    let capability = CapabilityToken::new(
        &task_id,
        permission_tier,
        vec!["*".to_string()],
        vec!["*".to_string()],
        5.0,
        3600,
    );

    let capability_token = capability.encode();

    let content = payload.content.clone();
    let _channel = payload.channel.clone();

    let repo = TaskRepository::new(&state.pool);
    let create_input = CreateTask {
        input_content: content.clone(),
        channel: payload.channel,
        thread_id: payload.thread_id,
        author: payload.author,
        skill_name: None,
        project: payload.project,
        priority: payload.priority,
        category: payload.category,
    };

    match repo.create(&task_id, create_input, tier.clone()).await {
        Ok(_) => {
            // Auto-tier: Route to LLM for all tasks (Deep)
            // This ensures conversational inputs like greetings get LLM responses
            tracing::info!(task_id = %task_id, tier = %tier_str, "Auto-routing to deep task (LLM)");

            let max_steps = payload.max_steps.unwrap_or(3);
            let budget_usd = payload.budget_usd.unwrap_or(1.0);

            state
                .message_bus
                .publish_deep_task(crate::message_bus::DeepTaskMessage {
                    task_id: task_id.clone(),
                    content: content.clone(),
                    max_steps,
                    budget_usd,
                    time_limit_secs: payload.time_limit_secs,
                    permission_tier: tier_str.clone(),
                });

            state.metrics.record_task(&tier_str, "running").await;

            Ok(Json(TaskResponse {
                task_id: task_id.clone(),
                status: "running".to_string(),
                tier: tier_str,
                capability_token,
                instant_response: None,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to create task: {}", e);
            Err(format!("Failed to create task: {}", e))
        }
    }
}

async fn list_tasks(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<TaskFilterRequest>,
) -> Result<Json<Vec<TaskStatusResponse>>, String> {
    let repo = TaskRepository::new(&state.pool);

    let limit = params.limit.unwrap_or(100);
    let offset = params.offset.unwrap_or(0);

    let tasks = repo.find_by_filter(
        params.project.as_deref(),
        params.status.as_deref(),
        params.priority.as_deref(),
        params.category.as_deref(),
        limit,
        offset,
    ).await.map_err(|e| format!("Failed to list tasks: {}", e))?;

    let responses: Vec<TaskStatusResponse> = tasks
        .into_iter()
        .map(|t| TaskStatusResponse {
            task_id: t.id,
            status: t.status,
            output: t.output_content,
            error: t.error_message,
            project: t.project,
            priority: t.priority,
            category: t.category,
            created_at: Some(t.created_at.to_rfc3339()),
        })
        .collect();
    Ok(Json(responses))
}

async fn get_task(
    State(state): State<AppState>,
    axum::extract::Path(task_id): axum::extract::Path<String>,
) -> Result<Json<TaskStatusResponse>, String> {
    let repo = TaskRepository::new(&state.pool);

    match repo.find_by_id(&task_id).await {
        Ok(task) => Ok(Json(TaskStatusResponse {
            task_id: task.id,
            status: task.status,
            output: task.output_content,
            error: task.error_message,
            project: task.project,
            priority: task.priority,
            category: task.category,
            created_at: Some(task.created_at.to_rfc3339()),
        })),
        Err(e) => Err(format!("Task not found: {}", e)),
    }
}

async fn cancel_task(
    State(state): State<AppState>,
    axum::extract::Path(task_id): axum::extract::Path<String>,
) -> Result<Json<TaskStatusResponse>, String> {
    let repo = TaskRepository::new(&state.pool);

    match repo.update_status(&task_id, TaskStatus::Cancelled).await {
        Ok(_) => {
            state.metrics.record_task("unknown", "cancelled").await;
            Ok(Json(TaskStatusResponse {
                task_id,
                status: "cancelled".to_string(),
                output: None,
                error: None,
                project: None,
                priority: None,
                category: None,
                created_at: None,
            }))
        }
        Err(e) => Err(format!("Failed to cancel task: {}", e)),
    }
}

async fn update_task(
    State(state): State<AppState>,
    axum::extract::Path(task_id): axum::extract::Path<String>,
    axum::extract::Json(update_req): axum::extract::Json<UpdateTaskRequest>,
) -> Result<Json<TaskStatusResponse>, String> {
    let repo = TaskRepository::new(&state.pool);

    repo.update_task_fields(
        &task_id,
        update_req.project.as_deref(),
        update_req.priority.as_deref(),
        update_req.category.as_deref(),
    ).await.map_err(|e| format!("Failed to update task: {}", e))?;

    match repo.find_by_id(&task_id).await {
        Ok(task) => Ok(Json(TaskStatusResponse {
            task_id: task.id,
            status: task.status,
            output: task.output_content,
            error: task.error_message,
            project: task.project,
            priority: task.priority,
            category: task.category,
            created_at: Some(task.created_at.to_rfc3339()),
        })),
        Err(e) => Err(format!("Task not found: {}", e)),
    }
}

async fn get_filter_options(State(state): State<AppState>) -> Json<serde_json::Value> {
    let repo = TaskRepository::new(&state.pool);
    
    let projects = repo.get_projects().await.unwrap_or_default();
    let categories = repo.get_categories().await.unwrap_or_default();
    
    Json(serde_json::json!({
        "projects": projects,
        "categories": categories,
        "priorities": ["low", "medium", "high", "urgent"],
        "statuses": ["pending", "running", "completed", "failed", "cancelled"],
    }))
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

async fn list_messages(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<apex_memory::msg_repo::Message>>, String> {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100);
    let offset = params.get("offset").and_then(|o| o.parse().ok()).unwrap_or(0);
    let channel = params.get("channel");

    let repo = MessageRepository::new(&state.pool);

    let messages = if let Some(ch) = channel {
        repo.find_by_channel(ch, limit, offset).await
    } else {
        repo.find_recent(limit).await
    }.map_err(|e| format!("Failed to list messages: {}", e))?;

    Ok(Json(messages))
}

async fn get_task_messages(
    State(state): State<AppState>,
    axum::extract::Path(task_id): axum::extract::Path<String>,
) -> Result<Json<Vec<apex_memory::msg_repo::Message>>, String> {
    let repo = MessageRepository::new(&state.pool);
    let messages = repo.find_by_task(&task_id)
        .await
        .map_err(|e| format!("Failed to get task messages: {}", e))?;

    Ok(Json(messages))
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

async fn list_skills(State(state): State<AppState>) -> Result<Json<Vec<SkillResponse>>, String> {
    let registry = SkillRegistry::new(&state.pool);

    match registry.find_all().await {
        Ok(skills) => {
            let responses: Vec<SkillResponse> = skills
                .into_iter()
                .map(|s| SkillResponse {
                    name: s.name,
                    version: s.version,
                    tier: s.tier,
                    enabled: s.enabled,
                    health_status: s.health_status,
                    last_health_check: s.last_health_check,
                })
                .collect();
            Ok(Json(responses))
        }
        Err(e) => Err(format!("Failed to list skills: {}", e)),
    }
}

async fn get_skill(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<SkillResponse>, String> {
    let registry = SkillRegistry::new(&state.pool);

    match registry.find_by_name(&name).await {
        Ok(Some(skill)) => Ok(Json(SkillResponse {
            name: skill.name,
            version: skill.version,
            tier: skill.tier,
            enabled: skill.enabled,
            health_status: skill.health_status,
            last_health_check: skill.last_health_check,
        })),
        Ok(None) => Err(format!("Skill not found: {}", name)),
        Err(e) => Err(format!("Failed to get skill: {}", e)),
    }
}

async fn register_skill(
    State(state): State<AppState>,
    Json(payload): Json<RegisterSkillRequest>,
) -> Result<Json<SkillResponse>, String> {
    let registry = SkillRegistry::new(&state.pool);
    let entry = SkillRegistryEntry::new(payload.name.clone(), payload.version, payload.tier);

    match registry.upsert(&entry).await {
        Ok(_) => Ok(Json(SkillResponse {
            name: entry.name,
            version: entry.version,
            tier: entry.tier,
            enabled: entry.enabled,
            health_status: entry.health_status,
            last_health_check: entry.last_health_check,
        })),
        Err(e) => Err(format!("Failed to register skill: {}", e)),
    }
}

async fn update_skill_health(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(payload): Json<UpdateHealthRequest>,
) -> Result<Json<SkillResponse>, String> {
    let registry = SkillRegistry::new(&state.pool);

    match registry.update_health(&name, &payload.health_status).await {
        Ok(_) => match registry.find_by_name(&name).await {
            Ok(Some(skill)) => Ok(Json(SkillResponse {
                name: skill.name,
                version: skill.version,
                tier: skill.tier,
                enabled: skill.enabled,
                health_status: skill.health_status,
                last_health_check: skill.last_health_check,
            })),
            Ok(None) => Err(format!("Skill not found: {}", name)),
            Err(e) => Err(format!("Failed to get skill: {}", e)),
        },
        Err(e) => Err(format!("Failed to update health: {}", e)),
    }
}

async fn delete_skill(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let registry = SkillRegistry::new(&state.pool);

    match registry.delete(&name).await {
        Ok(_) => Ok(Json(serde_json::json!({"deleted": true, "name": name}))),
        Err(e) => Err(format!("Failed to delete skill: {}", e)),
    }
}

async fn execute_skill(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteSkillRequest>,
) -> Result<Json<ExecuteSkillResponse>, String> {
    let registry = SkillRegistry::new(&state.pool);

    let skill = match registry.find_by_name(&payload.skill_name).await {
        Ok(Some(s)) => s,
        Ok(None) => return Err(format!("Skill not found: {}", payload.skill_name)),
        Err(e) => return Err(format!("Database error: {}", e)),
    };

    if !skill.enabled {
        return Err(format!("Skill is disabled: {}", payload.skill_name));
    }

    if !state
        .circuit_breakers
        .is_available(&payload.skill_name)
        .await
    {
        return Err(format!(
            "Circuit breaker open for skill: {}. Too many recent failures.",
            payload.skill_name
        ));
    }

    let user_tier = match payload.tier.as_str() {
        "T0" => PermissionTier::T0,
        "T1" => PermissionTier::T1,
        "T2" => PermissionTier::T2,
        "T3" => PermissionTier::T3,
        _ => PermissionTier::T0,
    };

    let required_tier = match skill.tier.as_str() {
        "T0" => PermissionTier::T0,
        "T1" => PermissionTier::T1,
        "T2" => PermissionTier::T2,
        "T3" => PermissionTier::T3,
        _ => PermissionTier::T0,
    };

    let tier_order = |t: &PermissionTier| match t {
        PermissionTier::T0 => 0,
        PermissionTier::T1 => 1,
        PermissionTier::T2 => 2,
        PermissionTier::T3 => 3,
    };

    if tier_order(&user_tier) < tier_order(&required_tier) {
        return Err(format!(
            "Insufficient permissions: skill requires {}, user has {}",
            skill.tier, payload.tier
        ));
    }

    let repo = TaskRepository::new(&state.pool);
    if let Err(e) = repo
        .update_status(&payload.task_id, TaskStatus::Running)
        .await
    {
        tracing::warn!("Failed to update task status: {}", e);
    }

    state
        .message_bus
        .publish_skill(crate::message_bus::SkillExecutionMessage {
            task_id: payload.task_id.clone(),
            skill_name: payload.skill_name.clone(),
            input: payload.input,
            permission_tier: payload.tier.clone(),
        });

    Ok(Json(ExecuteSkillResponse {
        success: true,
        output: Some(format!("Skill {} queued for execution", payload.skill_name)),
        error: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_task_request_serialization() {
        let req = TaskRequest {
            content: "Hello world".to_string(),
            channel: Some("test-channel".to_string()),
            thread_id: None,
            author: Some("testuser".to_string()),
            attachments: None,
            max_steps: Some(100),
            budget_usd: Some(0.5),
            time_limit_secs: Some(300),
            project: Some("my-project".to_string()),
            priority: Some("high".to_string()),
            category: Some("bug".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Hello world"));
        assert!(json.contains("test-channel"));
    }

    #[tokio::test]
    async fn test_task_response_serialization() {
        let resp = TaskResponse {
            task_id: "test123".to_string(),
            status: "pending".to_string(),
            tier: "instant".to_string(),
            capability_token: "tok123".to_string(),
            instant_response: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("test123"));
        assert!(json.contains("pending"));
    }

    #[tokio::test]
    async fn test_task_status_response_serialization() {
        let resp = TaskStatusResponse {
            task_id: "test123".to_string(),
            status: "completed".to_string(),
            output: Some("Hello".to_string()),
            error: None,
            project: Some("my-project".to_string()),
            priority: Some("high".to_string()),
            category: Some("bug".to_string()),
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("completed"));
        assert!(json.contains("Hello"));
        assert!(json.contains("my-project"));
    }

    #[tokio::test]
    async fn test_create_task_invalid_json() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let state = AppState {
            pool: pool.clone(),
            metrics: RouterMetrics::new(),
            message_bus: MessageBus::new(10),
            circuit_breakers: CircuitBreakerRegistry::new(),
            vm_pool: None,
            execution_streams: ExecutionStreamManager::new(),
            ws_manager: WebSocketManager::new(),
            moltbook: None,
            governance: std::sync::Arc::new(std::sync::Mutex::new(GovernanceEngine::default())),
            system_monitor: SystemMonitor::new(),
            cache: ResponseCache::new(60),
            rate_limiter: RateLimiter::new(60),
            workflow_repo: apex_memory::WorkflowRepository::new(&pool),
            webhook_manager: crate::webhook::WebhookManager::new(),
            notification_manager: crate::notification::NotificationManager::new(100),
        };
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(Body::from("invalid json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
