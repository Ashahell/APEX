//! Models for Vigilant Mode alert monitoring

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use ulid::Ulid;

/// Vigilant mode errors
#[derive(Debug, Error)]
pub enum VigilantError {
    #[error("Invalid alert rule: {0}")]
    InvalidRule(String),
    #[error("Rule not found: {0}")]
    RuleNotFound(String),
    #[error("Alert not found: {0}")]
    AlertNotFound(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Action execution failed: {0}")]
    ActionFailed(String),
    #[error("Cooldown active for rule: {0}")]
    CooldownActive(String),
}

pub type VigilantResult<T> = Result<T, VigilantError>;

/// Alert types that can be triggered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum AlertType {
    /// Infinite loop detected (same action repeated many times)
    InfiniteLoop { task_id: String, iterations: u32 },
    /// No progress made after many steps
    NoProgress { task_id: String, steps: u32 },
    /// Resource exhaustion warning
    ResourceExhaustion { task_id: String, resource: String },
    /// Task timeout warning
    TimeoutWarning {
        task_id: String,
        remaining_secs: u32,
    },
    /// Execution pattern detected (from death spiral detection)
    PatternDetected { pattern: String, task_id: String },
    /// Error rate spike
    ErrorSpike { task_id: String, error_count: u32 },
    /// High memory usage
    HighMemoryUsage { percentage: u8 },
    /// LLM service unavailable
    LLMUnavailable,
    /// Execution pool exhausted
    ExecutionPoolExhausted,
    /// Task stuck waiting for confirmation
    AwaitingConfirmation { task_id: String, wait_secs: u32 },
}

impl AlertType {
    /// Get the task ID associated with this alert, if any
    pub fn task_id(&self) -> Option<&str> {
        match self {
            AlertType::InfiniteLoop { task_id, .. } => Some(task_id),
            AlertType::NoProgress { task_id, .. } => Some(task_id),
            AlertType::ResourceExhaustion { task_id, .. } => Some(task_id),
            AlertType::TimeoutWarning { task_id, .. } => Some(task_id),
            AlertType::PatternDetected { task_id, .. } => Some(task_id),
            AlertType::ErrorSpike { task_id, .. } => Some(task_id),
            AlertType::AwaitingConfirmation { task_id, .. } => Some(task_id),
            _ => None,
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> String {
        match self {
            AlertType::InfiniteLoop {
                task_id,
                iterations,
            } => {
                format!(
                    "Infinite loop detected on task {} ({} iterations)",
                    task_id, iterations
                )
            }
            AlertType::NoProgress { task_id, steps } => {
                format!("No progress on task {} after {} steps", task_id, steps)
            }
            AlertType::ResourceExhaustion { task_id, resource } => {
                format!("Resource exhaustion on task {}: {}", task_id, resource)
            }
            AlertType::TimeoutWarning {
                task_id,
                remaining_secs,
            } => {
                format!(
                    "Task {} will timeout in {} seconds",
                    task_id, remaining_secs
                )
            }
            AlertType::PatternDetected { pattern, task_id } => {
                format!(
                    "Execution pattern '{}' detected on task {}",
                    pattern, task_id
                )
            }
            AlertType::ErrorSpike {
                task_id,
                error_count,
            } => {
                format!("Error spike on task {}: {} errors", task_id, error_count)
            }
            AlertType::HighMemoryUsage { percentage } => {
                format!("High memory usage: {}%", percentage)
            }
            AlertType::LLMUnavailable => "LLM service unavailable".to_string(),
            AlertType::ExecutionPoolExhausted => "Execution pool exhausted".to_string(),
            AlertType::AwaitingConfirmation { task_id, wait_secs } => {
                format!(
                    "Task {} awaiting confirmation for {} seconds",
                    task_id, wait_secs
                )
            }
        }
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub enum AlertSeverity {
    Info = 0,
    Warning = 1,
    Critical = 2,
}

impl Default for AlertSeverity {
    fn default() -> Self {
        AlertSeverity::Warning
    }
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "Info"),
            AlertSeverity::Warning => write!(f, "Warning"),
            AlertSeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// Alert action to take when triggered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum AlertAction {
    /// Log the alert
    Log,
    /// Send notification
    Notify,
    /// Pause the task
    PauseTask,
    /// Cancel the task
    CancelTask,
    /// Send webhook notification
    Webhook { url: String },
    /// Execute a shell command
    ExecuteCommand { command: String },
    /// Send email notification
    Email { to: String, subject: Option<String> },
}

/// Email configuration for alert notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP server host
    pub smtp_host: String,
    /// SMTP server port
    pub smtp_port: u16,
    /// SMTP username
    pub username: String,
    /// SMTP password (should be stored as secret)
    pub password: String,
    /// From address
    pub from_address: String,
    /// Use TLS
    pub use_tls: bool,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            username: String::new(),
            password: String::new(),
            from_address: "apex-alerts@localhost".to_string(),
            use_tls: true,
        }
    }
}

impl AlertAction {
    /// Validate the action
    pub fn validate(&self) -> VigilantResult<()> {
        match self {
            AlertAction::Webhook { url } => {
                if url.is_empty() {
                    return Err(VigilantError::InvalidRule(
                        "Webhook URL cannot be empty".to_string(),
                    ));
                }
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return Err(VigilantError::InvalidRule(
                        "Webhook URL must be HTTP/HTTPS".to_string(),
                    ));
                }
            }
            AlertAction::ExecuteCommand { command } => {
                if command.is_empty() {
                    return Err(VigilantError::InvalidRule(
                        "Command cannot be empty".to_string(),
                    ));
                }
                // Security: warn about dangerous commands
                let dangerous = ["rm -rf", "shutdown", "reboot", "mkfs", "dd of=/dev/"];
                for d in dangerous {
                    if command.contains(d) {
                        tracing::warn!(
                            "Alert action contains potentially dangerous command: {}",
                            d
                        );
                    }
                }
            }
            AlertAction::Email { to, .. } => {
                if to.is_empty() {
                    return Err(VigilantError::InvalidRule(
                        "Email recipient cannot be empty".to_string(),
                    ));
                }
                if !to.contains('@') {
                    return Err(VigilantError::InvalidRule(
                        "Invalid email address".to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Alert rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Type of alert this rule triggers on
    pub alert_type: AlertType,
    /// Severity level
    pub severity: AlertSeverity,
    /// Cooldown period in seconds
    pub cooldown_secs: u32,
    /// Actions to take
    pub actions: Vec<AlertAction>,
    /// Whether this rule is enabled
    pub enabled: bool,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

impl AlertRule {
    /// Create a new alert rule with validation
    pub fn new(
        id: String,
        name: String,
        alert_type: AlertType,
        severity: AlertSeverity,
        cooldown_secs: u32,
        actions: Vec<AlertAction>,
    ) -> VigilantResult<Self> {
        // Validate actions
        for action in &actions {
            action.validate()?;
        }

        let now = chrono::Utc::now().to_rfc3339();
        Ok(Self {
            id,
            name,
            alert_type,
            severity,
            cooldown_secs,
            actions,
            enabled: true,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Create a default infinite loop detection rule
    pub fn infinite_loop_detection() -> Self {
        Self::new(
            "builtin-loop-detection".to_string(),
            "Infinite Loop Detection".to_string(),
            AlertType::InfiniteLoop {
                task_id: String::new(),
                iterations: 100,
            },
            AlertSeverity::Critical,
            300, // 5 minute cooldown
            vec![AlertAction::Notify, AlertAction::CancelTask],
        )
        .unwrap()
    }

    /// Create a default no progress warning rule
    pub fn no_progress_warning() -> Self {
        Self::new(
            "builtin-no-progress".to_string(),
            "No Progress Warning".to_string(),
            AlertType::NoProgress {
                task_id: String::new(),
                steps: 10,
            },
            AlertSeverity::Warning,
            60,
            vec![AlertAction::Notify],
        )
        .unwrap()
    }

    /// Create a default timeout warning rule
    pub fn timeout_warning() -> Self {
        Self::new(
            "builtin-timeout-warning".to_string(),
            "Timeout Warning".to_string(),
            AlertType::TimeoutWarning {
                task_id: String::new(),
                remaining_secs: 60,
            },
            AlertSeverity::Warning,
            0,
            vec![AlertAction::Notify],
        )
        .unwrap()
    }
}

/// Alert status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Dismissed,
    Resolved,
}

impl Default for AlertStatus {
    fn default() -> Self {
        AlertStatus::Active
    }
}

impl std::fmt::Display for AlertStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertStatus::Active => write!(f, "Active"),
            AlertStatus::Acknowledged => write!(f, "Acknowledged"),
            AlertStatus::Dismissed => write!(f, "Dismissed"),
            AlertStatus::Resolved => write!(f, "Resolved"),
        }
    }
}

/// An active alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique identifier
    pub id: String,
    /// Rule that triggered this alert
    pub rule_id: String,
    /// Type of alert
    pub alert_type: AlertType,
    /// Severity level
    pub severity: AlertSeverity,
    /// Associated task ID, if any
    pub task_id: Option<String>,
    /// Human-readable message
    pub message: String,
    /// Additional payload
    pub payload: Option<String>,
    /// Alert status
    pub status: AlertStatus,
    /// When the alert was created
    pub created_at: String,
    /// When acknowledged
    pub acknowledged_at: Option<String>,
    /// Who acknowledged it
    pub acknowledged_by: Option<String>,
    /// When resolved
    pub resolved_at: Option<String>,
    /// Escalation tracking
    pub escalation_level: u8,
    pub escalated_at: Option<String>,
    pub last_escalation_at: Option<String>,
}

impl Alert {
    /// Create a new alert from a rule
    pub fn from_rule(rule: &AlertRule, task_id: Option<String>) -> Self {
        Self {
            id: Ulid::new().to_string(),
            rule_id: rule.id.clone(),
            alert_type: rule.alert_type.clone(),
            severity: rule.severity,
            task_id,
            message: rule.alert_type.description(),
            payload: None,
            status: AlertStatus::Active,
            created_at: chrono::Utc::now().to_rfc3339(),
            acknowledged_at: None,
            acknowledged_by: None,
            resolved_at: None,
            escalation_level: 0,
            escalated_at: None,
            last_escalation_at: None,
        }
    }

    /// Acknowledge the alert
    pub fn acknowledge(&mut self, by: Option<String>) {
        self.status = AlertStatus::Acknowledged;
        self.acknowledged_at = Some(chrono::Utc::now().to_rfc3339());
        self.acknowledged_by = by;
    }

    /// Dismiss the alert
    pub fn dismiss(&mut self) {
        self.status = AlertStatus::Dismissed;
        self.resolved_at = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Resolve the alert
    pub fn resolve(&mut self) {
        self.status = AlertStatus::Resolved;
        self.resolved_at = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Escalate the alert to the next level
    pub fn escalate(&mut self, level: u8) {
        self.escalation_level = level;
        let now = chrono::Utc::now().to_rfc3339();
        self.escalated_at = Some(now.clone());
        self.last_escalation_at = Some(now);
    }

    /// Check if alert should escalate based on time
    pub fn should_escalate(&self, wait_secs: u32) -> bool {
        if self.status != AlertStatus::Active {
            return false;
        }
        let created = chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());
        let elapsed = (chrono::Utc::now() - created).num_seconds() as u32;
        elapsed >= wait_secs
    }

    /// Get time since creation in seconds
    pub fn time_since_created_secs(&self) -> i64 {
        chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| (chrono::Utc::now() - dt.with_timezone(&chrono::Utc)).num_seconds())
            .unwrap_or(0)
    }

    /// Get time to acknowledge in seconds (if acknowledged)
    pub fn time_to_acknowledge_secs(&self) -> Option<i64> {
        if let (Some(created), Some(ack)) = (
            chrono::DateTime::parse_from_rfc3339(&self.created_at).ok(),
            &self.acknowledged_at,
        ) {
            ack.parse::<chrono::DateTime<chrono::FixedOffset>>()
                .ok()
                .map(|ack_dt| {
                    (ack_dt.with_timezone(&chrono::Utc) - created.with_timezone(&chrono::Utc))
                        .num_seconds()
                })
        } else {
            None
        }
    }
}

/// Alert history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHistory {
    pub id: String,
    pub alert_id: String,
    pub action: String,
    pub performed_by: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}

impl AlertHistory {
    pub fn new(alert_id: String, action: String, performed_by: Option<String>) -> Self {
        Self {
            id: Ulid::new().to_string(),
            alert_id,
            action,
            performed_by,
            metadata: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Alert statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertStats {
    pub total_alerts: u32,
    pub active_alerts: u32,
    pub by_severity: HashMap<String, u32>,
    pub by_rule: HashMap<String, u32>,
    pub acknowledged_today: u32,
    pub resolved_today: u32,
}

/// Escalation level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    /// Level number (1 = first escalation, 2 = second, etc.)
    pub level: u8,
    /// Time in seconds before escalating from previous level
    pub wait_secs: u32,
    /// Actions to take at this escalation level
    pub actions: Vec<AlertAction>,
}

/// Escalation configuration for an alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    /// Whether escalation is enabled
    pub enabled: bool,
    /// Maximum escalation levels
    pub max_level: u8,
    /// Escalation levels (ordered by level)
    pub levels: Vec<EscalationLevel>,
    /// Default escalation wait time (seconds) if not specified per level
    pub default_wait_secs: u32,
}

impl Default for EscalationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_level: 3,
            levels: vec![
                EscalationLevel {
                    level: 1,
                    wait_secs: 300, // 5 minutes
                    actions: vec![
                        AlertAction::Notify,
                        AlertAction::Email {
                            to: String::new(),
                            subject: Some("Alert Unacknowledged".to_string()),
                        },
                    ],
                },
                EscalationLevel {
                    level: 2,
                    wait_secs: 600, // 10 minutes
                    actions: vec![AlertAction::ExecuteCommand {
                        command: "echo 'ALERT ESCALATED'".to_string(),
                    }],
                },
                EscalationLevel {
                    level: 3,
                    wait_secs: 0,
                    actions: vec![AlertAction::CancelTask],
                },
            ],
            default_wait_secs: 300,
        }
    }
}

/// Alert history entry for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAnalytics {
    /// Total alerts in time range
    pub total_alerts: u32,
    /// Alerts by severity
    pub by_severity: HashMap<String, u32>,
    /// Alerts by status
    pub by_status: HashMap<String, u32>,
    /// Alerts by rule
    pub by_rule: HashMap<String, u32>,
    /// Average time to acknowledge (seconds)
    pub avg_ack_time_secs: f64,
    /// Average time to resolve (seconds)
    pub avg_resolve_time_secs: f64,
    /// Most active rules
    pub top_rules: Vec<(String, u32)>,
    /// Alerts over time (hourly buckets)
    pub hourly_buckets: Vec<HourlyBucket>,
}

/// Hourly alert count bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyBucket {
    pub hour: String,
    pub count: u32,
    pub critical: u32,
    pub warning: u32,
    pub info: u32,
}

/// Death spiral pattern detected - for auto-rule creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_type: String,
    pub severity: String,
    pub occurrences: u32,
    pub last_occurrence: String,
    pub affected_tasks: Vec<String>,
}

impl DetectedPattern {
    /// Convert detected pattern to an alert rule suggestion
    pub fn to_rule_suggestion(&self) -> AlertRuleSuggestion {
        AlertRuleSuggestion {
            pattern_type: self.pattern_type.clone(),
            suggested_name: format!("Auto: {} Detection", self.pattern_type.replace("_", " ")),
            suggested_severity: match self.severity.as_str() {
                "critical" => AlertSeverity::Critical,
                "warning" => AlertSeverity::Warning,
                _ => AlertSeverity::Info,
            },
            suggested_actions: vec![AlertAction::Notify, AlertAction::Log],
            cooldown_secs: 300,
            confidence: (self.occurrences as f32 * 10.0).min(100.0) as u8,
            reason: format!(
                "Detected {} times across {} tasks",
                self.occurrences,
                self.affected_tasks.len()
            ),
        }
    }
}

/// Suggestion for auto-creating an alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRuleSuggestion {
    pub pattern_type: String,
    pub suggested_name: String,
    pub suggested_severity: AlertSeverity,
    pub suggested_actions: Vec<AlertAction>,
    pub cooldown_secs: u32,
    pub confidence: u8,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_rule_validation() {
        let rule = AlertRule::new(
            "test".to_string(),
            "Test Rule".to_string(),
            AlertType::NoProgress {
                task_id: "t1".to_string(),
                steps: 5,
            },
            AlertSeverity::Warning,
            60,
            vec![AlertAction::Notify],
        );
        assert!(rule.is_ok());
    }

    #[test]
    fn test_alert_rule_invalid_webhook() {
        let rule = AlertRule::new(
            "test".to_string(),
            "Test Rule".to_string(),
            AlertType::NoProgress {
                task_id: "t1".to_string(),
                steps: 5,
            },
            AlertSeverity::Warning,
            60,
            vec![AlertAction::Webhook { url: String::new() }],
        );
        assert!(rule.is_err());
    }

    #[test]
    fn test_alert_acknowledge() {
        let rule = AlertRule::infinite_loop_detection();
        let mut alert = Alert::from_rule(&rule, Some("task-1".to_string()));

        assert_eq!(alert.status, AlertStatus::Active);
        alert.acknowledge(Some("user".to_string()));
        assert_eq!(alert.status, AlertStatus::Acknowledged);
        assert!(alert.acknowledged_by.is_some());
    }
}
