//! Hands MVP API for Computer Use.
//!
//! Minimal Axum router exposing Hands lifecycle endpoints.
//! Uses an in-memory store for MVP; replace with DB-backed persistence in later patches.

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// In-memory store for running Hands tasks.
static HAND_STORE: OnceLock<Mutex<HashMap<String, HandTask>>> = OnceLock::new();

/// A Hands task tracked by the MVP store.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HandTask {
    id: String,
    name: String,
    status: String,
    created_at: i64,
    updated_at: i64,
}

/// Request payload for starting a new Hands task.
#[derive(Debug, Deserialize)]
struct HandsStartRequest {
    name: String,
}

/// Response returned when a Hands task is started.
#[derive(Debug, Serialize)]
struct HandsStartResponse {
    id: String,
    status: String,
}

/// Response for a Hands status query.
#[derive(Debug, Serialize)]
struct HandsStatusResponse {
    id: String,
    name: String,
    status: String,
    updated_at: i64,
}

/// Build the Hands MVP router.
pub fn router() -> Router {
    Router::new()
        .route("/start", post(hands_start))
        .route("/status/:name", get(hands_status))
        .route("/stream/:name", get(hands_stream))
        .route("/logs/:name", get(hands_logs))
}

/// Start a new Hands task (MVP: in-memory store only).
async fn hands_start(Json(payload): Json<HandsStartRequest>) -> Json<HandsStartResponse> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let id = format!("hand-{}-{}", payload.name, now);
    let task = HandTask {
        id: id.clone(),
        name: payload.name.clone(),
        status: "running".to_string(),
        created_at: now,
        updated_at: now,
    };

    HAND_STORE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .insert(id.clone(), task);

    Json(HandsStartResponse {
        id,
        status: "running".to_string(),
    })
}

/// Query the status of a Hands task by name.
async fn hands_status(Path(name): Path<String>) -> Json<HandsStatusResponse> {
    let store = HAND_STORE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Some(hand) = store.lock().unwrap().values().find(|h| h.name == name) {
        Json(HandsStatusResponse {
            id: hand.id.clone(),
            name: hand.name.clone(),
            status: hand.status.clone(),
            updated_at: hand.updated_at,
        })
    } else {
        Json(HandsStatusResponse {
            id: String::new(),
            name,
            status: "not_found".to_string(),
            updated_at: 0,
        })
    }
}

/// Placeholder streaming endpoint for a Hands task.
async fn hands_stream(Path(_name): Path<String>) -> StatusCode {
    StatusCode::OK
}

/// Placeholder logs endpoint for a Hands task.
async fn hands_logs(Path(_name): Path<String>) -> Json<Vec<String>> {
    Json(vec![])
}
