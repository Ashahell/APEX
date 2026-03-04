use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct MessageBus {
    sender: broadcast::Sender<TaskMessage>,
    skill_sender: broadcast::Sender<SkillExecutionMessage>,
    deep_task_sender: broadcast::Sender<DeepTaskMessage>,
    confirmation_sender: broadcast::Sender<ConfirmationMessage>,
}

#[derive(Clone, Debug)]
pub struct TaskMessage {
    pub task_id: String,
    pub tier: String,
    pub content: String,
    pub channel: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillExecutionMessage {
    pub task_id: String,
    pub skill_name: String,
    pub input: serde_json::Value,
    pub permission_tier: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeepTaskMessage {
    pub task_id: String,
    pub content: String,
    pub max_steps: u32,
    pub budget_usd: f64,
    pub time_limit_secs: Option<u64>,
    pub permission_tier: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfirmationMessage {
    pub task_id: String,
    pub tier: String,
    pub action: String,
    pub skill_name: Option<String>,
    pub confirmed: bool,
    pub permission_tier: String,
}

impl MessageBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        let (skill_sender, _) = broadcast::channel(capacity);
        let (deep_task_sender, _) = broadcast::channel(capacity);
        let (confirmation_sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            skill_sender,
            deep_task_sender,
            confirmation_sender,
        }
    }

    pub fn publish(&self, message: TaskMessage) {
        let _ = self.sender.send(message);
    }

    pub fn publish_skill(&self, message: SkillExecutionMessage) {
        let _ = self.skill_sender.send(message);
    }

    pub fn publish_deep_task(&self, message: DeepTaskMessage) {
        let _ = self.deep_task_sender.send(message);
    }

    pub fn publish_confirmation(&self, message: ConfirmationMessage) {
        let _ = self.confirmation_sender.send(message);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TaskMessage> {
        self.sender.subscribe()
    }

    pub fn subscribe_skills(&self) -> broadcast::Receiver<SkillExecutionMessage> {
        self.skill_sender.subscribe()
    }

    pub fn subscribe_deep_tasks(&self) -> broadcast::Receiver<DeepTaskMessage> {
        self.deep_task_sender.subscribe()
    }

    pub fn subscribe_confirmations(&self) -> broadcast::Receiver<ConfirmationMessage> {
        self.confirmation_sender.subscribe()
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_bus_new() {
        let bus = MessageBus::new(10);
        let _receiver = bus.subscribe();
    }

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let bus = MessageBus::new(10);
        let mut receiver = bus.subscribe();
        
        bus.publish(TaskMessage {
            task_id: "test-1".to_string(),
            tier: "deep".to_string(),
            content: "test task".to_string(),
            channel: Some("default".to_string()),
        });
        
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.task_id, "test-1");
    }

    #[tokio::test]
    async fn test_publish_skill_message() {
        let bus = MessageBus::new(10);
        let mut receiver = bus.subscribe_skills();
        
        bus.publish_skill(SkillExecutionMessage {
            task_id: "test-1".to_string(),
            skill_name: "shell.execute".to_string(),
            input: serde_json::json!({"command": "ls"}),
            permission_tier: "T3".to_string(),
        });
        
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.skill_name, "shell.execute");
    }

    #[tokio::test]
    async fn test_publish_deep_task_message() {
        let bus = MessageBus::new(10);
        let mut receiver = bus.subscribe_deep_tasks();
        
        bus.publish_deep_task(DeepTaskMessage {
            task_id: "test-1".to_string(),
            content: "build a website".to_string(),
            max_steps: 10,
            budget_usd: 1.0,
            time_limit_secs: Some(60),
            permission_tier: "T2".to_string(),
        });
        
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.max_steps, 10);
    }

    #[tokio::test]
    async fn test_publish_confirmation_message() {
        let bus = MessageBus::new(10);
        let mut receiver = bus.subscribe_confirmations();
        
        bus.publish_confirmation(ConfirmationMessage {
            task_id: "test-1".to_string(),
            tier: "T2".to_string(),
            action: "delete file".to_string(),
            skill_name: Some("shell.execute".to_string()),
            confirmed: false,
            permission_tier: "T2".to_string(),
        });
        
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.action, "delete file");
    }

    #[test]
    fn test_message_bus_default() {
        let bus = MessageBus::default();
        let _receiver = bus.subscribe();
    }
}
