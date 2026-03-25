use axum::{routing::{get, post}, Json, Router, extract::Path, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::computer_use::orchestrator::{ComputerUseOrchestrator, ExecutionResult};
use crate::computer_use::screenshot::{ScreenshotConfig};
use crate::computer_use::vlm::VLMProvider;

static STORE: OnceLock<Mutex<HashMap<String, ComputerUseTask>>> = OnceLock::new();
// Simple Hands store for MVP
static HAND_STORE: OnceLock<Mutex<HashMap<String, HandTask>>> = OnceLock::new();
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HandTask {
  id: String,
  name: String,
  status: String,
  created_at: u64,
  updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComputerUseTask {
    id: String,
    task: String,
    status: String,
    steps: u32,
    cost: f64,
    final_state: Option<String>,
    created_at: u64,
    updated_at: u64,
}

#[derive(Debug, Deserialize)]
struct ExecuteRequest {
    task: String,
    max_steps: Option<u32>,
    max_cost_usd: Option<f64>,
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ExecuteResponse {
    task_id: String,
    status: String,
    steps: u32,
    cost: f64,
    final_state: Option<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse { ok: bool }

#[derive(Debug, Serialize)]
struct StatusResponse {
    id: String,
    status: String,
    steps: u32,
    cost: f64,
    final_state: Option<String>,
}

#[derive(Debug, Serialize)]
struct ScreenshotInfo { ts: u64, note: String }

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/execute", post(execute))
        // Hands lifecycle MVP endpoints
        .route("/hands/start", post(hands_start))
        .route("/hands/status/:name", get(hands_status))
        .route("/hands/stream/:name", get(hands_stream))
        .route("/hands/logs/:name", get(hands_logs))
        .route("/status/:id", get(status))
        .route("/stream/:id", get(stream))
        .route("/screenshots/:id", get(screenshots))
}

// Inline integration tests for MVP endpoints can be added in dedicated test crates later
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { ok: true })
}

#[derive(Debug, Deserialize)]
struct HandsStartRequest {
  name: String,
}

#[derive(Debug, Serialize)]
 struct HandsStartResponse {
  id: String,
  status: String,
}

async fn hands_start(Json(payload): Json<HandsStartRequest>) -> Json<HandsStartResponse> {
 let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
 let id = format!("hand-{}-{}", payload.name, now);
  let ht = HandTask { id: id.clone(), name: payload.name.clone(), status: "running".to_string(), created_at: now, updated_at: now };
  HAND_STORE.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap().insert(id.clone(), ht);
  // Update the in-memory hand store with an initial entry
  Json(HandsStartResponse { id, status: "running".to_string() })
}

#[derive(Debug, Serialize)]
struct HandsStatusResponse {
  id: String,
  name: String,
  status: String,
  updated_at: u64,
}

async fn hands_status(Path(name): Path<String>) -> Json<HandsStatusResponse> {
  let store = HAND_STORE.get_or_init(|| Mutex::new(HashMap::new()));
  if let Some(hand) = store.lock().unwrap().values().find(|h| h.name == name) {
    Json(HandsStatusResponse { id: hand.id.clone(), name: hand.name.clone(), status: hand.status.clone(), updated_at: hand.updated_at })
  } else {
    Json(HandsStatusResponse { id: String::new(), name, status: "not_found".to_string(), updated_at: 0 })
  }
}

async fn hands_stream(Path(_name): Path<String>) -> StatusCode {
  StatusCode::OK // placeholder streaming
}

#[derive(Debug, Serialize)]
struct HandsLogsResponse {
  entries: Vec<String>,
}

async fn hands_logs(Path(_name): Path<String>) -> Json<HandsLogsResponse> {
  Json(HandsLogsResponse { entries: Vec::new() })
}

async fn execute(Json(payload): Json<ExecuteRequest>) -> Json<ExecuteResponse> {
    // Create a fresh orchestrator and run the in-process MVP path
    let mut orch = ComputerUseOrchestrator::new();
    let res = orch.execute(&payload.task).await;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    let task_id = format!("cu-{}", now);
    let status = match &res {
        Ok(r) => {
            // Store a minimal record
        let mut store = STORE.get_or_init(|| Mutex::new(HashMap::new()));
            let entry = ComputerUseTask {
                id: task_id.clone(),
                task: payload.task.clone(),
                status: "completed".to_string(),
                steps: r.steps,
                cost: r.cost,
                final_state: r.final_state.clone(),
                created_at: now,
                updated_at: now,
            };
            store.lock().unwrap().insert(task_id.clone(), entry);
            "completed".to_string()
        }
        Err(_e) => {
            // store as failed
            let mut store = STORE.get_or_init(|| Mutex::new(HashMap::new()));
            let entry = ComputerUseTask {
                id: task_id.clone(),
                task: payload.task.clone(),
                status: "failed".to_string(),
                steps: 0,
                cost: 0.0,
                final_state: None,
                created_at: now,
                updated_at: now,
            };
            store.lock().unwrap().insert(task_id.clone(), entry);
            "failed".to_string()
        }
    };
    let steps = res.as_ref().map(|r| r.steps).unwrap_or(0);
    let cost = res.as_ref().map(|r| r.cost).unwrap_or(0.0);
    let final_state = res.ok().and_then(|r| r.final_state);
    Json(ExecuteResponse { task_id, status, steps, cost, final_state })
}

async fn status(Path(id): Path<String>) -> Json<StatusResponse> {
    let store = STORE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(t) = store.lock().unwrap().get(&id) {
        Json(StatusResponse { id: t.id.clone(), status: t.status.clone(), steps: t.steps, cost: t.cost, final_state: t.final_state.clone() })
    } else {
        // Not found
        Json(StatusResponse { id, status: "not_found".to_string(), steps: 0, cost: 0.0, final_state: None })
    }
}

async fn stream(Path(_id): Path<String>) -> StatusCode {
    // Placeholder: MVP does not implement streaming yet
    StatusCode::OK
}

async fn screenshots(Path(_id): Path<String>) -> Json<Vec<ScreenshotInfo>> {
    // Placeholder: no real screenshots stored yet
    Json(vec![])
}
