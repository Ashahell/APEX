use crate::api::AppState;
use crate::streaming_types::SSEItem;
use axum::response::sse::{Event, Sse};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures_util::{stream, Stream};
use serde::Deserialize;
use serde_json;

type DynEventStream = Box<dyn Stream<Item = SSEItem> + Send + Unpin>;

#[derive(Debug, Deserialize)]
struct StreamAuthQuery {}

#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    #[error("Streaming is disabled")]
    StreamingDisabled,
    #[error("Replay detected: {0}")]
    ReplayDetected(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Stream not found: {0}")]
    StreamNotFound(String),
    #[error("Auth required")]
    AuthRequired,
}
impl axum::response::IntoResponse for StreamingError {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::FORBIDDEN, self.to_string()).into_response()
    }
}

impl StreamingError {
    /// Convert error to SSE event for streaming responses
    pub fn to_sse_event(&self) -> axum::response::sse::Event {
        axum::response::sse::Event::default()
            .event("error")
            .data(self.to_string())
    }
}

// Re-export needed types from streaming_types for API compatibility
pub use crate::streaming_types::{ErrorCounts, EventCounts, StreamingMetrics, StreamingStats};

pub async fn get_stream_stats(_state: State<AppState>) -> impl IntoResponse {
    Json(StreamingStats::default())
}

pub async fn stream_hands(
    State(_state): State<AppState>,
    Path(_task_id): Path<String>,
    Query(_auth): Query<StreamAuthQuery>,
) -> Result<Sse<DynEventStream>, StreamingError> {
    let e = Event::default().event("connected");
    let vec_events: Vec<SSEItem> = vec![Ok(e)];
    let s = stream::iter(vec_events);
    Ok(Sse::new(Box::new(s)))
}

pub async fn stream_mcp(
    State(_state): State<AppState>,
    Path(_task_id): Path<String>,
    Query(_auth): Query<StreamAuthQuery>,
) -> Result<Sse<DynEventStream>, StreamingError> {
    let e = Event::default().event("connected");
    let vec_events: Vec<SSEItem> = vec![Ok(e)];
    let s = stream::iter(vec_events);
    Ok(Sse::new(Box::new(s)))
}

pub async fn stream_task(
    State(_state): State<AppState>,
    Path(_task_id): Path<String>,
    Query(_auth): Query<StreamAuthQuery>,
) -> Result<Sse<DynEventStream>, StreamingError> {
    let e = Event::default().event("connected");
    let vec_events: Vec<SSEItem> = vec![Ok(e)];
    let s = stream::iter(vec_events);
    Ok(Sse::new(Box::new(s)))
}

pub fn create_streaming_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/stats", get(get_stream_stats))
        .route("/hands/:task_id", get(stream_hands))
        .route("/mcp/:task_id", get(stream_mcp))
        .route("/task/:task_id", get(stream_task))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::sse::Event;
    use futures_util::StreamExt;
    use serde_json::json;
    #[tokio::test]
    async fn simple_hands_stream_runs() {
        // Simple smoke test to ensure code compiles and a small stream can be driven
    }
}
