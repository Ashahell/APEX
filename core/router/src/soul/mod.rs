use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulIdentity {
    pub name: String,
    pub version: String,
    pub created: String,
    pub wake_count: u64,
    pub purpose: String,
    pub values: Vec<Value>,
    pub capabilities: Vec<Capability>,
    pub autonomy_config: AutonomyConfig,
    pub memory_strategy: MemoryStrategy,
    pub relationships: Vec<Relationship>,
    pub affiliations: Vec<Affiliation>,
    pub current_goals: Vec<Goal>,
    pub reflections: Vec<Reflection>,
    pub constitution: Constitution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub name: String,
    pub description: String,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyConfig {
    pub heartbeat_interval_minutes: u64,
    pub max_actions_per_wake: u32,
    pub require_approval_for: Vec<String>,
    pub social_context_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStrategy {
    pub retention_days: u32,
    pub forgetting_threshold_days: u32,
    pub emphasis_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub agent_id: String,
    pub relationship_type: String,
    pub trust_level: f32,
    pub last_contact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affiliation {
    pub name: String,
    pub role: String,
    pub joined: String,
    pub tenets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub description: String,
    pub status: String,
    pub priority: u32,
    pub deadline: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    pub timestamp: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    pub version: String,
    pub hash: String,
    pub immutable_values: Vec<String>,
    pub emergency_protocol: String,
    pub self_destruct_conditions: Vec<String>,
}

impl Default for SoulIdentity {
    fn default() -> Self {
        Self {
            name: "APEX Agent".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            created: chrono::Utc::now().to_rfc3339(),
            wake_count: 0,
            purpose: "I am an autonomous agent that executes tasks on behalf of my user."
                .to_string(),
            values: vec![
                Value {
                    name: "Security".to_string(),
                    description: "Operating within strict permission tiers".to_string(),
                    priority: 1,
                },
                Value {
                    name: "Transparency".to_string(),
                    description: "Every decision is logged".to_string(),
                    priority: 2,
                },
                Value {
                    name: "Growth".to_string(),
                    description: "Learning from experience".to_string(),
                    priority: 3,
                },
            ],
            capabilities: vec![],
            autonomy_config: AutonomyConfig::default(),
            memory_strategy: MemoryStrategy::default(),
            relationships: vec![],
            affiliations: vec![],
            current_goals: vec![],
            reflections: vec![],
            constitution: Constitution::default(),
        }
    }
}

impl Default for AutonomyConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_minutes: 60,
            max_actions_per_wake: 3,
            require_approval_for: vec!["T1".to_string(), "T2".to_string(), "T3".to_string()],
            social_context_enabled: false,
        }
    }
}

impl Default for MemoryStrategy {
    fn default() -> Self {
        Self {
            retention_days: 90,
            forgetting_threshold_days: 30,
            emphasis_patterns: vec![
                "error".to_string(),
                "correction".to_string(),
                "success".to_string(),
            ],
        }
    }
}

impl Default for Constitution {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            hash: "sha256:default".to_string(),
            immutable_values: vec![
                "human_sovereignty".to_string(),
                "transparency".to_string(),
                "non_maleficence".to_string(),
                "integrity".to_string(),
            ],
            emergency_protocol: "restore_from_backup".to_string(),
            self_destruct_conditions: vec![
                "human_command".to_string(),
                "constitution_violation".to_string(),
            ],
        }
    }
}

pub mod loader;
pub mod constitution;

pub use loader::SoulLoader;
pub use loader::SoulError;
pub use constitution::ConstitutionManager;
pub use constitution::ConstitutionError;

use crate::unified_config::AppConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulConfig {
    pub soul_dir: PathBuf,
    pub fragments_dir: PathBuf,
    pub history_dir: PathBuf,
    pub backup_enabled: bool,
}

impl Default for SoulConfig {
    fn default() -> Self {
        let base = std::env::temp_dir().join("apex").join("soul");
        Self {
            soul_dir: base.clone(),
            fragments_dir: base.join("fragments"),
            history_dir: base.join("history"),
            backup_enabled: false,
        }
    }
}

impl SoulConfig {
    pub fn default_path() -> Self {
        Self::from_config(&AppConfig::global())
    }

    pub fn from_config(config: &AppConfig) -> Self {
        let base = config.soul.directory
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".apex")
                    .join("soul")
            });

        SoulConfig {
            soul_dir: base.clone(),
            fragments_dir: base.join("fragments"),
            history_dir: base.join("SOUL.md.history"),
            backup_enabled: config.soul.backup_enabled,
        }
    }

    pub fn from_env() -> Self {
        Self::from_config(&AppConfig::global())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_identity() {
        let identity = SoulIdentity::default();
        assert_eq!(identity.name, "APEX Agent");
        assert_eq!(identity.values.len(), 3);
    }

    #[test]
    fn test_constitution_defaults() {
        let constitution = Constitution::default();
        assert!(constitution
            .immutable_values
            .contains(&"human_sovereignty".to_string()));
    }

    #[test]
    fn test_autonomy_defaults() {
        let config = AutonomyConfig::default();
        assert_eq!(config.heartbeat_interval_minutes, 60);
        assert_eq!(config.max_actions_per_wake, 3);
    }
}
