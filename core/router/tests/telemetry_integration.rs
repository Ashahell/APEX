//! Telemetry Integration Tests (Phase 2)
//!
//! Integration tests for per-endpoint latency and error rate tracking.

use apex_router::metrics::{
    EndpointErrorTracker, EndpointLatencyTracker, ErrorStats, LatencyStats, RouterMetrics,
    TelemetrySurface,
};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Test 1: EndpointLatencyTracker records and computes stats
// ============================================================================

#[tokio::test]
async fn telemetry_latency_tracker_records_and_computes() {
    let tracker = EndpointLatencyTracker::new(100);

    // Record some latencies
    tracker.record(10).await;
    tracker.record(20).await;
    tracker.record(30).await;
    tracker.record(40).await;
    tracker.record(50).await;

    let stats = tracker.get_stats().await;

    assert_eq!(stats.count, 5);
    assert_eq!(stats.min_ms, 10);
    assert_eq!(stats.max_ms, 50);
    assert_eq!(stats.avg_ms, 30); // (10+20+30+40+50)/5 = 30
    assert_eq!(stats.p50_ms, 30);
    assert!(stats.p95_ms >= 30);
    assert!(stats.p99_ms >= 30);
}

#[tokio::test]
async fn telemetry_latency_tracker_empty_stats() {
    let tracker = EndpointLatencyTracker::new(100);
    let stats = tracker.get_stats().await;

    assert_eq!(stats.count, 0);
    assert_eq!(stats.avg_ms, 0);
    assert_eq!(stats.min_ms, 0);
    assert_eq!(stats.max_ms, 0);
}

#[tokio::test]
async fn telemetry_latency_tracker_respects_max_samples() {
    let tracker = EndpointLatencyTracker::new(3);

    tracker.record(100).await;
    tracker.record(200).await;
    tracker.record(300).await;
    tracker.record(400).await; // Should evict 100

    let stats = tracker.get_stats().await;
    assert_eq!(stats.count, 3);
    assert_eq!(stats.min_ms, 200); // 100 was evicted
    assert_eq!(stats.max_ms, 400);
}

// ============================================================================
// Test 2: EndpointErrorTracker records and computes stats
// ============================================================================

#[tokio::test]
async fn telemetry_error_tracker_records_and_computes() {
    let tracker = EndpointErrorTracker::new();

    tracker.record_request().await;
    tracker.record_request().await;
    tracker.record_request().await;
    tracker.record_request().await;
    tracker.record_request().await;

    tracker.record_error("5xx").await;
    tracker.record_error("4xx").await;

    let stats = tracker.get_stats().await;

    assert_eq!(stats.requests, 5);
    assert_eq!(stats.errors, 2);
    assert!((stats.error_rate_pct - 40.0).abs() < 0.01); // 2/5 = 40%
    assert_eq!(stats.error_types.get("5xx"), Some(&1));
    assert_eq!(stats.error_types.get("4xx"), Some(&1));
}

#[tokio::test]
async fn telemetry_error_tracker_no_errors() {
    let tracker = EndpointErrorTracker::new();

    tracker.record_request().await;
    tracker.record_request().await;

    let stats = tracker.get_stats().await;

    assert_eq!(stats.requests, 2);
    assert_eq!(stats.errors, 0);
    assert!((stats.error_rate_pct - 0.0).abs() < 0.01);
}

#[tokio::test]
async fn telemetry_error_tracker_no_requests() {
    let tracker = EndpointErrorTracker::new();
    let stats = tracker.get_stats().await;

    assert_eq!(stats.requests, 0);
    assert_eq!(stats.errors, 0);
    assert!((stats.error_rate_pct - 0.0).abs() < 0.01);
}

// ============================================================================
// Test 3: TelemetrySurface aggregates per-endpoint data
// ============================================================================

#[tokio::test]
async fn telemetry_surface_tracks_multiple_endpoints() {
    let surface = TelemetrySurface::new();

    // Record latency for /api/v1/tasks
    surface.record_latency("/api/v1/tasks", 50).await;
    surface.record_latency("/api/v1/tasks", 100).await;
    surface.record_request("/api/v1/tasks").await;
    surface.record_request("/api/v1/tasks").await;

    // Record latency for /api/v1/stream/stats
    surface.record_latency("/api/v1/stream/stats", 10).await;
    surface.record_request("/api/v1/stream/stats").await;
    surface.record_error("/api/v1/stream/stats", "5xx").await;

    let snapshot = surface.get_snapshot().await;

    assert!(snapshot.endpoint_latencies.contains_key("/api/v1/tasks"));
    assert!(snapshot.endpoint_latencies.contains_key("/api/v1/stream/stats"));
    assert!(snapshot.endpoint_errors.contains_key("/api/v1/stream/stats"));

    let tasks_latency = &snapshot.endpoint_latencies["/api/v1/tasks"];
    assert_eq!(tasks_latency.count, 2);
    assert_eq!(tasks_latency.avg_ms, 75); // (50+100)/2

    let stream_errors = &snapshot.endpoint_errors["/api/v1/stream/stats"];
    assert_eq!(stream_errors.errors, 1);
    assert_eq!(stream_errors.error_types.get("5xx"), Some(&1));
}

// ============================================================================
// Test 4: RouterMetrics includes telemetry
// ============================================================================

#[tokio::test]
async fn router_metrics_includes_telemetry_surface() {
    let metrics = RouterMetrics::new();

    // Record some telemetry data
    metrics
        .telemetry()
        .record_latency("/api/v1/tasks", 100)
        .await;
    metrics.telemetry().record_request("/api/v1/tasks").await;
    metrics
        .telemetry()
        .record_error("/api/v1/tasks", "4xx")
        .await;

    let snapshot = metrics.get_metrics().await;

    assert!(snapshot.telemetry.is_some());
    let telemetry = snapshot.telemetry.unwrap();
    assert!(telemetry.endpoint_latencies.contains_key("/api/v1/tasks"));
    assert!(telemetry.endpoint_errors.contains_key("/api/v1/tasks"));
}

// ============================================================================
// Test 5: Percentile calculations are correct
// ============================================================================

#[tokio::test]
async fn telemetry_percentile_calculations() {
    let tracker = EndpointLatencyTracker::new(1000);

    // Record 100 values: 1, 2, 3, ..., 100
    for i in 1..=100 {
        tracker.record(i).await;
    }

    let stats = tracker.get_stats().await;

    assert_eq!(stats.count, 100);
    assert_eq!(stats.min_ms, 1);
    assert_eq!(stats.max_ms, 100);
    assert_eq!(stats.avg_ms, 50); // (1+100)/2 = 50.5, truncated to 50
    assert!(stats.p50_ms >= 50 && stats.p50_ms <= 51);
    assert!(stats.p95_ms >= 95);
    assert!(stats.p99_ms >= 99);
}
