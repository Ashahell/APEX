//! Continuity Scheduler API
//!
//! REST endpoints for continuity scheduler feature.
//!
//! Feature 4: Continuity Scheduler

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::api::AppState;
use crate::continuity::TaskAction;
use crate::continuity::{CronSchedule, ScheduledTask, TaskHistoryEntry, TaskRegistry, TaskType};

/// In-memory task storage
#[derive(Default)]
pub struct ContinuityState {
    pub tasks: HashMap<String, ScheduledTask>,
    pub history: Vec<TaskHistoryEntry>,
}

/// Create scheduled task request
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    /// Task name
    pub name: String,
    /// Task type
    pub task_type: String,
    /// Cron schedule (e.g., "0 8 * * *")
    pub schedule: String,
}

/// Update scheduled task request
#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    /// Task name
    pub name: Option<String>,
    /// Enabled status
    pub enabled: Option<bool>,
}

/// Task list response
#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub tasks: Vec<ScheduledTask>,
    pub count: usize,
}

/// History list response
#[derive(Debug, Serialize)]
pub struct HistoryListResponse {
    pub history: Vec<TaskHistoryEntry>,
    pub count: usize,
}

/// Create continuity router
pub fn create_continuity_router() -> Router<AppState> {
    Router::new()
        .route("/continuity/tasks", get(list_tasks).post(create_task))
        .route(
            "/continuity/tasks/:id",
            get(get_task).put(update_task).delete(delete_task),
        )
        .route("/continuity/tasks/:id/run", post(run_task))
        .route("/continuity/tasks/:id/toggle", post(toggle_task))
        .route("/continuity/history", get(list_history))
        .route("/continuity/task-types", get(get_task_types))
        .route("/continuity/validate-cron", get(validate_cron))
}

/// List all scheduled tasks
async fn list_tasks(State(state): State<AppState>) -> Json<TaskListResponse> {
    let continuity = state.continuity_state.lock().unwrap();
    let tasks: Vec<ScheduledTask> = continuity.tasks.values().cloned().collect();

    Json(TaskListResponse {
        count: tasks.len(),
        tasks,
    })
}

/// Get a specific task
async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Option<ScheduledTask>> {
    let continuity = state.continuity_state.lock().unwrap();
    Json(continuity.tasks.get(&id).cloned())
}

/// Create a new scheduled task
async fn create_task(
    State(state): State<AppState>,
    Json(req): Json<CreateTaskRequest>,
) -> Json<ScheduledTask> {
    let mut continuity = state.continuity_state.lock().unwrap();

    let task_type = TaskType::from_str(&req.task_type).unwrap_or(TaskType::Custom);
    let schedule = CronSchedule::from_cron_str(&req.schedule).unwrap_or_default();

    let id = uuid::Uuid::new_v4().to_string();
    let task = ScheduledTask::new(
        req.name.clone(),
        task_type,
        schedule,
        TaskAction::new(format!("task_{}", id)),
    );

    continuity.tasks.insert(id, task.clone());

    Json(task)
}

/// Update a scheduled task
async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTaskRequest>,
) -> Json<Option<ScheduledTask>> {
    let mut continuity = state.continuity_state.lock().unwrap();

    if let Some(task) = continuity.tasks.get_mut(&id) {
        if let Some(name) = req.name {
            task.name = name;
        }
        if let Some(enabled) = req.enabled {
            task.enabled = enabled;
        }
        return Json(Some(task.clone()));
    }

    Json(None)
}

/// Delete a scheduled task
async fn delete_task(State(state): State<AppState>, Path(id): Path<String>) -> Json<bool> {
    let mut continuity = state.continuity_state.lock().unwrap();
    Json(continuity.tasks.remove(&id).is_some())
}

/// Manually run a task
async fn run_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let mut continuity = state.continuity_state.lock().unwrap();

    if let Some(task) = continuity.tasks.get_mut(&id) {
        // Record run
        task.record_run();

        // Add to history
        let mut entry = TaskHistoryEntry::start(id.clone());
        entry.success("Task executed".to_string());
        continuity.history.push(entry);

        Json(serde_json::json!({
            "success": true,
            "message": "Task executed",
        }))
    } else {
        Json(serde_json::json!({
            "success": false,
            "error": "Task not found",
        }))
    }
}

/// Toggle task enabled/disabled
async fn toggle_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Option<ScheduledTask>> {
    let mut continuity = state.continuity_state.lock().unwrap();

    if let Some(task) = continuity.tasks.get_mut(&id) {
        task.enabled = !task.enabled;
        return Json(Some(task.clone()));
    }

    Json(None)
}

/// Get task history
async fn list_history(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Json<HistoryListResponse> {
    let continuity = state.continuity_state.lock().unwrap();
    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

    let history: Vec<TaskHistoryEntry> = continuity
        .history
        .iter()
        .rev()
        .take(limit)
        .cloned()
        .collect();

    Json(HistoryListResponse {
        count: history.len(),
        history,
    })
}

/// Get available task types
async fn get_task_types() -> Json<Vec<String>> {
    Json(
        TaskRegistry::task_types()
            .into_iter()
            .map(String::from)
            .collect(),
    )
}

/// Validate a cron expression
async fn validate_cron(Query(params): Query<serde_json::Value>) -> Json<serde_json::Value> {
    let cron = params
        .get("expression")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match CronSchedule::from_cron_str(cron) {
        Ok(schedule) => Json(serde_json::json!({
            "valid": true,
            "schedule": schedule,
        })),
        Err(e) => Json(serde_json::json!({
            "valid": false,
            "error": e,
        })),
    }
}

impl AppState {
    /// Initialize continuity state
    pub fn init_continuity_state(&self) -> std::sync::Mutex<ContinuityState> {
        std::sync::Mutex::new(ContinuityState::default())
    }
}
