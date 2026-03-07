use axum::{
    extract::{Path, Query, State},
    response::Json,
    Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEntryResponse {
    pub id: i64,
    pub prev_hash: String,
    pub hash: String,
    pub timestamp: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details: Option<String>,
}

impl From<apex_memory::AuditEntry> for AuditEntryResponse {
    fn from(e: apex_memory::AuditEntry) -> Self {
        Self {
            id: e.id,
            prev_hash: e.prev_hash,
            hash: e.hash,
            timestamp: e.timestamp.to_rfc3339(),
            action: e.action,
            entity_type: e.entity_type,
            entity_id: e.entity_id,
            details: e.details,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAuditRequest {
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListAuditQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuditChainStatus {
    pub valid: bool,
    pub total_entries: i64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/audit", get(list_audit))
        .route("/api/v1/audit", post(create_audit))
        .route("/api/v1/audit/entity/:entity_type/:entity_id", get(get_audit_by_entity))
        .route("/api/v1/audit/chain", get(get_chain_status))
}

async fn list_audit(
    State(state): State<AppState>,
    Query(query): Query<ListAuditQuery>,
) -> Result<Json<Vec<AuditEntryResponse>>, String> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let entries = if let (Some(entity_type), Some(entity_id)) = (&query.entity_type, &query.entity_id) {
        state.audit_repo
            .find_by_entity(entity_type, entity_id)
            .await
            .map_err(|e| format!("Failed to list audit: {}", e))?
    } else {
        state.audit_repo
            .find_all(limit, offset)
            .await
            .map_err(|e| format!("Failed to list audit: {}", e))?
    };

    Ok(Json(entries.into_iter().map(|e| e.into()).collect()))
}

async fn create_audit(
    State(state): State<AppState>,
    Json(req): Json<CreateAuditRequest>,
) -> Result<Json<AuditEntryResponse>, String> {
    let entry = state.audit_repo
        .create(apex_memory::CreateAuditEntry {
            action: req.action,
            entity_type: req.entity_type,
            entity_id: req.entity_id,
            details: req.details,
        })
        .await
        .map_err(|e| format!("Failed to create audit: {}", e))?;

    Ok(Json(entry.into()))
}

async fn get_audit_by_entity(
    State(state): State<AppState>,
    Path((entity_type, entity_id)): Path<(String, String)>,
) -> Result<Json<Vec<AuditEntryResponse>>, String> {
    let entries = state.audit_repo
        .find_by_entity(&entity_type, &entity_id)
        .await
        .map_err(|e| format!("Failed to get audit: {}", e))?;

    Ok(Json(entries.into_iter().map(|e| e.into()).collect()))
}

async fn get_chain_status(
    State(state): State<AppState>,
) -> Result<Json<AuditChainStatus>, String> {
    let valid = state.audit_repo
        .verify_chain()
        .await
        .map_err(|e| format!("Failed to verify chain: {}", e))?;

    let total = state.audit_repo
        .count()
        .await
        .map_err(|e| format!("Failed to count audit: {}", e))?;

    Ok(Json(AuditChainStatus {
        valid,
        total_entries: total,
    }))
}
