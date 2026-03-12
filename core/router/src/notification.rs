use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;

/// External notification configuration (webhooks)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExternalNotificationConfig {
    pub discord_webhook_url: Option<String>,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub severity: String,
    pub read: bool,
    pub created_at_ms: i64,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotification {
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub severity: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct NotificationManager {
    notifications: Arc<RwLock<Vec<Notification>>>,
    max_notifications: usize,
    external_config: Arc<RwLock<ExternalNotificationConfig>>,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new(100)
    }
}

impl NotificationManager {
    pub fn new(max_notifications: usize) -> Self {
        Self {
            notifications: Arc::new(RwLock::new(Vec::new())),
            max_notifications,
            external_config: Arc::new(RwLock::new(ExternalNotificationConfig::default())),
        }
    }

    /// Get external notification configuration
    pub async fn get_external_config(&self) -> ExternalNotificationConfig {
        self.external_config.read().await.clone()
    }

    /// Update external notification configuration
    pub async fn set_external_config(&self, config: ExternalNotificationConfig) {
        *self.external_config.write().await = config;
    }

    /// Send external notification (Discord/Telegram webhook)
    pub async fn send_external_notification(&self, title: &str, message: &str) -> Result<(), String> {
        let config = self.external_config.read().await.clone();
        
        if !config.enabled {
            return Ok(());
        }

        // Send to Discord webhook if configured
        if let Some(discord_url) = &config.discord_webhook_url {
            let payload = serde_json::json!({
                "content": format!("**{}**\n{}", title, message)
            });
            
            let client = reqwest::Client::new();
            if let Err(e) = client.post(discord_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
            {
                tracing::warn!("Failed to send Discord notification: {}", e);
            }
        }

        // Send to Telegram if configured
        if let (Some(token), Some(chat_id)) = (&config.telegram_bot_token, &config.telegram_chat_id) {
            let telegram_url = format!("https://api.telegram.org/bot{}/sendMessage", token);
            let payload = serde_json::json!({
                "chat_id": chat_id,
                "text": format!("*{}*\n{}", title, message),
                "parse_mode": "Markdown"
            });
            
            let client = reqwest::Client::new();
            if let Err(e) = client.post(&telegram_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
            {
                tracing::warn!("Failed to send Telegram notification: {}", e);
            }
        }

        Ok(())
    }

    pub async fn list(&self, include_read: bool) -> Vec<Notification> {
        let notifications = self.notifications.read().await;
        if include_read {
            notifications.clone()
        } else {
            notifications.iter().filter(|n| !n.read).cloned().collect()
        }
    }

    pub async fn get(&self, id: &str) -> Option<Notification> {
        self.notifications.read().await.iter().find(|n| n.id == id).cloned()
    }

    pub async fn create(&self, create: CreateNotification) -> Notification {
        let notification = Notification {
            id: ulid::Ulid::new().to_string(),
            notification_type: create.notification_type,
            title: create.title,
            message: create.message,
            severity: create.severity,
            read: false,
            created_at_ms: Utc::now().timestamp_millis(),
            data: create.data,
        };
        
        let mut notifications = self.notifications.write().await;
        notifications.insert(0, notification.clone());
        
        if notifications.len() > self.max_notifications {
            notifications.pop();
        }
        
        notification
    }

    pub async fn mark_read(&self, id: &str) -> Option<Notification> {
        let mut notifications = self.notifications.write().await;
        if let Some(n) = notifications.iter_mut().find(|n| n.id == id) {
            n.read = true;
            return Some(n.clone());
        }
        None
    }

    pub async fn mark_all_read(&self) {
        let mut notifications = self.notifications.write().await;
        for n in notifications.iter_mut() {
            n.read = true;
        }
    }

    pub async fn delete(&self, id: &str) -> bool {
        let mut notifications = self.notifications.write().await;
        let len_before = notifications.len();
        notifications.retain(|n| n.id != id);
        notifications.len() < len_before
    }

    pub async fn clear_all(&self) {
        self.notifications.write().await.clear();
    }

    pub async fn unread_count(&self) -> usize {
        self.notifications.read().await.iter().filter(|n| !n.read).count()
    }

    pub async fn notify_task_complete(&self, task_id: &str, message: &str) {
        // Create in-app notification
        self.create(CreateNotification {
            notification_type: "task".to_string(),
            title: "Task Completed".to_string(),
            message: message.to_string(),
            severity: "info".to_string(),
            data: Some(serde_json::json!({ "task_id": task_id })),
        }).await;

        // Send external notification
        let _ = self.send_external_notification(
            &format!("Task Completed: {}", task_id),
            message,
        ).await;
    }

    pub async fn notify_task_failed(&self, task_id: &str, message: &str) {
        // Create in-app notification
        self.create(CreateNotification {
            notification_type: "task".to_string(),
            title: "Task Failed".to_string(),
            message: message.to_string(),
            severity: "error".to_string(),
            data: Some(serde_json::json!({ "task_id": task_id })),
        }).await;

        // Send external notification (always send failures)
        let _ = self.send_external_notification(
            &format!("Task Failed: {}", task_id),
            message,
        ).await;
    }

    pub async fn notify_confirmation(&self, task_id: &str, action: &str) {
        self.create(CreateNotification {
            notification_type: "confirmation".to_string(),
            title: "Confirmation Required".to_string(),
            message: format!("Action '{}' requires confirmation", action),
            severity: "warning".to_string(),
            data: Some(serde_json::json!({ "task_id": task_id, "action": action })),
        }).await;
    }

    pub async fn notify_system(&self, title: &str, message: &str, severity: &str) {
        self.create(CreateNotification {
            notification_type: "system".to_string(),
            title: title.to_string(),
            message: message.to_string(),
            severity: severity.to_string(),
            data: None,
        }).await;
    }
}
