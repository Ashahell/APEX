#[cfg(feature = "nats")]
use nats::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::message_bus::{
    ConfirmationMessage, DeepTaskMessage, SkillExecutionMessage, TaskMessage,
};
use crate::unified_config::AppConfig;

#[derive(Clone)]
pub enum MessageBusBackend {
    Local(broadcast::Sender<TaskMessage>),
    #[cfg(feature = "nats")]
    Nats(Arc<NatsMessageBus>),
}

#[cfg(feature = "nats")]
#[derive(Clone)]
pub struct NatsMessageBus {
    connection: Arc<RwLock<Option<Connection>>>,
    subject_prefix: String,
}

#[cfg(feature = "nats")]
impl NatsMessageBus {
    pub fn new(nats_url: &str, subject_prefix: &str) -> Result<Self, nats::Error> {
        let connection = Connection::connect(nats_url)?;
        Ok(Self {
            connection: Arc::new(RwLock::new(Some(connection))),
            subject_prefix: subject_prefix.to_string(),
        })
    }

    pub async fn publish(&self, subject: &str, payload: &[u8]) -> Result<(), String> {
        let conn_guard = self.connection.read().await;
        if let Some(conn) = conn_guard.as_ref() {
            conn.publish(subject, payload).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("NATS not connected".to_string())
        }
    }

    pub async fn subscribe(&self, subject: &str) -> Result<nats::Subscription, String> {
        let conn_guard = self.connection.read().await;
        if let Some(conn) = conn_guard.as_ref() {
            conn.subscribe(subject).map_err(|e| e.to_string())
        } else {
            Err("NATS not connected".to_string())
        }
    }

    pub fn subject(&self, name: &str) -> String {
        format!("{}.{}", self.subject_prefix, name)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NatsConfig {
    pub enabled: bool,
    pub url: String,
    pub subject_prefix: String,
}

impl NatsConfig {
    pub fn from_env() -> Self {
        Self::from_config(&AppConfig::global())
    }

    pub fn from_config(config: &AppConfig) -> Self {
        NatsConfig {
            enabled: config.nats.enabled,
            url: config.nats.url.clone(),
            subject_prefix: config.nats.subject_prefix.clone(),
        }
    }
}

pub struct DistributedMessageBus {
    local_bus: broadcast::Sender<TaskMessage>,
    #[cfg(feature = "nats")]
    nats_bus: Option<Arc<NatsMessageBus>>,
    subject_prefix: String,
}

impl DistributedMessageBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            local_bus: sender,
            #[cfg(feature = "nats")]
            nats_bus: None,
            subject_prefix: "apex".to_string(),
        }
    }

    #[cfg(feature = "nats")]
    pub fn with_nats(mut self, nats_url: &str, prefix: &str) -> Result<Self, nats::Error> {
        let nats = Arc::new(NatsMessageBus::new(nats_url, prefix)?);
        self.nats_bus = Some(nats);
        self.subject_prefix = prefix.to_string();
        Ok(self)
    }

    pub fn publish(&self, message: TaskMessage) {
        let _ = self.local_bus.send(message.clone());

        #[cfg(feature = "nats")]
        if let Some(nats) = &self.nats_bus {
            if let Ok(payload) = serde_json::to_vec(&message) {
                let subject = nats.subject("tasks");
                let _ = nats.publish(&subject, &payload);
            }
        }
    }

    pub fn publish_skill(&self, message: SkillExecutionMessage) {
        #[cfg(feature = "nats")]
        if let Some(nats) = &self.nats_bus {
            if let Ok(payload) = serde_json::to_vec(&message) {
                let subject = nats.subject("skills");
                let _ = nats.publish(&subject, &payload);
            }
        }
    }

    pub fn publish_deep_task(&self, message: DeepTaskMessage) {
        #[cfg(feature = "nats")]
        if let Some(nats) = &self.nats_bus {
            if let Ok(payload) = serde_json::to_vec(&message) {
                let subject = nats.subject("deep_tasks");
                let _ = nats.publish(&subject, &payload);
            }
        }
    }

    pub fn publish_confirmation(&self, message: ConfirmationMessage) {
        #[cfg(feature = "nats")]
        if let Some(nats) = &self.nats_bus {
            if let Ok(payload) = serde_json::to_vec(&message) {
                let subject = nats.subject("confirmations");
                let _ = nats.publish(&subject, &payload);
            }
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TaskMessage> {
        self.local_bus.subscribe()
    }

    #[cfg(feature = "nats")]
    pub async fn start_nats_listener(&self) {
        if let Some(nats) = &self.nats_bus {
            let nats_clone = nats.clone();
            let sender = self.local_bus.clone();

            tokio::spawn(async move {
                if let Ok(sub) = nats_clone.subscribe(&nats_clone.subject("tasks")).await {
                    for msg in sub.messages() {
                        if let Ok(task_msg) = serde_json::from_slice::<TaskMessage>(msg.data()) {
                            let _ = sender.send(task_msg);
                        }
                    }
                }
            });
        }
    }
}

impl Default for DistributedMessageBus {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_config_default_values() {
        let config = NatsConfig {
            enabled: false,
            url: "127.0.0.1:4222".to_string(),
            subject_prefix: "apex".to_string(),
        };
        
        assert!(!config.enabled);
        assert_eq!(config.url, "127.0.0.1:4222");
        assert_eq!(config.subject_prefix, "apex");
    }

    #[test]
    fn test_distributed_message_bus_local() {
        let bus = DistributedMessageBus::new(10);
        
        let receiver = bus.subscribe();
        assert!(!receiver.is_closed());
    }
}
