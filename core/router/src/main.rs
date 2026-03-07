use std::net::SocketAddr;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use apex_memory::db::Database;
use apex_memory::embedder::{Embedder, EmbeddingProvider};
use apex_memory::background_indexer::{BackgroundIndexer, IndexerConfig};
use apex_memory::narrative::NarrativeMemory;
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
use apex_router::skill_pool::SkillPool;
use apex_router::soul::loader::SoulLoader;
use apex_router::soul::SoulConfig;
use apex_router::totp::TotpManager;
use apex_router::unified_config::AppConfig;
use apex_router::vm_pool::{VmConfig, VmPool};
use apex_router::rate_limiter::RateLimiter;
use apex_router::response_cache::ResponseCache;
use apex_router::webhook::WebhookManager;
use apex_router::notification::NotificationManager;
use apex_router::system_health::SystemMonitor;
use apex_router::websocket::WebSocketManager;

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

    // C4: Load config first, before creating components
    let config = AppConfig::global();
    
    let validation_errors = config.validate();
    if !validation_errors.is_empty() {
        tracing::warn!("Configuration validation found {} issues:", validation_errors.len());
        for error in &validation_errors {
            tracing::warn!("  - {}", error.message);
        }
    } else {
        tracing::info!("Configuration validation passed");
    }

    // Now create components WITH config instead of using AppConfig::global()
    let vm_config = VmConfig::from_config(&config);
    tracing::info!(vm_config = ?vm_config, "VM Configuration");
    
    let vm_pool = VmPool::new(vm_config, 3, 1);
    if let Err(e) = vm_pool.initialize().await {
        tracing::warn!("Failed to initialize VM pool: {}", e);
    }

    let skill_pool = if config.skill_pool.enabled {
        let pool_config = apex_router::skill_pool::SkillPoolConfig {
            pool_size: config.skill_pool.pool_size,
            worker_script: std::path::PathBuf::from(&config.skill_pool.worker_script),
            skills_dir: std::path::PathBuf::from(&config.skill_pool.skills_dir),
            request_timeout_ms: config.skill_pool.request_timeout_ms,
            acquire_timeout_ms: config.skill_pool.acquire_timeout_ms,
            health_check_interval: std::time::Duration::from_secs(30),
        };
        match SkillPool::new(pool_config).await {
            Ok(pool) => {
                tracing::info!("SkillPool initialized with {} workers", pool.config().pool_size);
                Some(pool)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize SkillPool: {}, falling back to spawn", e);
                None
            }
        }
    } else {
        tracing::info!("SkillPool disabled via config");
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

    // Create TOTP manager
    let totp_manager = TotpManager::new();
    tracing::info!("TOTP manager initialized");

    // Create SoulLoader
    let soul_base_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".apex")
        .join("soul");
    std::fs::create_dir_all(&soul_base_dir).ok();
    let soul_config = SoulConfig {
        soul_dir: soul_base_dir.clone(),
        fragments_dir: soul_base_dir.join("fragments"),
        history_dir: soul_base_dir.join("history"),
        backup_enabled: config.soul.backup_enabled,
    };
    let soul_loader = SoulLoader::new(soul_config);
    tracing::info!("SoulLoader initialized");

    // Create heartbeat scheduler
    let heartbeat_config = apex_router::heartbeat::HeartbeatConfig::from_config(&config);
    let heartbeat_scheduler = HeartbeatScheduler::new(heartbeat_config);
    tracing::info!("Heartbeat scheduler initialized");

    // Create embedder from config
    let embedder = {
        let embedding_url = config.memory.embedding_url.clone();
        let embedding_model = config.memory.embedding_model.clone();
        let embedding_dim = config.memory.embedding_dim;
        
        let provider = if config.memory.embedding_provider == "openai" {
            EmbeddingProvider::OpenAI {
                api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
                model: embedding_model,
            }
        } else {
            EmbeddingProvider::Local {
                url: embedding_url,
                model: embedding_model,
            }
        };
        
        std::sync::Arc::new(Embedder::new(provider, embedding_dim))
    };
    tracing::info!("Embedder initialized with provider: {:?}", embedder);

    // Create background indexer
    let indexer_config = IndexerConfig {
        batch_size: config.memory.indexer_batch_size,
        embed_rate_limit_ms: config.memory.embed_rate_limit_ms,
        chunk_config: apex_memory::chunker::ChunkerConfig {
            chunk_size_tokens: config.memory.chunk_size,
            overlap_tokens: config.memory.chunk_overlap,
            min_chunk_tokens: 20,
            respect_headings: true,
            respect_code_blocks: true,
        },
        embedding_dim: config.memory.embedding_dim,
    };
    let background_indexer = std::sync::Arc::new(BackgroundIndexer::new(
        embedder.clone(),
        pool.clone(),
        indexer_config.clone(),
    ));
    tracing::info!("BackgroundIndexer initialized");

    // Create NarrativeMemory
    let narrative_config = apex_memory::narrative::NarrativeConfig {
        base_path: dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".apex")
            .join("memory"),
        retention_days: 90,
        forgetting_threshold_days: 30,
    };
    let narrative_memory = std::sync::Arc::new(NarrativeMemory::new(narrative_config.clone()));
    if let Err(e) = narrative_memory.initialize().await {
        tracing::warn!("Failed to initialize NarrativeMemory: {}", e);
    }
    tracing::info!("NarrativeMemory initialized at {:?}", narrative_config.base_path);

    // Start initial memory scan
    let indexer_for_scan = background_indexer.clone();
    let memory_dir = narrative_config.base_path.clone();
    tokio::spawn(async move {
        indexer_for_scan.initial_scan(&memory_dir).await;
    });

    let state = AppState {
        config: config.clone(),  // C4 Step 2: Add config to AppState
        pool: pool_for_workers.clone(),
        metrics: RouterMetrics::new(),
        message_bus: message_bus.clone(),
        circuit_breakers: circuit_breakers.clone(),
        vm_pool: Some(vm_pool.clone()),
        skill_pool: skill_pool.clone(),
        execution_streams: ExecutionStreamManager::new(),
        ws_manager: WebSocketManager::new(),
        moltbook,
        governance,
        system_monitor: SystemMonitor::new(),
        cache: ResponseCache::new(60),
        rate_limiter: RateLimiter::new(60),
        workflow_repo: apex_memory::WorkflowRepository::new(&pool_for_workers),
        preferences_repo: apex_memory::PreferencesRepository::new(&pool_for_workers),
        audit_repo: apex_memory::AuditRepository::new(&pool_for_workers),
        webhook_manager: WebhookManager::new(),
        notification_manager: NotificationManager::new(100),
        embedder,
        background_indexer,
        narrative_memory,
        totp_manager,
        soul_loader,
        heartbeat_scheduler,
    };
    
    let state_arc = std::sync::Arc::new(state);
    let state_for_router = state_arc.as_ref().clone();
    let state_for_deep_worker = state_arc.as_ref().clone();

    let worker = SkillWorker::new(pool_for_workers.clone(), skill_pool.clone(), message_bus.clone(), circuit_breakers.clone());
    tokio::spawn(worker.run());

    let deep_worker =
        DeepTaskWorker::new(pool_for_workers.clone(), message_bus.clone(), vm_pool, circuit_breakers.clone(), state_for_deep_worker.execution_streams.clone(), state_for_deep_worker.ws_manager.clone(), state_for_deep_worker.narrative_memory.clone());
    tokio::spawn(deep_worker.run());

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

    let app = create_router(state_for_router)
        .merge(apex_router::websocket::create_ws_router(state_arc));

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
            tracing::info!("Shutdown signal received");
            tracing::info!("Shutdown complete");
        })
        .await?;

    Ok(())
}
