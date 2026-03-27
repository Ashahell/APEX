//! Skill Manager API (Hermes-style Agent-Managed Skills)
//!
//! Endpoints for managing auto-created skills by the agent.

use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::api_error::ApiError;
use crate::skill_manager::{SkillCreateRequest, SkillError, SkillMetadata, SkillPatchRequest};

/// GET /api/v1/skills/auto-created - List all auto-created skills
async fn list_auto_created_skills(
    State(state): State<crate::api::AppState>,
) -> Result<Json<Vec<SkillMetadata>>, (axum::http::StatusCode, String)> {
    let manager = state.skill_manager.lock().await;
    manager
        .list_skills()
        .map(|skills| Json(skills))
        .map_err(|e| ApiError::internal(format!("Failed to list skills: {}", e)))
}

/// POST /api/v1/skills/auto-created - Create a new auto-created skill
async fn create_auto_created_skill(
    State(state): State<crate::api::AppState>,
    Json(req): Json<SkillCreateRequest>,
) -> Result<Json<SkillMetadata>, (axum::http::StatusCode, String)> {
    let manager = state.skill_manager.lock().await;
    manager
        .create_skill(req)
        .await
        .map(|metadata| Json(metadata))
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("already exists") {
                ApiError::bad_request(msg)
            } else if msg.contains("Invalid skill name") {
                ApiError::bad_request(msg)
            } else if msg.contains("SecurityBlocked") {
                ApiError::forbidden("Skill content blocked by security scan".to_string())
            } else {
                ApiError::internal(format!("Failed to create skill: {}", e))
            }
        })
}

/// GET /api/v1/skills/auto-created/:name - Get skill metadata
async fn get_auto_created_skill(
    State(state): State<crate::api::AppState>,
    Path(name): Path<String>,
) -> Result<Json<SkillMetadata>, (axum::http::StatusCode, String)> {
    let manager = state.skill_manager.lock().await;
    manager
        .list_skills()
        .map_err(|e| ApiError::internal(format!("Failed to list skills: {}", e)))?
        .into_iter()
        .find(|s| s.name == name)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Skill '{}' not found", name)))
}

/// GET /api/v1/skills/auto-created/:name/content - Get skill content
async fn get_skill_content(
    State(state): State<crate::api::AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let manager = state.skill_manager.lock().await;
    manager
        .get_skill_content(&name)
        .await
        .map(|content| Json(serde_json::json!({ "content": content })))
        .map_err(|e| ApiError::not_found(format!("Skill '{}' not found: {}", name, e)))
}

/// PUT /api/v1/skills/auto-created/:name - Patch skill content
async fn patch_auto_created_skill(
    State(state): State<crate::api::AppState>,
    Path(name): Path<String>,
    Json(patch): Json<SkillPatchRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let manager = state.skill_manager.lock().await;
    manager
        .patch_skill(&name, patch)
        .await
        .map(|_| {
            Json(serde_json::json!({ "success": true, "message": "Skill patched successfully" }))
        })
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                ApiError::not_found(format!("Skill '{}' not found", name))
            } else if msg.contains("Content not found") {
                ApiError::not_found("Content to replace not found in skill".to_string())
            } else if msg.contains("SecurityBlocked") {
                ApiError::forbidden("Patched content blocked by security scan".to_string())
            } else {
                ApiError::internal(format!("Failed to patch skill: {}", e))
            }
        })
}

/// DELETE /api/v1/skills/auto-created/:name - Delete a skill
async fn delete_auto_created_skill(
    State(state): State<crate::api::AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let manager = state.skill_manager.lock().await;
    manager
        .delete_skill(&name)
        .await
        .map(|_| {
            Json(serde_json::json!({ "success": true, "message": "Skill deleted successfully" }))
        })
        .map_err(|e| ApiError::not_found(format!("Skill '{}' not found: {}", name, e)))
}

/// GET /api/v1/skills/auto-created/search?q=query - Find similar skills
async fn search_similar_skills(
    State(state): State<crate::api::AppState>,
    axum::extract::Query(params): axum::extract::Query<SearchParams>,
) -> Result<Json<Vec<SkillMetadata>>, (axum::http::StatusCode, String)> {
    let manager = state.skill_manager.lock().await;
    manager
        .find_similar(&params.q)
        .map(Json)
        .map_err(|e| ApiError::internal(format!("Failed to search skills: {}", e)))
}

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
}

/// Create skill manager router
pub fn router() -> Router<crate::api::AppState> {
    Router::new()
        .route("/api/v1/skills/auto-created", get(list_auto_created_skills))
        .route(
            "/api/v1/skills/auto-created",
            post(create_auto_created_skill),
        )
        .route(
            "/api/v1/skills/auto-created/search",
            get(search_similar_skills),
        )
        .route(
            "/api/v1/skills/auto-created/{name}",
            get(get_auto_created_skill),
        )
        .route(
            "/api/v1/skills/auto-created/{name}",
            put(patch_auto_created_skill),
        )
        .route(
            "/api/v1/skills/auto-created/{name}",
            delete(delete_auto_created_skill),
        )
        .route(
            "/api/v1/skills/auto-created/{name}/content",
            get(get_skill_content),
        )
        .route("/api/v1/skills/suggestions", get(list_skill_suggestions))
        .route(
            "/api/v1/skills/suggestions/{task_id}",
            get(get_skill_suggestion),
        )
        .route(
            "/api/v1/skills/suggestions/{task_id}",
            delete(delete_skill_suggestion),
        )
}

/// GET /api/v1/skills/suggestions - List skill suggestions from deep tasks
async fn list_skill_suggestions(
    State(_state): State<crate::api::AppState>,
) -> Result<Json<Vec<SkillSuggestion>>, (axum::http::StatusCode, String)> {
    let suggestions_dir = dirs::data_local_dir()
        .unwrap_or_default()
        .join("apex")
        .join("skill_suggestions");

    if !suggestions_dir.exists() {
        return Ok(Json(Vec::new()));
    }

    let mut suggestions = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&suggestions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(suggestion) = serde_json::from_str::<SkillSuggestion>(&content) {
                        suggestions.push(suggestion);
                    }
                }
            }
        }
    }

    Ok(Json(suggestions))
}

/// GET /api/v1/skills/suggestions/:task_id - Get specific suggestion
async fn get_skill_suggestion(
    State(_state): State<crate::api::AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<SkillSuggestion>, (axum::http::StatusCode, String)> {
    let suggestion_path = dirs::data_local_dir()
        .unwrap_or_default()
        .join("apex")
        .join("skill_suggestions")
        .join(format!("{}.json", task_id));

    if !suggestion_path.exists() {
        return Err(ApiError::not_found(format!(
            "Suggestion for task '{}' not found",
            task_id
        )));
    }

    let content = std::fs::read_to_string(&suggestion_path)
        .map_err(|e| ApiError::internal(format!("Failed to read suggestion: {}", e)))?;

    let suggestion: SkillSuggestion = serde_json::from_str(&content)
        .map_err(|e| ApiError::internal(format!("Failed to parse suggestion: {}", e)))?;

    Ok(Json(suggestion))
}

/// DELETE /api/v1/skills/suggestions/:task_id - Delete a suggestion
async fn delete_skill_suggestion(
    State(_state): State<crate::api::AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let suggestion_path = dirs::data_local_dir()
        .unwrap_or_default()
        .join("apex")
        .join("skill_suggestions")
        .join(format!("{}.json", task_id));

    if !suggestion_path.exists() {
        return Err(ApiError::not_found(format!(
            "Suggestion for task '{}' not found",
            task_id
        )));
    }

    std::fs::remove_file(&suggestion_path)
        .map_err(|e| ApiError::internal(format!("Failed to delete suggestion: {}", e)))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillSuggestion {
    pub task_id: String,
    pub skill_name: String,
    pub task_content: String,
    pub skill_content: String,
    pub tools_used: Vec<String>,
    pub suggested_at: String,
}
