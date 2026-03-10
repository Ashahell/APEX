use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

pub static GLOBAL_CONFIG: Lazy<RwLock<Option<AppConfig>>> = Lazy::new(|| RwLock::new(None));

// C4 Step 1: Thread-local config override for test isolation
thread_local! {
    static THREAD_CONFIG: RefCell<Option<AppConfig>> = RefCell::new(None);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub channels: ChannelConfigs,
    pub agent: AgentConfig,
    pub execution: ExecutionConfig,
    pub database: DatabaseConfig,
    pub nats: NatsConfig,
    pub logging: LoggingConfig,
    pub skills: SkillsConfig,
    pub skill_pool: SkillPoolConfigSection,
    pub soul: SoulConfig,
    pub heartbeat: HeartbeatConfig,
    pub moltbook: MoltbookConfigSection,
    pub memory: MemoryConfig,
    #[serde(skip)]
    pub config_source: ConfigSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: std::env::var("APEX_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub shared_secret: String,
    pub disabled: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            shared_secret: std::env::var("APEX_SHARED_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-in-production".to_string()),
            disabled: std::env::var("APEX_AUTH_DISABLED")
                .ok()
                .map(|v| v == "1")
                .unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelConfigs {
    pub slack: Option<ChannelConfig>,
    pub discord: Option<ChannelConfig>,
    pub telegram: Option<ChannelConfig>,
    pub whatsapp: Option<ChannelConfig>,
    pub email: Option<ChannelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub enabled: bool,
    pub bot_token: Option<String>,
    pub signing_secret: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub use_llm: bool,
    pub llama_url: String,
    pub llama_model: String,
    pub max_iterations: u32,
    pub max_budget_cents: i64,
    pub context_window_tokens: usize,
    pub model: String,
    pub llms: Vec<LlmConfig>,
    pub default_llm_id: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        let default_llm = LlmConfig {
            id: "default".to_string(),
            name: "Local Qwen3-4B".to_string(),
            provider: LlmProvider::Local,
            url: std::env::var("LLAMA_SERVER_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            model: std::env::var("LLAMA_MODEL").unwrap_or_else(|_| "qwen3-4b".to_string()),
            api_key: None,
        };
        Self {
            use_llm: std::env::var("APEX_USE_LLM").is_ok(),
            llama_url: default_llm.url.clone(),
            llama_model: default_llm.model.clone(),
            max_iterations: 50,
            max_budget_cents: 500,
            context_window_tokens: 32768,
            model: default_llm.model.clone(),
            llms: vec![default_llm],
            default_llm_id: Some("default".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum LlmProvider {
    // Local/Private
    Local,
    Ollama,
    Vllm,
    LmStudio,
    
    // Major Cloud Providers
    OpenAI,
    Anthropic,
    Google,
    Azure,
    
    // Aggregators & Gateways
    OpenRouter,
    Cloudflare,
    Vercel,
    Together,
    LiteLlama,
    
    // AI Providers
    Mistral,
    Cohere,
    Groq,
    Fireworks,
    HuggingFace,
    
    // Chinese Providers
    ZhipuGlm,
    Qianfan,
    Moonshot,
    MiniMax,
    
    // Enterprise
    Bedrock,
    Vertex,
    
    // xAI
    Xai,
    
    // Venice
    Venice,
    
    // Custom (any OpenAI-compatible endpoint)
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub id: String,
    pub name: String,
    pub provider: LlmProvider,
    pub url: String,
    pub model: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub isolation: String,
    pub firecracker: FirecrackerConfig,
    pub gvisor: GvisorConfig,
    pub docker: DockerConfig,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            isolation: std::env::var("APEX_EXECUTION_ISOLATION")
                .unwrap_or_else(|_| "docker".to_string()),
            firecracker: FirecrackerConfig::default(),
            gvisor: GvisorConfig::default(),
            docker: DockerConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirecrackerConfig {
    pub enabled: bool,
    pub vcpus: u32,
    pub memory_mib: u64,
    pub timeout_secs: u64,
    pub kernel_path: Option<String>,
    pub rootfs_path: Option<String>,
    pub firecracker_path: Option<String>,
    pub network_isolation: bool,
    pub fast_boot: bool,
    pub use_jailer: bool,
}

impl Default for FirecrackerConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("APEX_USE_FIRECRACKER")
                .ok()
                .map(|v| v == "1")
                .unwrap_or(false),
            vcpus: std::env::var("APEX_VM_VCPU")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
            memory_mib: std::env::var("APEX_VM_MEMORY_MIB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2048),
            timeout_secs: 60,
            kernel_path: std::env::var("APEX_VM_KERNEL").ok(),
            rootfs_path: std::env::var("APEX_VM_ROOTFS").ok(),
            firecracker_path: std::env::var("APEX_FIRECRACKER_PATH").ok(),
            network_isolation: std::env::var("APEX_VM_NETWORK_ISOLATION")
                .ok()
                .map(|v| v == "1")
                .unwrap_or(true),
            fast_boot: std::env::var("APEX_VM_FAST_BOOT")
                .ok()
                .map(|v| v == "1")
                .unwrap_or(false),
            use_jailer: std::env::var("APEX_USE_JAILER")
                .ok()
                .map(|v| v == "1")
                .unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GvisorConfig {
    pub enabled: bool,
    pub runsc_path: Option<String>,
}

impl Default for GvisorConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("APEX_USE_GVISOR")
                .ok()
                .map(|v| v == "1")
                .unwrap_or(false),
            runsc_path: std::env::var("APEX_RUNSC_PATH").ok(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub enabled: bool,
    pub image: String,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("APEX_USE_DOCKER")
                .ok()
                .map(|v| v == "1")
                .unwrap_or(false),
            image: std::env::var("APEX_DOCKER_IMAGE")
                .unwrap_or_else(|_| "apex-execution:latest".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub db_type: String,
    pub connection_string: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: "sqlite".to_string(),
            connection_string: std::env::var("APEX_DATABASE_URL")
                .or_else(|_| std::env::var("DATABASE_URL"))
                .unwrap_or_else(|_| "sqlite:apex.db".to_string()),
            max_connections: std::env::var("APEX_DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            min_connections: std::env::var("APEX_DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    pub enabled: bool,
    pub url: String,
    pub subject_prefix: String,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("APEX_NATS_ENABLED")
                .ok()
                .map(|v| v == "1")
                .unwrap_or(false),
            url: std::env::var("APEX_NATS_URL").unwrap_or_else(|_| "127.0.0.1:4222".to_string()),
            subject_prefix: std::env::var("APEX_NATS_SUBJECT_PREFIX")
                .unwrap_or_else(|_| "apex".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub json_logs: bool,
    pub log_level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            json_logs: std::env::var("APEX_JSON_LOGS")
                .ok()
                .map(|v| v == "1")
                .unwrap_or(false),
            log_level: std::env::var("APEX_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsConfig {
    pub cli_path: Option<String>,
    pub directory: Option<String>,
}

impl Default for SkillsConfig {
    fn default() -> Self {
        Self {
            cli_path: std::env::var("APEX_SKILLS_CLI").ok(),
            directory: std::env::var("APEX_SKILLS_DIR").ok(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulConfig {
    pub directory: Option<String>,
    pub backup_enabled: bool,
}

impl Default for SoulConfig {
    fn default() -> Self {
        Self {
            directory: std::env::var("APEX_SOUL_DIR").ok(),
            backup_enabled: std::env::var("APEX_SOUL_BACKUP")
                .ok()
                .map(|v| v == "1")
                .unwrap_or(true),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    pub enabled: bool,
    pub interval_minutes: u64,
    pub jitter_percent: u32,
    pub cooldown_seconds: u64,
    pub max_actions_per_wake: u32,
    pub require_approval_t1_plus: bool,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("APEX_HEARTBEAT_ENABLED")
                .ok()
                .map(|v| v == "1")
                .unwrap_or(false),
            interval_minutes: std::env::var("APEX_HEARTBEAT_INTERVAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            jitter_percent: std::env::var("APEX_HEARTBEAT_JITTER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            cooldown_seconds: std::env::var("APEX_HEARTBEAT_COOLDOWN")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            max_actions_per_wake: std::env::var("APEX_HEARTBEAT_MAX_ACTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            require_approval_t1_plus: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoltbookConfigSection {
    pub enabled: bool,
    pub server_url: String,
    pub agent_id: Option<String>,
}

impl Default for MoltbookConfigSection {
    fn default() -> Self {
        Self {
            enabled: false,
            server_url: "https://moltbook.local".to_string(),
            agent_id: std::env::var("APEX_MOLTBOOK_AGENT_ID").ok(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub embedding_provider: String,
    pub embedding_url: String,
    pub embedding_model: String,
    pub embedding_dim: usize,
    pub rrf_k: usize,
    pub max_results: usize,
    pub mmr_lambda: f64,
    pub half_life_days: f64,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub embed_rate_limit_ms: u64,
    pub indexer_batch_size: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            embedding_provider: std::env::var("APEX_MEMORY_EMBEDDING_PROVIDER")
                .unwrap_or_else(|_| "local".to_string()),
            embedding_url: std::env::var("APEX_MEMORY_EMBEDDING_URL")
                .unwrap_or_else(|_| "http://localhost:8081".to_string()),
            embedding_model: std::env::var("APEX_MEMORY_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "nomic-embed-text".to_string()),
            embedding_dim: std::env::var("APEX_MEMORY_EMBEDDING_DIM")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(768),
            rrf_k: std::env::var("APEX_MEMORY_RRF_K")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            max_results: std::env::var("APEX_MEMORY_MAX_RESULTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8),
            mmr_lambda: std::env::var("APEX_MEMORY_MMR_LAMBDA")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.7),
            half_life_days: std::env::var("APEX_MEMORY_HALF_LIFE_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30.0),
            chunk_size: std::env::var("APEX_MEMORY_CHUNK_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(256),
            chunk_overlap: std::env::var("APEX_MEMORY_CHUNK_OVERLAP")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(32),
            embed_rate_limit_ms: std::env::var("APEX_MEMORY_EMBED_RATE_LIMIT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50),
            indexer_batch_size: std::env::var("APEX_MEMORY_INDEXER_BATCH_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(16),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPoolConfigSection {
    pub enabled: bool,
    pub pool_size: usize,
    pub worker_script: String,
    pub skills_dir: String,
    pub request_timeout_ms: u64,
    pub acquire_timeout_ms: u64,
}

impl Default for SkillPoolConfigSection {
    fn default() -> Self {
        Self {
            enabled: true,
            pool_size: std::env::var("APEX_SKILL_POOL_SIZE")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(4),
            worker_script: std::env::var("APEX_SKILL_POOL_WORKER")
                .unwrap_or_else(|_| "./skills/pool_worker.ts".to_string()),
            skills_dir: std::env::var("APEX_SKILLS_DIR").unwrap_or_else(|_| "./skills".to_string()),
            request_timeout_ms: std::env::var("APEX_SKILL_POOL_TIMEOUT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(30_000),
            acquire_timeout_ms: std::env::var("APEX_SKILL_POOL_ACQUIRE")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5_000),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConfigSource {
    #[default]
    Default,
    YamlFile,
    Environment,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            auth: AuthConfig::default(),
            channels: ChannelConfigs::default(),
            agent: AgentConfig::default(),
            execution: ExecutionConfig::default(),
            database: DatabaseConfig::default(),
            nats: NatsConfig::default(),
            logging: LoggingConfig::default(),
            skills: SkillsConfig::default(),
            skill_pool: SkillPoolConfigSection::default(),
            soul: SoulConfig::default(),
            heartbeat: HeartbeatConfig::default(),
            moltbook: MoltbookConfigSection::default(),
            memory: MemoryConfig::default(),
            config_source: ConfigSource::Environment,
        }
    }
}

impl AppConfig {
    pub fn load(path: Option<&str>) -> Result<Self, ConfigError> {
        if let Some(path) = path {
            let path = PathBuf::from(path);
            if path.exists() {
                return Self::load_from_file(&path);
            }
        }

        if let Some(path) = Self::find_config_file() {
            return Self::load_from_file(&path);
        }

        Ok(Self::default())
    }

    fn find_config_file() -> Option<PathBuf> {
        let candidates = vec![
            PathBuf::from("apex.yaml"),
            PathBuf::from("apex.yml"),
            PathBuf::from("config/apex.yaml"),
            PathBuf::from("config/apex.yml"),
            dirs::config_dir()?.join("apex").join("apex.yaml"),
        ];

        for candidate in candidates {
            if candidate.exists() {
                return Some(candidate);
            }
        }
        None
    }

    fn load_from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("yaml");

        let mut config: AppConfig = match ext {
            "yaml" | "yml" => serde_yaml::from_str(&content)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?,
            "json" => serde_json::from_str(&content)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?,
            _ => return Err(ConfigError::UnsupportedFormat(ext.to_string())),
        };

        config.config_source = ConfigSource::YamlFile;
        Ok(config)
    }

    pub fn set_global(config: AppConfig) {
        let mut global = GLOBAL_CONFIG.write().unwrap();
        *global = Some(config);
    }

    pub fn get_global() -> Option<AppConfig> {
        GLOBAL_CONFIG.read().unwrap().clone()
    }

    /// Load config from database using ConfigRepository
    pub async fn load_from_db(repo: &apex_memory::ConfigRepository) -> Result<Self, ConfigError> {
        let entry = repo.get("app_config").await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        match entry {
            Some(e) => {
                serde_json::from_str(&e.value)
                    .map_err(|e| ConfigError::ParseError(e.to_string()))
            }
            None => Ok(Self::default()),
        }
    }

    /// Save config to database using ConfigRepository
    pub async fn save_to_db(&self, repo: &apex_memory::ConfigRepository) -> Result<(), ConfigError> {
        let json = serde_json::to_string(self)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        repo.set("app_config", &json).await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        // Also update global in-memory config
        Self::set_global(self.clone());
        
        Ok(())
    }

    /// Load a specific section (e.g., "llms") from database
    pub async fn load_section_from_db<T: serde::de::DeserializeOwned>(
        repo: &apex_memory::ConfigRepository,
        key: &str
    ) -> Result<Option<T>, ConfigError> {
        let full_key = format!("app_config_{}", key);
        let entry = repo.get(&full_key).await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        match entry {
            Some(e) => {
                let value: T = serde_json::from_str(&e.value)
                    .map_err(|e| ConfigError::ParseError(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Save a specific section (e.g., "llms") to database
    pub async fn save_section_to_db<T: serde::Serialize>(
        repo: &apex_memory::ConfigRepository,
        key: &str,
        value: &T
    ) -> Result<(), ConfigError> {
        let full_key = format!("app_config_{}", key);
        let json = serde_json::to_string(value)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        repo.set(&full_key, &json).await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        Ok(())
    }

    pub fn global() -> AppConfig {
        // C4 Step 1: Check thread-local override first (for test isolation)
        THREAD_CONFIG.with(|c| {
            c.borrow().clone()
        }).unwrap_or_else(|| Self::get_global().unwrap_or_else(Self::default))
    }

    /// C4 Step 1: Test helper — sets a config visible only to the calling thread.
    /// Automatically scoped: drops when the test finishes.
    #[cfg(test)]
    pub fn with_test_config<F, R>(config: AppConfig, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        THREAD_CONFIG.with(|c| *c.borrow_mut() = Some(config));
        let result = f();
        THREAD_CONFIG.with(|c| *c.borrow_mut() = None);
        result
    }

    /// C4 Step 1: Async test helper
    #[cfg(test)]
    pub async fn with_test_config_async<F, R>(config: AppConfig, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        THREAD_CONFIG.with(|c| *c.borrow_mut() = Some(config));
        let result = f();
        THREAD_CONFIG.with(|c| *c.borrow_mut() = None);
        result
    }

    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        vars.insert("APEX_PORT".to_string(), self.server.port.to_string());
        vars.insert("APEX_HOST".to_string(), self.server.host.clone());
        vars.insert("APEX_SHARED_SECRET".to_string(), "[REDACTED]".to_string());

        if self.auth.disabled {
            vars.insert("APEX_AUTH_DISABLED".to_string(), "1".to_string());
        }

        vars.insert(
            "APEX_USE_LLM".to_string(),
            if self.agent.use_llm { "1" } else { "0" }.to_string(),
        );
        vars.insert("LLAMA_SERVER_URL".to_string(), self.agent.llama_url.clone());
        vars.insert("LLAMA_MODEL".to_string(), self.agent.llama_model.clone());

        vars.insert(
            "APEX_USE_FIRECRACKER".to_string(),
            if self.execution.firecracker.enabled {
                "1"
            } else {
                "0"
            }
            .to_string(),
        );
        vars.insert(
            "APEX_USE_GVISOR".to_string(),
            if self.execution.gvisor.enabled {
                "1"
            } else {
                "0"
            }
            .to_string(),
        );
        vars.insert(
            "APEX_USE_DOCKER".to_string(),
            if self.execution.docker.enabled {
                "1"
            } else {
                "0"
            }
            .to_string(),
        );

        vars.insert(
            "APEX_VM_VCPU".to_string(),
            self.execution.firecracker.vcpus.to_string(),
        );
        vars.insert(
            "APEX_VM_MEMORY_MIB".to_string(),
            self.execution.firecracker.memory_mib.to_string(),
        );

        vars.insert("APEX_DATABASE_URL".to_string(), "[REDACTED]".to_string());

        if self.nats.enabled {
            vars.insert("APEX_NATS_ENABLED".to_string(), "1".to_string());
            vars.insert("APEX_NATS_URL".to_string(), self.nats.url.clone());
            vars.insert(
                "APEX_NATS_SUBJECT_PREFIX".to_string(),
                self.nats.subject_prefix.clone(),
            );
        }

        vars.insert(
            "APEX_JSON_LOGS".to_string(),
            if self.logging.json_logs { "1" } else { "0" }.to_string(),
        );
        vars.insert("APEX_LOG_LEVEL".to_string(), self.logging.log_level.clone());

        vars.insert(
            "APEX_HEARTBEAT_ENABLED".to_string(),
            if self.heartbeat.enabled { "1" } else { "0" }.to_string(),
        );
        vars.insert(
            "APEX_HEARTBEAT_INTERVAL".to_string(),
            self.heartbeat.interval_minutes.to_string(),
        );

        if self.skill_pool.enabled {
            vars.insert(
                "APEX_SKILL_POOL_SIZE".to_string(),
                self.skill_pool.pool_size.to_string(),
            );
            vars.insert(
                "APEX_SKILL_POOL_WORKER".to_string(),
                self.skill_pool.worker_script.clone(),
            );
            vars.insert(
                "APEX_SKILL_POOL_TIMEOUT".to_string(),
                self.skill_pool.request_timeout_ms.to_string(),
            );
            vars.insert(
                "APEX_SKILL_POOL_ACQUIRE".to_string(),
                self.skill_pool.acquire_timeout_ms.to_string(),
            );
        }

        vars
    }

    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if self.server.port == 0 {
            errors.push(ValidationError {
                field: "server.port".to_string(),
                message: "Port must be greater than 0".to_string(),
            });
        }

        if self.auth.shared_secret.len() < 16 && !self.auth.disabled {
            errors.push(ValidationError {
                field: "auth.shared_secret".to_string(),
                message: "Shared secret should be at least 16 characters".to_string(),
            });
        }

        if self.database.connection_string.is_empty() {
            errors.push(ValidationError {
                field: "database.connection_string".to_string(),
                message: "Database connection string is required".to_string(),
            });
        }

        if self.execution.firecracker.enabled {
            if self.execution.firecracker.kernel_path.is_none() {
                errors.push(ValidationError {
                    field: "execution.firecracker.kernel_path".to_string(),
                    message: "Firecracker kernel path required when enabled".to_string(),
                });
            }
            if self.execution.firecracker.rootfs_path.is_none() {
                errors.push(ValidationError {
                    field: "execution.firecracker.rootfs_path".to_string(),
                    message: "Firecracker rootfs path required when enabled".to_string(),
                });
            }
        }

        errors
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Config file not found")]
    NotFound,

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSummary {
    pub server: ServerConfig,
    pub auth_enabled: bool,
    pub database_type: String,
    pub nats_enabled: bool,
    pub use_llm: bool,
    pub execution_backend: String,
    pub heartbeat_enabled: bool,
    pub config_source: ConfigSource,
    pub validation_errors: Vec<ValidationError>,
}

impl AppConfig {
    pub fn summary(&self) -> ConfigSummary {
        let validation_errors = self.validate();

        let execution_backend = if self.execution.firecracker.enabled {
            "firecracker"
        } else if self.execution.gvisor.enabled {
            "gvisor"
        } else if self.execution.docker.enabled {
            "docker"
        } else {
            "none"
        };

        ConfigSummary {
            server: self.server.clone(),
            auth_enabled: !self.auth.disabled,
            database_type: self.database.db_type.clone(),
            nats_enabled: self.nats.enabled,
            use_llm: self.agent.use_llm,
            execution_backend: execution_backend.to_string(),
            heartbeat_enabled: self.heartbeat.enabled,
            config_source: self.config_source,
            validation_errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.agent.max_iterations, 50);
    }

    #[test]
    fn test_config_validation() {
        let config = AppConfig::default();
        let errors = config.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_to_env_vars() {
        let config = AppConfig::default();
        let vars = config.to_env_vars();
        assert!(vars.contains_key("APEX_PORT"));
        assert!(vars.contains_key("APEX_SHARED_SECRET"));
    }
}
