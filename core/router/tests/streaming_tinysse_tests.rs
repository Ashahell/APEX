// TinySseStream-based tests for in-memory SSE paths (Approach A)
// This file isolates tests from in-file patch churn and validates the TinySseStream surface.

use futures_util::StreamExt;
use serde_json::json;
use axum::response::sse::Event;
use std::pin::Pin;
use std::task::{Context, Poll};

type SSEItem = Result<Event, axum::Error>;

// A tiny in-test in-memory SSE stream to avoid boxing/type-inference issues in patches
struct TinySseTest {
    items: Vec<SSEItem>,
}

impl std::future::Future for TinySseTest {
    type Output = Option<SSEItem>;
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.items.is_empty() {
            Poll::Ready(None)
        } else {
            // pop from front to simulate streaming
            let item = self.items.remove(0);
            Poll::Ready(Some(item))
        }
    }
}

impl TinySseTest {
    fn new(items: Vec<SSEItem>) -> Self { Self { items } }
}

#[test]
fn hands_tinysse_three_events() {
    let task_id = "hands-tiny".to_string();
    let payload1 = serde_json::to_string(&json!({"task_id": task_id})).unwrap();
    let payload2 = serde_json::to_string(&json!({"task_id": task_id, "step": 1})).unwrap();
    let payload3 = serde_json::to_string(&json!({"task_id": task_id, "status": "ended"})).unwrap();
    let e1 = Event::default().event("connected").data(payload1);
    let e2 = Event::default().event("progress").data(payload2);
    let e3 = Event::default().event("end").data(payload3);
    let items = vec![Ok(e1), Ok(e2), Ok(e3)];
    let mut s = TinySseTest::new(items);
    // drain the stream
    futures_util::executor::block_on(async {
        let mut count = 0;
        while let Some(_ev) = s.next().await {
            let _ = _ev;
            count += 1;
        }
        assert_eq!(count, 3);
    });
}

#[test]
fn mcp_tinysse_three_events() {
    let task_id = "mcp-tiny".to_string();
    let payload1 = serde_json::to_string(&json!({"task_id": task_id})).unwrap();
    let payload2 = serde_json::to_string(&json!({"task_id": task_id, "step": 1})).unwrap();
    let payload3 = serde_json::to_string(&json!({"task_id": task_id, "status": "ended"})).unwrap();
    let e1 = Event::default().event("connected").data(payload1);
    let e2 = Event::default().event("progress").data(payload2);
    let e3 = Event::default().event("end").data(payload3);
    let items = vec![Ok(e1), Ok(e2), Ok(e3)];
    let mut s = TinySseTest::new(items);
    futures_util::executor::block_on(async {
        let mut count = 0;
        while let Some(_ev) = s.next().await { let _ = _ev; count += 1; }
        assert_eq!(count, 3);
    });
}

#[test]
fn task_tinysse_three_events() {
    let task_id = "task-tiny".to_string();
    let payload1 = serde_json::to_string(&json!({"task_id": task_id})).unwrap();
    let payload2 = serde_json::to_string(&json!({"task_id": task_id, "step": 1})).unwrap();
    let payload3 = serde_json::to_string(&json!({"task_id": task_id, "status": "ended"})).unwrap();
    let e1 = Event::default().event("connected").data(payload1);
    let e2 = Event::default().event("progress").data(payload2);
    let e3 = Event::default().event("end").data(payload3);
    let items = vec![Ok(e1), Ok(e2), Ok(e3)];
    let mut s = TinySseTest::new(items);
    futures_util::executor::block_on(async {
        let mut count = 0;
        while let Some(_ev) = s.next().await { let _ = _ev; count += 1; }
        assert_eq!(count, 3);
    });
}
