//! Hook system for emitting and subscribing to monitor events

use crate::monitoring::models::{MonitorEvent, MonitorEventRecord};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Maximum events to keep in memory
const MAX_EVENTS: usize = 1000;

/// Hook emitter for broadcasting events to subscribers
#[derive(Debug, Default)]
pub struct HookEmitter {
    /// Event log for recent events
    event_log: Vec<MonitorEventRecord>,
    /// Subscribers (futures-compatible channel)
    subscribers: HashMap<String, flume::Sender<MonitorEvent>>,
}

impl HookEmitter {
    /// Create a new hook emitter
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to events
    pub fn subscribe(&mut self) -> flume::Receiver<MonitorEvent> {
        let (tx, rx) = flume::bounded(100);
        let id = ulid::Ulid::new().to_string();
        self.subscribers.insert(id, tx);
        rx
    }

    /// Unsubscribe by receiver
    pub fn unsubscribe(&mut self, _rx: &flume::Receiver<MonitorEvent>) {
        let to_remove: Vec<String> = self
            .subscribers
            .iter()
            .filter(|(_, tx)| tx.receiver_count() == 0 || tx.is_disconnected())
            .map(|(id, _)| id.clone())
            .collect();

        for id in to_remove {
            self.subscribers.remove(&id);
        }
    }

    /// Emit an event to all subscribers
    pub fn emit(&mut self, event: MonitorEvent) {
        let record = MonitorEventRecord::from(&event);

        // Add to log
        if self.event_log.len() >= MAX_EVENTS {
            self.event_log.remove(0);
        }
        self.event_log.push(record);

        // Broadcast to subscribers
        let payload = serde_json::to_string(&event).unwrap_or_default();
        for tx in self.subscribers.values() {
            if tx.send(event.clone()).is_err() {
                // Subscriber dropped, will be cleaned up on next emit
            }
            let _ = payload; // suppress unused warning in release
        }
    }

    /// Get recent events
    pub fn recent_events(&self, limit: usize) -> Vec<MonitorEventRecord> {
        self.event_log
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get events for a specific task
    pub fn events_for_task(&self, task_id: &str, limit: usize) -> Vec<MonitorEventRecord> {
        self.event_log
            .iter()
            .filter(|e| e.task_id.as_deref() == Some(task_id))
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}

/// Thread-safe wrapper
#[derive(Debug, Clone, Default)]
pub struct SharedHookEmitter(pub Arc<RwLock<HookEmitter>>);

impl SharedHookEmitter {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HookEmitter::new())))
    }

    /// Subscribe and get a receiver
    pub async fn subscribe(&self) -> flume::Receiver<MonitorEvent> {
        let mut emitter = self.0.write().await;
        emitter.subscribe()
    }

    /// Emit an event
    pub async fn emit(&self, event: MonitorEvent) {
        let mut emitter = self.0.write().await;
        emitter.emit(event);
    }

    /// Get recent events
    pub async fn recent_events(&self, limit: usize) -> Vec<MonitorEventRecord> {
        let emitter = self.0.read().await;
        emitter.recent_events(limit)
    }

    /// Get events for a task
    pub async fn events_for_task(&self, task_id: &str, limit: usize) -> Vec<MonitorEventRecord> {
        let emitter = self.0.read().await;
        emitter.events_for_task(task_id, limit)
    }
}

/// Event subscriber that processes events
pub struct EventSubscriber {
    receiver: flume::Receiver<MonitorEvent>,
}

impl EventSubscriber {
    pub fn new(receiver: flume::Receiver<MonitorEvent>) -> Self {
        Self { receiver }
    }

    /// Get the receiver for polling
    pub fn receiver(&self) -> &flume::Receiver<MonitorEvent> {
        &self.receiver
    }

    /// Try to get the next event without blocking
    pub fn try_next(&self) -> Option<MonitorEvent> {
        self.receiver.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_emit_and_subscribe() {
        let emitter = SharedHookEmitter::new();
        
        let rx = emitter.subscribe().await;
        
        emitter.emit(MonitorEvent::AgentStart {
            task_id: "task-1".to_string(),
            prompt: "Test prompt".to_string(),
        }).await;
        
        let event = rx.recv_async().await.unwrap();
        assert!(matches!(event, MonitorEvent::AgentStart { .. }));
    }

    #[test]
    fn test_event_log() {
        let mut emitter = HookEmitter::new();
        
        emitter.emit(MonitorEvent::AgentStart {
            task_id: "task-1".to_string(),
            prompt: "Test".to_string(),
        });
        
        let events = emitter.recent_events(10);
        assert_eq!(events.len(), 1);
    }
}
