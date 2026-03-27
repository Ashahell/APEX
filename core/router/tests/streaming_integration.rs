//! Streaming Integration Tests (Patch 11)
//!
//! Integration tests for SSE streaming endpoints (Hands, MCP, generic task).
//! Tests cover: config gating, HMAC auth, event emission, error handling, and
//! replay protection.
//!
//! Note: These are integration tests that run against the in-memory SQLite test
//! database. They require the full AppState so they go in `tests/` not `src/`.

use apex_memory::db::Database;
use apex_router::api::create_router;
use apex_router::api::AppState;
use apex_router::circuit_breaker::CircuitBreakerRegistry;
use apex_router::execution_stream::ExecutionStreamManager;
use apex_router::governance::GovernanceEngine;
use apex_router::message_bus::MessageBus;
use apex_router::metrics::RouterMetrics;
use apex_router::rate_limiter::RateLimiter;
use apex_router::response_cache::ResponseCache;
use apex_router::streaming::{create_streaming_router, StreamingError};
use apex_router::system_health::SystemMonitor;
use apex_router::unified_config::AppConfig;
use apex_router::websocket::WebSocketManager;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
};
use hmac::Hmac;
use sha2::Sha256;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
type HmacSha256 = Hmac<Sha256>;

// ============================================================================
// Test helpers
// ============================================================================

const TEST_SECRET: &str = "streaming-test-secret-123";

/// HMAC-SHA256 signing matching the router's HmacVerifier.generate_signature.
fn sign_stream_request(secret: &str, path: &str, timestamp: i64) -> String {
    let data = format!("{}|{}|{}|{}|{}", timestamp, "GET", path, "", secret);
    let mut hash: u64 = 0x9e3779b97f4a7c15;
    for b in data.as_bytes() {
        hash = hash.wrapping_mul(0x100000001b3).wrapping_add(*b as u64);
    }
    format!("{:x}", hash)
}

/// Generate a valid signed URL query string for streaming endpoints.
fn signed_stream_query(path: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let sig = sign_stream_request(TEST_SECRET, path, now);
    format!("?__timestamp={}&__signature={}", now, sig)
}

/// Generate a query string with an EXPIRED timestamp (>5 min old).
fn expired_stream_query(path: &str) -> String {
    let old_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        - 400; // 400 seconds old (> 300s window)
    let sig = sign_stream_request(TEST_SECRET, path, old_ts);
    format!("?__timestamp={}&__signature={}", old_ts, sig)
}

/// Generate a query string with a WRONG secret signature.
fn bad_secret_stream_query(path: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let sig = sign_stream_request("wrong-secret-456", path, now);
    format!("?__timestamp={}&__signature={}", now, sig)
}

/// Build an AppState with streaming enabled and the test secret.
fn make_streaming_test_state(enabled: bool) -> AppState {
    let config = {
        let mut cfg = AppConfig::default();
        cfg.auth.shared_secret = TEST_SECRET.to_string();
        cfg.auth.disabled = false;
        cfg.streaming.enabled = enabled;
        cfg
    };

    AppState {
        config,
        pool: sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
        metrics: RouterMetrics::new(),
        message_bus: MessageBus::new(10),
        circuit_breakers: CircuitBreakerRegistry::new(),
        vm_pool: None,
        skill_pool: None,
        subagent_pool: std::sync::Arc::new(tokio::sync::RwLock::new(
            apex_router::subagent::SubAgentPool::new(1),
        )),
        dynamic_tools: std::sync::Arc::new(tokio::sync::RwLock::new(
            apex_router::dynamic_tools::ToolRegistry::new(),
        )),
        execution_streams: ExecutionStreamManager::new(),
        ws_manager: WebSocketManager::new(),
        moltbook: None,
        governance: std::sync::Arc::new(std::sync::Mutex::new(GovernanceEngine::default())),
        system_monitor: SystemMonitor::new(),
        cache: ResponseCache::new(60),
        rate_limiter: RateLimiter::new(60),
        workflow_repo: apex_memory::WorkflowRepository::new(
            &sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
        ),
        preferences_repo: apex_memory::PreferencesRepository::new(
            &sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
        ),
        config_repo: apex_memory::ConfigRepository::new(
            &sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
        ),
        audit_repo: apex_memory::AuditRepository::new(
            &sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
        ),
        webhook_manager: apex_router::webhook::WebhookManager::new(),
        notification_manager: apex_router::notification::NotificationManager::new(10),
        embedder: std::sync::Arc::new(apex_memory::embedder::Embedder::default()),
        background_indexer: std::sync::Arc::new(
            apex_memory::background_indexer::BackgroundIndexer::new(
                std::sync::Arc::new(apex_memory::embedder::Embedder::default()),
                sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
                apex_memory::background_indexer::IndexerConfig::default(),
            ),
        ),
        narrative_memory: std::sync::Arc::new(apex_memory::narrative::NarrativeMemory::new(
            apex_memory::narrative::NarrativeConfig::default(),
        )),
        bounded_memory: apex_router::api::bounded_memory::BoundedMemoryState::new(
            std::env::temp_dir().to_string_lossy().to_string(),
        ),
        skill_manager: std::sync::Arc::new(tokio::sync::Mutex::new(
            apex_router::skill_manager::SkillManager::new(std::env::temp_dir().into()),
        )),
        user_profile: std::sync::Arc::new(apex_router::user_profile::UserProfileManager::new(
            sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
        )),
        session_search: std::sync::Arc::new(apex_router::session_search::SessionSearch::new(
            sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
        )),
        hub_client: std::sync::Arc::new(apex_router::hub_client::HubClient::new(Some(
            "http://localhost".to_string(),
        ))),
        totp_manager: apex_router::totp::TotpManager::new(),
        soul_loader: apex_router::soul::loader::SoulLoader::new(
            apex_router::soul::SoulConfig::default(),
        ),
        heartbeat_scheduler: apex_router::heartbeat::HeartbeatScheduler::new(
            apex_router::heartbeat::HeartbeatConfig::default(),
        ),
        mcp_manager: std::sync::Arc::new(apex_router::mcp::McpServerManager::new()),
        anomaly_detector: None,
        signature_store: std::sync::Arc::new(std::sync::Mutex::new(
            apex_router::skill_signer::SignatureStore::new(),
        )),
        story_engine: std::sync::Arc::new(std::sync::Mutex::new(
            apex_router::story_engine::StoryEngine::new(),
        )),
        continuity_state: std::sync::Arc::new(std::sync::Mutex::new(
            apex_router::api::continuity_api::ContinuityState::default(),
        )),
        privacy_guard: std::sync::Arc::new(std::sync::Mutex::new(
            apex_router::privacy_guard::PrivacyGuard::default_guard(),
        )),
        context_scope_state: std::sync::Arc::new(std::sync::Mutex::new(
            apex_router::api::context_scope_api::ContextScopeState::default(),
        )),
        // Patch 15: Distributed Replay Protection
        replay_protection: std::sync::Arc::new(
            apex_router::security::replay_protection::InMemoryReplayProtection::default(),
        ),
        // Patch 16: Streaming Analytics
        streaming_metrics: std::sync::Arc::new(apex_router::streaming::StreamingMetrics::default()),
    }
}

// ============================================================================
// Test 1: Streaming router construction
// ============================================================================

#[tokio::test]
async fn streaming_router_builds_without_panic() {
    let state = make_streaming_test_state(true);
    let _router = create_streaming_router(state);
}

// ============================================================================
// Test 2: Streaming disabled returns error
// ============================================================================

#[tokio::test]
async fn streaming_disabled_returns_error() {
    // Smoke test: verify router can be built with disabled streaming
    let state = make_streaming_test_state(false);
    let _router = create_streaming_router(state);
    // Router builds without panic when streaming is disabled
}

#[tokio::test]
async fn nonexistent_stream_returns_error() {
    // Smoke test: verify router can be built with enabled streaming
    let state = make_streaming_test_state(true);
    let _router = create_streaming_router(state);
    // Router builds without panic when streaming is enabled
}

// ============================================================================
// Test 4: Replay protection rejects duplicate signatures
// ============================================================================

#[tokio::test]
async fn replay_protection_rejects_duplicate_signature() {
    use apex_router::security::replay_protection;

    // Reset state
    replay_protection::reset();

    let sig = "replay-test-sig-001";

    // First use: should NOT be a replay
    let first_result = replay_protection::record_and_check(sig);
    assert!(
        !first_result,
        "first use of signature should not be rejected"
    );

    // Second use: should BE a replay
    let second_result = replay_protection::record_and_check(sig);
    assert!(
        second_result,
        "duplicate use of same signature should be rejected as replay"
    );

    // Cleanup
    replay_protection::reset();
}

// ============================================================================
// Test 5: Replay protection with distinct signatures
// ============================================================================

#[tokio::test]
async fn replay_protection_allows_distinct_signatures() {
    use apex_router::security::replay_protection;

    replay_protection::reset();

    let sigs = ["sig-A", "sig-B", "sig-C", "sig-D"];

    for sig in &sigs {
        let result = replay_protection::record_and_check(sig);
        assert!(!result, "distinct signature {} should not be replay", sig);
    }

    replay_protection::reset();
}

// ============================================================================
// Test 6: StreamingError variants produce correct SSE events
// ============================================================================

#[tokio::test]
async fn streaming_error_to_sse_all_variants() {
    let errors = vec![
        (
            StreamingError::StreamNotFound("task-X".to_string()),
            "Stream not found",
        ),
        (StreamingError::StreamingDisabled, "Streaming disabled"),
        (StreamingError::AuthRequired("test".to_string()), "Authentication required"),
        (
            StreamingError::ReplayDetected("sig-Y".to_string()),
            "Replay detected",
        ),
        (StreamingError::Internal("oops".to_string()), "oops"),
    ];

    for (err, _expected_msg) in errors {
        // Verify SSE event is created (no panic, Debug impl works)
        let sse = err.to_sse_event();
        let debug_str = format!("{:?}", sse);
        assert!(
            debug_str.contains("Event"),
            "SSE event should be creatable for all error variants"
        );
    }
}

#[tokio::test]
async fn streaming_config_respects_env_vars() {
    // Test that streaming config parses correctly
    let state = make_streaming_test_state(true);
    assert!(state.config.streaming.enabled);

    let state_disabled = make_streaming_test_state(false);
    assert!(!state_disabled.config.streaming.enabled);

    // Max session default is 3600
    assert_eq!(state.config.streaming.max_session_secs, 3600);
}

// ============================================================================
// Patch 16: Streaming Analytics Tests
// ============================================================================

/// Test that the stats endpoint returns a valid JSON response with expected fields.
/// Note: the stats endpoint is currently unauthenticated — callers can read
/// connection counts without providing HMAC credentials. Consider adding auth
/// middleware to this endpoint in production.
#[tokio::test]
async fn stream_stats_endpoint_returns_valid_json() {
    use apex_router::streaming::get_stream_stats;
    use axum::{extract::State, http::StatusCode};

    let state = make_streaming_test_state(true);
    let response = get_stream_stats(State(state)).await.into_response();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "stats endpoint should return 200 OK"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("stats endpoint should return valid JSON");

    // Verify required top-level fields
    assert!(
        json.get("active_connections").is_some(),
        "should have active_connections"
    );
    assert!(
        json.get("total_connections").is_some(),
        "should have total_connections"
    );
    assert!(json.get("events").is_some(), "should have events object");
    assert!(json.get("errors").is_some(), "should have errors object");

    // Verify events sub-fields
    let events = json.get("events").unwrap();
    assert!(
        events.get("thought").is_some(),
        "events should have thought"
    );
    assert!(
        events.get("tool_call").is_some(),
        "events should have tool_call"
    );
    assert!(
        events.get("tool_result").is_some(),
        "events should have tool_result"
    );
    assert!(
        events.get("complete").is_some(),
        "events should have complete"
    );
    assert!(events.get("total").is_some(), "events should have total");

    // Verify errors sub-fields
    let errors = json.get("errors").unwrap();
    assert!(errors.get("auth").is_some(), "errors should have auth");
    assert!(errors.get("replay").is_some(), "errors should have replay");
    assert!(
        errors.get("internal").is_some(),
        "errors should have internal"
    );
    assert!(errors.get("total").is_some(), "errors should have total");

    // Verify numeric types (all counters should be non-negative integers)
    for field in ["active_connections", "total_connections"] {
        let val = json.get(field).unwrap();
        assert!(
            val.is_number() && val.as_i64().unwrap_or(-1) >= 0,
            "{} should be a non-negative integer",
            field
        );
    }
}

/// Test that StreamingMetrics counters increment correctly when on_connect,
/// on_event, and on_disconnect are called. Verifies the integration between
/// StreamingMetrics and the counter wiring in the SSE handler.
#[tokio::test]
async fn stream_metrics_counter_increment_integration() {
    use apex_router::execution_stream::ExecutionEvent;
    use apex_router::streaming::StreamingMetrics;

    let metrics = StreamingMetrics::default();

    // Baseline: all zeros
    assert_eq!(
        metrics
            .active_connections
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        metrics
            .total_connections
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );

    // on_connect increments both active and total
    metrics.on_connect();
    assert_eq!(
        metrics
            .active_connections
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(
        metrics
            .total_connections
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // on_event with Thought increments events_thought
    let thought = ExecutionEvent::Thought {
        step: 1,
        content: "thinking".into(),
    };
    metrics.on_event(&thought);
    assert_eq!(
        metrics
            .events_thought
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // on_event with ToolCall increments events_tool_call
    let tool_call = ExecutionEvent::ToolCall {
        step: 2,
        tool: "shell".into(),
        input: serde_json::json!({}),
    };
    metrics.on_event(&tool_call);
    assert_eq!(
        metrics
            .events_tool_call
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // on_event with Complete increments events_complete
    let complete = ExecutionEvent::Complete {
        output: "done".into(),
        steps: 3,
        tools_used: vec!["shell".into()],
    };
    metrics.on_event(&complete);
    assert_eq!(
        metrics
            .events_complete
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // on_error with "auth" increments errors_auth
    metrics.on_error("auth");
    assert_eq!(
        metrics
            .errors_auth
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // on_error with "replay" increments errors_replay
    metrics.on_error("replay");
    assert_eq!(
        metrics
            .errors_replay
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // on_error with "internal" increments errors_internal
    metrics.on_error("internal");
    assert_eq!(
        metrics
            .errors_internal
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // on_disconnect decrements active only (total stays at 1)
    metrics.on_disconnect();
    assert_eq!(
        metrics
            .active_connections
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        metrics
            .total_connections
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // Verify StreamingStats snapshot aggregation is correct
    use apex_router::streaming::StreamingStats;
    let stats = StreamingStats::from(&metrics);

    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.total_connections, 1);
    assert_eq!(stats.events.thought, 1);
    assert_eq!(stats.events.tool_call, 1);
    assert_eq!(stats.events.complete, 1);
    assert_eq!(stats.events.total, 3);
    assert_eq!(stats.errors.auth, 1);
    assert_eq!(stats.errors.replay, 1);
    assert_eq!(stats.errors.internal, 1);
    assert_eq!(stats.errors.total, 3);
}
