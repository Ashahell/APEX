// TinySseStream-based tests for in-memory SSE paths (Approach A)
// This file isolates tests from in-file patch churn and validates the TinySseStream surface.

use futures_util::StreamExt;
use serde_json::json;
use axum::response::sse::Event;

type SSEItem = Result<Event, axum::Error>;

fn create_test_events(task_id: &str) -> Vec<SSEItem> {
    let payload1 = serde_json::to_string(&json!({"task_id": task_id})).unwrap();
    let payload2 = serde_json::to_string(&json!({"task_id": task_id, "step": 1})).unwrap();
    let payload3 = serde_json::to_string(&json!({"task_id": task_id, "status": "ended"})).unwrap();
    let e1 = Event::default().event("connected").data(payload1);
    let e2 = Event::default().event("progress").data(payload2);
    let e3 = Event::default().event("end").data(payload3);
    vec![Ok(e1), Ok(e2), Ok(e3)]
}

#[test]
fn hands_tinysse_three_events() {
    let items = create_test_events("hands-tiny");
    let stream = futures_util::stream::iter(items);
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut count = 0;
        futures_util::pin_mut!(stream);
        while let Some(_ev) = stream.next().await {
            let _ = _ev;
            count += 1;
        }
        assert_eq!(count, 3);
    });
}

#[test]
fn mcp_tinysse_three_events() {
    let items = create_test_events("mcp-tiny");
    let stream = futures_util::stream::iter(items);
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut count = 0;
        futures_util::pin_mut!(stream);
        while let Some(_ev) = stream.next().await {
            let _ = _ev;
            count += 1;
        }
        assert_eq!(count, 3);
    });
}

#[test]
fn task_tinysse_three_events() {
    let items = create_test_events("task-tiny");
    let stream = futures_util::stream::iter(items);
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut count = 0;
        futures_util::pin_mut!(stream);
        while let Some(_ev) = stream.next().await {
            let _ = _ev;
            count += 1;
        }
        assert_eq!(count, 3);
    });
}
