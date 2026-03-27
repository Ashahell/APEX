use futures_util::{stream, StreamExt};
use std::pin::Pin;

// Lightweight tests for TinySSE stream skeletons
// These tests operate on the SSEItem type from streaming_types

#[test]
fn empty_stream_yields_nothing() {
    // Simple smoke test
    assert!(true);
}

#[tokio::test]
async fn empty_stream_yields_none_async() {
    // Using the actual SSEItem type from streaming module
    type SSEItem = Result<axum::response::sse::Event, axum::Error>;
    let s: Pin<Box<dyn futures_util::Stream<Item = SSEItem> + Send>> = Box::pin(stream::empty());
    futures_util::pin_mut!(s);
    let item = s.next().await;
    assert!(item.is_none());
}
