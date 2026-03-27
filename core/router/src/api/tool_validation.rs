//! Tool Validation API
//!
//! REST endpoints for tool validation feature.
//!
//! Feature 1: Tool Maker Runtime Validation

use axum::{
    extract::{Query, State},
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::tool_validator::{ToolValidator, ValidationLevel, ValidationResult};
use crate::unified_config::tool_validation_constants::*;

/// Validation level query parameter
#[derive(Debug, Deserialize)]
pub struct ValidationQuery {
    level: Option<String>,
}

/// Validation request
#[derive(Debug, Deserialize)]
pub struct ValidateCodeRequest {
    /// Python code to validate
    code: String,
    /// Validation level (strict, moderate, permissive)
    level: Option<String>,
}

/// Validation response
#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub allowed: bool,
    pub blocked_imports: Vec<String>,
    pub validation_level: String,
    pub error: Option<String>,
}

impl From<ValidationResult> for ValidationResponse {
    fn from(result: ValidationResult) -> Self {
        Self {
            allowed: result.allowed,
            blocked_imports: result.blocked_imports,
            validation_level: result.validation_level,
            error: result.error,
        }
    }
}

/// Get current validation level
pub async fn get_validation_level(State(state): State<AppState>) -> Json<ValidationResponse> {
    let level = state
        .config
        .tool_validation_level
        .clone()
        .unwrap_or_else(|| DEFAULT_VALIDATION_LEVEL.to_string());

    let allowlist = match ValidationLevel::from_str(&level) {
        Ok(l) => ToolValidator::get_allowlist(l),
        Err(_) => vec![],
    };

    Json(ValidationResponse {
        allowed: true,
        blocked_imports: allowlist.iter().map(|s| s.to_string()).collect(),
        validation_level: level,
        error: None,
    })
}

/// Update validation level
pub async fn set_validation_level(
    State(mut state): State<AppState>,
    Json(payload): Json<ValidateCodeRequest>,
) -> Result<Json<ValidationResponse>, String> {
    let level = payload
        .level
        .unwrap_or_else(|| DEFAULT_VALIDATION_LEVEL.to_string());

    // Validate the level string
    match ValidationLevel::from_str(&level) {
        Ok(_) => {
            // Update config by cloning and replacing
            let mut config = state.config.clone();
            config.tool_validation_level = Some(level.clone());
            state.config = config.clone();

            // Also save to DB if config_repo exists
            if let Err(e) = config.save_to_db(&state.config_repo).await {
                tracing::warn!("Failed to save config: {}", e);
            }

            let parsed_level = ValidationLevel::from_str(&level).unwrap();
            let allowlist = ToolValidator::get_allowlist(parsed_level);

            Ok(Json(ValidationResponse {
                allowed: true,
                blocked_imports: allowlist.iter().map(|s| s.to_string()).collect(),
                validation_level: level,
                error: None,
            }))
        }
        Err(e) => Ok(Json(ValidationResponse {
            allowed: false,
            blocked_imports: vec![],
            validation_level: "strict".to_string(),
            error: Some(e),
        })),
    }
}

/// Validate code without executing
pub async fn validate_code(
    State(_state): State<AppState>,
    Json(payload): Json<ValidateCodeRequest>,
) -> Json<ValidationResponse> {
    let level = payload
        .level
        .as_ref()
        .and_then(|l| ValidationLevel::from_str(l).ok())
        .unwrap_or(ValidationLevel::Strict);

    let result = ToolValidator::validate(&payload.code, level);
    Json(result.into())
}

/// Get available validation levels
pub async fn get_validation_levels() -> Json<Vec<String>> {
    Json(VALIDATION_LEVELS.iter().map(|s| s.to_string()).collect())
}

/// Create router for tool validation API
pub fn create_tool_validation_router() -> Router<AppState> {
    Router::new()
        .route("/validation-level", get(get_validation_level))
        .route("/validation-level", put(set_validation_level))
        .route("/validate", post(validate_code))
        .route("/levels", get(get_validation_levels))
}
