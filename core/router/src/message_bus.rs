use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct MessageBus {
    sender: broadcast::Sender<TaskMessage>,
    skill_sender: broadcast::Sender<SkillExecutionMessage>,
    deep_task_sender: broadcast::Sender<DeepTaskMessage>,
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeepTaskMessage {
    pub task_id: String,
    pub content: String,
    pub max_steps: u32,
    pub budget_usd: f64,
    pub time_limit_secs: Option<u64>,
}

impl MessageBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        let (skill_sender, _) = broadcast::channel(capacity);
        let (deep_task_sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            skill_sender,
            deep_task_sender,
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

    pub fn subscribe(&self) -> broadcast::Receiver<TaskMessage> {
        self.sender.subscribe()
    }

    pub fn subscribe_skills(&self) -> broadcast::Receiver<SkillExecutionMessage> {
        self.skill_sender.subscribe()
    }

    pub fn subscribe_deep_tasks(&self) -> broadcast::Receiver<DeepTaskMessage> {
        self.deep_task_sender.subscribe()
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new(100)
    }
}
