//! Continuity Scheduler - Cron-based autonomous task scheduling
//!
//! Feature 4: Continuity Scheduler
//! Provides scheduled tasks like morning greetings, check-ins, alarms

use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::unified_config::continuity_constants::*;

/// Type of scheduled task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    MorningGreeting,
    EveningCheckin,
    WeeklySummary,
    DreamMode,
    Alarm,
    RandomCheckin,
    Custom,
}

impl Default for TaskType {
    fn default() -> Self {
        TaskType::Custom
    }
}

impl TaskType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "morning_greeting" => Ok(TaskType::MorningGreeting),
            "evening_checkin" => Ok(TaskType::EveningCheckin),
            "weekly_summary" => Ok(TaskType::WeeklySummary),
            "dream_mode" => Ok(TaskType::DreamMode),
            "alarm" => Ok(TaskType::Alarm),
            "random_checkin" => Ok(TaskType::RandomCheckin),
            "custom" => Ok(TaskType::Custom),
            _ => Err(format!("Unknown task type: {}", s)),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::MorningGreeting => "morning_greeting",
            TaskType::EveningCheckin => "evening_checkin",
            TaskType::WeeklySummary => "weekly_summary",
            TaskType::DreamMode => "dream_mode",
            TaskType::Alarm => "alarm",
            TaskType::RandomCheckin => "random_checkin",
            TaskType::Custom => "custom",
        }
    }
}

/// Cron schedule (simplified 5-field: minute hour day month weekday)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronSchedule {
    pub minute: u32,
    pub hour: u32,
    pub day_of_month: Option<u32>,
    pub month: Option<u32>,
    pub day_of_week: Option<u32>,
}

impl Default for CronSchedule {
    fn default() -> Self {
        Self {
            minute: 0,
            hour: DEFAULT_MORNING_HOUR as u32,
            day_of_month: None,
            month: None,
            day_of_week: None,
        }
    }
}

impl CronSchedule {
    /// Parse from string (e.g., "0 8 * * *" for daily at 8:00)
    pub fn from_cron_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 5 {
            return Err("Cron must have 5 fields: minute hour day month weekday".to_string());
        }

        let minute = parts[0]
            .parse::<u32>()
            .map_err(|_| "Invalid minute")?
            .min(CRON_MINUTE_MAX);
        let hour = parts[1]
            .parse::<u32>()
            .map_err(|_| "Invalid hour")?
            .min(CRON_HOUR_MAX);

        let day_of_month = if parts[2] == "*" {
            None
        } else {
            Some(parts[2].parse::<u32>().map_err(|_| "Invalid day")?)
        };

        let month = if parts[3] == "*" {
            None
        } else {
            Some(parts[3].parse::<u32>().map_err(|_| "Invalid month")?)
        };

        let day_of_week = if parts[4] == "*" {
            None
        } else {
            Some(parts[4].parse::<u32>().map_err(|_| "Invalid weekday")?)
        };

        Ok(Self {
            minute,
            hour,
            day_of_month,
            month,
            day_of_week,
        })
    }

    /// Check if schedule matches current time
    pub fn matches_now(&self) -> bool {
        // Simple check - just return false for now (full impl needs proper chrono usage)
        // This is a placeholder - in production, would use proper datetime comparison
        false
    }

    /// Get next run time (simple implementation)
    pub fn next_run(&self) -> i64 {
        // Return a simple timestamp 24 hours from now
        (Utc::now() + chrono::Duration::days(1)).timestamp()
    }

    /// Create a morning schedule (8:00 AM)
    pub fn morning() -> Self {
        Self {
            minute: 0,
            hour: DEFAULT_MORNING_HOUR,
            day_of_month: None,
            month: None,
            day_of_week: None,
        }
    }

    /// Create an evening schedule (9:00 PM)
    pub fn evening() -> Self {
        Self {
            minute: 0,
            hour: DEFAULT_EVENING_HOUR,
            day_of_month: None,
            month: None,
            day_of_week: None,
        }
    }
}

/// Action to perform when task runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAction {
    /// Action type
    pub action_type: String,
    /// Action payload (JSON)
    pub payload: Option<String>,
}

impl TaskAction {
    pub fn new(action_type: String) -> Self {
        Self {
            action_type,
            payload: None,
        }
    }

    pub fn with_payload(action_type: String, payload: serde_json::Value) -> Self {
        Self {
            action_type,
            payload: Some(payload.to_string()),
        }
    }
}

/// A scheduled task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub task_type: TaskType,
    pub schedule: CronSchedule,
    pub action: TaskAction,
    pub enabled: bool,
    pub last_run: Option<i64>,
    pub next_run: Option<i64>,
    pub run_count: u32,
    pub created_at: i64,
}

impl ScheduledTask {
    pub fn new(
        name: String,
        task_type: TaskType,
        schedule: CronSchedule,
        action: TaskAction,
    ) -> Self {
        let now = Utc::now().timestamp();
        let next = schedule.next_run();
        Self {
            id: Ulid::new().to_string(),
            name,
            task_type,
            schedule,
            action,
            enabled: true,
            last_run: None,
            next_run: Some(next),
            run_count: 0,
            created_at: now,
        }
    }

    /// Check if task should run now
    pub fn should_run(&self) -> bool {
        self.enabled && self.schedule.matches_now()
    }

    /// Record that task ran
    pub fn record_run(&mut self) {
        self.last_run = Some(Utc::now().timestamp());
        self.run_count += 1;
        self.next_run = Some(self.schedule.next_run());
    }

    /// Enable/disable task
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Task execution history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHistoryEntry {
    pub id: String,
    pub task_id: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub status: String, // running, success, failed
    pub result: Option<String>,
    pub error: Option<String>,
}

impl TaskHistoryEntry {
    pub fn start(task_id: String) -> Self {
        Self {
            id: Ulid::new().to_string(),
            task_id,
            started_at: Utc::now().timestamp(),
            completed_at: None,
            status: "running".to_string(),
            result: None,
            error: None,
        }
    }

    pub fn success(&mut self, result: String) {
        self.completed_at = Some(Utc::now().timestamp());
        self.status = "success".to_string();
        self.result = Some(result);
    }

    pub fn failure(&mut self, error: String) {
        self.completed_at = Some(Utc::now().timestamp());
        self.status = "failed".to_string();
        self.error = Some(error);
    }

    pub fn duration_ms(&self) -> Option<i64> {
        self.completed_at.and_then(|c| Some(c - self.started_at))
    }
}

/// Task registry - manages scheduled tasks
pub struct TaskRegistry;

impl TaskRegistry {
    /// Create a morning greeting task
    pub fn create_morning_greeting(name: String) -> ScheduledTask {
        ScheduledTask::new(
            name,
            TaskType::MorningGreeting,
            CronSchedule::morning(),
            TaskAction::new("send_greeting".to_string()),
        )
    }

    /// Create an evening check-in task
    pub fn create_evening_checkin(name: String) -> ScheduledTask {
        ScheduledTask::new(
            name,
            TaskType::EveningCheckin,
            CronSchedule::evening(),
            TaskAction::new("send_checkin".to_string()),
        )
    }

    /// Validate task name
    pub fn is_valid_name(name: &str) -> bool {
        !name.is_empty() && name.len() <= 100
    }

    /// Get available task types
    pub fn task_types() -> Vec<&'static str> {
        TASK_TYPES.to_vec()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_from_str() {
        assert_eq!(
            TaskType::from_str("morning_greeting").unwrap(),
            TaskType::MorningGreeting
        );
        assert_eq!(
            TaskType::from_str("evening_checkin").unwrap(),
            TaskType::EveningCheckin
        );
        assert!(TaskType::from_str("invalid").is_err());
    }

    #[test]
    fn test_cron_schedule_default() {
        let schedule = CronSchedule::default();
        assert_eq!(schedule.hour, DEFAULT_MORNING_HOUR);
    }

    #[test]
    fn test_cron_from_string() {
        let schedule = CronSchedule::from_cron_str("30 8 * * *").unwrap();
        assert_eq!(schedule.minute, 30);
        assert_eq!(schedule.hour, 8);
    }

    #[test]
    fn test_cron_invalid() {
        assert!(CronSchedule::from_cron_str("invalid").is_err());
        assert!(CronSchedule::from_cron_str("1 2").is_err());
    }

    #[test]
    fn test_scheduled_task_creation() {
        let task = ScheduledTask::new(
            "Morning".to_string(),
            TaskType::MorningGreeting,
            CronSchedule::morning(),
            TaskAction::new("greet".to_string()),
        );
        assert_eq!(task.name, "Morning");
        assert!(task.enabled);
    }

    #[test]
    fn test_task_record_run() {
        let mut task = ScheduledTask::new(
            "Test".to_string(),
            TaskType::Custom,
            CronSchedule::default(),
            TaskAction::new("test".to_string()),
        );
        assert_eq!(task.run_count, 0);
        task.record_run();
        assert_eq!(task.run_count, 1);
        assert!(task.last_run.is_some());
    }

    #[test]
    fn test_task_history_entry() {
        let mut entry = TaskHistoryEntry::start("task123".to_string());
        assert_eq!(entry.status, "running");
        entry.success("Done".to_string());
        assert_eq!(entry.status, "success");
        assert_eq!(entry.result, Some("Done".to_string()));
    }

    #[test]
    fn test_task_registry_names() {
        assert!(TaskRegistry::is_valid_name("Valid Task"));
        assert!(!TaskRegistry::is_valid_name(""));
        assert!(!TaskRegistry::is_valid_name(&"a".repeat(101)));
    }

    #[test]
    fn test_task_types_list() {
        let types = TaskRegistry::task_types();
        assert!(types.contains(&"morning_greeting"));
        assert!(types.contains(&"evening_checkin"));
        assert!(types.contains(&"alarm"));
    }
}
