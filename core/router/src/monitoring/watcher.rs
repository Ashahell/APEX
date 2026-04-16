//! Watcher Registry - manages watch patterns

use crate::monitoring::models::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Registry for all watch patterns
#[derive(Debug, Default)]
pub struct WatcherRegistry {
    /// Active watchers by ID
    watchers: HashMap<String, WatchPattern>,
    /// Match counters for threshold-based notifications
    match_counts: HashMap<String, u32>,
}

impl WatcherRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new watcher
    pub fn add(&mut self, mut watcher: WatchPattern) -> MonitoringResult<()> {
        watcher.compile()?;
        self.watchers.insert(watcher.id.clone(), watcher);
        Ok(())
    }

    /// Remove a watcher by ID
    pub fn remove(&mut self, id: &str) -> Option<WatchPattern> {
        self.match_counts.remove(id);
        self.watchers.remove(id)
    }

    /// Get a watcher by ID
    pub fn get(&self, id: &str) -> Option<&WatchPattern> {
        self.watchers.get(id)
    }

    /// Get a mutable watcher by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut WatchPattern> {
        self.watchers.get_mut(id)
    }

    /// List all watchers
    pub fn list(&self) -> Vec<&WatchPattern> {
        self.watchers.values().collect()
    }

    /// List only enabled watchers
    pub fn list_enabled(&self) -> Vec<&WatchPattern> {
        self.watchers.values().filter(|w| w.enabled).collect()
    }

    /// Update a watcher
    pub fn update(&mut self, id: &str, updates: WatchPatternUpdate) -> MonitoringResult<()> {
        let watcher = self
            .watchers
            .get_mut(id)
            .ok_or_else(|| MonitoringError::PatternNotFound(id.to_string()))?;

        if let Some(name) = updates.name {
            watcher.name = name;
        }
        if let Some(pattern) = updates.pattern {
            watcher.pattern = pattern.clone();
            watcher.regex = None;
            watcher.compile()?;
        }
        if let Some(watch_scope) = updates.watch_scope {
            watcher.watch_scope = watch_scope;
        }
        if let Some(notify_on) = updates.notify_on {
            watcher.notify_on = notify_on;
        }
        if let Some(mode) = updates.notification_mode {
            watcher.notification_mode = mode;
        }
        if let Some(enabled) = updates.enabled {
            watcher.enabled = enabled;
        }

        watcher.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    /// Check which watchers match the given event
    pub fn check_matches(
        &mut self,
        event: &MonitorEvent,
        project: Option<&str>,
    ) -> Vec<(String, NotifyMode)> {
        let mut matches = Vec::new();
        let text = event.text_content();
        let task_id = event.task_id();

        for watcher in self.watchers.values_mut() {
            if !watcher.enabled {
                continue;
            }

            // Check scope
            if !watcher.handles_scope(project, task_id) {
                continue;
            }

            // Check notify_on condition
            let should_notify = match &watcher.notify_on {
                NotifyOn::Match => watcher.matches(&text),
                NotifyOn::Completion => matches!(event, MonitorEvent::AgentEnd { .. }),
                NotifyOn::Error => matches!(event, MonitorEvent::AgentEnd { success: false, .. }),
                NotifyOn::Timeout => false, // TODO: implement timeout tracking
                NotifyOn::Threshold { count } => {
                    if watcher.matches(&text) {
                        let counter = self.match_counts.entry(watcher.id.clone()).or_insert(0);
                        *counter += 1;
                        *counter >= *count
                    } else {
                        false
                    }
                }
            };

            if should_notify {
                if let NotifyOn::Threshold { .. } = &watcher.notify_on {
                    // Reset counter after threshold notification
                    self.match_counts.insert(watcher.id.clone(), 0);
                }
                matches.push((watcher.id.clone(), watcher.notification_mode.clone()));
            }
        }

        matches
    }

    /// Get statistics
    pub fn stats(&self) -> MonitoringStats {
        let watchers: Vec<_> = self.watchers.values().collect();
        MonitoringStats {
            total_watchers: watchers.len() as u32,
            active_watchers: watchers.iter().filter(|w| w.enabled).count() as u32,
            events_last_hour: 0, // TODO: track with timestamp
            patterns_matched: self.match_counts.values().sum(),
            notifications_sent: 0, // TODO: track
        }
    }
}

/// Thread-safe wrapper for WatcherRegistry
#[derive(Debug, Clone, Default)]
pub struct SharedWatcherRegistry(pub Arc<RwLock<WatcherRegistry>>);

impl SharedWatcherRegistry {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(WatcherRegistry::new())))
    }

    pub async fn add(&self, watcher: WatchPattern) -> MonitoringResult<()> {
        let mut registry = self.0.write().await;
        registry.add(watcher)
    }

    pub async fn remove(&self, id: &str) -> Option<WatchPattern> {
        let mut registry = self.0.write().await;
        registry.remove(id)
    }

    pub async fn get(&self, id: &str) -> Option<WatchPattern> {
        let registry = self.0.read().await;
        registry.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<WatchPattern> {
        let registry = self.0.read().await;
        registry.list().into_iter().cloned().collect()
    }

    pub async fn list_enabled(&self) -> Vec<WatchPattern> {
        let registry = self.0.read().await;
        registry.list_enabled().into_iter().cloned().collect()
    }

    pub async fn update(&self, id: &str, updates: WatchPatternUpdate) -> MonitoringResult<()> {
        let mut registry = self.0.write().await;
        registry.update(id, updates)
    }

    pub async fn check_matches(
        &self,
        event: &MonitorEvent,
        project: Option<&str>,
    ) -> Vec<(String, NotifyMode)> {
        let mut registry = self.0.write().await;
        registry.check_matches(event, project)
    }

    pub async fn stats(&self) -> MonitoringStats {
        let registry = self.0.read().await;
        registry.stats()
    }
}

/// Update fields for a watch pattern
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WatchPatternUpdate {
    pub name: Option<String>,
    pub pattern: Option<String>,
    pub watch_scope: Option<WatchScope>,
    pub notify_on: Option<NotifyOn>,
    pub notification_mode: Option<NotifyMode>,
    pub enabled: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_pattern_creation() {
        let pattern = WatchPattern::new(
            "test-1".to_string(),
            "Test Pattern".to_string(),
            r"\berror\b".to_string(),
            WatchScope::All,
            NotifyOn::Match,
            NotifyMode::All,
        );
        assert!(pattern.is_ok());
    }

    #[test]
    fn test_watch_pattern_matching() {
        let mut pattern = WatchPattern::new(
            "test-1".to_string(),
            "Test Pattern".to_string(),
            r"(?i)\berror\b".to_string(), // Case-insensitive regex
            WatchScope::All,
            NotifyOn::Match,
            NotifyMode::All,
        )
        .unwrap();
        pattern.compile().unwrap();

        assert!(pattern.matches("An error occurred"));
        assert!(pattern.matches("ERROR: failed"));
        assert!(!pattern.matches("No errors here"));
        assert!(!pattern.matches("Terrorist attack")); // partial word match
    }

    #[test]
    fn test_invalid_regex() {
        let pattern = WatchPattern::new(
            "test-1".to_string(),
            "Test Pattern".to_string(),
            r"[invalid".to_string(),
            WatchScope::All,
            NotifyOn::Match,
            NotifyMode::All,
        );
        assert!(pattern.is_err());
    }

    #[test]
    fn test_watch_scope() {
        let mut registry = WatcherRegistry::new();

        registry
            .add(WatchPattern::new(
                "all".to_string(),
                "All".to_string(),
                r".*".to_string(),
                WatchScope::All,
                NotifyOn::Match,
                NotifyMode::All,
            ).unwrap())
            .unwrap();

        registry
            .add(WatchPattern::new(
                "project".to_string(),
                "Project".to_string(),
                r".*".to_string(),
                WatchScope::Project("test-project".to_string()),
                NotifyOn::Match,
                NotifyMode::All,
            ).unwrap())
            .unwrap();

        let all = registry.get("all").unwrap();
        assert!(all.handles_scope(None, None));
        assert!(all.handles_scope(Some("anything"), Some("anything")));

        let project = registry.get("project").unwrap();
        assert!(!project.handles_scope(None, None));
        assert!(project.handles_scope(Some("test-project"), Some("task-1")));
    }
}
