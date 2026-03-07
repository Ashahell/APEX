use axum::{
    extract::{Path, State},
    response::Json,
    response::IntoResponse,
    Json as AxumJson, Router,
    routing::{get, put, delete},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingResponse {
    pub key: String,
    pub value: String,
    pub encrypted: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetSettingRequest {
    pub value: String,
    pub encrypt: Option<bool>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/settings/:key", get(get_setting))
        .route("/api/v1/settings/:key", put(set_setting))
        .route("/api/v1/settings/:key", delete(delete_setting))
        .route("/api/v1/settings", get(list_settings))
}

async fn get_setting(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<(StatusCode, AxumJson<SettingResponse>), (StatusCode, String)> {
    let pref = state.preferences_repo
        .get(&key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get setting: {}", e)))?;

    match pref {
        Some(p) => {
            let value = if p.encrypted {
                apex_memory::PreferencesRepository::decode(&p.value)
            } else {
                p.value
            };
            Ok((StatusCode::OK, AxumJson(SettingResponse {
                key: p.key,
                value,
                encrypted: p.encrypted,
            })))
        }
        None => Err((StatusCode::NOT_FOUND, format!("Setting not found: {}", key))),
    }
}

async fn set_setting(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<SetSettingRequest>,
) -> Result<Json<SettingResponse>, String> {
    let encrypt = req.encrypt.unwrap_or(false);
    
    state.preferences_repo
        .set(&key, &req.value, encrypt)
        .await
        .map_err(|e| format!("Failed to set setting: {}", e))?;

    Ok(Json(SettingResponse {
        key,
        value: req.value,
        encrypted: encrypt,
    }))
}

async fn delete_setting(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    state.preferences_repo
        .delete(&key)
        .await
        .map_err(|e| format!("Failed to delete setting: {}", e))?;

    Ok(Json(serde_json::json!({ "deleted": true, "key": key })))
}

async fn list_settings(
    State(_state): State<AppState>,
) -> Result<Json<Vec<SettingResponse>>, String> {
    // For now, return empty list - could add a list_all method later
    Ok(Json(vec![]))
}
