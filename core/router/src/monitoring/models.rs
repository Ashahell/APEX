//! Models for Background Process Monitoring

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MonitoringError {
    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(String),
    #[error("Pattern not found: {0}")]
    PatternNotFound(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Notification error: {0}")]
    Notification(String),
}

pub type MonitoringResult<T> = Result<T, MonitoringError>;

/// Scope of what to watch
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum WatchScope {
    /// Watch all tasks
    All,
    /// Watch tasks in a specific project
    Project(String),
    /// Watch specific task IDs
    TaskIds(Vec<String>),
}

impl Default for WatchScope {
    fn default() -> Self {
        WatchScope::All
    }
}

/// When to trigger notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum NotifyOn {
    /// Trigger when pattern matches
    Match,
    /// Trigger on task completion
    Completion,
    /// Trigger on task error
    Error,
    /// Trigger on task timeout
    Timeout,
    /// Trigger after N matches
    Threshold { count: u32 },
}

impl Default for NotifyOn {
    fn default() -> Self {
        NotifyOn::Match
    }
}

/// Notification delivery mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum NotifyMode {
    /// All events
    All,
    /// Only final result
    Result,
    /// Only errors
    Error,
    /// Disabled
    Off,
}

impl Default for NotifyMode {
    fn default() -> Self {
        NotifyMode::Result
    }
}

/// A watch pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchPattern {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Regex pattern to match against
    pub pattern: String,
    /// Compiled regex (not serialized)
    #[serde(skip)]
    pub regex: Option<Arc<Regex>>,
    /// What to watch
    pub watch_scope: WatchScope,
    /// When to notify
    pub notify_on: NotifyOn,
    /// Notification delivery mode
    pub notification_mode: NotifyMode,
    /// Whether this watcher is active
    pub enabled: bool,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last update timestamp (ISO 8601)
    pub updated_at: String,
}

impl WatchPattern {
    /// Create a new watch pattern with validation
    pub fn new(
        id: String,
        name: String,
        pattern: String,
        watch_scope: WatchScope,
        notify_on: NotifyOn,
        notification_mode: NotifyMode,
    ) -> MonitoringResult<Self> {
        // Validate regex pattern
        Regex::new(&pattern).map_err(|e| MonitoringError::InvalidPattern(e.to_string()))?;

        let now = chrono::Utc::now().to_rfc3339();
        Ok(Self {
            id,
            name,
            pattern,
            regex: None,
            watch_scope,
            notify_on,
            notification_mode,
            enabled: true,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Compile the regex pattern
    pub fn compile(&mut self) -> MonitoringResult<()> {
        if self.regex.is_none() {
            let re = Regex::new(&self.pattern)
                .map_err(|e| MonitoringError::InvalidPattern(e.to_string()))?;
            self.regex = Some(Arc::new(re));
        }
        Ok(())
    }

    /// Check if this pattern matches the given text
    pub fn matches(&self, text: &str) -> bool {
        if !self.enabled {
            return false;
        }
        self.regex
            .as_ref()
            .map(|re| re.is_match(text))
            .unwrap_or(false)
    }

    /// Check if this watcher should handle the given scope
    pub fn handles_scope(&self, project: Option<&str>, task_id: Option<&str>) -> bool {
        match &self.watch_scope {
            WatchScope::All => true,
            WatchScope::Project(p) => project.map(|s| s == p).unwrap_or(false),
            WatchScope::TaskIds(ids) => task_id
                .map(|tid| ids.contains(&tid.to_string()))
                .unwrap_or(false),
        }
    }
}

/// Monitor event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum MonitorEvent {
    /// Agent started processing a task
    AgentStart { task_id: String, prompt: String },
    /// Agent executed a step
    AgentStep {
        task_id: String,
        step: u32,
        action: String,
        output: Option<String>,
    },
    /// Agent completed task
    AgentEnd {
        task_id: String,
        result: String,
        success: bool,
    },
    /// Session started
    SessionStart { session_id: String },
    /// Session ended
    SessionEnd { session_id: String },
    /// Pattern matched in output
    PatternMatched {
        watcher_id: String,
        task_id: String,
        match_text: String,
    },
}

impl MonitorEvent {
    /// Get the task ID associated with this event, if any
    pub fn task_id(&self) -> Option<&str> {
        match self {
            MonitorEvent::AgentStart { task_id, .. } => Some(task_id),
            MonitorEvent::AgentStep { task_id, .. } => Some(task_id),
            MonitorEvent::AgentEnd { task_id, .. } => Some(task_id),
            _ => None,
        }
    }

    /// Get the text content of this event for pattern matching
    pub fn text_content(&self) -> String {
        match self {
            MonitorEvent::AgentStart { prompt, .. } => prompt.clone(),
            MonitorEvent::AgentStep { action, output, .. } => {
                format!("{}\n{}", action, output.as_deref().unwrap_or(""))
            }
            MonitorEvent::AgentEnd { result, .. } => result.clone(),
            MonitorEvent::PatternMatched { match_text, .. } => match_text.clone(),
            MonitorEvent::SessionStart { session_id } => session_id.clone(),
            MonitorEvent::SessionEnd { session_id } => session_id.clone(),
        }
    }
}

/// A recorded monitor event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorEventRecord {
    pub id: String,
    pub event_type: String,
    pub task_id: Option<String>,
    pub session_id: Option<String>,
    pub payload: String,
    pub matched_watcher_id: Option<String>,
    pub created_at: String,
}

impl From<&MonitorEvent> for MonitorEventRecord {
    fn from(event: &MonitorEvent) -> Self {
        let (event_type, task_id, session_id, payload) = match event {
            MonitorEvent::AgentStart { task_id, prompt } => (
                "AgentStart".to_string(),
                Some(task_id.clone()),
                None,
                serde_json::to_string(prompt).unwrap_or_default(),
            ),
            MonitorEvent::AgentStep {
                task_id,
                step,
                action,
                output,
            } => (
                "AgentStep".to_string(),
                Some(task_id.clone()),
                None,
                serde_json::json!({ "step": step, "action": action, "output": output }).to_string(),
            ),
            MonitorEvent::AgentEnd {
                task_id,
                result,
                success,
            } => (
                "AgentEnd".to_string(),
                Some(task_id.clone()),
                None,
                serde_json::json!({ "result": result, "success": success }).to_string(),
            ),
            MonitorEvent::SessionStart { session_id } => (
                "SessionStart".to_string(),
                None,
                Some(session_id.clone()),
                serde_json::json!({ "session_id": session_id }).to_string(),
            ),
            MonitorEvent::SessionEnd { session_id } => (
                "SessionEnd".to_string(),
                None,
                Some(session_id.clone()),
                serde_json::json!({ "session_id": session_id }).to_string(),
            ),
            MonitorEvent::PatternMatched {
                watcher_id,
                task_id,
                match_text,
            } => (
                "PatternMatched".to_string(),
                Some(task_id.clone()),
                None,
                serde_json::json!({ "watcher_id": watcher_id, "match_text": match_text })
                    .to_string(),
            ),
        };

        Self {
            id: ulid::Ulid::new().to_string(),
            event_type,
            task_id,
            session_id,
            payload,
            matched_watcher_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Notification to be sent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub watcher_id: Option<String>,
    pub event: MonitorEvent,
    pub mode: NotifyMode,
    pub message: String,
    pub timestamp: String,
}

impl Notification {
    pub fn new(event: MonitorEvent, mode: NotifyMode, message: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            watcher_id: event.task_id().map(|s| s.to_string()),
            event,
            mode,
            message,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringStats {
    pub total_watchers: u32,
    pub active_watchers: u32,
    pub events_last_hour: u32,
    pub patterns_matched: u32,
    pub notifications_sent: u32,
}
