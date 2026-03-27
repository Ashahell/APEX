use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: String,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub enabled: bool,
    pub secret: Option<String>,
    pub created_at_ms: i64,
    pub last_triggered_ms: Option<i64>,
    pub failure_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhook {
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: String,
    pub timestamp: i64,
    pub data: serde_json::Value,
}

#[derive(Clone)]
pub struct WebhookManager {
    webhooks: Arc<RwLock<Vec<Webhook>>>,
    http_client: Client,
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WebhookManager {
    pub fn new() -> Self {
        Self {
            webhooks: Arc::new(RwLock::new(Vec::new())),
            http_client: Client::new(),
        }
    }

    pub async fn list_webhooks(&self) -> Vec<Webhook> {
        self.webhooks.read().await.clone()
    }

    pub async fn get_webhook(&self, id: &str) -> Option<Webhook> {
        self.webhooks
            .read()
            .await
            .iter()
            .find(|w| w.id == id)
            .cloned()
    }

    pub async fn create_webhook(&self, create: CreateWebhook) -> Webhook {
        let webhook = Webhook {
            id: ulid::Ulid::new().to_string(),
            name: create.name,
            url: create.url,
            events: create.events,
            enabled: true,
            secret: create.secret,
            created_at_ms: Utc::now().timestamp_millis(),
            last_triggered_ms: None,
            failure_count: 0,
        };
        self.webhooks.write().await.push(webhook.clone());
        webhook
    }

    pub async fn delete_webhook(&self, id: &str) -> bool {
        let mut webhooks = self.webhooks.write().await;
        let len_before = webhooks.len();
        webhooks.retain(|w| w.id != id);
        webhooks.len() < len_before
    }

    pub async fn toggle_webhook(&self, id: &str) -> Option<Webhook> {
        let mut webhooks = self.webhooks.write().await;
        if let Some(webhook) = webhooks.iter_mut().find(|w| w.id == id) {
            webhook.enabled = !webhook.enabled;
            return Some(webhook.clone());
        }
        None
    }

    pub async fn trigger_event(&self, event: &str, data: serde_json::Value) {
        let webhooks = self.webhooks.read().await;
        let mut handles = Vec::new();
        let data = data.clone();

        for webhook in webhooks
            .iter()
            .filter(|w| w.enabled && w.events.contains(&event.to_string()))
        {
            let url = webhook.url.clone();
            let secret = webhook.secret.clone();
            let http_client = self.http_client.clone();
            let event = event.to_string();
            let data = data.clone();

            handles.push(tokio::spawn(async move {
                let payload = WebhookPayload {
                    event,
                    timestamp: Utc::now().timestamp_millis(),
                    data,
                };

                let mut request = http_client.post(&url).json(&payload);

                if let Some(secret) = secret {
                    request = request.header("X-Webhook-Secret", secret);
                }

                match request.send().await {
                    Ok(resp) if resp.status().is_success() => {
                        tracing::info!("Webhook delivered: {}", url);
                    }
                    Ok(resp) => {
                        tracing::warn!("Webhook failed: {} - status {}", url, resp.status());
                    }
                    Err(e) => {
                        tracing::error!("Webhook error: {} - {}", url, e);
                    }
                }
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }
    }

    pub async fn record_failure(&self, id: &str) {
        let mut webhooks = self.webhooks.write().await;
        if let Some(webhook) = webhooks.iter_mut().find(|w| w.id == id) {
            webhook.failure_count += 1;
            if webhook.failure_count >= 5 {
                webhook.enabled = false;
                tracing::warn!("Webhook {} disabled after 5 failures", id);
            }
        }
    }
}
