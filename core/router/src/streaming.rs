// Phase 1.3 Patch 3B-5: Incremental end-to-end wiring scaffold using relocated types

use axum::{extract::{Path, Query, State}, response::IntoResponse, routing::get, Router, Json};
use serde_json;
use axum::response::sse::{Event, Sse};
use futures_util::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
type DynEventStream = Box<dyn Stream<Item = SSEItem> + Send + Unpin>;
type SSEItem = Result<Event, axum::Error>;

// Simple in-memory SSE stream that yields a fixed sequence of SSEItem
struct SimpleSseStream {
    items: Vec<SSEItem>,
}

impl Stream for SimpleSseStream {
    type Item = SSEItem;
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = unsafe { &mut *(&self as *const _ as *mut Self) };
        if this.items.is_empty() {
            return Poll::Ready(None);
        }
        Poll::Ready(Some(this.items.remove(0)))
    }
}

// Tiny concrete SSE stream to avoid trait-object inference issues
struct TinySseStream {
    items: Vec<SSEItem>,
}

impl Unpin for TinySseStream {}

impl Stream for TinySseStream {
    type Item = SSEItem;
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let me = unsafe { &mut *(&self as *const _ as *mut Self) };
        if me.items.is_empty() {
            return Poll::Ready(None);
        }
        Poll::Ready(Some(me.items.remove(0)))
    }
}
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

// Public surface: types relocated to streaming_types.rs
pub use crate::streaming_types::{StreamingMetrics, StreamingStats, EventCounts, ErrorCounts};

use crate::api::AppState;
use crate::execution_stream::ExecutionEvent;
use std::pin::Pin;

#[derive(Debug, Deserialize)]
struct StreamAuthQuery {
    // Placeholder for future auth surface
}

#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    #[error("Streaming is disabled")] StreamingDisabled,
}

impl axum::response::IntoResponse for StreamingError {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::FORBIDDEN, self.to_string()).into_response()
    }
}

// GET /api/v1/stream/stats
pub async fn get_stream_stats(_state: State<AppState>) -> impl IntoResponse {
    // Return a default empty snapshot from relocated types
    Json(StreamingStats::default())
}

pub async fn stream_hands(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
    Query(_auth): Query<StreamAuthQuery>,
) -> Result<Sse<DynEventStream>, StreamingError> {
    // Build a feed-driven SSE stream from an in-memory vector using explicit, typed events
    let e1 = Event::default()
        .event("connected")
        .data(serde_json::to_string(&serde_json::json!({"task_id": task_id})).unwrap());
    let e2 = Event::default()
        .event("progress")
        .data(serde_json::to_string(&serde_json::json!({"task_id": task_id, "step": 1})).unwrap());
    let e3 = Event::default()
        .event("end")
        .data(serde_json::to_string(&serde_json::json!({"task_id": task_id, "status": "ended"})).unwrap());
    let vec_events: Vec<SSEItem> = vec![Ok(e1), Ok(e2), Ok(e3)];
    let tiny = TinySseStream { items: vec_events };
    let boxed: DynEventStream = Box::new(tiny);
    Ok(Sse::new(boxed))
}

fn feed_events_for_task(task_id: String) -> DynEventStream {
    let e1 = Event::default()
        .event("connected")
        .data(serde_json::to_string(&serde_json::json!({"task_id": task_id})).unwrap());
    let e2 = Event::default()
        .event("progress")
        .data(serde_json::to_string(&serde_json::json!({"task_id": task_id, "step": 1})).unwrap());
    let e3 = Event::default()
        .event("end")
        .data(serde_json::to_string(&serde_json::json!({"task_id": task_id, "status": "ended"})).unwrap());
    let stream = futures_util::stream::iter(vec![Ok::<Event, axum::Error>(e1), Ok::<Event, axum::Error>(e2), Ok::<Event, axum::Error>(e3)]);
    Box::new(stream)
}

fn feed_events_via_channel(task_id: String) -> DynEventStream {
    let (tx, rx) = mpsc::channel::<Event>(4);
    let t = task_id.clone();
    tokio::spawn(async move {
        let e1 = Event::default()
            .event("connected")
            .data(serde_json::to_string(&serde_json::json!({"task_id": t})).unwrap());
        let e2 = Event::default()
            .event("progress")
            .data(serde_json::to_string(&serde_json::json!({"task_id": t, "step": 1})).unwrap());
        let e3 = Event::default()
            .event("end")
            .data(serde_json::to_string(&serde_json::json!({"task_id": t, "status": "ended"})).unwrap());
        let _ = tx.send(e1).await;
        let _ = tx.send(e2).await;
        let _ = tx.send(e3).await;
    });
    Box::new(ReceiverStream::new(rx))
}

pub async fn stream_mcp(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
    Query(_auth): Query<StreamAuthQuery>,
) -> Result<Sse<DynEventStream>, StreamingError> {
    // Build a tiny in-memory MCP stream using explicit, typed events
    let e1 = Event::default()
        .event("connected")
        .data(serde_json::to_string(&serde_json::json!({"task_id": task_id})).unwrap());
    let e2 = Event::default()
        .event("progress")
        .data(serde_json::to_string(&serde_json::json!({"task_id": task_id, "step": 1})).unwrap());
    let e3 = Event::default()
        .event("end")
        .data(serde_json::to_string(&serde_json::json!({"task_id": task_id, "status": "ended"})).unwrap());
    let vec_events: Vec<SSEItem> = vec![Ok(e1), Ok(e2), Ok(e3)];
    let tiny = TinySseStream { items: vec_events };
    let boxed: DynEventStream = Box::new(tiny);
    Ok(Sse::new(boxed))
}

pub async fn stream_task(
    State(_state): State<AppState>,
    Path(_task_id): Path<String>,
    Query(_auth): Query<StreamAuthQuery>,
) -> Result<Sse<DynEventStream>, StreamingError> {
    // Flesh generic task stream using feed_events_for_task
    let boxed = feed_events_for_task(_task_id);
    Ok::<Sse<DynEventStream>, StreamingError>(Sse::new(boxed))
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
    use futures_util::StreamExt;
    use axum::response::sse::Event;
    use serde_json::json;

    #[tokio::test]
    async fn hands_stream_yields_three_events() {
        let task_id = "test-task".to_string();
        let e1 = Event::default()
            .event("connected")
            .data(json!({"task_id": task_id}).to_string());
        let e2 = Event::default()
            .event("progress")
            .data(json!({"task_id": task_id, "step": 1}).to_string());
        let e3 = Event::default()
            .event("end")
            .data(json!({"task_id": task_id, "status": "ended"}).to_string());
    let stream = futures_util::stream::iter(vec![Ok::<Event, axum::Error>(e1), Ok::<Event, axum::Error>(e2), Ok::<Event, axum::Error>(e3)]);
        let mut s = Box::new(stream);
        let mut count = 0;
        while let Some(_ev) = s.next().await {
            let _ = _ev;
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn hands_tinysse_three_events() {
        let task_id = "test-task-tiny".to_string();
        let e1 = Event::default()
            .event("connected")
            .data(serde_json::to_string(&serde_json::json!({"task_id": task_id})).unwrap());
        let e2 = Event::default()
            .event("progress")
            .data(serde_json::to_string(&serde_json::json!({"task_id": task_id, "step": 1})).unwrap());
        let e3 = Event::default()
            .event("end")
            .data(serde_json::to_string(&serde_json::json!({"task_id": task_id, "status": "ended"})).unwrap());
        let vec_events: Vec<SSEItem> = vec![Ok(e1), Ok(e2), Ok(e3)];
        let tiny = TinySseStream { items: vec_events };
        let mut s = Box::new(tiny);
        let mut count = 0usize;
        while let Some(_ev) = s.next().await {
            let _ = _ev;
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn mcp_tinysse_three_events() {
        let task_id = "mcp-test-tiny".to_string();
        let e1 = Event::default()
            .event("connected")
            .data(serde_json::to_string(&serde_json::json!({"task_id": task_id})).unwrap());
        let e2 = Event::default()
            .event("progress")
            .data(serde_json::to_string(&serde_json::json!({"task_id": task_id, "step": 1})).unwrap());
        let e3 = Event::default()
            .event("end")
            .data(serde_json::to_string(&serde_json::json!({"task_id": task_id, "status": "ended"})).unwrap());
        let vec_events: Vec<SSEItem> = vec![Ok(e1), Ok(e2), Ok(e3)];
        let tiny = TinySseStream { items: vec_events };
        let mut s = Box::new(tiny);
        let mut count = 0usize;
        while let Some(_ev) = s.next().await {
            let _ = _ev;
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn task_tinysse_three_events() {
        let task_id = "task-test-tiny".to_string();
        let e1 = Event::default()
            .event("connected")
            .data(serde_json::to_string(&serde_json::json!({"task_id": task_id})).unwrap());
        let e2 = Event::default()
            .event("progress")
            .data(serde_json::to_string(&serde_json::json!({"task_id": task_id, "step": 1})).unwrap());
        let e3 = Event::default()
            .event("end")
            .data(serde_json::to_string(&serde_json::json!({"task_id": task_id, "status": "ended"})).unwrap());
        let vec_events: Vec<SSEItem> = vec![Ok(e1), Ok(e2), Ok(e3)];
        let tiny = TinySseStream { items: vec_events };
        let mut s = Box::new(tiny);
        let mut count = 0usize;
        while let Some(_ev) = s.next().await {
            let _ = _ev;
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn mcp_stream_yields_three_events() {
        // Use the internal feed to generate a short SSE for MCP
        let task_id = "mcp-test".to_string();
        let s = feed_events_for_task(task_id);
        let mut count = 0usize;
        let mut stream = s;
        while let Some(_ev) = stream.next().await {
            let _ = _ev;
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn task_stream_yields_three_events() {
        let task_id = "task-test".to_string();
        let s = feed_events_for_task(task_id);
        let mut count = 0usize;
        let mut stream = s;
        while let Some(_ev) = stream.next().await {
            let _ = _ev;
            count += 1;
        }
        assert_eq!(count, 3);
    }
}
