use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc as StdArc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::api::AppState;

#[derive(Clone)]
pub struct WebSocketManager {
    clients: StdArc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
    notification_channel: StdArc<broadcast::Sender<String>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (notification_channel, _) = broadcast::channel(100);
        Self {
            clients: StdArc::new(RwLock::new(HashMap::new())),
            notification_channel: StdArc::new(notification_channel),
        }
    }

    pub async fn add_client(&self, task_id: String, sender: broadcast::Sender<String>) {
        let mut clients = self.clients.write().await;
        clients.insert(task_id, sender);
    }

    pub async fn remove_client(&self, task_id: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(task_id);
    }

    pub async fn broadcast_task_update(&self, task_id: &str, message: &str) {
        let clients = self.clients.read().await;
        if let Some(sender) = clients.get(task_id) {
            let _ = sender.send(message.to_string());
        }
    }

    pub async fn broadcast_all(&self, message: &str) {
        let clients = self.clients.read().await;
        for sender in clients.values() {
            let _ = sender.send(message.to_string());
        }
    }

    pub fn broadcast_notification(&self, message: &str) {
        let _ = self.notification_channel.send(message.to_string());
    }

    pub fn notification_receiver(&self) -> broadcast::Receiver<String> {
        self.notification_channel.subscribe()
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<StdArc<AppState>>,
) -> impl IntoResponse {
    let ws_manager = state.ws_manager.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, ws_manager))
}

async fn handle_socket(socket: WebSocket, ws_manager: WebSocketManager) {
    let (sender, mut receiver) = socket.split();
    let task_id = StdArc::new(RwLock::new(String::new()));
    let (tx, mut rx) = broadcast::channel::<String>(100);

    // Subscribe to notification broadcasts
    let mut notification_rx = ws_manager.notification_receiver();

    let task_id_clone = task_id.clone();
    let tx_orig = tx.clone();

    // Use Arc to share sender across tasks
    let sender = StdArc::new(tokio::sync::Mutex::new(sender));
    let sender_for_tasks = sender.clone();
    let sender_for_notifs = sender.clone();

    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                if let Message::Text(text) = msg {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(id) = json.get("task_id").and_then(|v| v.as_str()) {
                            let mut id_guard = task_id_clone.write().await;
                            *id_guard = id.to_string();
                            drop(id_guard);
                            let tx_clone = tx_orig.clone();
                            ws_manager.add_client(id.to_string(), tx_clone).await;
                        }
                    }
                }
            }
        }
    });

    // Handle task-specific messages
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let mut sender = sender_for_tasks.lock().await;
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle notification broadcasts
    tokio::spawn(async move {
        while let Ok(msg) = notification_rx.recv().await {
            let mut sender = sender_for_notifs.lock().await;
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
}

pub fn create_ws_router(state: StdArc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/ws", get(ws_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_websocket_manager_add_remove_client() {
        let manager = WebSocketManager::new();
        let (tx, _rx) = broadcast::channel(10);

        manager.add_client("task-1".to_string(), tx).await;

        manager.remove_client("task-1").await;

        let clients = manager.clients.read().await;
        assert!(clients.is_empty());
    }

    #[tokio::test]
    async fn test_websocket_manager_broadcast() {
        let manager = WebSocketManager::new();
        let (tx, mut rx) = broadcast::channel(10);

        manager.add_client("task-1".to_string(), tx).await;
        manager.broadcast_task_update("task-1", "Hello").await;

        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, "Hello");
    }

    #[tokio::test]
    async fn test_websocket_manager_broadcast_all() {
        let manager = WebSocketManager::new();
        let (tx1, mut rx1) = broadcast::channel(10);
        let (tx2, mut rx2) = broadcast::channel(10);

        manager.add_client("task-1".to_string(), tx1).await;
        manager.add_client("task-2".to_string(), tx2).await;

        manager.broadcast_all("Broadcast").await;

        let msg1 = rx1.recv().await.unwrap();
        let msg2 = rx2.recv().await.unwrap();

        assert_eq!(msg1, "Broadcast");
        assert_eq!(msg2, "Broadcast");
    }
}
