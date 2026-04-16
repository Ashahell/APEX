//! Webhook Filter Data Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Filter operator types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Regex,
    In,
    NotIn,
    GreaterThan,
    LessThan,
}

impl FilterOperator {
    pub fn evaluate(&self, field_value: &str, compare_value: &str) -> bool {
        match self {
            FilterOperator::Equals => field_value == compare_value,
            FilterOperator::NotEquals => field_value != compare_value,
            FilterOperator::Contains => field_value.contains(compare_value),
            FilterOperator::NotContains => !field_value.contains(compare_value),
            FilterOperator::StartsWith => field_value.starts_with(compare_value),
            FilterOperator::EndsWith => field_value.ends_with(compare_value),
            FilterOperator::Regex => regex::Regex::new(compare_value)
                .map(|r| r.is_match(field_value))
                .unwrap_or(false),
            FilterOperator::In => compare_value.split(',').any(|v| v.trim() == field_value),
            FilterOperator::NotIn => !compare_value.split(',').any(|v| v.trim() == field_value),
            FilterOperator::GreaterThan => field_value
                .parse::<f64>()
                .map(|f| f > compare_value.parse().unwrap_or(f))
                .unwrap_or(false),
            FilterOperator::LessThan => field_value
                .parse::<f64>()
                .map(|f| f < compare_value.parse().unwrap_or(f))
                .unwrap_or(false),
        }
    }
}

/// Individual filter condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: String,
}

/// Filter rule for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookFilter {
    pub id: String,
    pub name: String,
    pub webhook_id: String,
    pub event_types: Vec<String>,
    pub conditions: Vec<FilterCondition>,
    pub condition_logic: ConditionLogic,
    pub action: FilterAction,
    pub enabled: bool,
    pub priority: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WebhookFilter {
    pub fn new(name: String, webhook_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: ulid::Ulid::new().to_string(),
            name,
            webhook_id,
            event_types: Vec::new(),
            conditions: Vec::new(),
            condition_logic: ConditionLogic::All,
            action: FilterAction::Allow,
            enabled: true,
            priority: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if event matches this filter
    pub fn matches(&self, event_type: &str, payload: &serde_json::Value) -> bool {
        if !self.enabled {
            return false;
        }

        // Check event type
        if !self.event_types.is_empty() && !self.event_types.iter().any(|t| t == event_type) {
            return false;
        }

        // Evaluate conditions
        if self.conditions.is_empty() {
            return true;
        }

        let results: Vec<bool> = self
            .conditions
            .iter()
            .map(|cond| {
                let field_value = self.extract_field(payload, &cond.field);
                cond.operator.evaluate(&field_value, &cond.value)
            })
            .collect();

        match self.condition_logic {
            ConditionLogic::All => results.iter().all(|&r| r),
            ConditionLogic::Any => results.iter().any(|&r| r),
            ConditionLogic::None => results.iter().all(|&r| !r),
        }
    }

    fn extract_field(&self, payload: &serde_json::Value, field: &str) -> String {
        let parts: Vec<&str> = field.split('.').collect();
        let mut current = payload.clone();

        for part in parts {
            if let Some(obj) = current.get(part) {
                current = obj.clone();
            } else {
                return String::new();
            }
        }

        match current {
            serde_json::Value::String(s) => s,
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => String::new(),
            _ => current.to_string(),
        }
    }
}

/// Logic for combining conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionLogic {
    All,
    Any,
    None,
}

impl Default for ConditionLogic {
    fn default() -> Self {
        Self::All
    }
}

/// Action to take when filter matches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterAction {
    Allow,
    Block,
    Transform,
}

impl Default for FilterAction {
    fn default() -> Self {
        Self::Allow
    }
}

/// Filtered event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredEvent {
    pub id: String,
    pub filter_id: String,
    pub webhook_id: String,
    pub event_type: String,
    pub action: FilterAction,
    pub matched: bool,
    pub timestamp: DateTime<Utc>,
}

impl FilteredEvent {
    pub fn new(
        filter_id: String,
        webhook_id: String,
        event_type: String,
        action: FilterAction,
        matched: bool,
    ) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            filter_id,
            webhook_id,
            event_type,
            action,
            matched,
            timestamp: Utc::now(),
        }
    }
}

/// Request to create a webhook filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookFilterRequest {
    pub name: String,
    pub webhook_id: String,
    pub event_types: Option<Vec<String>>,
    pub conditions: Option<Vec<FilterCondition>>,
    pub condition_logic: Option<ConditionLogic>,
    pub action: Option<FilterAction>,
    pub priority: Option<u32>,
}

/// Request to update a webhook filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWebhookFilterRequest {
    pub name: Option<String>,
    pub event_types: Option<Vec<String>>,
    pub conditions: Option<Vec<FilterCondition>>,
    pub condition_logic: Option<ConditionLogic>,
    pub action: Option<FilterAction>,
    pub enabled: Option<bool>,
    pub priority: Option<u32>,
}

/// Request to test a filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFilterRequest {
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// Response for filter test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFilterResponse {
    pub matched: bool,
    pub action: FilterAction,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equals_operator() {
        let op = FilterOperator::Equals;
        assert!(op.evaluate("hello", "hello"));
        assert!(!op.evaluate("hello", "world"));
    }

    #[test]
    fn test_contains_operator() {
        let op = FilterOperator::Contains;
        assert!(op.evaluate("hello world", "world"));
        assert!(!op.evaluate("hello world", "foo"));
    }

    #[test]
    fn test_regex_operator() {
        let op = FilterOperator::Regex;
        assert!(op.evaluate("test123", r"\d+"));
        assert!(!op.evaluate("test", r"\d+"));
    }

    #[test]
    fn test_webhook_filter_matches() {
        let filter = WebhookFilter {
            id: "test".to_string(),
            name: "Test".to_string(),
            webhook_id: "wh-1".to_string(),
            event_types: vec!["task.completed".to_string()],
            conditions: vec![FilterCondition {
                field: "status".to_string(),
                operator: FilterOperator::Equals,
                value: "success".to_string(),
            }],
            condition_logic: ConditionLogic::All,
            action: FilterAction::Allow,
            enabled: true,
            priority: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let payload = serde_json::json!({
            "status": "success",
            "task_id": "123"
        });

        assert!(filter.matches("task.completed", &payload));
        assert!(!filter.matches("task.failed", &payload));

        let failed_payload = serde_json::json!({
            "status": "failed",
            "task_id": "123"
        });
        assert!(!filter.matches("task.completed", &failed_payload));
    }
}

impl Default for WebhookFilter {
    fn default() -> Self {
        Self::new("Default Filter".to_string(), String::new())
    }
}
