use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TaskStatus::Pending),
            "running" => Some(TaskStatus::Running),
            "completed" => Some(TaskStatus::Completed),
            "failed" => Some(TaskStatus::Failed),
            "cancelled" => Some(TaskStatus::Cancelled),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_as_str() {
        assert_eq!(TaskStatus::Pending.as_str(), "pending");
        assert_eq!(TaskStatus::Running.as_str(), "running");
        assert_eq!(TaskStatus::Completed.as_str(), "completed");
        assert_eq!(TaskStatus::Failed.as_str(), "failed");
        assert_eq!(TaskStatus::Cancelled.as_str(), "cancelled");
    }

    #[test]
    fn test_task_status_try_from_str() {
        assert_eq!(
            TaskStatus::try_from_str("pending"),
            Some(TaskStatus::Pending)
        );
        assert_eq!(
            TaskStatus::try_from_str("running"),
            Some(TaskStatus::Running)
        );
        assert_eq!(
            TaskStatus::try_from_str("completed"),
            Some(TaskStatus::Completed)
        );
        assert_eq!(TaskStatus::try_from_str("failed"), Some(TaskStatus::Failed));
        assert_eq!(
            TaskStatus::try_from_str("cancelled"),
            Some(TaskStatus::Cancelled)
        );
        assert_eq!(TaskStatus::try_from_str("unknown"), None);
    }

    #[test]
    fn test_task_tier_as_str() {
        assert_eq!(TaskTier::Instant.as_str(), "instant");
        assert_eq!(TaskTier::Shallow.as_str(), "shallow");
        assert_eq!(TaskTier::Deep.as_str(), "deep");
    }

    #[test]
    fn test_task_tier_try_from_str() {
        assert_eq!(TaskTier::try_from_str("instant"), Some(TaskTier::Instant));
        assert_eq!(TaskTier::try_from_str("shallow"), Some(TaskTier::Shallow));
        assert_eq!(TaskTier::try_from_str("deep"), Some(TaskTier::Deep));
        assert_eq!(TaskTier::try_from_str("unknown"), None);
    }

    #[test]
    fn test_task_priority_as_str() {
        assert_eq!(TaskPriority::Low.as_str(), "low");
        assert_eq!(TaskPriority::Medium.as_str(), "medium");
        assert_eq!(TaskPriority::High.as_str(), "high");
        assert_eq!(TaskPriority::Urgent.as_str(), "urgent");
    }

    #[test]
    fn test_task_priority_try_from_str() {
        assert_eq!(TaskPriority::try_from_str("low"), Some(TaskPriority::Low));
        assert_eq!(
            TaskPriority::try_from_str("medium"),
            Some(TaskPriority::Medium)
        );
        assert_eq!(TaskPriority::try_from_str("high"), Some(TaskPriority::High));
        assert_eq!(
            TaskPriority::try_from_str("urgent"),
            Some(TaskPriority::Urgent)
        );
        assert_eq!(TaskPriority::try_from_str("unknown"), None);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskTier {
    Instant,
    Shallow,
    Deep,
}

impl TaskTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskTier::Instant => "instant",
            TaskTier::Shallow => "shallow",
            TaskTier::Deep => "deep",
        }
    }

    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "instant" => Some(TaskTier::Instant),
            "shallow" => Some(TaskTier::Shallow),
            "deep" => Some(TaskTier::Deep),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Low => "low",
            TaskPriority::Medium => "medium",
            TaskPriority::High => "high",
            TaskPriority::Urgent => "urgent",
        }
    }

    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "low" => Some(TaskPriority::Low),
            "medium" => Some(TaskPriority::Medium),
            "high" => Some(TaskPriority::High),
            "urgent" => Some(TaskPriority::Urgent),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: String,
    pub status: String,
    pub tier: String,
    pub input_content: String,
    pub output_content: Option<String>,
    pub channel: Option<String>,
    pub thread_id: Option<String>,
    pub author: Option<String>,
    pub skill_name: Option<String>,
    pub error_message: Option<String>,
    pub cost_estimate_usd: Option<f64>,
    pub actual_cost_usd: Option<f64>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTask {
    pub input_content: String,
    pub channel: Option<String>,
    pub thread_id: Option<String>,
    pub author: Option<String>,
    pub skill_name: Option<String>,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTask {
    pub status: Option<TaskStatus>,
    pub output_content: Option<String>,
    pub error_message: Option<String>,
    pub cost_estimate_usd: Option<f64>,
    pub actual_cost_usd: Option<f64>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
}
