use axum::routing::get;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use apex_memory::db::Database;
use apex_router::api::{create_router, AppState};
use apex_router::circuit_breaker::CircuitBreakerRegistry;
use apex_router::deep_task_worker::DeepTaskWorker;
use apex_router::execution_stream::ExecutionStreamManager;
use apex_router::governance::GovernanceEngine;
use apex_router::heartbeat::HeartbeatScheduler;
use apex_router::message_bus::MessageBus;
use apex_router::metrics::RouterMetrics;
use apex_router::moltbook::MoltbookClient;
use apex_router::skill_worker::SkillWorker;
use apex_router::unified_config::AppConfig;
use apex_router::vm_pool::{VmConfig, VmPool};
use apex_router::rate_limiter::RateLimiter;
use apex_router::response_cache::ResponseCache;
use apex_router::system_health::SystemMonitor;
use apex_router::websocket::WebSocketManager;

async fn root() -> &'static str {
    "APEX Router v0.1.0 - See /api/v1/tasks for task endpoints"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_logs = std::env::var("APEX_JSON_LOGS").is_ok();
    
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "apex_router=info,tower_http=debug".into()),
        );
    
    if json_logs {
        subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        subscriber
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    let use_llm = std::env::var("APEX_USE_LLM").is_ok();
    let llama_url = std::env::var("LLAMA_SERVER_URL").ok();
    let llama_model = std::env::var("LLAMA_MODEL").ok();

    tracing::info!(use_llm = use_llm, llama_url = ?llama_url, llama_model = ?llama_model, "LLM Configuration");

    let db_path = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("apex")
        .join("apex.db");

    std::fs::create_dir_all(db_path.parent().unwrap())?;

    let db = Database::new(&db_path).await?;
    db.run_migrations().await?;

    let pool = db.pool().clone();
    let pool_for_workers = pool.clone();
    tracing::info!("Database initialized at {:?}", pool);

    let message_bus = MessageBus::new(100);
    let circuit_breakers = CircuitBreakerRegistry::new();

    let vm_config = VmConfig::from_env();
    tracing::info!(vm_config = ?vm_config, "VM Configuration");
    
    let vm_pool = VmPool::new(vm_config, 3, 1);
    if let Err(e) = vm_pool.initialize().await {
        tracing::warn!("Failed to initialize VM pool: {}", e);
    }

    let config = AppConfig::global();
    
    let heartbeat_scheduler = if config.heartbeat.enabled {
        let heartbeat_config = apex_router::heartbeat::HeartbeatConfig::from_config(&config);
        let scheduler = HeartbeatScheduler::new(heartbeat_config);
        tracing::info!("Heartbeat scheduler enabled, interval: {} minutes", config.heartbeat.interval_minutes);
        Some(scheduler)
    } else {
        tracing::info!("Heartbeat daemon disabled");
        None
    };
    
    let moltbook = if config.moltbook.enabled {
        let moltbook_config = apex_router::moltbook::MoltbookConfig {
            enabled: config.moltbook.enabled,
            server_url: config.moltbook.server_url.clone(),
            agent_id: config.moltbook.agent_id.clone().unwrap_or_default(),
            client_cert_path: None,
            client_key_path: None,
            ca_cert_path: None,
            poll_interval_secs: 60,
            max_connections: 10,
        };
        match MoltbookClient::new(moltbook_config) {
            Ok(client) => {
                tracing::info!("Moltbook client initialized");
                Some(client)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize Moltbook client: {}", e);
                None
            }
        }
    } else {
        None
    };

    let governance = std::sync::Arc::new(std::sync::Mutex::new(GovernanceEngine::default()));

    let state = AppState {
        pool: pool_for_workers.clone(),
        metrics: RouterMetrics::new(),
        message_bus: message_bus.clone(),
        circuit_breakers: circuit_breakers.clone(),
        vm_pool: Some(vm_pool.clone()),
        execution_streams: ExecutionStreamManager::new(),
        ws_manager: WebSocketManager::new(),
        moltbook,
        governance,
        system_monitor: SystemMonitor::new(),
        cache: ResponseCache::new(60),
        rate_limiter: RateLimiter::new(60),
        workflow_repo: apex_memory::WorkflowRepository::new(&pool_for_workers),
    };

    let worker = SkillWorker::new(pool_for_workers.clone(), message_bus.clone(), circuit_breakers.clone());
    tokio::spawn(worker.run());

    let deep_worker =
        DeepTaskWorker::new(pool_for_workers.clone(), message_bus.clone(), vm_pool, circuit_breakers.clone(), state.execution_streams.clone(), state.ws_manager.clone());
    tokio::spawn(deep_worker.run());

    if let Some(ref scheduler) = heartbeat_scheduler {
        let scheduler_clone = scheduler.clone();
        tokio::spawn(async move {
            scheduler_clone.start().await;
        });
    }

    let cleanup_pool = pool_for_workers.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let repo = apex_memory::task_repo::TaskRepository::new(&cleanup_pool);
            match repo.cleanup_old_completed(7).await {
                Ok(count) if count > 0 => {
                    tracing::info!("Cleaned up {} old completed tasks", count);
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("Failed to cleanup old tasks: {}", e);
                }
            }
        }
    });

    let app = create_router(state).route("/", get(root));

    let port: u16 = std::env::var("APEX_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Starting router on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            tracing::info!("Shutdown signal received, stopping heartbeat...");
            if let Some(ref scheduler) = heartbeat_scheduler {
                scheduler.stop().await;
            }
            tracing::info!("Shutdown complete");
        })
        .await?;

    Ok(())
}
