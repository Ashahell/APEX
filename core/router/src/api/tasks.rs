use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use ulid::Ulid;

use crate::apex_security::capability::{CapabilityToken, PermissionTier};
use crate::classifier::TaskClassifier;
use apex_memory::task_repo::TaskRepository;
use apex_memory::tasks::{CreateTask, TaskStatus, TaskTier};

use super::{
    AppState, TaskFilterRequest, TaskRequest, TaskResponse, TaskStatusResponse, UpdateTaskRequest,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/tasks", get(list_tasks).post(create_task))
        .route("/api/v1/tasks/filter-options", get(get_filter_options))
        .route("/api/v1/tasks/:id", get(get_task).put(update_task))
        .route("/api/v1/tasks/:id/cancel", post(cancel_task))
        .route("/api/v1/tasks/:id/confirm", post(confirm_task))
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
    Query(params): Query<TaskFilterRequest>,
) -> Result<Json<Vec<TaskStatusResponse>>, String> {
    let repo = TaskRepository::new(&state.pool);

    let limit = params.limit.unwrap_or(100);
    let offset = params.offset.unwrap_or(0);

    let tasks = repo
        .find_by_filter(
            params.project.as_deref(),
            params.status.as_deref(),
            params.priority.as_deref(),
            params.category.as_deref(),
            limit,
            offset,
        )
        .await
        .map_err(|e| format!("Failed to list tasks: {}", e))?;

    let responses: Vec<TaskStatusResponse> = tasks
        .into_iter()
        .map(|t| TaskStatusResponse {
            task_id: t.id,
            status: t.status,
            content: Some(t.input_content),
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
    Path(task_id): Path<String>,
) -> Result<Json<TaskStatusResponse>, String> {
    let repo = TaskRepository::new(&state.pool);

    match repo.find_by_id(&task_id).await {
        Ok(task) => Ok(Json(TaskStatusResponse {
            task_id: task.id,
            status: task.status,
            content: Some(task.input_content),
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
    Path(task_id): Path<String>,
) -> Result<Json<TaskStatusResponse>, String> {
    let repo = TaskRepository::new(&state.pool);

    match repo.update_status(&task_id, TaskStatus::Cancelled).await {
        Ok(_) => {
            state.metrics.record_task("unknown", "cancelled").await;
            Ok(Json(TaskStatusResponse {
                task_id,
                status: "cancelled".to_string(),
                content: None,
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

async fn confirm_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskStatusResponse>, String> {
    let repo = TaskRepository::new(&state.pool);
    
    let task = match repo.find_by_id(&task_id).await {
        Ok(t) => t,
        Err(_) => {
            return Err(format!("Task not found: {}", task_id));
        }
    };
    
    if task.status != "pending" {
        return Err(format!("Task {} is not pending, current status: {}", task_id, task.status));
    }
    
    match repo.update_status(&task_id, TaskStatus::Running).await {
        Ok(_) => {
            state.metrics.record_task("unknown", "running").await;
            Ok(Json(TaskStatusResponse {
                task_id,
                status: "running".to_string(),
                content: None,
                output: None,
                error: None,
                project: None,
                priority: None,
                category: None,
                created_at: None,
            }))
        }
        Err(e) => Err(format!("Failed to confirm task: {}", e)),
    }
}

async fn update_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(update_req): Json<UpdateTaskRequest>,
) -> Result<Json<TaskStatusResponse>, String> {
    let repo = TaskRepository::new(&state.pool);

    if let Some(status) = update_req.status {
        let task_status = apex_memory::tasks::TaskStatus::try_from_str(&status)
            .ok_or_else(|| format!("Invalid status: {}", status))?;
        repo.update_status(&task_id, task_status)
            .await
            .map_err(|e| format!("Failed to update status: {}", e))?;
    }

    repo.update_task_fields(
        &task_id,
        update_req.project.as_deref(),
        update_req.priority.as_deref(),
        update_req.category.as_deref(),
    )
    .await
    .map_err(|e| format!("Failed to update task: {}", e))?;

    match repo.find_by_id(&task_id).await {
        Ok(task) => Ok(Json(TaskStatusResponse {
            task_id: task.id,
            status: task.status,
            content: Some(task.input_content),
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
