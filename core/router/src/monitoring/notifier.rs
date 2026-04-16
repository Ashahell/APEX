//! Notification dispatcher for sending alerts

use crate::monitoring::models::{MonitorEvent, Notification, NotifyMode};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Notification channel
#[derive(Debug, Clone)]
pub enum NotificationChannel {
    /// WebSocket push to UI
    WebSocket,
    /// SSE stream
    SSE,
    /// HTTP webhook
    Webhook { url: String },
    /// Telegram bot
    Telegram { bot_token: String, chat_id: String },
    /// Email via SMTP
    Email { smtp_config: SmtpConfig, recipients: Vec<String> },
}

/// SMTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub use_tls: bool,
}

use serde::{Deserialize, Serialize};

/// Notification dispatcher
#[derive(Debug, Default)]
pub struct NotificationDispatcher {
    /// Configured channels
    channels: Vec<NotificationChannel>,
    /// Statistics
    stats: NotificationStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationStats {
    pub total_sent: u64,
    pub by_channel: HashMap<String, u64>,
    pub by_mode: HashMap<String, u64>,
}

use std::collections::HashMap;

impl NotificationDispatcher {
    /// Create a new dispatcher
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a notification channel
    pub fn add_channel(&mut self, channel: NotificationChannel) {
        self.channels.push(channel);
    }

    /// Clear all channels
    pub fn clear_channels(&mut self) {
        self.channels.clear();
    }

    /// Dispatch a notification based on mode
    pub async fn dispatch(&mut self, notification: &Notification) -> DispatchResult {
        let mut results = Vec::new();

        // Check if notification should be sent based on mode
        let should_send = match &notification.mode {
            NotifyMode::All => true,
            NotifyMode::Result => matches!(
                notification.event,
                MonitorEvent::AgentEnd { .. }
            ),
            NotifyMode::Error => matches!(
                notification.event,
                MonitorEvent::AgentEnd { success: false, .. }
            ),
            NotifyMode::Off => false,
        };

        if !should_send {
            return DispatchResult::Skipped;
        }

        // Send to all channels
        for channel in &self.channels {
            let result = self.send_to_channel(channel, notification).await;
            results.push(result);
        }

        // Update stats
        self.stats.total_sent += 1;
        *self.stats.by_mode.entry(format!("{:?}", notification.mode)).or_insert(0) += 1;

        if results.iter().all(|r| matches!(r, DispatchResult::Success)) {
            DispatchResult::Success
        } else {
            DispatchResult::PartialFailure
        }
    }

    /// Send to a specific channel
    async fn send_to_channel(
        &self,
        channel: &NotificationChannel,
        notification: &Notification,
    ) -> DispatchResult {
        match channel {
            NotificationChannel::WebSocket => {
                // Handled by WebSocketManager in the main app
                // This is a placeholder for the interface
                DispatchResult::Success
            }
            NotificationChannel::SSE => {
                // Handled by SSE stream manager
                DispatchResult::Success
            }
            NotificationChannel::Webhook { url } => {
                self.send_webhook(url, notification).await
            }
            NotificationChannel::Telegram { bot_token, chat_id } => {
                self.send_telegram(bot_token, chat_id, notification).await
            }
            NotificationChannel::Email { smtp_config, recipients } => {
                self.send_email(smtp_config, recipients, notification).await
            }
        }
    }

    /// Send HTTP webhook
    async fn send_webhook(&self, url: &str, notification: &Notification) -> DispatchResult {
        let client = reqwest::Client::new();
        
        match client
            .post(url)
            .json(&notification)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => DispatchResult::Success,
            Ok(resp) => DispatchResult::Failed(format!("HTTP {}", resp.status())),
            Err(e) => DispatchResult::Failed(e.to_string()),
        }
    }

    /// Send Telegram message
    async fn send_telegram(
        &self,
        bot_token: &str,
        chat_id: &str,
        notification: &Notification,
    ) -> DispatchResult {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            bot_token
        );

        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": notification.message,
            "parse_mode": "Markdown",
        });

        match client
            .post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => DispatchResult::Success,
            Ok(resp) => DispatchResult::Failed(format!("Telegram API {}", resp.status())),
            Err(e) => DispatchResult::Failed(e.to_string()),
        }
    }

    /// Send email
    async fn send_email(
        &self,
        _config: &SmtpConfig,
        _recipients: &[String],
        _notification: &Notification,
    ) -> DispatchResult {
        // Email sending would use lettre or similar SMTP library
        // For now, placeholder implementation
        DispatchResult::Success
    }

    /// Get statistics
    pub fn stats(&self) -> &NotificationStats {
        &self.stats
    }
}

/// Result of dispatch attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DispatchResult {
    Success,
    Failed(String),
    Skipped,
    /// Partial failure - some channels succeeded, some failed
    PartialFailure,
}

/// Thread-safe wrapper
#[derive(Debug, Clone, Default)]
pub struct SharedNotificationDispatcher(pub Arc<RwLock<NotificationDispatcher>>);

impl SharedNotificationDispatcher {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(NotificationDispatcher::new())))
    }

    pub async fn add_channel(&self, channel: NotificationChannel) {
        let mut dispatcher = self.0.write().await;
        dispatcher.add_channel(channel);
    }

    pub async fn clear_channels(&self) {
        let mut dispatcher = self.0.write().await;
        dispatcher.clear_channels();
    }

    pub async fn dispatch(&self, notification: &Notification) -> DispatchResult {
        let mut dispatcher = self.0.write().await;
        dispatcher.dispatch(notification).await
    }

    pub async fn stats(&self) -> NotificationStats {
        let dispatcher = self.0.read().await;
        dispatcher.stats().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_notification_mode_filtering() {
        let dispatcher = SharedNotificationDispatcher::new();

        let all_notification = Notification::new(
            MonitorEvent::AgentStart {
                task_id: "test".to_string(),
                prompt: "Test".to_string(),
            },
            NotifyMode::All,
            "Test message".to_string(),
        );

        let result = dispatcher.dispatch(&all_notification).await;
        assert!(matches!(result, DispatchResult::Success));

        let off_notification = Notification::new(
            MonitorEvent::AgentStart {
                task_id: "test".to_string(),
                prompt: "Test".to_string(),
            },
            NotifyMode::Off,
            "Test message".to_string(),
        );

        let result = dispatcher.dispatch(&off_notification).await;
        assert!(matches!(result, DispatchResult::Skipped));
    }
}
