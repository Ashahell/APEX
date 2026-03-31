//! Telemetry middleware for per-endpoint latency and error tracking (Phase 2)
//!
//! Records latency histograms and error rates for all API endpoints.
//! Data is exposed via `/api/v1/metrics` in the `telemetry` section.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::Layer;
use tower::Service;

use crate::metrics::RouterMetrics;

/// Layer that adds telemetry tracking to each request
#[derive(Clone)]
pub struct TelemetryLayer {
    metrics: RouterMetrics,
}

impl TelemetryLayer {
    pub fn new(metrics: RouterMetrics) -> Self {
        Self { metrics }
    }
}

impl<S> Layer<S> for TelemetryLayer {
    type Service = TelemetryService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TelemetryService {
            inner,
            metrics: self.metrics.clone(),
        }
    }
}

/// Service that records telemetry for each request
#[derive(Clone)]
pub struct TelemetryService<S> {
    inner: S,
    metrics: RouterMetrics,
}

impl<S> Service<Request<Body>> for TelemetryService<S>
where
    S: Service<Request<Body>, Response = axum::response::Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = axum::response::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let metrics = self.metrics.clone();
        let endpoint = req.uri().path().to_string();
        let start = Instant::now();

        let future = self.inner.call(req);

        Box::pin(async move {
            let response = future.await?;
            let elapsed_ms = start.elapsed().as_millis() as u64;
            let status = response.status();

            // Normalize endpoint path (replace dynamic segments)
            let normalized = normalize_endpoint(&endpoint);

            // Record latency
            metrics
                .telemetry()
                .record_latency(&normalized, elapsed_ms)
                .await;

            // Record request and potential error
            metrics.telemetry().record_request(&normalized).await;

            if status.is_server_error() {
                metrics
                    .telemetry()
                    .record_error(&normalized, "5xx")
                    .await;
            } else if status.is_client_error() {
                metrics
                    .telemetry()
                    .record_error(&normalized, "4xx")
                    .await;
            }

            Ok(response)
        })
    }
}

/// Normalize endpoint path by replacing ULIDs, UUIDs, and numeric IDs with :id
fn normalize_endpoint(path: &str) -> String {
    let mut parts: Vec<&str> = path.split('/').collect();

    for part in parts.iter_mut().skip(1) {
        // Skip empty segments from leading slash
        if part.is_empty() {
            continue;
        }

        // Check if it looks like a ULID (26 chars, alphanumeric)
        if part.len() == 26 && part.chars().all(|c| c.is_alphanumeric()) {
            *part = ":id";
        }
        // Check if it looks like a UUID
        else if part.len() == 36
            && part.chars().all(|c| c.is_alphanumeric() || c == '-')
            && part.matches('-').count() == 4
        {
            *part = ":id";
        }
        // Check if it's purely numeric
        else if !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()) {
            *part = ":id";
        }
    }

    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_endpoint() {
        assert_eq!(
            normalize_endpoint("/api/v1/tasks/01ARZ3NDEKTSV4RRFFQ69G5FAV"),
            "/api/v1/tasks/:id"
        );

        assert_eq!(
            normalize_endpoint("/api/v1/tasks"),
            "/api/v1/tasks"
        );

        assert_eq!(
            normalize_endpoint("/api/v1/stream/hands/test-task"),
            "/api/v1/stream/hands/test-task"
        );

        assert_eq!(
            normalize_endpoint("/api/v1/mcp/servers/123/tools"),
            "/api/v1/mcp/servers/:id/tools"
        );
    }
}
