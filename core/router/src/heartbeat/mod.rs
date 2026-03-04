pub mod scheduler;
pub use scheduler::HeartbeatScheduler;

use crate::unified_config::AppConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    pub enabled: bool,
    pub interval_minutes: u64,
    pub jitter_percent: u32,
    pub cooldown_seconds: u64,
    pub max_actions_per_wake: u32,
    pub require_approval_t1_plus: bool,
    pub social_context_enabled: bool,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self::from_config(&AppConfig::global())
    }
}

impl HeartbeatConfig {
    pub fn from_config(config: &AppConfig) -> Self {
        HeartbeatConfig {
            enabled: config.heartbeat.enabled,
            interval_minutes: config.heartbeat.interval_minutes,
            jitter_percent: config.heartbeat.jitter_percent,
            cooldown_seconds: config.heartbeat.cooldown_seconds,
            max_actions_per_wake: config.heartbeat.max_actions_per_wake,
            require_approval_t1_plus: config.heartbeat.require_approval_t1_plus,
            social_context_enabled: false,
        }
    }

    pub fn from_env() -> Self {
        Self::default()
    }

    pub fn effective_interval(&self) -> u64 {
        if self.jitter_percent == 0 {
            return self.interval_minutes;
        }
        let jitter = (self.interval_minutes as f64 * self.jitter_percent as f64 / 100.0) as u64;
        let min = self.interval_minutes.saturating_sub(jitter);
        let max = self.interval_minutes + jitter;
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        min + (seed % (max - min + 1))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AutonomyAction {
    SelfMaintenance {
        action_type: String,
        description: String,
    },
    GoalAdvancement {
        goal_id: String,
        description: String,
    },
    SocialCoordination {
        agent_id: String,
        action: String,
    },
    Learning {
        capability: String,
        description: String,
    },
    IdentityModification {
        field: String,
        reason: String,
    },
}

impl AutonomyAction {
    pub fn tier_requirement(&self) -> &str {
        match self {
            AutonomyAction::SelfMaintenance { .. } => "T0",
            AutonomyAction::GoalAdvancement { .. } => "T1",
            AutonomyAction::SocialCoordination { .. } => "T1",
            AutonomyAction::Learning { .. } => "T2",
            AutonomyAction::IdentityModification { .. } => "T3",
        }
    }

    pub fn requires_approval(&self, config: &HeartbeatConfig) -> bool {
        match self.tier_requirement() {
            "T0" => false,
            tier if tier == "T1" || tier == "T2" => config.require_approval_t1_plus,
            "T3" => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutonomousTask {
    pub id: String,
    pub action: AutonomyAction,
    pub status: TaskStatus,
    pub created_at: String,
    pub approved: bool,
    pub executed_at: Option<String>,
    pub result: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Approved,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Approved => "approved",
            TaskStatus::Executing => "executing",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WakeCycle {
    pub wake_id: String,
    pub started_at: String,
    pub actions_planned: Vec<AutonomyAction>,
    pub actions_executed: u32,
    pub completed_at: Option<String>,
    pub success: bool,
}

impl WakeCycle {
    pub fn new() -> Self {
        Self {
            wake_id: ulid::Ulid::new().to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
            actions_planned: vec![],
            actions_executed: 0,
            completed_at: None,
            success: true,
        }
    }

    pub fn complete(&mut self, success: bool) {
        self.completed_at = Some(chrono::Utc::now().to_rfc3339());
        self.success = success;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HeartbeatConfig::default();
        assert_eq!(config.interval_minutes, 60);
        assert_eq!(config.max_actions_per_wake, 3);
    }

    #[test]
    fn test_action_tier() {
        let action = AutonomyAction::SelfMaintenance {
            action_type: "memory_compress".to_string(),
            description: "Compress old memories".to_string(),
        };
        assert_eq!(action.tier_requirement(), "T0");
    }

    #[test]
    fn test_wake_cycle() {
        let mut cycle = WakeCycle::new();
        cycle.complete(true);
        assert!(cycle.completed_at.is_some());
        assert!(cycle.success);
    }
}
