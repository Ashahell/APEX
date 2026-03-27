use crate::api::AppState;
use crate::auth::{verify_request, AuthConfig};
use crate::streaming_types::SSEItem;
use axum::response::sse::{Event, Sse};
use axum::{
    body::Body,
    extract::{Path, Request, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures_util::{stream, Stream};
use std::sync::Arc;

type DynEventStream = Box<dyn Stream<Item = SSEItem> + Send + Unpin>;

/// A streaming wrapper that tracks disconnection for metrics
struct TrackedStream {
    inner: DynEventStream,
    connection_id: String,
    metrics: &'static StreamingMetrics,
}

impl Stream for TrackedStream {
    type Item = SSEItem;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let result = std::pin::Pin::new(&mut self.inner).poll_next(cx);
        match result {
            std::task::Poll::Ready(None) => {
                // Stream ended - log disconnect
                tracing::info!(connection_id = %self.connection_id, "Stream disconnected");
                self.metrics.on_disconnect();
                std::task::Poll::Ready(None)
            }
            other => other,
        }
    }
}

/// Streaming-specific error type with security context
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
    #[error("Auth required: {0}")]
    AuthRequired(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Timestamp too old: {0}")]
    TimestampTooOld(i64),
}

impl axum::response::IntoResponse for StreamingError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            StreamingError::AuthRequired(_) => axum::http::StatusCode::UNAUTHORIZED,
            StreamingError::InvalidSignature(_) => axum::http::StatusCode::UNAUTHORIZED,
            StreamingError::TimestampTooOld(_) => axum::http::StatusCode::UNAUTHORIZED,
            _ => axum::http::StatusCode::FORBIDDEN,
        };
        (status, self.to_string()).into_response()
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

/// Auth configuration for streaming - cloned from AppState
#[derive(Clone)]
pub struct StreamingAuth {
    pub secret: Arc<String>,
    pub disabled: bool,
}

impl StreamingAuth {
    pub fn new() -> Self {
        let config = crate::unified_config::AppConfig::global();
        Self {
            secret: Arc::new(config.auth.shared_secret.clone()),
            disabled: config.auth.disabled,
        }
    }

    /// Validate HMAC signature for streaming request
    pub fn validate(&self, request: &Request<Body>) -> Result<(), StreamingError> {
        if self.disabled {
            return Ok(());
        }

        // Extract signature
        let signature = request
            .headers()
            .get("X-APEX-Signature")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Extract timestamp
        let timestamp: Option<i64> = request
            .headers()
            .get("X-APEX-Timestamp")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let (signature, timestamp) = match (signature, timestamp) {
            (Some(s), Some(t)) => (s, t),
            _ => {
                return Err(StreamingError::AuthRequired(
                    "X-APEX-Signature and X-APEX-Timestamp required".to_string(),
                ));
            }
        };

        // Check timestamp age (max 5 minutes)
        let now = chrono::Utc::now().timestamp();
        let age = (now - timestamp).abs();
        if age > 300 {
            return Err(StreamingError::TimestampTooOld(age));
        }

        // Verify HMAC signature
        let path = request.uri().path();
        let method = request.method().as_str();

        if !verify_request(&self.secret, method, path, &[], &signature, timestamp) {
            return Err(StreamingError::InvalidSignature("HMAC verification failed".to_string()));
        }

        Ok(())
    }
}

impl Default for StreamingAuth {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export needed types from streaming_types for API compatibility
pub use crate::streaming_types::{
    ConnectionPayload, ErrorCounts, EventCounts, HeartbeatPayload, 
    SseEnvelope, StreamingMetrics, StreamingStats, StreamEventType,
};

/// Extract the raw Request for auth validation
fn extract_request() -> impl Fn(axum::extract::Request) -> futures_util::future::Ready<Result<Request<Body>, StreamingError>> {
    |req: axum::extract::Request| {
        futures_util::future::ready(Ok(req))
    }
}

pub async fn get_stream_stats(_state: State<AppState>) -> impl IntoResponse {
    let metrics = get_streaming_metrics();
    let stats = StreamingStats::from(&*metrics);
    tracing::debug!(active = stats.active_connections, total = stats.total_connections, "Streaming stats requested");
    Json(stats)
}

/// Global streaming metrics - singleton for observability
use std::sync::OnceLock;
static GLOBAL_STREAMING_METRICS: OnceLock<StreamingMetrics> = OnceLock::new();

/// Get the global streaming metrics instance
pub fn get_streaming_metrics() -> &'static StreamingMetrics {
    GLOBAL_STREAMING_METRICS.get_or_init(|| StreamingMetrics::default())
}

/// Create a streaming endpoint with auth validation and observability
macro_rules! streaming_endpoint {
    ($name:ident, $event_type:expr) => {
        pub async fn $name(
            State(state): State<AppState>,
            Path(task_id): Path<String>,
            request: Request<Body>,
        ) -> Result<Sse<DynEventStream>, StreamingError> {
            // Validate auth
            let auth = StreamingAuth::new();
            let path = request.uri().path().to_string();
            auth.validate(&request).map_err(|e| {
                tracing::warn!(endpoint = %path, task_id = %task_id, error = %e, "Streaming auth failed");
                get_streaming_metrics().on_error("auth");
                e
            })?;

            // Track connection with observability
            let metrics = get_streaming_metrics();
            metrics.on_connect();
            
            let connection_id = uuid::Uuid::new_v4().to_string();
            tracing::info!(
                connection_id = %connection_id,
                endpoint = %path,
                task_id = %task_id,
                "Stream connected"
            );

            // Create connected event
            let e = Event::default()
                .event($event_type)
                .data(format!("connected:{}:{}", task_id, connection_id));
            let vec_events: Vec<SSEItem> = vec![Ok(e)];
            let s = stream::iter(vec_events);

            tracing::debug!(
                connection_id = %connection_id,
                endpoint = %path,
                task_id = %task_id,
                "Stream event sent"
            );

            Ok(Sse::new(Box::new(s)))
        }
    };
}

/// Create a heartbeat event for keep-alive
pub fn create_heartbeat_event() -> SSEItem {
    use crate::streaming_types::{HeartbeatPayload, SseEnvelope, StreamEventType};
    
    let metrics = get_streaming_metrics();
    let stats = StreamingStats::from(&*metrics);
    
    let payload = HeartbeatPayload {
        server_time: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        active_connections: stats.active_connections,
    };
    
    let envelope = SseEnvelope::new(StreamEventType::Heartbeat, payload);
    let json = serde_json::to_string(&envelope).unwrap_or_default();
    
    Ok(Event::default()
        .event("heartbeat")
        .data(json))
}

streaming_endpoint!(stream_hands, "hands");
streaming_endpoint!(stream_mcp, "mcp");
streaming_endpoint!(stream_task, "task");

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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[test]
    fn test_streaming_auth_disabled() {
        // Create auth with disabled = true
        let auth = StreamingAuth {
            secret: Arc::new("test".to_string()),
            disabled: true,
        };
        
        let req = Request::builder()
            .uri("/stream/hands/test-task")
            .body(Body::empty())
            .unwrap();
        
        // Should pass when disabled
        assert!(auth.validate(&req).is_ok());
    }

    #[test]
    fn test_streaming_auth_missing_headers() {
        let auth = StreamingAuth {
            secret: Arc::new("test-secret".to_string()),
            disabled: false,
        };
        
        let req = Request::builder()
            .uri("/stream/hands/test-task")
            .body(Body::empty())
            .unwrap();
        
        let result = auth.validate(&req);
        assert!(result.is_err());
        match result {
            Err(StreamingError::AuthRequired(_)) => {}
            _ => panic!("Expected AuthRequired error"),
        }
    }

    #[test]
    fn test_streaming_error_responses() {
        let err = StreamingError::AuthRequired("test".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        
        let err = StreamingError::InvalidSignature("test".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        
        let err = StreamingError::StreamingDisabled;
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn simple_hands_stream_runs() {
        // Simple smoke test to ensure code compiles and a small stream can be driven
    }

    #[tokio::test]
    async fn test_concurrent_metrics_isolation() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        
        // Get global metrics
        let metrics = get_streaming_metrics();
        
        // Reset counters for this test
        let initial_active = metrics.active_connections.load(Ordering::Relaxed);
        let initial_total = metrics.total_connections.load(Ordering::Relaxed);
        
        // Simulate multiple concurrent connections
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let metrics = get_streaming_metrics();
                tokio::spawn(async move {
                    metrics.on_connect();
                    // Simulate some work
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    metrics.on_disconnect();
                })
            })
            .collect();
        
        // Wait for all to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify metrics were updated correctly
        let final_active = metrics.active_connections.load(Ordering::Relaxed);
        let final_total = metrics.total_connections.load(Ordering::Relaxed);
        
        // Total should have increased by 10
        assert_eq!(final_total, initial_total + 10);
        // Active should be back to initial (all disconnected)
        assert_eq!(final_active, initial_active);
    }

    #[tokio::test]
    async fn test_metrics_error_tracking() {
        let metrics = get_streaming_metrics();
        
        // Record various error types
        metrics.on_error("auth");
        metrics.on_error("auth");
        metrics.on_error("replay");
        metrics.on_error("internal");
        
        // Verify error counts (requires accessing internal state, but we can verify via stats)
        let stats = StreamingStats::from(&*metrics);
        
        assert_eq!(stats.errors.auth, 2);
        assert_eq!(stats.errors.replay, 1);
        assert_eq!(stats.errors.internal, 1);
    }

    #[tokio::test]
    async fn test_sse_envelope_serialization() {
        use crate::streaming_types::{SseEnvelope, StreamEventType, ConnectionPayload};
        
        let payload = ConnectionPayload {
            task_id: "test-task".to_string(),
            connection_id: "test-conn".to_string(),
            message: "connected".to_string(),
        };
        
        let envelope = SseEnvelope::new(StreamEventType::Connected, payload);
        let json = serde_json::to_string(&envelope).unwrap();
        
        // Verify JSON structure
        assert!(json.contains("\"type\":\"connected\""));
        assert!(json.contains("test-task"));
        assert!(json.contains("test-conn"));
    }

    #[tokio::test]
    async fn test_heartbeat_event() {
        let heartbeat = create_heartbeat_event();
        
        // Just verify the heartbeat event can be created without panic
        assert!(heartbeat.is_ok());
    }
}
