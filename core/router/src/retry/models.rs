//! Retry Policy Data Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Retry status for tracking attempt results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RetryStatus {
    Pending,
    Running,
    Success,
    Failed,
    Exhausted,
    Cancelled,
}

impl Default for RetryStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Statuses that trigger a retry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RetryableStatus {
    Failed,
    Timeout,
    Error,
}

impl RetryableStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RetryableStatus::Failed => "failed",
            RetryableStatus::Timeout => "timeout",
            RetryableStatus::Error => "error",
        }
    }
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub max_attempts: u32,
    pub initial_delay_secs: u64,
    pub backoff_multiplier: f64,
    pub max_delay_secs: u64,
    pub jitter: bool,
    pub retry_on_statuses: Vec<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl RetryPolicy {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: ulid::Ulid::new().to_string(),
            name,
            description: None,
            max_attempts: 3,
            initial_delay_secs: 5,
            backoff_multiplier: 2.0,
            max_delay_secs: 300,
            jitter: true,
            retry_on_statuses: vec!["failed".to_string(), "timeout".to_string()],
            enabled: true,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Calculate delay for a given attempt number
    pub fn calculate_delay(&self, attempt: u32) -> u64 {
        let base_delay: f64 =
            (self.initial_delay_secs as f64) * self.backoff_multiplier.powi(attempt as i32 - 1);
        let capped_delay = base_delay.min(self.max_delay_secs as f64) as u64;

        if self.jitter {
            // Add random jitter of 0-25% of the delay
            use std::time::{SystemTime, UNIX_EPOCH};
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .subsec_nanos();
            let jitter_factor = 0.75 + (nanos % 25) as f64 / 100.0;
            (capped_delay as f64 * jitter_factor) as u64
        } else {
            capped_delay
        }
    }

    /// Check if a status should trigger a retry
    pub fn should_retry(&self, status: &str) -> bool {
        self.retry_on_statuses.iter().any(|s| s == status)
    }
}

/// Individual retry attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryAttempt {
    pub id: String,
    pub task_id: String,
    pub policy_id: String,
    pub attempt_number: u32,
    pub status: RetryStatus,
    pub error_message: Option<String>,
    pub delay_used_secs: u64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl RetryAttempt {
    pub fn new(task_id: String, policy_id: String, attempt_number: u32) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            task_id,
            policy_id,
            attempt_number,
            status: RetryStatus::Pending,
            error_message: None,
            delay_used_secs: 0,
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn mark_running(&mut self) {
        self.status = RetryStatus::Running;
    }

    pub fn mark_success(&mut self) {
        self.status = RetryStatus::Success;
        self.completed_at = Some(Utc::now());
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = RetryStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
    }

    pub fn mark_exhausted(&mut self) {
        self.status = RetryStatus::Exhausted;
        self.completed_at = Some(Utc::now());
    }

    pub fn mark_cancelled(&mut self) {
        self.status = RetryStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }
}

/// Request to create a new retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRetryPolicyRequest {
    pub name: String,
    pub description: Option<String>,
    pub max_attempts: Option<u32>,
    pub initial_delay_secs: Option<u64>,
    pub backoff_multiplier: Option<f64>,
    pub max_delay_secs: Option<u64>,
    pub jitter: Option<bool>,
    pub retry_on_statuses: Option<Vec<String>>,
}

/// Request to update a retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRetryPolicyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_attempts: Option<u32>,
    pub initial_delay_secs: Option<u64>,
    pub backoff_multiplier: Option<f64>,
    pub max_delay_secs: Option<u64>,
    pub jitter: Option<bool>,
    pub retry_on_statuses: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

/// Request to apply a retry policy to a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyRetryPolicyRequest {
    pub task_id: String,
    pub force: Option<bool>,
}

/// Response for applying a retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyRetryPolicyResponse {
    pub success: bool,
    pub attempt_id: Option<String>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub message: String,
}

/// Retry policy statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStats {
    pub policy_id: String,
    pub total_attempts: u32,
    pub successful_retries: u32,
    pub failed_retries: u32,
    pub exhausted_retries: u32,
    pub avg_delay_secs: f64,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_delay_no_jitter() {
        let policy = RetryPolicy {
            id: "test".to_string(),
            name: "Test".to_string(),
            max_attempts: 3,
            initial_delay_secs: 5,
            backoff_multiplier: 2.0,
            max_delay_secs: 100,
            jitter: false,
            ..Default::default()
        };

        // Attempt 1: 5 * 2^0 = 5
        assert_eq!(policy.calculate_delay(1), 5);
        // Attempt 2: 5 * 2^1 = 10
        assert_eq!(policy.calculate_delay(2), 10);
        // Attempt 3: 5 * 2^2 = 20
        assert_eq!(policy.calculate_delay(3), 20);
    }

    #[test]
    fn test_calculate_delay_with_cap() {
        let policy = RetryPolicy {
            id: "test".to_string(),
            name: "Test".to_string(),
            max_attempts: 10,
            initial_delay_secs: 10,
            backoff_multiplier: 2.0,
            max_delay_secs: 60,
            jitter: false,
            ..Default::default()
        };

        // Should cap at 60
        assert_eq!(policy.calculate_delay(5), 60); // Would be 80 without cap
    }

    #[test]
    fn test_should_retry() {
        let policy = RetryPolicy {
            retry_on_statuses: vec!["failed".to_string(), "timeout".to_string()],
            ..Default::default()
        };

        assert!(policy.should_retry("failed"));
        assert!(policy.should_retry("timeout"));
        assert!(!policy.should_retry("success"));
        assert!(!policy.should_retry("cancelled"));
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new("Default Policy".to_string())
    }
}
