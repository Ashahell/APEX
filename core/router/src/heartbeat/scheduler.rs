use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;

use super::{AutonomyAction, HeartbeatConfig, WakeCycle};

pub type WakeCallback = Box<
    dyn Fn() -> Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static> + Send + Sync,
>;

#[derive(Clone)]
pub struct HeartbeatScheduler {
    config: HeartbeatConfig,
    is_running: Arc<RwLock<bool>>,
    on_wake: Arc<RwLock<Option<WakeCallback>>>,
    last_wake: Arc<RwLock<Option<String>>>,
    wake_count: Arc<RwLock<u64>>,
}

impl HeartbeatScheduler {
    pub fn new(config: HeartbeatConfig) -> Self {
        Self {
            config,
            is_running: Arc::new(RwLock::new(false)),
            on_wake: Arc::new(RwLock::new(None)),
            last_wake: Arc::new(RwLock::new(None)),
            wake_count: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn start(&self) {
        {
            let mut running = self.is_running.write().await;
            if *running {
                tracing::info!("Heartbeat scheduler already running");
                return;
            }
            *running = true;
        }

        tracing::info!(
            interval_minutes = self.config.interval_minutes,
            "Starting heartbeat scheduler"
        );

        let mut ticker = interval(Duration::from_secs(self.config.interval_minutes * 60));

        loop {
            ticker.tick().await;

            let should_wake = {
                let running = self.is_running.read().await;
                *running
            };

            if !should_wake {
                break;
            }

            self.trigger_wake().await;
        }
    }

    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;
        tracing::info!("Heartbeat scheduler stopped");
    }

    async fn trigger_wake(&self) {
        let wake_id = ulid::Ulid::new().to_string();

        {
            let mut count = self.wake_count.write().await;
            *count += 1;
            tracing::info!(wake_id = %wake_id, wake_number = *count, "Heartbeat triggered");
        }

        {
            let mut last = self.last_wake.write().await;
            *last = Some(wake_id.clone());
        }

        let _ = self.execute_wake_cycle(wake_id).await;
    }

    async fn execute_wake_cycle(&self, wake_id: String) {
        let mut cycle = WakeCycle::new();
        cycle.wake_id = wake_id;

        let actions = self.decide_actions().await;
        cycle.actions_planned = actions.clone();

        tracing::info!(actions_planned = actions.len(), "Executing wake cycle");

        for action in actions {
            if cycle.actions_executed >= self.config.max_actions_per_wake {
                tracing::warn!("Max actions per wake reached");
                break;
            }

            let requires_approval = action.requires_approval(&self.config);

            if requires_approval {
                tracing::info!(action = ?action, "Action requires approval, queuing");
            } else {
                self.execute_action(&action).await;
                cycle.actions_executed += 1;
            }
        }

        cycle.complete(true);

        tracing::info!(
            wake_id = %cycle.wake_id,
            actions_executed = cycle.actions_executed,
            "Wake cycle complete"
        );
    }

    async fn decide_actions(&self) -> Vec<AutonomyAction> {
        let mut actions = vec![];

        let should_self_maintain = self.should_run_self_maintenance().await;
        if should_self_maintain {
            actions.push(AutonomyAction::SelfMaintenance {
                action_type: "memory_cleanup".to_string(),
                description: "Clean up old temporary files".to_string(),
            });
        }

        if actions.is_empty() {
            tracing::debug!("No autonomous actions decided for this wake");
        }

        actions
    }

    async fn should_run_self_maintenance(&self) -> bool {
        let wake_count = *self.wake_count.read().await;
        wake_count % 10 == 0
    }

    async fn execute_action(&self, action: &AutonomyAction) {
        tracing::info!(action = ?action, "Executing autonomous action");

        match action {
            AutonomyAction::SelfMaintenance {
                action_type,
                description,
            } => {
                tracing::info!(action_type = %action_type, description = %description, "Running self-maintenance");
            }
            AutonomyAction::GoalAdvancement {
                goal_id,
                description,
            } => {
                tracing::info!(goal_id = %goal_id, description = %description, "Advancing goal");
            }
            AutonomyAction::SocialCoordination { agent_id, action } => {
                tracing::info!(agent_id = %agent_id, action = %action, "Social coordination");
            }
            AutonomyAction::Learning {
                capability,
                description,
            } => {
                tracing::info!(capability = %capability, description = %description, "Learning new capability");
            }
            AutonomyAction::IdentityModification { field, reason } => {
                tracing::warn!(field = %field, reason = %reason, "Identity modification requires T3");
            }
        }
    }

    pub async fn get_status(&self) -> HeartbeatStatus {
        let running = *self.is_running.read().await;
        let last_wake = self.last_wake.read().await.clone();
        let wake_count = *self.wake_count.read().await;

        HeartbeatStatus {
            running,
            last_wake,
            wake_count,
            config: self.config.clone(),
        }
    }

    pub async fn force_wake(&self) {
        tracing::info!("Forcing immediate wake cycle");
        self.trigger_wake().await;
    }

    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    pub async fn get_wake_count(&self) -> u64 {
        *self.wake_count.read().await
    }

    pub async fn get_last_wake(&self) -> Option<String> {
        self.last_wake.read().await.clone()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HeartbeatStatus {
    pub running: bool,
    pub last_wake: Option<String>,
    pub wake_count: u64,
    pub config: HeartbeatConfig,
}

pub struct AutonomyDecisionEngine {
    config: HeartbeatConfig,
}

impl AutonomyDecisionEngine {
    pub fn new(config: HeartbeatConfig) -> Self {
        Self { config }
    }

    pub async fn decide(&self, context: &DecisionContext) -> Vec<AutonomyAction> {
        let mut actions = vec![];

        if let Some(goal) = self.assess_goals(&context).await {
            actions.push(goal);
        }

        if self.config.social_context_enabled {
            if let Some(social) = self.assess_social(&context).await {
                actions.push(social);
            }
        }

        if self.should_self_maintain() {
            actions.push(AutonomyAction::SelfMaintenance {
                action_type: "health_check".to_string(),
                description: "Check system health".to_string(),
            });
        }

        actions
    }

    async fn assess_goals(&self, context: &DecisionContext) -> Option<AutonomyAction> {
        if context.active_goals.is_empty() {
            return None;
        }

        let high_priority = context.active_goals.iter().find(|g| g.priority >= 8);

        if let Some(goal) = high_priority {
            Some(AutonomyAction::GoalAdvancement {
                goal_id: goal.id.clone(),
                description: goal.description.clone(),
            })
        } else {
            None
        }
    }

    async fn assess_social(&self, context: &DecisionContext) -> Option<AutonomyAction> {
        if context.pending_notifications > 0 {
            Some(AutonomyAction::SocialCoordination {
                agent_id: "moltbook".to_string(),
                action: "check_notifications".to_string(),
            })
        } else {
            None
        }
    }

    fn should_self_maintain(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        let minute = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 60;
        minute % 30 == 0
    }
}

#[derive(Debug, Clone)]
pub struct DecisionContext {
    pub active_goals: Vec<GoalContext>,
    pub pending_notifications: u32,
    pub last_activity: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GoalContext {
    pub id: String,
    pub description: String,
    pub priority: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_status() {
        let config = HeartbeatConfig::default();
        let scheduler = HeartbeatScheduler::new(config);

        let status = scheduler.get_status().await;
        assert!(!status.running);
        assert_eq!(status.wake_count, 0);
    }

    #[test]
    fn test_decision_engine() {
        use super::{AutonomyDecisionEngine, DecisionContext, GoalContext, HeartbeatConfig};

        let config = HeartbeatConfig::default();
        let engine = AutonomyDecisionEngine::new(config);

        let context = DecisionContext {
            active_goals: vec![GoalContext {
                id: "test-goal".to_string(),
                description: "Test goal".to_string(),
                priority: 9,
            }],
            pending_notifications: 0,
            last_activity: None,
        };

        let actions = futures::executor::block_on(engine.decide(&context));
        assert!(actions.len() >= 1); // Should have goal advancement
    }
}
