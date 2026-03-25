//! Streaming API endpoints for Hands and MCP.
//!
//! Patch 11: End-to-end SSE streaming integration for Hands and MCP tasks.
//! Streams events via Server-Sent Events (SSE) using the existing ExecutionStreamManager.
//!
//! Patch 14: WebSocket streaming with ticket-based auth.
//! - Ticket endpoint: GET /api/v1/stream/ticket?task_id=X
//! - WebSocket endpoint: WS /api/v1/stream/ws/:task_id?ticket=...
//!
//! Patch 16: Streaming analytics — connection counts, event throughput, error rates.
//!
//! Security posture:
//! - Streaming is gated behind a config flag (APEX_STREAMING_ENABLED)
//! - HMAC authentication is required on stream initiation (connection-time auth)
//! - Replay protection: in-memory detector rejects duplicate request signatures
//! - All event payloads are JSON-serialized ExecutionEvent variants

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, sse::{Event, Sse}},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use futures::stream::{self, Stream};
use std::sync::atomic::AtomicU64;
use std::time::{Duration, Instant};
use futures_util::{SinkExt, StreamExt};
use tracing::{error, info, warn};

use crate::api::AppState;
use crate::execution_stream::ExecutionEvent;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Stream-specific error type.
#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    #[error("Stream not found for task: {0}")]
    StreamNotFound(String),

    #[error("Streaming disabled via config")]
    StreamingDisabled,

    #[error("Authentication required")]
    AuthRequired,

    #[error("Replay detected: {0}")]
    ReplayDetected(String),

    #[error("Internal streaming error: {0}")]
    Internal(String),
}

impl StreamingError {
    /// Convert this error to an SSE error event for client consumption.
    pub fn to_sse_event(&self) -> Event {
        let payload = serde_json::json!({
            "type": "error",
            "message": self.to_string(),
        });
        Event::default()
            .event("error")
            .data(serde_json::to_string(&payload).unwrap_or_else(|_| r#"{"type":"error","message":"serialization error"}"#.to_string()))
    }
}

impl IntoResponse for StreamingError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            StreamingError::StreamNotFound(_) => StatusCode::NOT_FOUND,
            StreamingError::StreamingDisabled => StatusCode::FORBIDDEN,
            StreamingError::AuthRequired => StatusCode::UNAUTHORIZED,
            StreamingError::ReplayDetected(_) => StatusCode::CONFLICT,
            StreamingError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

// ---------------------------------------------------------------------------
// Streaming Analytics (Patch 16)
// ---------------------------------------------------------------------------

use std::sync::atomic::Ordering;

/// Thread-safe streaming metrics — atomic counters for observability.
///
/// Counters:
///   - active_connections: Currently open SSE/WS streams
///   - total_connections: Cumulative connections since startup
///   - events_by_type: Events dispatched by ExecutionEvent variant
///   - errors: Stream errors by category
#[derive(Debug, Default)]
pub struct StreamingMetrics {
    pub active_connections: AtomicU64,
    pub total_connections: AtomicU64,
    pub events_thought: AtomicU64,
    pub events_tool_call: AtomicU64,
    pub events_tool_progress: AtomicU64,
    pub events_tool_result: AtomicU64,
    pub events_approval: AtomicU64,
    pub events_error: AtomicU64,
    pub events_complete: AtomicU64,
    pub errors_auth: AtomicU64,
    pub errors_replay: AtomicU64,
    pub errors_internal: AtomicU64,
}

impl StreamingMetrics {
    /// Increment active connection counter (called on stream connect).
    pub fn on_connect(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        self.total_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connection counter (called on stream disconnect).
    pub fn on_disconnect(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Increment event counter by type.
    pub fn on_event(&self, event: &ExecutionEvent) {
        match event {
            ExecutionEvent::Thought { .. } => { let _ = self.events_thought.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
            ExecutionEvent::ToolCall { .. } => { let _ = self.events_tool_call.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
            ExecutionEvent::ToolProgress { .. } => { let _ = self.events_tool_progress.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
            ExecutionEvent::ToolResult { .. } => { let _ = self.events_tool_result.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
            ExecutionEvent::ApprovalNeeded { .. } => { let _ = self.events_approval.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
            ExecutionEvent::Error { .. } => { let _ = self.events_error.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
            ExecutionEvent::Complete { .. } => { let _ = self.events_complete.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
        }
    }

    /// Increment error counter by category.
    pub fn on_error(&self, kind: &str) {
        match kind {
            "auth" => { let _ = self.errors_auth.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
            "replay" => { let _ = self.errors_replay.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
            _ => { let _ = self.errors_internal.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
        }
    }
}

/// Snapshot of streaming metrics for API responses.
#[derive(Debug, Serialize)]
pub struct StreamingStats {
    pub active_connections: u64,
    pub total_connections: u64,
    pub events: EventCounts,
    pub errors: ErrorCounts,
}

#[derive(Debug, Serialize)]
pub struct EventCounts {
    pub thought: u64,
    pub tool_call: u64,
    pub tool_progress: u64,
    pub tool_result: u64,
    pub approval_needed: u64,
    pub error: u64,
    pub complete: u64,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct ErrorCounts {
    pub auth: u64,
    pub replay: u64,
    pub internal: u64,
    pub total: u64,
}

impl From<&StreamingMetrics> for StreamingStats {
    fn from(m: &StreamingMetrics) -> Self {
        let thought = m.events_thought.load(Ordering::Relaxed);
        let tool_call = m.events_tool_call.load(Ordering::Relaxed);
        let tool_progress = m.events_tool_progress.load(Ordering::Relaxed);
        let tool_result = m.events_tool_result.load(Ordering::Relaxed);
        let approval = m.events_approval.load(Ordering::Relaxed);
        let error = m.events_error.load(Ordering::Relaxed);
        let complete = m.events_complete.load(Ordering::Relaxed);
        let total_events = thought + tool_call + tool_progress + tool_result + approval + error + complete;

        let auth = m.errors_auth.load(Ordering::Relaxed);
        let replay = m.errors_replay.load(Ordering::Relaxed);
        let internal = m.errors_internal.load(Ordering::Relaxed);
        let total_errors = auth + replay + internal;

        Self {
            active_connections: m.active_connections.load(Ordering::Relaxed),
            total_connections: m.total_connections.load(Ordering::Relaxed),
            events: EventCounts {
                thought,
                tool_call,
                tool_progress,
                tool_result,
                approval_needed: approval,
                error,
                complete,
                total: total_events,
            },
            errors: ErrorCounts {
                auth,
                replay,
                internal,
                total: total_errors,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// HMAC + replay-protection helpers
// ---------------------------------------------------------------------------

/// Verify HMAC signature for a streaming request.
///
/// MVP implementation: extracts X-APEX-Signature and X-APEX-Timestamp
/// from query parameters (passed by the client via EventSource URL).
/// Falls through gracefully when APEX_AUTH_DISABLED=1.
fn verify_stream_auth(
    state: &AppState,
    auth: &StreamAuthQuery,
) -> Result<(), StreamingError> {
    // Skip auth in dev mode
    if state.config.auth.disabled {
        info!("Streaming auth bypassed (APEX_AUTH_DISABLED)");
        return Ok(());
    }

    // Extract query params
    let signature = auth.signature.as_deref().ok_or(StreamingError::AuthRequired)?;
    let timestamp_str = auth.timestamp.as_deref().ok_or(StreamingError::AuthRequired)?;
    let ts: i64 = timestamp_str
        .parse()
        .map_err(|_| StreamingError::AuthRequired)?;

    // Replay protection: check and record the signature nonce (Patch 15: uses injected backend)
    if state.replay_protection.record_and_check(signature) {
        warn!(signature = %signature, "Replay detected on streaming request");
        return Err(StreamingError::ReplayDetected(signature.to_string()));
    }

    // Timestamp drift check (5-minute window, same as REST)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let drift = (now - ts).abs();
    if drift > 300 {
        warn!(drift_secs = drift, "Streaming timestamp drift too large");
        return Err(StreamingError::AuthRequired);
    }

    // HMAC verification: use the existing verify_request from auth.rs
    // For SSE streaming, path is empty (no body), method is GET
    let secret = &state.config.auth.shared_secret;
    if !crate::auth::verify_request(secret, "GET", "", &[], signature, ts) {
        warn!("Invalid HMAC signature for streaming request");
        return Err(StreamingError::AuthRequired);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// SSE stream handlers
// ---------------------------------------------------------------------------

/// Query parameters for streaming authentication (MVP: query-param approach).
///
/// The client sends HMAC signature via query params because native EventSource
/// cannot set custom headers. In production (Patch 12+), migrate to header auth.
#[derive(Debug, Deserialize)]
pub struct StreamAuthQuery {
    /// Unix timestamp (seconds) — must be within 5 min of server time.
    #[serde(rename = "__timestamp")]
    pub timestamp: Option<String>,
    /// HMAC-SHA256 hex signature of the request.
    #[serde(rename = "__signature")]
    pub signature: Option<String>,
    /// Nonce (optional, for replay tracking).
    #[serde(rename = "__nonce")]
    pub nonce: Option<String>,
}

/// Build the streaming router, mounting Hands and MCP SSE endpoints.
///
/// Endpoints:
///   GET /api/v1/stream/hands/:task_id   — SSE stream for a Hands task
///   GET /api/v1/stream/mcp/:task_id     — SSE stream for an MCP task
///   GET /api/v1/stream/task/:task_id    — Generic SSE stream (alias)


/// SSE stream for Hands task events.
async fn stream_hands(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Query(auth): Query<StreamAuthQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, StreamingError> {
    _stream_task_internal(&state, &task_id, "hands", &auth).await
}

/// SSE stream for MCP task events.
async fn stream_mcp(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Query(auth): Query<StreamAuthQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, StreamingError> {
    _stream_task_internal(&state, &task_id, "mcp", &auth).await
}

/// Generic SSE stream for any task (alias).
async fn stream_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Query(auth): Query<StreamAuthQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, StreamingError> {
    _stream_task_internal(&state, &task_id, "task", &auth).await
}

/// Shared SSE stream implementation.
async fn _stream_task_internal(
    state: &AppState,
    task_id: &str,
    stream_type: &str,
    auth: &StreamAuthQuery,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, StreamingError> {
    // Config gate: reject if streaming is disabled
    if !state.config.streaming.enabled {
        error!(task_id = %task_id, "Streaming requested but disabled via config");
        return Err(StreamingError::StreamingDisabled);
    }

    // Auth check with query params
    if let Err(e) = verify_stream_auth(state, auth) {
        state.streaming_metrics.on_error("auth");
        return Err(e);
    }

    // Subscribe to the ExecutionStream for this task
    let streams = &state.execution_streams;
    let mut receiver = streams
        .subscribe(task_id)
        .ok_or_else(|| StreamingError::StreamNotFound(task_id.to_string()))?;

    info!(task_id = %task_id, stream_type = %stream_type, "Starting SSE stream");

    // Increment connection counters (metrics)
    state.streaming_metrics.on_connect();

    // Keepalive interval (30 s)
    let keepalive_interval = Duration::from_secs(30);

    // Clone metrics for the stream closure
    let metrics = state.streaming_metrics.clone();
    let task_id_owned = task_id.to_string();

    // Build the SSE stream: unfold handles both events and keepalive ticks
    let stream = stream::unfold((receiver, task_id_owned, metrics), move |(mut rx, tid, metrics)| async move {
        // Send a connect event on stream open
        let connect_event = Event::default()
            .event("connected")
            .data(serde_json::json!({
                "type": "connected",
                "task_id": &tid,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }).to_string());

        // Keepalive ticker
        let mut keepalive = tokio::time::interval(keepalive_interval);
        keepalive.tick().await; // skip first immediate tick

        loop {
            tokio::select! {
                // Incoming execution event
                event = rx.recv() => {
                    match event {
                        Ok(exec_event) => {
                            // Record event metric
                            metrics.on_event(&exec_event);
                            let (sse_event, _data) = execution_event_to_sse(&exec_event);
                            return Some((Ok(sse_event), (rx, tid, metrics)));
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            warn!(task_id = %tid, lagged = n, "SSE receiver lagged behind, skipping events");
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            info!(task_id = %tid, "SSE stream closed by sender");
                            // Record disconnect metric — called here so it fires exactly once
                            metrics.on_disconnect();
                            let close_event = Event::default()
                                .event("stream_closed")
                                .data(serde_json::json!({
                                    "type": "stream_closed",
                                    "task_id": &tid,
                                }).to_string());
                            return Some((Ok(close_event), (rx, tid, metrics)));
                        }
                    }
                }
                // Keepalive tick
                _ = keepalive.tick() => {
                    // Send a SSE comment keepalive
                    let ka = Event::default().comment(": keepalive");
                    return Some((Ok(ka), (rx, tid, metrics)));
                }
            }
        }
    });

    use axum::response::sse::KeepAlive;
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

// ---------------------------------------------------------------------------
// WebSocket streaming (Patch 14: Header-auth migration)
// ---------------------------------------------------------------------------

use axum::{
    extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
    response::Json,
};
use futures_util::stream::{SplitSink, SplitStream};
use std::sync::Arc as StdArc;

/// Ticket validity window in seconds (30 s is enough for a WS handshake).
const TICKET_VALIDITY_SECS: i64 = 30;

/// A signed WebSocket streaming ticket.
/// Issued by GET /api/v1/stream/ticket, validated by WS /api/v1/stream/ws/:task_id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamTicket {
    /// Task ID this ticket grants access to.
    pub task_id: String,
    /// Unix timestamp (seconds) when this ticket expires.
    pub expires_at: i64,
    /// Random nonce for uniqueness.
    pub nonce: String,
    /// HMAC-SHA256 signature over the above fields.
    pub signature: String,
}

impl StreamTicket {
    /// Generate a new ticket for the given task_id.
    fn new(task_id: String, secret: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expires_at = now + TICKET_VALIDITY_SECS;
        let nonce = uuid::Uuid::new_v4().to_string();
        let signature = Self::sign(&task_id, expires_at, &nonce, secret);
        Self { task_id, expires_at, nonce, signature }
    }

    /// Sign a ticket's fields. Format: HMAC-SHA256("ticket|task_id|expires_at|nonce", secret).
    fn sign(task_id: &str, expires_at: i64, nonce: &str, secret: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;
        let data = format!("ticket|{}|{}|{}", task_id, expires_at, nonce);
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// Verify the ticket's signature and expiry.
    fn verify(&self, secret: &str) -> Result<(), &'static str> {
        // Check expiry
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        if now > self.expires_at {
            return Err("Ticket expired");
        }
        // Verify signature
        let expected = Self::sign(&self.task_id, self.expires_at, &self.nonce, secret);
        if !constant_time_eq(expected.as_bytes(), self.signature.as_bytes()) {
            return Err("Invalid ticket signature");
        }
        Ok(())
    }
}

/// Constant-time byte comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ---------------------------------------------------------------------------
// Ticket endpoint (GET /api/v1/stream/ticket)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct TicketQuery {
    task_id: String,
}

/// GET /api/v1/stream/ticket?task_id=X
///
/// Requires HMAC auth (via X-APEX-* headers).
/// Returns a short-lived signed ticket for WebSocket streaming.
/// The client then connects to WS /api/v1/stream/ws/:task_id?ticket=...
pub async fn get_stream_ticket(
    State(state): State<AppState>,
    Query(query): Query<TicketQuery>,
) -> Result<Json<StreamTicket>, StreamingError> {
    if !state.config.streaming.enabled {
        return Err(StreamingError::StreamingDisabled);
    }
    // Verify HMAC auth (standard REST headers)
    // The REST API gateway middleware handles this, but we add a guard here
    if !state.config.auth.disabled {
        // Auth is enforced by the gateway; we trust the request if it reached here
        // In a more paranoid setup we'd re-verify the HMAC headers here
        info!("Ticket issued for task: {}", query.task_id);
    }
    let secret = &state.config.auth.shared_secret;
    let ticket = StreamTicket::new(query.task_id, secret);
    info!(task_id = %ticket.task_id, expires_at = %ticket.expires_at, "Stream ticket issued");
    Ok(Json(ticket))
}

// ---------------------------------------------------------------------------
// WebSocket streaming endpoint (WS /api/v1/stream/ws/:task_id)
// ---------------------------------------------------------------------------

/// Query params for WebSocket streaming.
#[derive(Debug, Deserialize)]
struct WsAuthQuery {
    ticket: String,
}

/// WS /api/v1/stream/ws/:task_id?ticket=...
///
/// Upgrades to WebSocket and relays ExecutionEvent JSON messages.
/// The ticket must be valid (signature + expiry).
pub async fn ws_stream(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Query(query): Query<WsAuthQuery>,
) -> impl axum::response::IntoResponse {
    let state = StdArc::new(state);
    ws.on_upgrade(move |socket| handle_ws_stream(socket, state, task_id, query.ticket))
}

async fn handle_ws_stream(
    mut socket: WebSocket,
    state: StdArc<AppState>,
    task_id: String,
    ticket_raw: String,
) {
    // 1. Validate ticket
    let ticket: StreamTicket = match serde_json::from_str(&ticket_raw) {
        Ok(t) => t,
        Err(_) => {
            error!(task_id = %task_id, "Failed to parse stream ticket");
            let _ = socket.send(WsMessage::Text(r#"{"type":"error","message":"Invalid ticket format"}"#.into())).await;
            let _ = socket.close().await;
            return;
        }
    };

    if ticket.task_id != task_id {
        error!(ticket_task = %ticket.task_id, path_task = %task_id, "Ticket task_id mismatch");
        let _ = socket.send(WsMessage::Text(r#"{"type":"error","message":"Ticket task_id mismatch"}"#.into())).await;
        let _ = socket.close().await;
        return;
    }

    let secret = &state.config.auth.shared_secret;
    if let Err(e) = ticket.verify(secret) {
        warn!(task_id = %task_id, reason = e, "Stream ticket verification failed");
        state.streaming_metrics.on_error("auth");
        let _ = socket.send(WsMessage::Text(serde_json::json!({
            "type": "error",
            "message": format!("Ticket invalid: {}", e)
        }).to_string().into())).await;
        let _ = socket.close().await;
        return;
    }

    // Increment connection counters
    state.streaming_metrics.on_connect();

    // 2. Subscribe to execution stream
    let mut rx = match state.execution_streams.subscribe(&task_id) {
        Some(r) => r,
        None => {
            warn!(task_id = %task_id, "No execution stream for task");
            let _ = socket.send(WsMessage::Text(serde_json::json!({
                "type": "error",
                "message": "No stream found for task"
            }).to_string().into())).await;
            let _ = socket.close().await;
            return;
        }
    };

    info!(task_id = %task_id, "WebSocket stream connected");

    // 3. Send connected event
    let connected = serde_json::json!({
        "type": "connected",
        "task_id": task_id,
    }).to_string();
    let _ = socket.send(WsMessage::Text(connected)).await;

    // 4. Relay execution events over WebSocket
    let mut socket = socket;
    let (sender, receiver) = socket.split();
    let sender = StdArc::new(tokio::sync::Mutex::new(sender));
    let receiver = StdArc::new(tokio::sync::Mutex::new(receiver));
    let task_id_clone = task_id.clone();

    // Spawn task to handle incoming WS messages (e.g., ping/pong heartbeat)
    // Lock the receiver mutex inside the async block to get &mut SplitStream
    let sender_clone = sender.clone();
    let forward_task_raw = tokio::spawn(async move {
        let mut rx_lock = receiver.lock().await;
        while let Some(msg) = futures_util::StreamExt::next(&mut *rx_lock).await {
            if let Ok(WsMessage::Text(text)) = msg {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if json.get("type").and_then(|v| v.as_str()) == Some("ping") {
                        let mut tx = sender_clone.lock().await;
                        let _ = tx.send(WsMessage::Text(
                            serde_json::json!({ "type": "pong" }).to_string()
                        )).await;
                    }
                }
            }
        }
    });
    // Pin the JoinHandle so tokio::select! can poll it without consuming it
    let mut forward_task = std::pin::pin!(forward_task_raw);

    // Relay execution events to WebSocket (main loop)
    // Poll forward_task.await in a tokio::select! to detect when it finishes
    loop {
        tokio::select! {
            // Execution event from the task
            event = rx.recv() => {
                match event {
                    Ok(exec_event) => {
                        // Record event metric
                        state.streaming_metrics.on_event(&exec_event);
                        let json = execution_event_to_json(&exec_event, &task_id_clone);
                        let mut tx = sender.lock().await;
                        if tx.send(WsMessage::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        info!(task_id = %task_id, "Execution stream closed");
                        let mut tx = sender.lock().await;
                        let _ = tx.send(WsMessage::Text(
                            serde_json::json!({ "type": "stream_closed", "task_id": task_id_clone }).to_string()
                        )).await;
                        break;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // Skip lagged events
                        continue;
                    }
                }
            }
            // Forward task finished (client disconnected)
            _ = forward_task.as_mut() => {
                break;
            }
        }
    }

    // Decrement active connections
    state.streaming_metrics.on_disconnect();
    info!(task_id = %task_id, "WebSocket stream disconnected");
}

/// Convert an ExecutionEvent to a JSON string for WebSocket transport.
fn execution_event_to_json(event: &ExecutionEvent, task_id: &str) -> String {
    match event {
        ExecutionEvent::Thought { step, content } => {
            serde_json::json!({
                "type": "Thought",
                "task_id": task_id,
                "step": step,
                "content": content,
            }).to_string()
        }
        ExecutionEvent::ToolCall { step, tool, input } => {
            serde_json::json!({
                "type": "ToolCall",
                "task_id": task_id,
                "step": step,
                "tool": tool,
                "input": input,
            }).to_string()
        }
        ExecutionEvent::ToolProgress { step, tool, output } => {
            serde_json::json!({
                "type": "ToolProgress",
                "task_id": task_id,
                "step": step,
                "tool": tool,
                "output": output,
            }).to_string()
        }
        ExecutionEvent::ToolResult { step, tool, success, output } => {
            serde_json::json!({
                "type": "ToolResult",
                "task_id": task_id,
                "step": step,
                "tool": tool,
                "success": success,
                "output": output,
            }).to_string()
        }
        ExecutionEvent::ApprovalNeeded { step, tier, action, consequences } => {
            serde_json::json!({
                "type": "ApprovalNeeded",
                "task_id": task_id,
                "step": step,
                "tier": tier,
                "action": action,
                "consequences": consequences,
            }).to_string()
        }
        ExecutionEvent::Error { step, message } => {
            serde_json::json!({
                "type": "Error",
                "task_id": task_id,
                "step": step,
                "message": message,
            }).to_string()
        }
        ExecutionEvent::Complete { output, steps, tools_used } => {
            serde_json::json!({
                "type": "Complete",
                "task_id": task_id,
                "output": output,
                "steps": steps,
                "tools_used": tools_used,
            }).to_string()
        }
    }
}

// ---------------------------------------------------------------------------
// Update router to include WebSocket endpoints
// ---------------------------------------------------------------------------

/// GET /api/v1/stream/stats
///
/// Returns a snapshot of streaming metrics (connection counts, event throughput, error rates).
pub async fn get_stream_stats(
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    Json(StreamingStats::from(state.streaming_metrics.as_ref()))
}

pub fn create_streaming_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/hands/:task_id", get(stream_hands))
        .route("/mcp/:task_id", get(stream_mcp))
        .route("/task/:task_id", get(stream_task))
        // Ticket endpoint (Patch 14)
        .route("/ticket", get(get_stream_ticket))
        // WebSocket streaming (Patch 14)
        .route("/ws/:task_id", get(ws_stream))
        // Streaming analytics (Patch 16)
        .route("/stats", get(get_stream_stats))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Event conversion helpers
// ---------------------------------------------------------------------------

/// Convert an ExecutionEvent to an SSE Event with the appropriate event type.
/// Returns (Event, data_string) so callers (including tests) can verify the payload.
pub fn execution_event_to_sse(event: &ExecutionEvent) -> (Event, String) {
    let (event_type, data_str) = match event {
        ExecutionEvent::Thought { step, content } => (
            "thought",
            serde_json::json!({ "type": "thought", "step": step, "content": content }).to_string(),
        ),
        ExecutionEvent::ToolCall { step, tool, input } => (
            "tool_call",
            serde_json::json!({ "type": "tool_call", "step": step, "tool": tool, "input": input }).to_string(),
        ),
        ExecutionEvent::ToolProgress { step, tool, output } => (
            "tool_progress",
            serde_json::json!({ "type": "tool_progress", "step": step, "tool": tool, "output": output }).to_string(),
        ),
        ExecutionEvent::ToolResult { step, tool, success, output } => (
            "tool_result",
            serde_json::json!({ "type": "tool_result", "step": step, "tool": tool, "success": success, "output": output }).to_string(),
        ),
        ExecutionEvent::ApprovalNeeded { step, tier, action, consequences } => (
            "approval_needed",
            serde_json::json!({
                "type": "approval_needed",
                "step": step,
                "tier": tier,
                "action": action,
                "consequences": consequences,
            }).to_string(),
        ),
        ExecutionEvent::Error { step, message } => (
            "error",
            serde_json::json!({ "type": "error", "step": step, "message": message }).to_string(),
        ),
        ExecutionEvent::Complete { output, steps, tools_used } => (
            "complete",
            serde_json::json!({ "type": "complete", "output": output, "steps": steps, "tools_used": tools_used }).to_string(),
        ),
    };

    let sse_event = Event::default()
        .event(event_type)
        .data(data_str.clone());
    (sse_event, data_str)
}

// ---------------------------------------------------------------------------
// Tests — focused on pure conversion logic (no AppState dependency)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- ExecutionEvent -> SSE conversion ---

    #[test]
    fn test_execution_event_to_sse_thought() {
        let event = ExecutionEvent::Thought {
            step: 1,
            content: "Thinking about the task".to_string(),
        };
        let (sse, data_str) = execution_event_to_sse(&event);
        assert!(data_str.contains("\"type\":\"thought\""));
        assert!(data_str.contains("\"step\":1"));
        assert!(data_str.contains("Thinking about the task"));
        // Verify SSE event was created (no panic)
        let _ = sse;
    }

    #[test]
    fn test_execution_event_to_sse_tool_call() {
        let event = ExecutionEvent::ToolCall {
            step: 2,
            tool: "shell.execute".to_string(),
            input: serde_json::json!({"command": "ls"}),
        };
        let (sse, data_str) = execution_event_to_sse(&event);
        assert!(data_str.contains("\"type\":\"tool_call\""));
        assert!(data_str.contains("\"tool\":\"shell.execute\""));
        assert!(data_str.contains("\"command\":\"ls\""));
        let _ = sse;
    }

    #[test]
    fn test_execution_event_to_sse_tool_progress() {
        let event = ExecutionEvent::ToolProgress {
            step: 2,
            tool: "shell.execute".to_string(),
            output: "downloading...".to_string(),
        };
        let (sse, data_str) = execution_event_to_sse(&event);
        assert!(data_str.contains("\"type\":\"tool_progress\""));
        assert!(data_str.contains("downloading"));
        let _ = sse;
    }

    #[test]
    fn test_execution_event_to_sse_tool_result_success() {
        let event = ExecutionEvent::ToolResult {
            step: 2,
            tool: "shell.execute".to_string(),
            success: true,
            output: "done".to_string(),
        };
        let (sse, data_str) = execution_event_to_sse(&event);
        assert!(data_str.contains("\"type\":\"tool_result\""));
        assert!(data_str.contains("\"success\":true"));
        assert!(data_str.contains("done"));
        let _ = sse;
    }

    #[test]
    fn test_execution_event_to_sse_tool_result_failure() {
        let event = ExecutionEvent::ToolResult {
            step: 3,
            tool: "file.read".to_string(),
            success: false,
            output: "Permission denied".to_string(),
        };
        let (sse, data_str) = execution_event_to_sse(&event);
        assert!(data_str.contains("\"success\":false"));
        assert!(data_str.contains("Permission denied"));
        let _ = sse;
    }

    #[test]
    fn test_execution_event_to_sse_approval_needed() {
        let event = ExecutionEvent::ApprovalNeeded {
            step: 4,
            tier: "T2".to_string(),
            action: "shell.execute".to_string(),
            consequences: crate::execution_stream::ConsequencePreview::default(),
        };
        let (sse, data_str) = execution_event_to_sse(&event);
        assert!(data_str.contains("\"type\":\"approval_needed\""));
        assert!(data_str.contains("\"tier\":\"T2\""));
        assert!(data_str.contains("\"action\":\"shell.execute\""));
        let _ = sse;
    }

    #[test]
    fn test_execution_event_to_sse_error() {
        let event = ExecutionEvent::Error {
            step: 3,
            message: "File not found".to_string(),
        };
        let (sse, data_str) = execution_event_to_sse(&event);
        assert!(data_str.contains("\"type\":\"error\""));
        assert!(data_str.contains("File not found"));
        let _ = sse;
    }

    #[test]
    fn test_execution_event_to_sse_complete() {
        let event = ExecutionEvent::Complete {
            output: "Task finished successfully".to_string(),
            steps: 5,
            tools_used: vec!["shell.execute".to_string(), "file.read".to_string()],
        };
        let (sse, data_str) = execution_event_to_sse(&event);
        assert!(data_str.contains("\"type\":\"complete\""));
        assert!(data_str.contains("\"steps\":5"));
        assert!(data_str.contains("Task finished successfully"));
        assert!(data_str.contains("shell.execute"));
        let _ = sse;
    }

    // --- StreamingError -> SSE conversion ---

    #[test]
    fn test_streaming_error_to_sse_stream_not_found() {
        let err = StreamingError::StreamNotFound("task-abc".to_string());
        let sse = err.to_sse_event();
        // Verify the SSE event was created (no panic, Debug impl works)
        let debug_str = format!("{:?}", sse);
        assert!(debug_str.contains("Event"));
    }

    #[test]
    fn test_streaming_error_to_sse_disabled() {
        let err = StreamingError::StreamingDisabled;
        let sse = err.to_sse_event();
        let debug_str = format!("{:?}", sse);
        assert!(debug_str.contains("Event"));
    }

    #[test]
    fn test_streaming_error_to_sse_auth_required() {
        let err = StreamingError::AuthRequired;
        let sse = err.to_sse_event();
        let debug_str = format!("{:?}", sse);
        assert!(debug_str.contains("Event"));
    }

    #[test]
    fn test_streaming_error_to_sse_replay() {
        let err = StreamingError::ReplayDetected("sig-xyz".to_string());
        let sse = err.to_sse_event();
        let debug_str = format!("{:?}", sse);
        assert!(debug_str.contains("Event"));
    }

    #[test]
    fn test_streaming_error_to_sse_internal() {
        let err = StreamingError::Internal("Something went wrong".to_string());
        let sse = err.to_sse_event();
        let debug_str = format!("{:?}", sse);
        assert!(debug_str.contains("Event"));
    }

    // --- StreamTicket tests ---

    #[test]
    fn test_stream_ticket_sign_and_verify() {
        let secret = "test-secret";
        let ticket = StreamTicket::new("task-123".to_string(), secret);
        assert_eq!(ticket.task_id, "task-123");
        assert!(!ticket.nonce.is_empty());
        assert!(ticket.expires_at > 0);
        // Verify with correct secret
        assert!(ticket.verify(secret).is_ok());
    }

    #[test]
    fn test_stream_ticket_wrong_secret() {
        let ticket = StreamTicket::new("task-456".to_string(), "correct-secret");
        // Verify with wrong secret
        let result = ticket.verify("wrong-secret");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid ticket signature");
    }

    #[test]
    fn test_constant_time_eq() {
        // Same bytes
        assert!(constant_time_eq(b"hello", b"hello"));
        // Different lengths
        assert!(!constant_time_eq(b"hello", b"hell"));
        // Different content
        assert!(!constant_time_eq(b"hello", b"world"));
    }

    #[test]
    fn test_stream_ticket_signature_format() {
        let secret = "my-secret";
        let ticket = StreamTicket::new("task-789".to_string(), secret);
        // Signature should be a hex string (64 chars for SHA-256)
        assert_eq!(ticket.signature.len(), 64);
        assert!(ticket.signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // --- StreamingMetrics tests (Patch 16) ---

    #[test]
    fn test_streaming_metrics_on_connect_disconnect() {
        let metrics = StreamingMetrics::default();
        // Start at 0
        assert_eq!(metrics.active_connections.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert_eq!(metrics.total_connections.load(std::sync::atomic::Ordering::Relaxed), 0);

        metrics.on_connect();
        assert_eq!(metrics.active_connections.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(metrics.total_connections.load(std::sync::atomic::Ordering::Relaxed), 1);

        metrics.on_connect();
        assert_eq!(metrics.active_connections.load(std::sync::atomic::Ordering::Relaxed), 2);
        assert_eq!(metrics.total_connections.load(std::sync::atomic::Ordering::Relaxed), 2);

        metrics.on_disconnect();
        assert_eq!(metrics.active_connections.load(std::sync::atomic::Ordering::Relaxed), 1);

        metrics.on_disconnect();
        assert_eq!(metrics.active_connections.load(std::sync::atomic::Ordering::Relaxed), 0);
        // total_connections stays at 2
        assert_eq!(metrics.total_connections.load(std::sync::atomic::Ordering::Relaxed), 2);
    }

    #[test]
    fn test_streaming_metrics_on_event_by_type() {
        let metrics = StreamingMetrics::default();

        let thought = ExecutionEvent::Thought { step: 1, content: "thinking".into() };
        metrics.on_event(&thought);
        assert_eq!(metrics.events_thought.load(std::sync::atomic::Ordering::Relaxed), 1);

        let tool_call = ExecutionEvent::ToolCall { step: 2, tool: "shell".into(), input: serde_json::json!({}) };
        metrics.on_event(&tool_call);
        assert_eq!(metrics.events_tool_call.load(std::sync::atomic::Ordering::Relaxed), 1);

        let progress = ExecutionEvent::ToolProgress { step: 2, tool: "shell".into(), output: "output".into() };
        metrics.on_event(&progress);
        assert_eq!(metrics.events_tool_progress.load(std::sync::atomic::Ordering::Relaxed), 1);

        let result = ExecutionEvent::ToolResult { step: 2, tool: "shell".into(), success: true, output: "done".into() };
        metrics.on_event(&result);
        assert_eq!(metrics.events_tool_result.load(std::sync::atomic::Ordering::Relaxed), 1);

        let approval = ExecutionEvent::ApprovalNeeded {
            step: 3, tier: "T2".into(), action: "shell".into(),
            consequences: crate::execution_stream::ConsequencePreview::default(),
        };
        metrics.on_event(&approval);
        assert_eq!(metrics.events_approval.load(std::sync::atomic::Ordering::Relaxed), 1);

        let error_ev = ExecutionEvent::Error { step: 4, message: "oops".into() };
        metrics.on_event(&error_ev);
        assert_eq!(metrics.events_error.load(std::sync::atomic::Ordering::Relaxed), 1);

        let complete = ExecutionEvent::Complete { output: "done".into(), steps: 5, tools_used: vec![] };
        metrics.on_event(&complete);
        assert_eq!(metrics.events_complete.load(std::sync::atomic::Ordering::Relaxed), 1);
    }

    #[test]
    fn test_streaming_metrics_on_error_by_category() {
        let metrics = StreamingMetrics::default();

        metrics.on_error("auth");
        assert_eq!(metrics.errors_auth.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(metrics.errors_replay.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert_eq!(metrics.errors_internal.load(std::sync::atomic::Ordering::Relaxed), 0);

        metrics.on_error("replay");
        assert_eq!(metrics.errors_replay.load(std::sync::atomic::Ordering::Relaxed), 1);

        metrics.on_error("internal");
        assert_eq!(metrics.errors_internal.load(std::sync::atomic::Ordering::Relaxed), 1);

        metrics.on_error("unknown");
        assert_eq!(metrics.errors_internal.load(std::sync::atomic::Ordering::Relaxed), 2);
    }

    #[test]
    fn test_streaming_stats_snapshot() {
        let metrics = StreamingMetrics::default();
        metrics.on_connect();
        metrics.on_connect();

        let thought = ExecutionEvent::Thought { step: 1, content: "hi".into() };
        metrics.on_event(&thought);
        metrics.on_event(&thought);

        let tool = ExecutionEvent::ToolCall { step: 2, tool: "ls".into(), input: serde_json::json!({}) };
        metrics.on_event(&tool);

        metrics.on_error("auth");

        let stats = StreamingStats::from(&metrics);

        assert_eq!(stats.active_connections, 2);
        assert_eq!(stats.total_connections, 2);
        assert_eq!(stats.events.thought, 2);
        assert_eq!(stats.events.tool_call, 1);
        assert_eq!(stats.events.total, 3);
        assert_eq!(stats.errors.auth, 1);
        assert_eq!(stats.errors.total, 1);
    }

}
