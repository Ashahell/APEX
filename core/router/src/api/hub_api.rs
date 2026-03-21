//! Skills Hub API (Hermes-style Skills Hub)
//!
//! Endpoints for browsing and installing skills from the hub.

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::api::api_error::ApiError;
use crate::hub_client::{HubClient, HubError, HubSkill};

/// GET /api/v1/hub/skills - List skills from hub
async fn list_hub_skills(
    State(state): State<crate::api::AppState>,
    Query(params): Query<HubSearchParams>,
) -> Result<Json<Vec<HubSkill>>, (axum::http::StatusCode, String)> {
    let client = HubClient::new(state.config.hub.base_url.clone());
    
    let skills = if let Some(query) = &params.q {
        client.search_skills(query).await
    } else if let Some(category) = &params.category {
        client.get_skills_by_category(category).await
    } else {
        client.get_featured_skills().await
    };

    skills
        .map(Json)
        .map_err(|e| match e {
            HubError::HubUnavailable { status } => {
                ApiError::internal(format!("Hub unavailable: HTTP {}", status))
            }
            _ => ApiError::internal(format!("Hub error: {}", e)),
        })
}

/// GET /api/v1/hub/skills/:id - Get skill details
async fn get_hub_skill(
    State(state): State<crate::api::AppState>,
    Path(id): Path<String>,
) -> Result<Json<HubSkill>, (axum::http::StatusCode, String)> {
    let client = HubClient::new(state.config.hub.base_url.clone());
    
    client.get_skill(&id)
        .await
        .map(Json)
        .map_err(|e| match e {
            HubError::SkillNotFound { .. } => ApiError::not_found(format!("Skill '{}' not found", id)),
            HubError::HubUnavailable { status } => {
                ApiError::internal(format!("Hub unavailable: HTTP {}", status))
            }
            _ => ApiError::internal(format!("Hub error: {}", e)),
        })
}

/// GET /api/v1/hub/status - Check hub status
async fn hub_status(
    State(_state): State<crate::api::AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // For now, return mock status
    Ok(Json(serde_json::json!({
        "connected": false,
        "url": "https://skills.sh/api/v1",
        "message": "Hub integration is available. Configure hub URL in settings."
    })))
}

#[derive(Debug, Deserialize)]
pub struct HubSearchParams {
    pub q: Option<String>,
    pub category: Option<String>,
}

use serde::Serialize;

/// Create hub router
pub fn router() -> Router<crate::api::AppState> {
    Router::new()
        .route("/api/v1/hub/skills", get(list_hub_skills))
        .route("/api/v1/hub/skills/{id}", get(get_hub_skill))
        .route("/api/v1/hub/status", get(hub_status))
}
