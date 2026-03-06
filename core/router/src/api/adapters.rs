#![allow(unused_imports)]

use axum::{
    extract::Path,
    routing::{get, post, put},
    Json, Router,
};

use super::{AdapterConfig, AppState, UpdateAdapterRequest, ADAPTERS};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/adapters", get(list_adapters))
        .route("/api/v1/adapters/:name", get(get_adapter).put(update_adapter))
        .route("/api/v1/adapters/:name/toggle", post(toggle_adapter))
}

async fn list_adapters() -> Result<Json<Vec<AdapterConfig>>, String> {
    let adapters = ADAPTERS.read().map_err(|e| e.to_string())?;
    let list: Vec<AdapterConfig> = adapters.values().cloned().collect();
    Ok(Json(list))
}

async fn get_adapter(Path(name): Path<String>) -> Result<Json<AdapterConfig>, String> {
    let adapters = ADAPTERS.read().map_err(|e| e.to_string())?;
    let adapter = adapters
        .get(&name)
        .ok_or_else(|| "Adapter not found".to_string())?
        .clone();
    Ok(Json(adapter))
}

async fn update_adapter(
    Path(name): Path<String>,
    Json(req): Json<UpdateAdapterRequest>,
) -> Result<Json<AdapterConfig>, String> {
    let mut adapters = ADAPTERS.write().map_err(|e| e.to_string())?;
    let adapter = adapters
        .get_mut(&name)
        .ok_or_else(|| "Adapter not found".to_string())?;

    if let Some(enabled) = req.enabled {
        adapter.enabled = enabled;
    }
    if let Some(config) = req.config {
        adapter.config = config;
    }

    Ok(Json(adapter.clone()))
}

async fn toggle_adapter(Path(name): Path<String>) -> Result<Json<AdapterConfig>, String> {
    let mut adapters = ADAPTERS.write().map_err(|e| e.to_string())?;
    let adapter = adapters
        .get_mut(&name)
        .ok_or_else(|| "Adapter not found".to_string())?;

    adapter.enabled = !adapter.enabled;
    Ok(Json(adapter.clone()))
}
