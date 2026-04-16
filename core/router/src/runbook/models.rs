//! Runbook Data Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Main runbook structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runbook {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// Which alert type triggers this runbook (e.g., "task_timeout", "error_spike")
    pub trigger_alert_type: Option<String>,
    pub steps: Vec<RunbookStep>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Runbook {
    pub fn new(id: String, name: String, steps: Vec<RunbookStep>) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description: None,
            trigger_alert_type: None,
            steps,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Individual runbook step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunbookStep {
    /// Cancel a running task
    CancelTask { task_id: String },
    /// Create a new task
    CreateTask {
        input: String,
        priority: Option<String>,
        project: Option<String>,
    },
    /// Send a notification
    Notify { message: String },
    /// Execute a shell command
    ExecuteCommand { command: String },
    /// Wait for a duration
    Delay { ms: u64 },
    /// Call a webhook
    Webhook {
        url: String,
        method: Option<String>,
        body: Option<String>,
        headers: Option<std::collections::HashMap<String, String>>,
    },
    /// Cancel and recreate task
    RestartTask { task_id: String },
}

/// Runbook execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookExecution {
    pub id: String,
    pub runbook_id: String,
    pub runbook_name: String,
    pub alert_id: Option<String>,
    pub status: ExecutionStatus,
    pub current_step: u32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub step_results: Vec<StepResult>,
}

impl RunbookExecution {
    pub fn new(runbook_id: String, runbook_name: String, alert_id: Option<String>) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            runbook_id,
            runbook_name,
            alert_id,
            status: ExecutionStatus::Pending,
            current_step: 0,
            started_at: Utc::now(),
            completed_at: None,
            error: None,
            step_results: Vec::new(),
        }
    }

    pub fn mark_running(&mut self) {
        self.status = ExecutionStatus::Running;
    }

    pub fn mark_completed(&mut self) {
        self.status = ExecutionStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = ExecutionStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error);
    }

    pub fn add_step_result(&mut self, result: StepResult) {
        self.current_step = result.step_index + 1;
        self.step_results.push(result);
    }

    pub fn duration_ms(&self) -> u64 {
        let end = self.completed_at.unwrap_or_else(Utc::now);
        (end - self.started_at).num_milliseconds() as u64
    }
}

/// Execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Result of a single step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_index: u32,
    pub step_type: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

impl StepResult {
    pub fn success(step_index: u32, step_type: &str, output: String, duration_ms: u64) -> Self {
        Self {
            step_index,
            step_type: step_type.to_string(),
            success: true,
            output: Some(output),
            error: None,
            duration_ms,
        }
    }

    pub fn failure(step_index: u32, step_type: &str, error: String, duration_ms: u64) -> Self {
        Self {
            step_index,
            step_type: step_type.to_string(),
            success: false,
            output: None,
            error: Some(error),
            duration_ms,
        }
    }
}

/// Request to create a runbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRunbookRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_alert_type: Option<String>,
    pub steps: Vec<RunbookStep>,
}

/// Request to update a runbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRunbookRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger_alert_type: Option<String>,
    pub steps: Option<Vec<RunbookStep>>,
    pub enabled: Option<bool>,
}

/// Request to execute a runbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRunbookRequest {
    pub alert_id: Option<String>,
}

/// Runbook template for quick creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookTemplate {
    pub name: String,
    pub description: String,
    pub steps: Vec<RunbookStep>,
}

impl RunbookTemplate {
    pub fn restart_failed_task() -> Self {
        Self {
            name: "Restart Failed Task".to_string(),
            description: "Cancels failed task and recreates it with same input".to_string(),
            steps: vec![
                RunbookStep::CancelTask {
                    task_id: "{{alert.task_id}}".to_string(),
                },
                RunbookStep::Delay { ms: 1000 },
                RunbookStep::CreateTask {
                    input: "{{alert.original_input}}".to_string(),
                    priority: Some("high".to_string()),
                    project: None,
                },
                RunbookStep::Notify {
                    message: "Task restarted automatically by runbook".to_string(),
                },
            ],
        }
    }

    pub fn clear_resource() -> Self {
        Self {
            name: "Clear Resource".to_string(),
            description: "Clears stuck resource (cache, queue, etc.)".to_string(),
            steps: vec![
                RunbookStep::ExecuteCommand {
                    command: "echo 'Clearing resource'".to_string(),
                },
                RunbookStep::Notify {
                    message: "Resource cleared automatically".to_string(),
                },
            ],
        }
    }

    pub fn escalate_notification() -> Self {
        Self {
            name: "Escalate Notification".to_string(),
            description: "Sends escalation notification to on-call".to_string(),
            steps: vec![
                RunbookStep::Delay { ms: 300000 }, // 5 min delay
                RunbookStep::Webhook {
                    url: "{{config.escalation_webhook}}".to_string(),
                    method: Some("POST".to_string()),
                    body: Some("{{alert.json}}".to_string()),
                    headers: None,
                },
                RunbookStep::Notify {
                    message: "Escalation notification sent".to_string(),
                },
            ],
        }
    }
}
