//! Privacy Guard API
//!
//! REST endpoints for privacy guard feature.
//!
//! Feature 6: Privacy Toggle

use axum::{
    extract::{State, Query},
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::privacy_guard::{PrivacyCheckResult, PrivacyConfig, PrivacyGuard};
use crate::unified_config::privacy_constants::*;

/// Update privacy config request
#[derive(Debug, Deserialize)]
pub struct UpdatePrivacyConfigRequest {
    /// Enable/disable privacy mode
    pub enabled: Option<bool>,
    /// Blocked providers
    pub blocked_providers: Option<Vec<String>>,
    /// Allow local-only connections
    pub allow_local_only: Option<bool>,
    /// Enable audit logging
    pub audit_log_enabled: Option<bool>,
}

/// Privacy status response
#[derive(Debug, Serialize)]
pub struct PrivacyStatusResponse {
    pub enabled: bool,
    pub blocked_providers: Vec<String>,
    pub allow_local_only: bool,
    pub audit_log_enabled: bool,
    pub cloud_providers: Vec<String>,
}

/// Check provider response
#[derive(Debug, Serialize)]
pub struct ProviderCheckResponse {
    pub provider: String,
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Create privacy router
pub fn create_privacy_router() -> Router<AppState> {
    Router::new()
        .route("/privacy/status", get(get_privacy_status))
        .route("/privacy/config", get(get_privacy_config).put(update_privacy_config))
        .route("/privacy/check", get(check_provider))
        .route("/privacy/providers", get(list_providers))
        .route("/privacy/audit", get(get_audit_log))
}

/// Get current privacy status
async fn get_privacy_status(State(state): State<AppState>) -> Json<PrivacyStatusResponse> {
    let guard = state.privacy_guard.lock().unwrap();
    let config = guard.config();
    
    Json(PrivacyStatusResponse {
        enabled: config.enabled,
        blocked_providers: config.blocked_providers.clone(),
        allow_local_only: config.allow_local_only,
        audit_log_enabled: config.audit_log_enabled,
        cloud_providers: PrivacyGuard::cloud_providers().into_iter().map(String::from).collect(),
    })
}

/// Get current privacy config
async fn get_privacy_config(State(state): State<AppState>) -> Json<PrivacyConfig> {
    let guard = state.privacy_guard.lock().unwrap();
    Json(guard.config().clone())
}

/// Update privacy config
async fn update_privacy_config(
    State(state): State<AppState>,
    Json(req): Json<UpdatePrivacyConfigRequest>,
) -> Json<PrivacyConfig> {
    let mut guard = state.privacy_guard.lock().unwrap();
    
    let mut config = guard.config().clone();
    
    if let Some(enabled) = req.enabled {
        config.enabled = enabled;
    }
    if let Some(providers) = req.blocked_providers {
        config.blocked_providers = providers;
    }
    if let Some(allow_local) = req.allow_local_only {
        config.allow_local_only = allow_local;
    }
    if let Some(audit) = req.audit_log_enabled {
        config.audit_log_enabled = audit;
    }
    
    guard.update_config(config.clone());
    
    Json(config)
}

/// Check if a provider is allowed
async fn check_provider(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Json<ProviderCheckResponse> {
    let provider = params.get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let guard = state.privacy_guard.lock().unwrap();
    let result = guard.check(provider);
    
    Json(ProviderCheckResponse {
        provider: provider.to_string(),
        allowed: result.allowed,
        reason: result.reason,
    })
}

/// List available cloud providers
async fn list_providers(State(_state): State<AppState>) -> Json<Vec<String>> {
    Json(PrivacyGuard::cloud_providers().into_iter().map(String::from).collect())
}

/// Get audit log entries (stub - would need actual implementation)
async fn get_audit_log(
    State(_state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Json<serde_json::Value> {
    let limit = params.get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(50) as usize;
    
    // Return empty log for now
    Json(serde_json::json!({
        "entries": [],
        "count": 0,
        "limit": limit,
    }))
}

impl AppState {
    /// Initialize privacy guard
    pub fn init_privacy_guard(&self) -> std::sync::Mutex<PrivacyGuard> {
        std::sync::Mutex::new(PrivacyGuard::default_guard())
    }
}
