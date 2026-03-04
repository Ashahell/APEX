use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

use crate::message_bus::{DeepTaskMessage, MessageBus, SkillExecutionMessage};

pub struct T3ConfirmationWorker {
    message_bus: MessageBus,
}

impl T3ConfirmationWorker {
    pub fn new(message_bus: MessageBus) -> Self {
        Self { message_bus }
    }

    pub async fn run(self) {
        let mut rx = self.message_bus.subscribe_confirmations();
        let message_bus = self.message_bus;

        loop {
            match rx.recv().await {
                Ok(message) => {
                    if message.tier == "T3" && message.confirmed {
                        tracing::info!(
                            task_id = %message.task_id,
                            "T3 confirmation received, waiting 5 seconds before execution"
                        );

                        sleep(Duration::from_secs(5)).await;

                        tracing::info!(
                            task_id = %message.task_id,
                            "T3 delay complete, proceeding with execution"
                        );

                        if let Some(skill_name) = &message.skill_name {
                            message_bus.publish_skill(SkillExecutionMessage {
                                task_id: message.task_id.clone(),
                                skill_name: skill_name.clone(),
                                input: serde_json::Value::Null,
                                permission_tier: message.tier.clone(),
                            });
                        } else {
                            message_bus.publish_deep_task(DeepTaskMessage {
                                task_id: message.task_id.clone(),
                                content: message.action,
                                max_steps: 3,
                                budget_usd: 1.0,
                                time_limit_secs: None,
                                permission_tier: message.tier.clone(),
                            });
                        }
                    } else if message.confirmed {
                        if let Some(skill_name) = &message.skill_name {
                            message_bus.publish_skill(SkillExecutionMessage {
                                task_id: message.task_id.clone(),
                                skill_name: skill_name.clone(),
                                input: serde_json::Value::Null,
                                permission_tier: message.tier.clone(),
                            });
                        } else {
                            message_bus.publish_deep_task(DeepTaskMessage {
                                task_id: message.task_id.clone(),
                                content: message.action,
                                max_steps: 3,
                                budget_usd: 1.0,
                                time_limit_secs: None,
                                permission_tier: message.tier.clone(),
                            });
                        }
                    }
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("T3 worker: message bus closed");
                    break;
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("T3 worker: lagged behind, skipping message");
                }
            }
        }
    }
}
