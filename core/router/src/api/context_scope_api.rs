//! Context Scope API
//!
//! REST endpoints for context scope isolation feature.
//!
//! Feature 3: Context Scope Isolation

use axum::{
    extract::{Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::api::AppState;
use crate::context_scope::{Scope, ScopeContext};

/// In-memory scope storage
#[derive(Default)]
pub struct ContextScopeState {
    pub entries: HashMap<String, ScopeEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScopeEntry {
    pub key: String,
    pub value: String,
    pub scope: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Set scope request
#[derive(Debug, Deserialize)]
pub struct SetScopeRequest {
    /// The scope type: global, session:ID, or channel:ID
    pub scope: String,
    /// The key to store
    pub key: String,
    /// The value to store
    pub value: String,
}

/// Delete scope entry request
#[derive(Debug, Deserialize)]
pub struct DeleteScopeRequest {
    /// The scope to delete from
    pub scope: String,
    /// The key to delete
    pub key: String,
}

/// Scope stats response
#[derive(Debug, Serialize)]
pub struct ScopeStatsResponse {
    pub global_count: usize,
    pub session_count: usize,
    pub channel_count: usize,
    pub total_entries: usize,
}

/// Scope list response
#[derive(Debug, Serialize)]
pub struct ScopeListResponse {
    pub entries: Vec<ScopeEntry>,
    pub scope: String,
    pub count: usize,
}

/// Create context scope router
pub fn create_context_scope_router() -> Router<AppState> {
    Router::new()
        .route("/context/scope/set", put(set_scope_entry))
        .route("/context/scope/get", get(get_scope_entry))
        .route("/context/scope/delete", delete(delete_scope_entry))
        .route("/context/scope/list", get(list_scope_entries))
        .route("/context/scope/stats", get(get_scope_stats))
        .route("/context/scope/current", get(get_current_scope))
        .route("/context/scope/clear", post(clear_scope_entries))
}

/// Set a scope entry
async fn set_scope_entry(
    State(state): State<AppState>,
    Json(req): Json<SetScopeRequest>,
) -> Json<ScopeEntry> {
    let mut scope_state = state.context_scope_state.lock().unwrap();

    let now = chrono::Utc::now().timestamp();
    let scope_key = format!("{}:{}", req.scope, req.key);

    let entry = ScopeEntry {
        key: req.key.clone(),
        value: req.value,
        scope: req.scope.clone(),
        created_at: now,
        updated_at: now,
    };

    scope_state.entries.insert(scope_key, entry.clone());

    Json(entry)
}

/// Get a scope entry
async fn get_scope_entry(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Json<Option<ScopeEntry>> {
    let scope_state = state.context_scope_state.lock().unwrap();

    let scope = params
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("global");
    let key = params.get("key").and_then(|v| v.as_str()).unwrap_or("");

    let scope_key = format!("{}:{}", scope, key);

    Json(scope_state.entries.get(&scope_key).cloned())
}

/// Delete a scope entry
async fn delete_scope_entry(
    State(state): State<AppState>,
    Json(req): Json<DeleteScopeRequest>,
) -> Json<bool> {
    let mut scope_state = state.context_scope_state.lock().unwrap();

    let scope_key = format!("{}:{}", req.scope, req.key);

    Json(scope_state.entries.remove(&scope_key).is_some())
}

/// List entries in a scope
async fn list_scope_entries(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Json<ScopeListResponse> {
    let scope_state = state.context_scope_state.lock().unwrap();

    let scope_prefix = params
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("global")
        .to_string();

    let entries: Vec<ScopeEntry> = scope_state
        .entries
        .values()
        .filter(|e| e.scope == scope_prefix)
        .cloned()
        .collect();

    Json(ScopeListResponse {
        entries: entries.clone(),
        scope: scope_prefix,
        count: entries.len(),
    })
}

/// Get scope statistics
async fn get_scope_stats(State(state): State<AppState>) -> Json<ScopeStatsResponse> {
    let scope_state = state.context_scope_state.lock().unwrap();

    let global_count = scope_state
        .entries
        .values()
        .filter(|e| e.scope == "global")
        .count();
    let session_count = scope_state
        .entries
        .values()
        .filter(|e| e.scope.starts_with("session:"))
        .count();
    let channel_count = scope_state
        .entries
        .values()
        .filter(|e| e.scope.starts_with("channel:"))
        .count();

    Json(ScopeStatsResponse {
        global_count,
        session_count,
        channel_count,
        total_entries: scope_state.entries.len(),
    })
}

/// Get current active scope (from request context)
async fn get_current_scope(Query(params): Query<serde_json::Value>) -> Json<serde_json::Value> {
    let session_id = params.get("session_id").and_then(|v| v.as_str());
    let channel_id = params.get("channel_id").and_then(|v| v.as_str());

    let scope = if let Some(ch) = channel_id {
        format!("channel:{}", ch)
    } else if let Some(sess) = session_id {
        format!("session:{}", sess)
    } else {
        "global".to_string()
    };

    Json(serde_json::json!({
        "scope": scope,
        "scope_type": if scope.starts_with("channel:") {
            "channel"
        } else if scope.starts_with("session:") {
            "session"
        } else {
            "global"
        }
    }))
}

/// Clear all entries in a scope
async fn clear_scope_entries(
    State(state): State<AppState>,
    Json(scope): Json<String>,
) -> Json<bool> {
    let mut scope_state = state.context_scope_state.lock().unwrap();

    let initial_count = scope_state.entries.len();
    scope_state.entries.retain(|_, v| v.scope != scope);

    let cleared = scope_state.entries.len() < initial_count;

    Json(cleared)
}

impl AppState {
    /// Initialize context scope state
    pub fn init_context_scope_state(&self) -> std::sync::Mutex<ContextScopeState> {
        std::sync::Mutex::new(ContextScopeState::default())
    }
}
