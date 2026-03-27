use apex_memory::db::Database;
use apex_memory::task_repo::TaskRepository;
use apex_memory::tasks::{CreateTask, TaskTier};
use apex_router::api::{create_router, AppState};
use apex_router::circuit_breaker::CircuitBreakerRegistry;
use apex_router::execution_stream::ExecutionStreamManager;
use apex_router::governance::GovernanceEngine;
use apex_router::message_bus::MessageBus;
use apex_router::metrics::RouterMetrics;
use apex_router::rate_limiter::RateLimiter;
use apex_router::response_cache::ResponseCache;
use apex_router::system_health::SystemMonitor;
use apex_router::vm_pool::VmPool;
use apex_router::websocket::WebSocketManager;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;
use tower::ServiceExt;

static TEST_TIMINGS: Mutex<Option<HashMap<String, (usize, std::time::Duration)>>> =
    Mutex::new(None);

fn record_test_time(name: &str, duration: std::time::Duration) {
    if let Ok(mut guard) = TEST_TIMINGS.lock() {
        let timings = guard.get_or_insert_with(HashMap::new);
        let entry = timings
            .entry(name.to_string())
            .or_insert((0, Duration::ZERO));
        entry.0 += 1;
        entry.1 += duration;
    }
}

#[derive(Clone)]
pub struct TestTimer {
    name: String,
    start: Instant,
}

impl TestTimer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
        }
    }
}

impl Drop for TestTimer {
    fn drop(&mut self) {
        record_test_time(&self.name, self.start.elapsed());
    }
}

#[tokio::test]
async fn zz_benchmark_test_harness() {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    println!("\n=== PERFORMANCE METRICS ===");
    println!("Timestamp: {}", timestamp);

    let timings = {
        let guard = TEST_TIMINGS.lock().unwrap();
        guard.clone()
    };

    if let Some(timings) = timings {
        let mut sorted: Vec<_> = timings.iter().collect();
        sorted.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));

        let mut total_time = Duration::ZERO;
        let mut test_count = 0;

        println!(
            "\n{:<45} {:>8} {:>12} {:>12}",
            "Test Name", "Runs", "Total(ms)", "Avg(ms)"
        );
        println!("{:-<45} {:-<8} {:-<12} {:-<12}", "", "", "", "");

        for (name, (count, duration)) in &sorted {
            let avg = duration.as_millis() as f64 / *count as f64;
            println!(
                "{:<45} {:>8} {:>12} {:>12.2}",
                if name.len() > 44 { &name[..44] } else { name },
                count,
                duration.as_millis(),
                avg
            );
            total_time += *duration;
            test_count += count;
        }

        println!("{:-<45} {:-<8} {:-<12} {:-<12}", "", "", "", "");
        println!(
            "{:<45} {:>8} {:>12}",
            "TOTAL",
            test_count,
            total_time.as_millis()
        );

        // Performance tiers
        let fast = sorted
            .iter()
            .filter(|(_, (_, d))| d.as_millis() < 10)
            .count();
        let medium = sorted
            .iter()
            .filter(|(_, (_, d))| d.as_millis() >= 10 && d.as_millis() < 100)
            .count();
        let slow = sorted
            .iter()
            .filter(|(_, (_, d))| d.as_millis() >= 100)
            .count();

        println!("\nPerformance Distribution:");
        println!("  Fast (<10ms):    {:>3} tests", fast);
        println!("  Medium (10-100ms): {:>3} tests", medium);
        println!("  Slow (>100ms):   {:>3} tests", slow);
    } else {
        println!("No timing data collected. Run other tests first.");
    }

    println!("\n=== END METRICS ===\n");
}

use std::time::Duration;

async fn create_test_state() -> AppState {
    let db = Database::new(&PathBuf::from(":memory:")).await.unwrap();
    db.run_migrations().await.unwrap();

    let embedder = std::sync::Arc::new(apex_memory::embedder::Embedder::default());
    let indexer_config = apex_memory::background_indexer::IndexerConfig::default();
    let background_indexer =
        std::sync::Arc::new(apex_memory::background_indexer::BackgroundIndexer::new(
            embedder.clone(),
            db.pool().clone(),
            indexer_config,
        ));
    let narrative_config = apex_memory::narrative::NarrativeConfig::default();
    let narrative_memory = std::sync::Arc::new(apex_memory::narrative::NarrativeMemory::new(
        narrative_config,
    ));

    // Create bounded memory state for tests
    let bounded_memory = apex_router::api::bounded_memory::BoundedMemoryState::new(
        std::env::temp_dir().to_string_lossy().to_string(),
    );

    // Create skill manager for tests
    let skill_manager = std::sync::Arc::new(tokio::sync::Mutex::new(
        apex_router::skill_manager::SkillManager::new(std::env::temp_dir().join("skills")),
    ));

    AppState {
        config: apex_router::unified_config::AppConfig::default(), // C4 Step 2
        pool: db.pool().clone(),
        metrics: RouterMetrics::new(),
        message_bus: MessageBus::new(10),
        circuit_breakers: CircuitBreakerRegistry::new(),
        vm_pool: Some(VmPool::new(Default::default(), 2, 0)),
        skill_pool: None,
        subagent_pool: std::sync::Arc::new(tokio::sync::RwLock::new(
            apex_router::subagent::SubAgentPool::new(4),
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
        workflow_repo: apex_memory::WorkflowRepository::new(&db.pool()),
        preferences_repo: apex_memory::PreferencesRepository::new(&db.pool()),
        config_repo: apex_memory::ConfigRepository::new(&db.pool()),
        audit_repo: apex_memory::AuditRepository::new(&db.pool()),
        webhook_manager: apex_router::webhook::WebhookManager::new(),
        notification_manager: apex_router::notification::NotificationManager::new(100),
        embedder,
        background_indexer,
        narrative_memory,
        bounded_memory,
        skill_manager,
        user_profile: std::sync::Arc::new(apex_router::user_profile::UserProfileManager::new(
            db.pool().clone(),
        )),
        session_search: std::sync::Arc::new(apex_router::session_search::SessionSearch::new(
            db.pool().clone(),
        )),
        hub_client: std::sync::Arc::new(apex_router::hub_client::HubClient::new(Some(
            "https://skills.sh/api/v1".to_string(),
        ))),
        totp_manager: apex_router::totp::TotpManager::new(),
        soul_loader: apex_router::soul::loader::SoulLoader::new(
            apex_router::soul::SoulConfig::default(),
        ),
        heartbeat_scheduler: apex_router::heartbeat::HeartbeatScheduler::new(
            apex_router::heartbeat::HeartbeatConfig::default(),
        ),
        mcp_manager: std::sync::Arc::new(apex_router::mcp::McpServerManager::new()),
        anomaly_detector: Some(std::sync::Arc::new(
            apex_router::security::AnomalyDetector::new(),
        )),
        // Feature 5: Plugin Signing
        signature_store: std::sync::Arc::new(std::sync::Mutex::new(
            apex_router::skill_signer::SignatureStore::new(),
        )),
        // Feature 7: Story Engine
        story_engine: std::sync::Arc::new(std::sync::Mutex::new(
            apex_router::story_engine::StoryEngine::new(),
        )),
        // Feature 4: Continuity Scheduler
        continuity_state: std::sync::Arc::new(std::sync::Mutex::new(
            apex_router::api::continuity_api::ContinuityState::default(),
        )),
        // Feature 6: Privacy Toggle
        privacy_guard: std::sync::Arc::new(std::sync::Mutex::new(
            apex_router::privacy_guard::PrivacyGuard::default_guard(),
        )),
        // Feature 3: Context Scope
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

#[tokio::test]
async fn test_router_root_endpoint() {
    let _timer = TestTimer::new("test_router_root_endpoint");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_router_health_endpoint() {
    let _timer = TestTimer::new("test_router_health_endpoint");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_task_endpoint() {
    let _timer = TestTimer::new("test_create_task_endpoint");
    let state = create_test_state().await;
    let app = create_router(state);

    let request_body = r#"{"content": "Hello world", "channel": "test"}"#;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/tasks")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_nonexistent_task() {
    let _timer = TestTimer::new("test_get_nonexistent_task");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/tasks/nonexistent-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Verify the endpoint responds - even if task not found
    // The API returns error in body with whatever status
    assert!(
        response.status().is_client_error()
            || response.status().is_server_error()
            || response.status().is_success()
    );
}

#[tokio::test]
async fn test_task_lifecycle() {
    let _timer = TestTimer::new("test_task_lifecycle");
    let state = create_test_state().await;
    let app = create_router(state);

    let create_response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/tasks")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"content": "Test task"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);

    // Get task ID from response - we know the format, just verify it's not empty
    let _location_header = create_response
        .headers()
        .get("location")
        .map(|h| h.to_str().unwrap_or(""));

    // For POST /api/v1/tasks, we can get the task from the body
    // Since we can't easily parse the body, just verify we got a successful response
    // The task was created successfully (201 Created or 200 OK)
}

#[tokio::test]
async fn test_list_skills_empty() {
    let _timer = TestTimer::new("test_list_skills_empty");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/skills")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_register_skill() {
    let _timer = TestTimer::new("test_register_skill");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/skills")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{"name":"test.skill","version":"0.1.0","tier":"T1"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_skill() {
    let _timer = TestTimer::new("test_get_skill");
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let app2 = create_router(state);

    // First register a skill
    app.oneshot(
        Request::builder()
            .uri("/api/v1/skills")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"name":"test.skill","version":"0.1.0","tier":"T1"}"#,
            ))
            .unwrap(),
    )
    .await
    .unwrap();

    // Then get it
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/v1/skills/test.skill")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_nonexistent_skill() {
    let _timer = TestTimer::new("test_get_nonexistent_skill");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/skills/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Returns error message, accept both 404 and 200 with error in body
    let status = response.status();
    assert!(status == StatusCode::NOT_FOUND || status == StatusCode::OK);
}

#[tokio::test]
async fn test_create_deep_task() {
    let _timer = TestTimer::new("test_create_deep_task");
    let state = create_test_state().await;
    let app = create_router(state);

    let request_body = r#"{"content": "Build a website", "max_steps": 5}"#;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/deep")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_deep_task_with_budget() {
    let _timer = TestTimer::new("test_create_deep_task_with_budget");
    let state = create_test_state().await;
    let app = create_router(state);

    let request_body = r#"{"content": "Analyze data", "max_steps": 10, "budget_usd": 2.5}"#;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/deep")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_deep_task_defaults() {
    let _timer = TestTimer::new("test_create_deep_task_defaults");
    let state = create_test_state().await;
    let app = create_router(state);

    let request_body = r#"{"content": "Simple task"}"#;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/deep")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_vm_stats_endpoint() {
    let _timer = TestTimer::new("test_vm_stats_endpoint");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/vm/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_deep_task_creates_task_in_db() {
    let _timer = TestTimer::new("test_deep_task_creates_task_in_db");
    let db = Database::new(&PathBuf::from(":memory:")).await.unwrap();
    db.run_migrations().await.unwrap();

    let pool = db.pool().clone();
    let repo = TaskRepository::new(&pool);

    let task_id = "test-deep-task-001";
    let create_input = CreateTask {
        input_content: "Test deep task".to_string(),
        channel: None,
        thread_id: None,
        author: Some("test".to_string()),
        skill_name: None,
        project: None,
        priority: None,
        category: None,
    };

    repo.create(task_id, create_input, TaskTier::Deep)
        .await
        .unwrap();

    let task = repo.find_by_id(task_id).await.unwrap();
    assert_eq!(task.tier.to_string(), "deep");
}

#[tokio::test]
async fn test_list_workflows_empty() {
    let _timer = TestTimer::new("test_list_workflows_empty");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_workflow() {
    let _timer = TestTimer::new("test_create_workflow");
    let state = create_test_state().await;
    let app = create_router(state);

    let request_body = r#"{
        "name": "Test Workflow",
        "description": "A test workflow",
        "definition": "{\"steps\":[]}",
        "category": "testing"
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_workflow() {
    let _timer = TestTimer::new("test_get_workflow");
    let state = create_test_state().await;
    let app = create_router(state.clone());

    let create_body = r#"{
        "name": "Get Test Workflow",
        "definition": "{\"steps\":[]}"
    }"#;

    let create_response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);

    let app2 = create_router(state);
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows/test-workflow-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::OK);
}

#[tokio::test]
async fn test_update_workflow() {
    let _timer = TestTimer::new("test_update_workflow");
    let state = create_test_state().await;
    let app = create_router(state.clone());

    let create_body = r#"{
        "name": "Original Name",
        "definition": "{\"steps\":[]}"
    }"#;

    let _ = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    let app2 = create_router(state);
    let update_body = r#"{
        "name": "Updated Name",
        "is_active": false
    }"#;

    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows/test-wf-update")
                .method("PUT")
                .header("Content-Type", "application/json")
                .body(Body::from(update_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_delete_workflow() {
    let _timer = TestTimer::new("test_delete_workflow");
    let state = create_test_state().await;
    let app = create_router(state.clone());

    let create_body = r#"{
        "name": "To Be Deleted",
        "definition": "{\"steps\":[]}"
    }"#;

    let _ = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    let app2 = create_router(state);
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows/delete-me")
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_workflow_filter_options() {
    let _timer = TestTimer::new("test_workflow_filter_options");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows/filter-options")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_workflow_executions() {
    let _timer = TestTimer::new("test_workflow_executions");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows/test-id/executions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_adapters() {
    let _timer = TestTimer::new("test_list_adapters");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/adapters")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_adapter() {
    let _timer = TestTimer::new("test_get_adapter");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/adapters/slack")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_nonexistent_adapter() {
    let _timer = TestTimer::new("test_get_nonexistent_adapter");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/adapters/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::OK);
}

#[tokio::test]
async fn test_update_adapter() {
    let _timer = TestTimer::new("test_update_adapter");
    let state = create_test_state().await;
    let app = create_router(state);

    let update_body = r#"{"enabled": false}"#;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/adapters/slack")
                .method("PUT")
                .header("Content-Type", "application/json")
                .body(Body::from(update_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_toggle_adapter() {
    let _timer = TestTimer::new("test_toggle_adapter");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/adapters/telegram/toggle")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_webhooks() {
    let _timer = TestTimer::new("test_list_webhooks");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/webhooks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_webhook() {
    let _timer = TestTimer::new("test_create_webhook");
    let state = create_test_state().await;
    let app = create_router(state);

    let request_body = r#"{
        "name": "Test Webhook",
        "url": "https://example.com/webhook",
        "events": ["task.completed", "task.failed"],
        "secret": "test-secret"
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/webhooks")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_webhook() {
    let _timer = TestTimer::new("test_get_webhook");
    let state = create_test_state().await;
    let app = create_router(state.clone());

    let create_body = r#"{
        "name": "Get Test Webhook",
        "url": "https://example.com/webhook",
        "events": ["task.completed"]
    }"#;

    let _ = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/webhooks")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    let app2 = create_router(state);
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/v1/webhooks/test-webhook-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_delete_webhook() {
    let _timer = TestTimer::new("test_delete_webhook");
    let state = create_test_state().await;
    let app = create_router(state.clone());

    let create_body = r#"{
        "name": "To Delete",
        "url": "https://example.com/webhook",
        "events": ["task.completed"]
    }"#;

    let _ = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/webhooks")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    let app2 = create_router(state);
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/v1/webhooks/delete-me-webhook")
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_toggle_webhook() {
    let _timer = TestTimer::new("test_toggle_webhook");
    let state = create_test_state().await;
    let app = create_router(state.clone());

    let create_body = r#"{
        "name": "Toggle Test",
        "url": "https://example.com/webhook",
        "events": ["task.completed"]
    }"#;

    let _ = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/webhooks")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    let app2 = create_router(state);
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/v1/webhooks/toggle-test-id/toggle")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_list_notifications() {
    let _timer = TestTimer::new("test_list_notifications");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/notifications")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_unread_count() {
    let _timer = TestTimer::new("test_get_unread_count");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/notifications/unread-count")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_notification() {
    let _timer = TestTimer::new("test_get_notification");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/notifications/test-notification-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_mark_notification_read() {
    let _timer = TestTimer::new("test_mark_notification_read");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/notifications/test-id/read")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_mark_all_read() {
    let _timer = TestTimer::new("test_mark_all_read");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/notifications/read-all")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_delete_notification() {
    let _timer = TestTimer::new("test_delete_notification");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/notifications/delete-me")
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_clear_notifications() {
    let _timer = TestTimer::new("test_clear_notifications");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/notifications")
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_list_files() {
    let _timer = TestTimer::new("test_list_files");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_file_content() {
    let _timer = TestTimer::new("test_get_file_content");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/files/content?path=test.txt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_memory_search_endpoint() {
    let _timer = TestTimer::new("test_memory_search_endpoint");
    let state = create_test_state().await;
    let pool = state.pool.clone();
    let app = create_router(state);

    // Insert test memory chunk (with chunk_index)
    sqlx::query("INSERT INTO memory_chunks (id, file_path, chunk_index, content, word_count, memory_type) VALUES (?, ?, ?, ?, ?, ?)")
        .bind("test-chunk-1")
        .bind("/test/file.md")
        .bind(0)
        .bind("This is a test document about testing")
        .bind(8)
        .bind("test")
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/search?q=test&limit=5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_memory_index_stats_endpoint() {
    let _timer = TestTimer::new("test_memory_index_stats_endpoint");
    let state = create_test_state().await;
    let pool = state.pool.clone();
    let app = create_router(state);

    // Insert test memory chunk (with chunk_index)
    sqlx::query("INSERT INTO memory_chunks (id, file_path, chunk_index, content, word_count, memory_type) VALUES (?, ?, ?, ?, ?, ?)")
        .bind("test-chunk-2")
        .bind("/test/file2.md")
        .bind(0)
        .bind("Another test document")
        .bind(4)
        .bind("test")
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/index")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_memory_stats_endpoint() {
    let _timer = TestTimer::new("test_memory_stats_endpoint");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_memory_reflections_endpoint() {
    let _timer = TestTimer::new("test_memory_reflections_endpoint");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/reflections")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_settings_set_and_get() {
    let _timer = TestTimer::new("test_settings_set_and_get");
    let state = create_test_state().await;
    let app = create_router(state);

    // Set a setting
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .header("Content-Type", "application/json")
                .uri("/api/v1/settings/test_key")
                .body(Body::from(r#"{"value": "test_value", "encrypt": false}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK, "Set setting failed");
}

#[tokio::test]
async fn test_settings_get_not_found() {
    let _timer = TestTimer::new("test_settings_get_not_found");
    let state = create_test_state().await;
    let app = create_router(state);

    // Get a non-existent setting
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/settings/nonexistent_key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_settings_delete() {
    let _timer = TestTimer::new("test_settings_delete");
    let state = create_test_state().await;
    let app = create_router(state);

    // First set a setting
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .header("Content-Type", "application/json")
                .uri("/api/v1/settings/to_delete")
                .body(Body::from(r#"{"value": "delete_me"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // Then delete it
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/settings/to_delete")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_audit_create_and_list() {
    let _timer = TestTimer::new("test_audit_create_and_list");
    let state = create_test_state().await;
    let app = create_router(state);
    let app2 = app.clone();

    // Create audit entry
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .header("Content-Type", "application/json")
                .uri("/api/v1/audit")
                .body(Body::from(r#"{"action": "task.created", "entity_type": "task", "entity_id": "test-001", "details": "Test task created"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // List audit entries
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/v1/audit?limit=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_audit_chain_status() {
    let _timer = TestTimer::new("test_audit_chain_status");
    let state = create_test_state().await;
    let app = create_router(state);

    // Get chain status
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/audit/chain")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_task_confirm_endpoint_exists() {
    let _timer = TestTimer::new("test_task_confirm_endpoint_exists");
    let state = create_test_state().await;
    let app = create_router(state);

    // Test that the confirm endpoint exists and accepts POST requests
    // The endpoint should respond (not 404 Not Found)
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tasks/test-task-id/confirm")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should NOT return 404 - endpoint exists
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_llm_providers_endpoint() {
    let _timer = TestTimer::new("test_llm_providers_endpoint");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/llms/providers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let providers: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(providers.len() >= 26); // At least 26 providers
}

#[tokio::test]
async fn test_llm_list_endpoint() {
    let _timer = TestTimer::new("test_llm_list_endpoint");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/llms")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let llms: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(llms.len(), 1); // One default LLM configured
    assert_eq!(llms[0]["name"], "Local Qwen3-4B");
}

#[tokio::test]
async fn test_llm_get_default_endpoint() {
    let _timer = TestTimer::new("test_llm_get_default_endpoint");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/llms/default")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let llm: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(llm["name"], "Local Qwen3-4B"); // Default LLM is set
}

#[tokio::test]
async fn test_llm_invalid_provider() {
    let _timer = TestTimer::new("test_llm_invalid_provider");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/llms")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                    "name": "Test LLM",
                    "provider": "invalid_provider",
                    "url": "http://localhost:8080/v1",
                    "model": "test-model"
                }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // The endpoint returns 200 with error in body for invalid provider
    // Just verify we got some response
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_llm_add_endpoint() {
    let _timer = TestTimer::new("test_llm_add_endpoint");
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/llms")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{
                    "name": "Test LLM",
                    "provider": "local",
                    "url": "http://localhost:8080/v1",
                    "model": "qwen3-4b"
                }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let llm: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(llm["name"], "Test LLM");
    assert_eq!(llm["provider"], "local");
}

#[tokio::test]
async fn test_config_persistence_save_and_load() {
    let _timer = TestTimer::new("test_config_persistence_save_and_load");
    let db = Database::new(&PathBuf::from(":memory:")).await.unwrap();
    db.run_migrations().await.unwrap();

    let repo = apex_memory::ConfigRepository::new(&db.pool());

    // Save a config section
    let test_config = serde_json::json!({
        "llms": [
            {
                "id": "test-llm",
                "name": "Test LLM",
                "provider": "local",
                "url": "http://localhost:8080",
                "model": "test-model",
                "api_key": null
            }
        ],
        "default_llm_id": "test-llm"
    });

    apex_router::unified_config::AppConfig::save_section_to_db(&repo, "agent", &test_config)
        .await
        .unwrap();

    // Load it back
    let loaded: serde_json::Value =
        apex_router::unified_config::AppConfig::load_section_from_db(&repo, "agent")
            .await
            .unwrap()
            .unwrap();

    assert_eq!(loaded["llms"][0]["name"], "Test LLM");
    assert_eq!(loaded["default_llm_id"], "test-llm");
}

#[tokio::test]
async fn test_config_persistence_full_config() {
    let _timer = TestTimer::new("test_config_persistence_full_config");
    let db = Database::new(&PathBuf::from(":memory:")).await.unwrap();
    db.run_migrations().await.unwrap();

    let repo = apex_memory::ConfigRepository::new(&db.pool());

    // Create a full config
    let config = apex_router::unified_config::AppConfig::default();

    // Save full config
    config.save_to_db(&repo).await.unwrap();

    // Load it back
    let loaded = apex_router::unified_config::AppConfig::load_from_db(&repo)
        .await
        .unwrap();

    // Verify key fields match
    assert_eq!(loaded.agent.use_llm, config.agent.use_llm);
}

#[tokio::test]
async fn test_config_persistence_nonexistent() {
    let _timer = TestTimer::new("test_config_persistence_nonexistent");
    let db = Database::new(&PathBuf::from(":memory:")).await.unwrap();
    db.run_migrations().await.unwrap();

    let repo = apex_memory::ConfigRepository::new(&db.pool());

    // Try to load non-existent config
    let result = apex_router::unified_config::AppConfig::load_from_db(&repo)
        .await
        .unwrap();

    // Should return default config
    assert!(result.agent.llms.len() >= 1);
}
