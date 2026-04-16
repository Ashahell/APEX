//! Alert rule engine for matching and triggering alerts

use crate::vigilant::models::{Alert, AlertRule, AlertSeverity, AlertStatus, AlertType, VigilantResult};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static! {
    /// Default alert rules (initialized lazily to avoid const fn limitations)
    pub static ref DEFAULT_ALERT_RULES: Vec<AlertRule> = vec![
        AlertRule::infinite_loop_detection(),
        AlertRule::no_progress_warning(),
        AlertRule::timeout_warning(),
    ];
}

/// Rule engine for managing and evaluating alert rules
#[derive(Debug, Default)]
pub struct AlertRuleEngine {
    /// Active rules by ID
    rules: HashMap<String, AlertRule>,
    /// Last trigger time by rule ID (for cooldown)
    last_triggered: HashMap<String, i64>,
}

impl AlertRuleEngine {
    /// Create a new rule engine with default rules
    pub fn new() -> Self {
        let mut engine = Self::default();
        for rule in DEFAULT_ALERT_RULES.iter() {
            engine.rules.insert(rule.id.clone(), rule.clone());
        }
        engine
    }

    /// Add a rule
    pub fn add(&mut self, rule: AlertRule) -> VigilantResult<()> {
        // Validate the rule
        for action in &rule.actions {
            action.validate()?;
        }
        self.rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Remove a rule
    pub fn remove(&mut self, id: &str) -> Option<AlertRule> {
        self.last_triggered.remove(id);
        self.rules.remove(id)
    }

    /// Get a rule by ID
    pub fn get(&self, id: &str) -> Option<&AlertRule> {
        self.rules.get(id)
    }

    /// List all rules
    pub fn list(&self) -> Vec<&AlertRule> {
        self.rules.values().collect()
    }

    /// List only enabled rules
    pub fn list_enabled(&self) -> Vec<&AlertRule> {
        self.rules.values().filter(|r| r.enabled).collect()
    }

    /// Update a rule
    pub fn update(&mut self, id: &str, updates: AlertRuleUpdate) -> VigilantResult<()> {
        let rule = self
            .rules
            .get_mut(id)
            .ok_or_else(|| crate::vigilant::models::VigilantError::RuleNotFound(id.to_string()))?;

        if let Some(name) = updates.name {
            rule.name = name;
        }
        if let Some(severity) = updates.severity {
            rule.severity = severity;
        }
        if let Some(cooldown) = updates.cooldown_secs {
            rule.cooldown_secs = cooldown;
        }
        if let Some(actions) = updates.actions {
            for action in &actions {
                action.validate()?;
            }
            rule.actions = actions;
        }
        if let Some(enabled) = updates.enabled {
            rule.enabled = enabled;
        }

        rule.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    /// Check if a rule is in cooldown
    pub fn is_in_cooldown(&self, rule_id: &str) -> bool {
        if let Some(rule) = self.rules.get(rule_id) {
            if rule.cooldown_secs == 0 {
                return false;
            }
            if let Some(last) = self.last_triggered.get(rule_id) {
                let now = chrono::Utc::now().timestamp();
                let elapsed = now - last;
                return elapsed < rule.cooldown_secs as i64;
            }
        }
        false
    }

    /// Mark a rule as triggered (start cooldown)
    pub fn mark_triggered(&mut self, rule_id: &str) {
        let now = chrono::Utc::now().timestamp();
        self.last_triggered.insert(rule_id.to_string(), now);
    }

    /// Reset cooldown for a rule
    pub fn reset_cooldown(&mut self, rule_id: &str) {
        self.last_triggered.remove(rule_id);
    }

    /// Check an alert type against all rules and return matching alerts
    pub fn check(&mut self, alert_type: &AlertType) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // First pass: collect matching rule IDs and create alerts
        let mut to_trigger: Vec<(String, Alert)> = Vec::new();
        let alert_type_str = serde_json::to_string(alert_type).unwrap_or_default();

        for (rule_id, rule) in &self.rules {
            if !rule.enabled {
                continue;
            }

            // Check cooldown
            if self.is_in_cooldown(rule_id) {
                continue;
            }

            // Check if rule matches the alert type
            if self.rule_matches_type(rule, alert_type) {
                let task_id = alert_type.task_id().map(|s| s.to_string());
                let mut alert = Alert::from_rule(rule, task_id);
                alert.payload = Some(alert_type_str.clone());
                to_trigger.push((rule_id.clone(), alert));
            }
        }

        // Second pass: mark triggered and collect alerts
        for (rule_id, alert) in to_trigger {
            self.mark_triggered(&rule_id);
            alerts.push(alert);
        }

        alerts
    }

    /// Check if a rule matches an alert type
    fn rule_matches_type(&self, rule: &AlertRule, alert_type: &AlertType) -> bool {
        // Compare by variant name since the inner values might differ
        let rule_type_str = format!("{:?}", rule.alert_type);
        let rule_type_name = rule_type_str
            .split('{')
            .next()
            .map(|s| s.trim())
            .unwrap_or("");
        
        let alert_type_str = format!("{:?}", alert_type);
        let alert_type_name = alert_type_str
            .split('{')
            .next()
            .map(|s| s.trim())
            .unwrap_or("");

        rule_type_name == alert_type_name
    }

    /// Get default rules
    pub fn default_rules() -> Vec<AlertRule> {
        DEFAULT_ALERT_RULES.to_vec()
    }
}

/// Thread-safe wrapper for AlertRuleEngine
#[derive(Debug, Clone, Default)]
pub struct SharedAlertRuleEngine(pub Arc<RwLock<AlertRuleEngine>>);

impl SharedAlertRuleEngine {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(AlertRuleEngine::new())))
    }

    pub async fn add(&self, rule: AlertRule) -> VigilantResult<()> {
        let mut engine = self.0.write().await;
        engine.add(rule)
    }

    pub async fn remove(&self, id: &str) -> Option<AlertRule> {
        let mut engine = self.0.write().await;
        engine.remove(id)
    }

    pub async fn get(&self, id: &str) -> Option<AlertRule> {
        let engine = self.0.read().await;
        engine.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<AlertRule> {
        let engine = self.0.read().await;
        engine.list().into_iter().cloned().collect()
    }

    pub async fn list_enabled(&self) -> Vec<AlertRule> {
        let engine = self.0.read().await;
        engine.list_enabled().into_iter().cloned().collect()
    }

    pub async fn update(&self, id: &str, updates: AlertRuleUpdate) -> VigilantResult<()> {
        let mut engine = self.0.write().await;
        engine.update(id, updates)
    }

    pub async fn check(&self, alert_type: &AlertType) -> Vec<Alert> {
        let mut engine = self.0.write().await;
        engine.check(alert_type)
    }

    pub async fn reset_cooldown(&self, rule_id: &str) {
        let mut engine = self.0.write().await;
        engine.reset_cooldown(rule_id);
    }
}

/// Update fields for an alert rule
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlertRuleUpdate {
    pub name: Option<String>,
    pub severity: Option<AlertSeverity>,
    pub cooldown_secs: Option<u32>,
    pub actions: Option<Vec<crate::vigilant::models::AlertAction>>,
    pub enabled: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules() {
        let engine = AlertRuleEngine::new();
        let rules = engine.list();
        assert!(rules.len() >= 3);
    }

    #[test]
    fn test_cooldown() {
        let mut engine = AlertRuleEngine::new();
        
        // Mark as triggered
        engine.mark_triggered("builtin-loop-detection");
        
        // Should be in cooldown
        assert!(engine.is_in_cooldown("builtin-loop-detection"));
    }

    #[tokio::test]
    async fn test_shared_engine() {
        let engine = SharedAlertRuleEngine::new();
        
        let rules = engine.list().await;
        assert!(!rules.is_empty());

        let alerts = engine.check(&AlertType::InfiniteLoop {
            task_id: "test".to_string(),
            iterations: 100,
        }).await;

        assert!(!alerts.is_empty());
    }
}
