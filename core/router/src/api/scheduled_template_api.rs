//! Scheduled Template API Endpoints
//! 
//! v1.10.0: Scheduled Task Templates API

use tokio::sync::RwLock;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use chrono::Utc;

use crate::api::api_error::ApiError;
use crate::api::AppState;
use crate::scheduled_template::{
    ScheduledTemplate, ScheduledExecution,
    CreateScheduledTemplateRequest, UpdateScheduledTemplateRequest,
    TriggerTemplateRequest, TriggerTemplateResponse, ScheduleType,
};

/// Shared scheduled template state
pub struct ScheduledTemplateState {
    pub templates: RwLock<Vec<ScheduledTemplate>>,
    pub executions: RwLock<Vec<ScheduledExecution>>,
}

impl ScheduledTemplateState {
    pub fn new() -> Self {
        Self {
            templates: RwLock::new(Vec::new()),
            executions: RwLock::new(Vec::new()),
        }
    }

    pub async fn list_templates(&self) -> Vec<ScheduledTemplate> {
        self.templates.read().await.clone()
    }

    pub async fn get_template(&self, id: &str) -> Option<ScheduledTemplate> {
        self.templates.read().await.iter()
            .find(|t| t.id == id)
            .cloned()
    }

    pub async fn create_template(&self, mut template: ScheduledTemplate) -> ScheduledTemplate {
        template.id = ulid::Ulid::new().to_string();
        template.created_at = Utc::now();
        template.updated_at = Utc::now();
        template.next_run_at = template.calculate_next_run();
        self.templates.write().await.push(template.clone());
        template
    }

    pub async fn update_template(&self, id: &str, update: UpdateScheduledTemplateRequest) -> Option<ScheduledTemplate> {
        let mut templates = self.templates.write().await;
        if let Some(template) = templates.iter_mut().find(|t| t.id == id) {
            if let Some(name) = update.name {
                template.name = name;
            }
            if let Some(desc) = update.description {
                template.description = Some(desc);
            }
            if let Some(content) = update.task_content {
                template.task_content = content;
            }
            if let Some(schedule_type) = update.schedule_type {
                template.schedule_type = schedule_type;
            }
            if let Some(interval) = update.interval_secs {
                template.schedule_config.interval_secs = interval;
            }
            if let Some(cron) = update.cron_expr {
                template.schedule_config.cron_expr = Some(cron);
            }
            if let Some(run_at) = update.run_at {
                template.schedule_config.run_at = Some(run_at);
            }
            if let Some(max) = update.max_runs {
                template.max_runs = Some(max);
            }
            if let Some(enabled) = update.enabled {
                template.enabled = enabled;
            }
            template.updated_at = Utc::now();
            template.next_run_at = template.calculate_next_run();
            return Some(template.clone());
        }
        None
    }

    pub async fn delete_template(&self, id: &str) -> bool {
        let mut templates = self.templates.write().await;
        let pos = templates.iter().position(|t| t.id == id);
        if let Some(pos) = pos {
            templates.remove(pos);
            true
        } else {
            false
        }
    }

    pub async fn get_templates_to_run(&self) -> Vec<ScheduledTemplate> {
        let templates = self.templates.read().await;
        templates.iter()
            .filter(|t| t.should_run())
            .cloned()
            .collect()
    }

    pub async fn record_execution(&self, execution: ScheduledExecution) {
        // Update template run count
        let mut templates = self.templates.write().await;
        if let Some(template) = templates.iter_mut().find(|t| t.id == execution.template_id) {
            template.record_run();
        }
        drop(templates);
        
        self.executions.write().await.push(execution);
    }

    pub async fn get_executions(&self, template_id: Option<&str>) -> Vec<ScheduledExecution> {
        let executions = self.executions.read().await;
        match template_id {
            Some(id) => executions.iter().filter(|e| e.template_id == id).cloned().collect(),
            None => executions.clone(),
        }
    }

    pub async fn get_execution(&self, id: &str) -> Option<ScheduledExecution> {
        self.executions.read().await.iter()
            .find(|e| e.id == id)
            .cloned()
    }
}

impl Default for ScheduledTemplateState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the scheduled template router
pub fn create_scheduled_template_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/scheduled-templates", get(list_templates))
        .route("/api/v1/scheduled-templates/:id", get(get_template))
        .route("/api/v1/scheduled-templates", post(create_template))
        .route("/api/v1/scheduled-templates/:id", put(update_template))
        .route("/api/v1/scheduled-templates/:id", delete(delete_template))
        .route("/api/v1/scheduled-templates/:id/trigger", post(trigger_template))
        .route("/api/v1/scheduled-templates/pending", get(list_pending))
        .route("/api/v1/scheduled-executions", get(list_executions))
        .route("/api/v1/scheduled-executions/:id", get(get_execution))
}

/// GET /api/v1/scheduled-templates - List all templates
async fn list_templates(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScheduledTemplate>>, (axum::http::StatusCode, String)> {
    let templates = state.scheduled_template_state.list_templates().await;
    Ok(Json(templates))
}

/// GET /api/v1/scheduled-templates/:id - Get a specific template
async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ScheduledTemplate>, (axum::http::StatusCode, String)> {
    state.scheduled_template_state.get_template(&id).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Template {} not found", id)))
}

/// POST /api/v1/scheduled-templates - Create a new template
async fn create_template(
    State(state): State<AppState>,
    Json(req): Json<CreateScheduledTemplateRequest>,
) -> Result<Json<ScheduledTemplate>, (axum::http::StatusCode, String)> {
    let mut template = ScheduledTemplate::new(req.name, req.task_content);
    template.description = req.description;
    
    if let Some(schedule_type) = req.schedule_type {
        template.schedule_type = schedule_type;
    }
    if let Some(interval) = req.interval_secs {
        template.schedule_config.interval_secs = interval;
    }
    if let Some(cron) = req.cron_expr {
        template.schedule_config.cron_expr = Some(cron);
    }
    if let Some(run_at) = req.run_at {
        template.schedule_config.run_at = Some(run_at);
    }
    if let Some(max) = req.max_runs {
        template.max_runs = Some(max);
    }

    let created = state.scheduled_template_state.create_template(template).await;
    Ok(Json(created))
}

/// PUT /api/v1/scheduled-templates/:id - Update a template
async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateScheduledTemplateRequest>,
) -> Result<Json<ScheduledTemplate>, (axum::http::StatusCode, String)> {
    state.scheduled_template_state.update_template(&id, req).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Template {} not found", id)))
}

/// DELETE /api/v1/scheduled-templates/:id - Delete a template
async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    if state.scheduled_template_state.delete_template(&id).await {
        Ok(Json(serde_json::json!({ "success": true })))
    } else {
        Err(ApiError::not_found(format!("Template {} not found", id)))
    }
}

/// POST /api/v1/scheduled-templates/:id/trigger - Trigger a template manually
async fn trigger_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(_req): Json<TriggerTemplateRequest>,
) -> Result<Json<TriggerTemplateResponse>, (axum::http::StatusCode, String)> {
    let _template = state.scheduled_template_state.get_template(&id).await
        .ok_or_else(|| ApiError::not_found(format!("Template {} not found", id)))?;

    // Create execution record
    let task_id = ulid::Ulid::new().to_string();
    let mut execution = ScheduledExecution::new(id.clone(), task_id.clone());
    execution.mark_running();
    
    let execution_id = execution.id.clone();
    state.scheduled_template_state.record_execution(execution).await;

    Ok(Json(TriggerTemplateResponse {
        success: true,
        execution_id: Some(execution_id),
        task_id: Some(task_id),
        message: "Template triggered successfully".to_string(),
    }))
}

/// GET /api/v1/scheduled-templates/pending - List templates ready to run
async fn list_pending(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScheduledTemplate>>, (axum::http::StatusCode, String)> {
    let templates = state.scheduled_template_state.get_templates_to_run().await;
    Ok(Json(templates))
}

/// GET /api/v1/scheduled-executions - List executions
#[derive(Debug, serde::Deserialize)]
pub struct ListExecutionsQuery {
    pub template_id: Option<String>,
}

async fn list_executions(
    State(state): State<AppState>,
    Query(query): Query<ListExecutionsQuery>,
) -> Result<Json<Vec<ScheduledExecution>>, (axum::http::StatusCode, String)> {
    let executions = state.scheduled_template_state.get_executions(query.template_id.as_deref()).await;
    Ok(Json(executions))
}

/// GET /api/v1/scheduled-executions/:id - Get a specific execution
async fn get_execution(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ScheduledExecution>, (axum::http::StatusCode, String)> {
    state.scheduled_template_state.get_execution(&id).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Execution {} not found", id)))
}
