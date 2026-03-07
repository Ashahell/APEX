use axum::{
    extract::State,
    response::IntoResponse,
    response::Json as AxumJson,
    routing::{get, post},
    Json, Router,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::totp::TotpManager;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/totp/setup", post(setup_totp))
        .route("/api/v1/totp/verify", post(verify_totp))
        .route("/api/v1/totp/status", get(get_totp_status))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpSetupResponse {
    pub secret: String,
    pub otpauth_uri: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct TotpVerifyRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpVerifyResponse {
    pub valid: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpStatusResponse {
    pub configured: bool,
}

async fn setup_totp(State(state): State<AppState>) -> Result<(StatusCode, AxumJson<TotpSetupResponse>), (StatusCode, String)> {
    let user_id = "default_user";
    
    match state.totp_manager.generate_secret(user_id).await {
        Ok(secret) => {
            let otpauth_uri = TotpManager::generate_otpauth_uri(&secret, user_id, "APEX");
            Ok((StatusCode::OK, AxumJson(TotpSetupResponse {
                secret,
                otpauth_uri,
                message: "TOTP secret generated. Scan the QR code with your authenticator app.".to_string(),
            })))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e))),
    }
}

async fn verify_totp(
    State(state): State<AppState>,
    Json(payload): Json<TotpVerifyRequest>,
) -> Result<AxumJson<TotpVerifyResponse>, (StatusCode, String)> {
    let user_id = "default_user";
    
    match state.totp_manager.verify(user_id, &payload.token).await {
        Ok(valid) => Ok(AxumJson(TotpVerifyResponse {
            valid,
            message: if valid { "Token verified successfully".to_string() } else { "Invalid token".to_string() },
        })),
        Err(e) => Ok(AxumJson(TotpVerifyResponse {
            valid: false,
            message: format!("Error: {}", e),
        })),
    }
}

async fn get_totp_status(State(state): State<AppState>) -> Result<AxumJson<TotpStatusResponse>, (StatusCode, String)> {
    let user_id = "default_user";
    let configured = state.totp_manager.has_secret(user_id).await;
    Ok(AxumJson(TotpStatusResponse { configured }))
}
