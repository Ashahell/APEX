//! Alert Correlation Data Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertSeverity::Critical => "critical",
            AlertSeverity::High => "high",
            AlertSeverity::Medium => "medium",
            AlertSeverity::Low => "low",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "critical" => Some(AlertSeverity::Critical),
            "high" => Some(AlertSeverity::High),
            "medium" => Some(AlertSeverity::Medium),
            "low" => Some(AlertSeverity::Low),
            _ => None,
        }
    }
}

/// Condition for matching alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationCondition {
    /// Pattern to match against alert source
    pub source_pattern: Option<String>,
    /// Pattern to match against alert message
    pub message_pattern: Option<String>,
    /// List of severities to match
    pub severity_in: Option<Vec<String>>,
    /// Time window in seconds
    pub time_window_secs: Option<u64>,
}

impl CorrelationCondition {
    pub fn matches(&self, source: &str, message: &str, severity: &str) -> bool {
        // Source pattern match
        if let Some(ref pattern) = self.source_pattern {
            if !source.contains(pattern) {
                return false;
            }
        }

        // Message pattern match
        if let Some(ref pattern) = self.message_pattern {
            if !message.contains(pattern) {
                return false;
            }
        }

        // Severity match
        if let Some(ref severities) = self.severity_in {
            if !severities.iter().any(|s| s == severity) {
                return false;
            }
        }

        true
    }
}

/// Action to take when condition matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationAction {
    /// Suppress individual alerts in the group
    pub suppress: bool,
    /// Aggregate alerts into a single notification
    pub aggregate: bool,
    /// Send notification when group is formed
    pub notify: bool,
    /// Auto-resolve the group after time window
    pub auto_resolve: bool,
}

impl Default for CorrelationAction {
    fn default() -> Self {
        Self {
            suppress: false,
            aggregate: true,
            notify: true,
            auto_resolve: false,
        }
    }
}

/// Alert correlation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCorrelationRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub condition: CorrelationCondition,
    pub action: CorrelationAction,
    pub enabled: bool,
    pub priority: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl AlertCorrelationRule {
    pub fn new(name: String, condition: CorrelationCondition) -> Self {
        let now = Utc::now();
        Self {
            id: ulid::Ulid::new().to_string(),
            name,
            description: None,
            condition,
            action: CorrelationAction::default(),
            enabled: true,
            priority: 0,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Check if an alert matches this rule
    pub fn matches(&self, source: &str, message: &str, severity: &str) -> bool {
        self.enabled && self.condition.matches(source, message, severity)
    }
}

/// Group of correlated alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertGroup {
    pub id: String,
    pub rule_id: String,
    pub group_key: String,
    pub alert_count: u32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub alerts: Vec<AlertEntry>,
}

impl AlertGroup {
    pub fn new(rule_id: String, group_key: String) -> Self {
        let now = Utc::now();
        Self {
            id: ulid::Ulid::new().to_string(),
            rule_id,
            group_key,
            alert_count: 1,
            first_seen: now,
            last_seen: now,
            resolved: false,
            resolved_at: None,
            alerts: Vec::new(),
        }
    }

    pub fn add_alert(&mut self, alert: AlertEntry) {
        self.alert_count += 1;
        self.last_seen = Utc::now();
        self.alerts.push(alert);
    }

    pub fn resolve(&mut self) {
        self.resolved = true;
        self.resolved_at = Some(Utc::now());
    }
}

/// Individual alert entry in a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEntry {
    pub id: String,
    pub source: String,
    pub message: String,
    pub severity: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl AlertEntry {
    pub fn new(source: String, message: String, severity: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            source,
            message,
            severity,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

/// Request to create a correlation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCorrelationRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub condition: CorrelationCondition,
    pub action: Option<CorrelationAction>,
    pub priority: Option<u32>,
}

/// Request to update a correlation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCorrelationRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub condition: Option<CorrelationCondition>,
    pub action: Option<CorrelationAction>,
    pub enabled: Option<bool>,
    pub priority: Option<u32>,
}

/// Incoming alert to process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessAlertRequest {
    pub source: String,
    pub message: String,
    pub severity: String,
    pub metadata: Option<HashMap<String, String>>,
}

/// Response for processing an alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessAlertResponse {
    pub alert_id: String,
    pub matched: bool,
    pub group_id: Option<String>,
    pub suppressed: bool,
    pub message: String,
}

/// Correlation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationStats {
    pub total_rules: u32,
    pub enabled_rules: u32,
    pub active_groups: u32,
    pub resolved_groups: u32,
    pub alerts_processed: u32,
    pub alerts_suppressed: u32,
    pub alerts_grouped: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_matches_source() {
        let condition = CorrelationCondition {
            source_pattern: Some("api".to_string()),
            message_pattern: None,
            severity_in: None,
            time_window_secs: None,
        };

        assert!(condition.matches("api-gateway", "test message", "high"));
        assert!(!condition.matches("database", "test message", "high"));
    }

    #[test]
    fn test_condition_matches_severity() {
        let condition = CorrelationCondition {
            source_pattern: None,
            message_pattern: None,
            severity_in: Some(vec!["critical".to_string(), "high".to_string()]),
            time_window_secs: None,
        };

        assert!(condition.matches("source", "message", "critical"));
        assert!(condition.matches("source", "message", "high"));
        assert!(!condition.matches("source", "message", "low"));
    }

    #[test]
    fn test_rule_matches() {
        let rule = AlertCorrelationRule::new(
            "Test Rule".to_string(),
            CorrelationCondition {
                source_pattern: Some("api".to_string()),
                ..Default::default()
            },
        );

        assert!(rule.matches("api-server", "error", "high"));
        assert!(!rule.matches("db-server", "error", "high"));

        // Disabled rule should not match
        let mut disabled_rule = rule.clone();
        disabled_rule.enabled = false;
        assert!(!disabled_rule.matches("api-server", "error", "high"));
    }

    #[test]
    fn test_alert_group() {
        let mut group = AlertGroup::new("rule-1".to_string(), "api-errors".to_string());
        assert_eq!(group.alert_count, 1);

        group.add_alert(AlertEntry::new(
            "api-1".to_string(),
            "error".to_string(),
            "high".to_string(),
        ));
        assert_eq!(group.alert_count, 2);

        group.resolve();
        assert!(group.resolved);
        assert!(group.resolved_at.is_some());
    }
}

impl Default for AlertCorrelationRule {
    fn default() -> Self {
        Self::new("Default Rule".to_string(), CorrelationCondition::default())
    }
}

impl Default for CorrelationCondition {
    fn default() -> Self {
        Self {
            source_pattern: None,
            message_pattern: None,
            severity_in: None,
            time_window_secs: None,
        }
    }
}
