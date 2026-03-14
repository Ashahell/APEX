use axum::{
    extract::{State, Path, Query},
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use apex_memory::secrets_repo::{
    SecretsRepository, SecretRef, SecretRotationLog, SecretAccessLog,
    get_category_info, get_predefined_secret_ids, is_predefined_secret,
};

use crate::api::AppState;

// ============ Router ============

pub fn router() -> Router<AppState> {
    Router::new()
        // Secret references
        .route("/api/v1/secrets", get(list_secrets))
        .route("/api/v1/secrets/categories", get(list_categories))
        .route("/api/v1/secrets/:id", get(get_secret))
        .route("/api/v1/secrets/:id", put(update_secret))
        .route("/api/v1/secrets/:id", delete(delete_secret))
        .route("/api/v1/secrets/category/:category", get(get_by_category))
        
        // Rotation logs
        .route("/api/v1/secrets/rotation/:secret_name", get(get_rotation_history))
        .route("/api/v1/secrets/rotation/recent", get(get_recent_rotations))
        
        // Access logs
        .route("/api/v1/secrets/access/:secret_ref_id", get(get_access_history))
        .route("/api/v1/secrets/access/recent", get(get_recent_accesses))
        .route("/api/v1/secrets/access/failed", get(get_failed_accesses))
        
        // Utility
        .route("/api/v1/secrets/predefined", get(list_predefined))
}

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct UpdateSecretRequest {
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogAccessRequest {
    pub access_type: String,
    pub accessed_by: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogRotationRequest {
    pub secret_name: String,
    pub rotated_by: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub old_value_hash: Option<String>,
    pub new_value_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SecretResponse {
    pub id: String,
    pub ref_key: String,
    pub secret_name: String,
    pub env_var: Option<String>,
    pub description: Option<String>,
    pub targets: Vec<String>,
    pub category: String,
    pub category_label: String,
    pub category_icon: String,
    pub is_predefined: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<SecretRef> for SecretResponse {
    fn from(s: SecretRef) -> Self {
        let targets: Vec<String> = serde_json::from_str(&s.targets).unwrap_or_default();
        let category = s.category.clone();
        let category_for_info = category.clone();
        let (label, icon) = get_category_info(&category_for_info);
        let is_predefined = is_predefined_secret(&s.id);
        
        Self {
            id: s.id,
            ref_key: s.ref_key,
            secret_name: s.secret_name,
            env_var: s.env_var,
            description: s.description,
            targets,
            category,
            category_label: label.to_string(),
            category_icon: icon.to_string(),
            is_predefined,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

// ============ Handlers ============

// List all secrets
async fn list_secrets(
    State(state): State<AppState>,
) -> Result<Json<Vec<SecretResponse>>, String> {
    let repo = SecretsRepository::new(&state.pool);
    
    let secrets = repo
        .list_secrets()
        .await
        .map_err(|e| format!("Failed to list secrets: {}", e))?;
    
    Ok(Json(secrets.into_iter().map(|s| s.into()).collect()))
}

// Get secret by ID
async fn get_secret(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SecretResponse>, String> {
    let repo = SecretsRepository::new(&state.pool);
    
    let secret = repo
        .get_secret(&id)
        .await
        .map_err(|e| format!("Secret not found: {}", e))?;
    
    Ok(Json(secret.into()))
}

// Update secret
async fn update_secret(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSecretRequest>,
) -> Result<Json<SecretResponse>, String> {
    let repo = SecretsRepository::new(&state.pool);
    
    // Check if it's predefined - only allow updating description
    if is_predefined_secret(&id) && req.description.is_some() {
        // Predefined secrets can have descriptions updated
    }
    
    let secret = repo
        .update_description(&id, req.description.as_deref().unwrap_or(""))
        .await
        .map_err(|e| format!("Failed to update secret: {}", e))?;
    
    Ok(Json(secret.into()))
}

// Delete secret
async fn delete_secret(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    // Only allow deleting custom secrets
    if is_predefined_secret(&id) {
        return Err("Cannot delete predefined secrets".to_string());
    }
    
    let repo = SecretsRepository::new(&state.pool);
    
    repo
        .delete_secret(&id)
        .await
        .map_err(|e| format!("Failed to delete secret: {}", e))?;
    
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// List categories
async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, String> {
    let repo = SecretsRepository::new(&state.pool);
    
    let categories = repo
        .list_categories()
        .await
        .map_err(|e| format!("Failed to list categories: {}", e))?;
    
    Ok(Json(categories))
}

// Get secrets by category
async fn get_by_category(
    State(state): State<AppState>,
    Path(category): Path<String>,
) -> Result<Json<Vec<SecretResponse>>, String> {
    let repo = SecretsRepository::new(&state.pool);
    
    let secrets = repo
        .get_by_category(&category)
        .await
        .map_err(|e| format!("Failed to get secrets: {}", e))?;
    
    Ok(Json(secrets.into_iter().map(|s| s.into()).collect()))
}

// ============ Rotation Log Handlers ============

// Get rotation history
async fn get_rotation_history(
    State(state): State<AppState>,
    Path(secret_name): Path<String>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<SecretRotationLog>>, String> {
    let repo = SecretsRepository::new(&state.pool);
    
    let history = repo
        .get_rotation_history(&secret_name, query.limit)
        .await
        .map_err(|e| format!("Failed to get rotation history: {}", e))?;
    
    Ok(Json(history))
}

// Get recent rotations
async fn get_recent_rotations(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<SecretRotationLog>>, String> {
    let repo = SecretsRepository::new(&state.pool);
    
    let rotations = repo
        .get_recent_rotations(query.limit.unwrap_or(20))
        .await
        .map_err(|e| format!("Failed to get rotations: {}", e))?;
    
    Ok(Json(rotations))
}

// ============ Access Log Handlers ============

// Get access history
async fn get_access_history(
    State(state): State<AppState>,
    Path(secret_ref_id): Path<String>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<SecretAccessLog>>, String> {
    let repo = SecretsRepository::new(&state.pool);
    
    let history = repo
        .get_access_history(&secret_ref_id, query.limit)
        .await
        .map_err(|e| format!("Failed to get access history: {}", e))?;
    
    Ok(Json(history))
}

// Get recent accesses
async fn get_recent_accesses(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<SecretAccessLog>>, String> {
    let repo = SecretsRepository::new(&state.pool);
    
    let accesses = repo
        .get_recent_accesses(query.limit.unwrap_or(20))
        .await
        .map_err(|e| format!("Failed to get accesses: {}", e))?;
    
    Ok(Json(accesses))
}

// Get failed accesses
async fn get_failed_accesses(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<SecretAccessLog>>, String> {
    let repo = SecretsRepository::new(&state.pool);
    
    let accesses = repo
        .get_failed_accesses(query.limit.unwrap_or(20))
        .await
        .map_err(|e| format!("Failed to get failed accesses: {}", e))?;
    
    Ok(Json(accesses))
}

// ============ Utility Handlers ============

// List predefined secret IDs
async fn list_predefined() -> Json<Vec<String>> {
    Json(get_predefined_secret_ids().into_iter().map(String::from).collect())
}
