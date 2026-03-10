use axum::{extract::State, Json, routing::post, Router};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::mcp::validation::validate_registry_input;

#[derive(Debug, Deserialize)]
pub struct HttpValidateRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct HttpValidateResponse {
    pub valid: bool,
    pub error: Option<String>,
}

pub async fn http_validate_registry_endpoint(
    State(_state): State<AppState>,
    Json(payload): Json<HttpValidateRequest>,
) -> Json<HttpValidateResponse> {
    let ok = validate_registry_input(&payload.name).is_ok();
    Json(HttpValidateResponse {
        valid: ok,
        error: if ok { None } else { Some("invalid registry name".to_string()) },
    })
}
