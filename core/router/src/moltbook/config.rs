use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoltbookConfig {
    pub enabled: bool,
    pub server_url: String,
    pub agent_id: String,
    pub client_cert_path: Option<PathBuf>,
    pub client_key_path: Option<PathBuf>,
    pub ca_cert_path: Option<PathBuf>,
    pub poll_interval_secs: u64,
    pub max_connections: u32,
}

impl Default for MoltbookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            server_url: "https://moltbook.local".to_string(),
            agent_id: String::new(),
            client_cert_path: None,
            client_key_path: None,
            ca_cert_path: None,
            poll_interval_secs: 60,
            max_connections: 10,
        }
    }
}
