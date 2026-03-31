//! Plugin Signing API
//!
//! REST endpoints for plugin/skill signing feature.
//!
//! Feature 5: Plugin Signing (ed25519)

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::skill_signer::{
    SignatureStatus, SignatureStore, SigningKeyPair, SkillSignature, SkillSigner,
    VerificationResult,
};
use crate::unified_config::signing_constants::*;

/// Response for verification key
#[derive(Debug, Serialize)]
pub struct VerifyKeyResponse {
    pub public_key: String,
    pub algorithm: String,
}

/// Sign skill request
#[derive(Debug, Deserialize)]
pub struct SignSkillRequest {
    /// Skill name to sign
    pub skill_name: String,
    /// Skill content to sign
    pub content: String,
}

/// Verify skill request
#[derive(Debug, Deserialize)]
pub struct VerifySkillRequest {
    /// Skill name
    pub skill_name: String,
    /// Skill content
    pub content: String,
    /// Signature to verify
    pub signature: SkillSignature,
}

/// Signature status response
#[derive(Debug, Serialize)]
pub struct SignatureStatusResponse {
    pub skill_name: String,
    pub status: String,
    pub signed_at: Option<i64>,
    pub expires_at: Option<i64>,
}

/// Create signing router
pub fn create_signing_router() -> Router<AppState> {
    Router::new()
        .route("/keys/verify-key", get(get_verify_key))
        .route("/skills/:name/sign", post(sign_skill))
        .route("/skills/:name/verify", post(verify_skill))
        .route("/skills/:name/signature", get(get_signature))
        .route("/signatures/stats", get(get_signature_stats))
}

/// Get verification key
async fn get_verify_key(State(_state): State<AppState>) -> Json<VerifyKeyResponse> {
    let signer = SkillSigner::new(std::path::PathBuf::from(KEYS_DIR));
    let public_key = signer
        .get_public_key()
        .unwrap_or_else(|_| "unavailable".to_string());

    Json(VerifyKeyResponse {
        public_key,
        algorithm: SIGNATURE_ALGORITHM.to_string(),
    })
}

/// Sign a skill
async fn sign_skill(
    State(state): State<AppState>,
    Path(skill_name): Path<String>,
    Json(req): Json<SignSkillRequest>,
) -> Json<SkillSignature> {
    let signer = SkillSigner::new(std::path::PathBuf::from(KEYS_DIR));

    // Sign the skill content
    let signature = signer
        .sign_skill(&req.skill_name, &req.content)
        .unwrap_or_else(|e| {
            // Return a dummy signature on error
            SkillSignature {
                skill_name: req.skill_name,
                signature: format!("error: {}", e),
                signed_at: 0,
                expires_at: 0,
                signer_public_key: "error".to_string(),
            }
        });

    // Store the signature
    let mut store = state.signature_store.lock().unwrap();
    store.set_signature(skill_name.clone(), signature.clone());

    Json(signature)
}

/// Verify a skill's signature
async fn verify_skill(
    State(_state): State<AppState>,
    Path(_skill_name): Path<String>,
    Json(req): Json<VerifySkillRequest>,
) -> Json<VerificationResult> {
    let signer = SkillSigner::new(std::path::PathBuf::from(KEYS_DIR));

    let result = signer.verify_signature(&req.skill_name, &req.content, &req.signature);

    Json(result)
}

/// Get signature for a skill
async fn get_signature(
    State(state): State<AppState>,
    Path(skill_name): Path<String>,
) -> Json<Option<SkillSignature>> {
    let store = state.signature_store.lock().unwrap();
    Json(store.get_signature(&skill_name).cloned())
}

/// Get signature statistics
async fn get_signature_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let store = state.signature_store.lock().unwrap();

    Json(serde_json::json!({
        "total_signed": store.signed_count(),
        "signed_skills": store.signed_skills(),
    }))
}

impl AppState {
    /// Initialize signature store
    pub fn init_signature_store(&self) -> std::sync::Mutex<SignatureStore> {
        std::sync::Mutex::new(SignatureStore::new())
    }
}
