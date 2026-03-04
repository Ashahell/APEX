use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub channels: ChannelConfigs,
    pub agent: AgentConfig,
    pub execution: ExecutionConfig,
    pub database: DatabaseConfigSection,
    pub nats: NatsConfig,
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
            port: 3000,
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
    pub max_iterations: u32,
    pub max_budget_cents: i64,
    pub context_window_tokens: usize,
    pub model: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            max_budget_cents: 500,
            context_window_tokens: 4096,
            model: "qwen3-4b".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub isolation: String,
    pub firecracker: Option<FirecrackerConfig>,
    pub gvisor: Option<GvisorConfig>,
    pub docker: Option<DockerConfig>,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            isolation: "docker".to_string(),
            firecracker: Some(FirecrackerConfig::default()),
            gvisor: None,
            docker: Some(DockerConfig::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirecrackerConfig {
    pub vcpus: u32,
    pub memory_mib: u64,
    pub timeout_secs: u64,
    pub kernel_path: Option<String>,
    pub rootfs_path: Option<String>,
}

impl Default for FirecrackerConfig {
    fn default() -> Self {
        Self {
            vcpus: 2,
            memory_mib: 2048,
            timeout_secs: 60,
            kernel_path: std::env::var("APEX_VM_KERNEL").ok(),
            rootfs_path: std::env::var("APEX_VM_ROOTFS").ok(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GvisorConfig {
    pub runsc_path: Option<String>,
}

impl Default for GvisorConfig {
    fn default() -> Self {
        Self {
            runsc_path: std::env::var("APEX_RUNSC_PATH").ok(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub image: String,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            image: std::env::var("APEX_DOCKER_IMAGE")
                .unwrap_or_else(|_| "apex-execution:latest".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfigSection {
    pub db_type: String,
    pub connection_string: String,
}

impl Default for DatabaseConfigSection {
    fn default() -> Self {
        Self {
            db_type: "sqlite".to_string(),
            connection_string: std::env::var("APEX_DATABASE_URL")
                .or_else(|_| std::env::var("DATABASE_URL"))
                .unwrap_or_else(|_| "sqlite:apex.db".to_string()),
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
                .map(|v| v == "1")
                .unwrap_or(false),
            url: std::env::var("APEX_NATS_URL").unwrap_or_else(|_| "127.0.0.1:4222".to_string()),
            subject_prefix: std::env::var("APEX_NATS_SUBJECT_PREFIX")
                .unwrap_or_else(|_| "apex".to_string()),
        }
    }
}

impl Config {
    pub fn load(path: Option<&str>) -> Result<Self, ConfigError> {
        let config_path = path.map(PathBuf::from).or_else(|| Self::find_config_file());

        if let Some(path) = config_path {
            if path.exists() {
                return Self::load_from_file(&path);
            }
        }

        Ok(Self::default())
    }

    fn find_config_file() -> Option<PathBuf> {
        let candidates = vec![
            PathBuf::from("apex.yaml"),
            PathBuf::from("apex.yml"),
            PathBuf::from("config/apex.yaml"),
            PathBuf::from("config/apex.yml"),
            PathBuf::from(".apex.yaml"),
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

        let mut config: Config = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.extension().and_then(|s| s.to_str()) == Some("yml")
        {
            serde_yaml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?
        } else {
            serde_json::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?
        };

        config.apply_env_overrides();
        Ok(config)
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(secret) = std::env::var("APEX_SHARED_SECRET") {
            self.auth.shared_secret = secret;
        }
        if std::env::var("APEX_AUTH_DISABLED").is_ok() {
            self.auth.disabled = std::env::var("APEX_AUTH_DISABLED").unwrap() == "1";
        }
        if let Ok(port) = std::env::var("APEX_PORT") {
            if let Ok(p) = port.parse() {
                self.server.port = p;
            }
        }
        if std::env::var("APEX_NATS_ENABLED").is_ok() {
            self.nats.enabled = std::env::var("APEX_NATS_ENABLED").unwrap() == "1";
        }
        if let Ok(url) = std::env::var("APEX_NATS_URL") {
            self.nats.url = url;
        }
    }

    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        vars.insert("APEX_PORT".to_string(), self.server.port.to_string());
        vars.insert(
            "APEX_SHARED_SECRET".to_string(),
            self.auth.shared_secret.clone(),
        );

        if self.auth.disabled {
            vars.insert("APEX_AUTH_DISABLED".to_string(), "1".to_string());
        }

        if self.nats.enabled {
            vars.insert("APEX_NATS_ENABLED".to_string(), "1".to_string());
            vars.insert("APEX_NATS_URL".to_string(), self.nats.url.clone());
            vars.insert(
                "APEX_NATS_SUBJECT_PREFIX".to_string(),
                self.nats.subject_prefix.clone(),
            );
        }

        vars.insert(
            "APEX_DATABASE_URL".to_string(),
            self.database.connection_string.clone(),
        );

        vars
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            auth: AuthConfig::default(),
            channels: ChannelConfigs::default(),
            agent: AgentConfig::default(),
            execution: ExecutionConfig::default(),
            database: DatabaseConfigSection::default(),
            nats: NatsConfig::default(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Config not found")]
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.agent.max_iterations, 50);
    }

    #[test]
    fn test_config_env_override() {
        std::env::set_var("APEX_PORT", "4000");
        let config = Config::default();
        assert_eq!(config.server.port, 4000);
    }

    #[test]
    fn test_to_env_vars() {
        let config = Config::default();
        let vars = config.to_env_vars();
        assert!(vars.contains_key("APEX_PORT"));
        assert!(vars.contains_key("APEX_SHARED_SECRET"));
    }
}
