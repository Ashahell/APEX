//! User Profile API (Hermes-style User Preferences)
//!
//! Endpoints for managing user preferences.

use axum::{
    extract::State,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::api_error::ApiError;
use crate::user_profile::{
    CommunicationStyle, ResponseFormat, UserProfile, UserProfileManager, Verbosity,
};

/// GET /api/v1/user/profile - Get user profile
async fn get_user_profile(
    State(state): State<crate::api::AppState>,
) -> Result<Json<UserProfile>, (axum::http::StatusCode, String)> {
    let profile = state.user_profile.get_profile().await;
    Ok(Json(profile))
}

/// PUT /api/v1/user/profile - Update user profile
async fn update_user_profile(
    State(state): State<crate::api::AppState>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, (axum::http::StatusCode, String)> {
    let mut profile = state.user_profile.get_profile().await;

    if let Some(style) = req.communication_style {
        profile.communication_style = style;
    }
    if let Some(verbosity) = req.verbosity {
        profile.verbosity = verbosity;
    }
    if let Some(format) = req.response_format {
        profile.response_format = format;
    }
    if let Some(include_reasoning) = req.include_reasoning {
        profile.include_reasoning = include_reasoning;
    }
    if let Some(language) = req.language {
        profile.language = language;
    }

    state.user_profile.set_profile(profile.clone())
        .await
        .map_err(|e| ApiError::internal(format!("Failed to save profile: {}", e)))?;

    Ok(Json(profile))
}

/// GET /api/v1/user/profile/system-prompt - Get system prompt additions
async fn get_system_prompt(
    State(state): State<crate::api::AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let additions = state.user_profile.get_system_prompt_additions().await;
    Ok(Json(serde_json::json!({ "additions": additions })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub communication_style: Option<CommunicationStyle>,
    pub verbosity: Option<Verbosity>,
    pub response_format: Option<ResponseFormat>,
    pub include_reasoning: Option<bool>,
    pub language: Option<String>,
}

/// Create user profile router
pub fn router() -> Router<crate::api::AppState> {
    Router::new()
        .route("/api/v1/user/profile", get(get_user_profile))
        .route("/api/v1/user/profile", put(update_user_profile))
        .route("/api/v1/user/profile/system-prompt", get(get_system_prompt))
}
