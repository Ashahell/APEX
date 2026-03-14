use axum::{
    extract::{State, Path},
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use apex_memory::slack_block_repo::{SlackBlockRepository, SlackBlockTemplate};

use crate::api::AppState;

// ============ Router ============

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/slack/templates", get(list_templates))
        .route("/api/v1/slack/templates", post(create_template))
        .route("/api/v1/slack/templates/:id", get(get_template))
        .route("/api/v1/slack/templates/:id", put(update_template))
        .route("/api/v1/slack/templates/:id", delete(delete_template))
        .route("/api/v1/slack/templates/:id/render", post(render_template))
}

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub template: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub template: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RenderTemplateRequest {
    pub variables: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub id: String,
    pub name: String,
    pub template: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct RenderResponse {
    pub rendered: String,
}

impl From<SlackBlockTemplate> for TemplateResponse {
    fn from(t: SlackBlockTemplate) -> Self {
        Self {
            id: t.id,
            name: t.name,
            template: t.template,
            description: t.description,
            created_at: t.created_at,
        }
    }
}

// ============ Handlers ============

// List all templates
async fn list_templates(
    State(state): State<AppState>,
) -> Result<Json<Vec<TemplateResponse>>, String> {
    let repo = SlackBlockRepository::new(&state.pool);
    
    let templates = repo
        .list_templates()
        .await
        .map_err(|e| format!("Failed to list templates: {}", e))?;
    
    Ok(Json(templates.into_iter().map(|t| t.into()).collect()))
}

// Get template by ID
async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TemplateResponse>, String> {
    let repo = SlackBlockRepository::new(&state.pool);
    
    let template = repo
        .get_template(&id)
        .await
        .map_err(|e| format!("Template not found: {}", e))?;
    
    Ok(Json(template.into()))
}

// Create template
async fn create_template(
    State(state): State<AppState>,
    Json(req): Json<CreateTemplateRequest>,
) -> Result<Json<TemplateResponse>, String> {
    let repo = SlackBlockRepository::new(&state.pool);
    
    let id = Ulid::new().to_string();
    
    let template = repo
        .create_template(&id, &req.name, &req.template, req.description.as_deref())
        .await
        .map_err(|e| format!("Failed to create template: {}", e))?;
    
    Ok(Json(template.into()))
}

// Update template
async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTemplateRequest>,
) -> Result<Json<TemplateResponse>, String> {
    let repo = SlackBlockRepository::new(&state.pool);
    
    let template = repo
        .update_template(&id, req.name.as_deref(), req.template.as_deref(), req.description.as_deref())
        .await
        .map_err(|e| format!("Failed to update template: {}", e))?;
    
    Ok(Json(template.into()))
}

// Delete template
async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = SlackBlockRepository::new(&state.pool);
    
    repo
        .delete_template(&id)
        .await
        .map_err(|e| format!("Failed to delete template: {}", e))?;
    
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// Render template with variables
async fn render_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<RenderTemplateRequest>,
) -> Result<Json<RenderResponse>, String> {
    let repo = SlackBlockRepository::new(&state.pool);
    
    let template = repo
        .get_template(&id)
        .await
        .map_err(|e| format!("Template not found: {}", e))?;
    
    let rendered = repo
        .render_template(&template.template, &req.variables)
        .map_err(|e| format!("Failed to render template: {}", e))?;
    
    Ok(Json(RenderResponse { rendered }))
}
