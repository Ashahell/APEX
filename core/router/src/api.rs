use axum::{
    extract::State,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use ulid::Ulid;

use crate::apex_security::capability::{CapabilityToken, PermissionTier};
use crate::circuit_breaker::CircuitBreakerRegistry;
use crate::classifier::TaskClassifier;
use crate::message_bus::DeepTaskMessage;
use crate::message_bus::MessageBus;
use crate::metrics::RouterMetrics;
use crate::vm_pool::VmPool;
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
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
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
            pool,
            metrics: RouterMetrics::new(),
            message_bus: MessageBus::new(10),
            circuit_breakers: CircuitBreakerRegistry::new(),
            vm_pool: None,
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
