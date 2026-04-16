//! Scheduled Task Template Data Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schedule type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleType {
    Interval,
    Cron,
    Onetime,
}

impl Default for ScheduleType {
    fn default() -> Self {
        Self::Interval
    }
}

/// Task template with scheduling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub task_content: String,
    pub schedule_type: ScheduleType,
    pub schedule_config: ScheduleConfig,
    pub enabled: bool,
    pub max_runs: Option<u32>,
    pub run_count: u32,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl ScheduledTemplate {
    pub fn new(name: String, task_content: String) -> Self {
        let now = Utc::now();
        Self {
            id: ulid::Ulid::new().to_string(),
            name,
            description: None,
            task_content,
            schedule_type: ScheduleType::Interval,
            schedule_config: ScheduleConfig::default(),
            enabled: true,
            max_runs: None,
            run_count: 0,
            last_run_at: None,
            next_run_at: None,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Calculate next run time
    pub fn calculate_next_run(&self) -> Option<DateTime<Utc>> {
        let now = Utc::now();

        match self.schedule_type {
            ScheduleType::Interval => {
                // If never run before, run immediately
                if self.last_run_at.is_none() {
                    Some(now)
                } else {
                    let base = self.last_run_at.unwrap();
                    Some(
                        base + chrono::Duration::seconds(self.schedule_config.interval_secs as i64),
                    )
                }
            }
            ScheduleType::Cron => {
                // Simple cron: just use interval as fallback for now
                if self.last_run_at.is_none() {
                    Some(now)
                } else {
                    let base = self.last_run_at.unwrap();
                    Some(
                        base + chrono::Duration::seconds(self.schedule_config.interval_secs as i64),
                    )
                }
            }
            ScheduleType::Onetime => {
                if self.last_run_at.is_none() {
                    self.schedule_config.run_at
                } else {
                    None
                }
            }
        }
    }

    /// Check if template should run
    pub fn should_run(&self) -> bool {
        if !self.enabled {
            return false;
        }

        // Check max runs
        if let Some(max) = self.max_runs {
            if self.run_count >= max {
                return false;
            }
        }

        // Check if time to run - calculate next_run_at if not set
        let next = self.next_run_at.or_else(|| self.calculate_next_run());
        if let Some(next) = next {
            return Utc::now() >= next;
        }

        false
    }

    /// Record a run
    pub fn record_run(&mut self) {
        self.run_count += 1;
        self.last_run_at = Some(Utc::now());
        self.next_run_at = self.calculate_next_run();
    }
}

/// Schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// Interval in seconds (for Interval type)
    pub interval_secs: u64,
    /// Cron expression (for Cron type)
    pub cron_expr: Option<String>,
    /// One-time run at (for Onetime type)
    pub run_at: Option<DateTime<Utc>>,
    /// Start time for daily/weekly schedules
    pub start_time: Option<String>,
    /// Days of week (0 = Sunday, 6 = Saturday)
    pub days_of_week: Option<Vec<u8>>,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            interval_secs: 3600, // 1 hour
            cron_expr: None,
            run_at: None,
            start_time: None,
            days_of_week: None,
        }
    }
}

/// Scheduled task execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledExecution {
    pub id: String,
    pub template_id: String,
    pub task_id: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: Option<String>,
    pub error: Option<String>,
}

impl ScheduledExecution {
    pub fn new(template_id: String, task_id: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            template_id,
            task_id,
            status: ExecutionStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
            output: None,
            error: None,
        }
    }

    pub fn mark_running(&mut self) {
        self.status = ExecutionStatus::Running;
    }

    pub fn mark_completed(&mut self, output: String) {
        self.status = ExecutionStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.output = Some(output);
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = ExecutionStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error);
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

impl Default for ExecutionStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Request to create a scheduled template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub task_content: String,
    pub schedule_type: Option<ScheduleType>,
    pub interval_secs: Option<u64>,
    pub cron_expr: Option<String>,
    pub run_at: Option<DateTime<Utc>>,
    pub max_runs: Option<u32>,
}

/// Request to update a scheduled template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduledTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub task_content: Option<String>,
    pub schedule_type: Option<ScheduleType>,
    pub interval_secs: Option<u64>,
    pub cron_expr: Option<String>,
    pub run_at: Option<DateTime<Utc>>,
    pub max_runs: Option<u32>,
    pub enabled: Option<bool>,
}

/// Request to trigger a template manually
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerTemplateRequest {
    pub force: Option<bool>,
}

/// Response for triggering a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerTemplateResponse {
    pub success: bool,
    pub execution_id: Option<String>,
    pub task_id: Option<String>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_run() {
        let mut template = ScheduledTemplate::new("Test".to_string(), "echo hello".to_string());
        template.schedule_config.interval_secs = 3600;

        assert!(template.should_run());

        template.enabled = false;
        assert!(!template.should_run());
    }

    #[test]
    fn test_calculate_next_run() {
        let mut template = ScheduledTemplate::new("Test".to_string(), "echo hello".to_string());
        template.schedule_config.interval_secs = 60;

        let next = template.calculate_next_run();
        assert!(next.is_some());
    }

    #[test]
    fn test_max_runs() {
        let mut template = ScheduledTemplate::new("Test".to_string(), "echo hello".to_string());
        template.max_runs = Some(2);
        template.run_count = 2;

        assert!(!template.should_run());
    }
}

impl Default for ScheduledTemplate {
    fn default() -> Self {
        Self::new("Default Template".to_string(), String::new())
    }
}
