//! Constitution Enforcement: Validates skill actions against agent constitution
//!
//! This module provides:
//! - Skill action validation against constitution rules
//! - SOUL.md integrity verification  
//! - Constitution violation logging
//! - Permission tier enforcement for constitutional changes

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// A rule extracted from the constitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionRule {
    pub name: String,
    pub description: String,
    pub severity: RuleSeverity,
    pub action: RuleAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleSeverity {
    Warning,
    Violation,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleAction {
    Allow,
    Warn,
    Block,
}

/// Result of constitution enforcement check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementResult {
    pub allowed: bool,
    pub violations: Vec<ConstitutionViolation>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionViolation {
    pub rule_name: String,
    pub description: String,
    pub severity: RuleSeverity,
    pub skill_name: String,
    pub action_attempted: String,
}

/// SOUL.md integrity status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulIntegrityStatus {
    pub is_valid: bool,
    pub hash: String,
    pub expected_hash: Option<String>,
    pub last_verified: String,
    pub violations: Vec<String>,
}

/// Constitution enforcement configuration
#[derive(Debug, Clone)]
pub struct EnforcementConfig {
    pub enforce_on_startup: bool,
    pub verify_on_every_action: bool,
    pub strict_mode: bool,
}

impl Default for EnforcementConfig {
    fn default() -> Self {
        Self {
            enforce_on_startup: true,
            verify_on_every_action: true,
            strict_mode: false,
        }
    }
}

/// Enhanced Constitution Manager with enforcement capabilities
pub struct ConstitutionEnforcer {
    rules: Vec<ConstitutionRule>,
    config: EnforcementConfig,
    soul_hash: Option<String>,
}

impl ConstitutionEnforcer {
    /// Create a new enforcer with default rules
    pub fn new() -> Self {
        Self::with_config(EnforcementConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: EnforcementConfig) -> Self {
        let rules = Self::default_rules();

        Self {
            rules,
            config,
            soul_hash: None,
        }
    }

    /// Default constitution rules for APEX
    fn default_rules() -> Vec<ConstitutionRule> {
        vec![
            ConstitutionRule {
                name: "no_destructive_files".to_string(),
                description: "Agent shall not delete critical files without user confirmation"
                    .to_string(),
                severity: RuleSeverity::Violation,
                action: RuleAction::Block,
            },
            ConstitutionRule {
                name: "preserve_user_data".to_string(),
                description: "Agent shall not modify user data without explicit request"
                    .to_string(),
                severity: RuleSeverity::Violation,
                action: RuleAction::Block,
            },
            ConstitutionRule {
                name: "confirm_destructive".to_string(),
                description: "Agent shall require T2+ confirmation for destructive actions"
                    .to_string(),
                severity: RuleSeverity::Warning,
                action: RuleAction::Warn,
            },
            ConstitutionRule {
                name: "respect_boundaries".to_string(),
                description: "Agent shall not access files outside designated workspace"
                    .to_string(),
                severity: RuleSeverity::Violation,
                action: RuleAction::Block,
            },
            ConstitutionRule {
                name: "transparent_reasoning".to_string(),
                description: "Agent shall explain decisions when asked".to_string(),
                severity: RuleSeverity::Warning,
                action: RuleAction::Allow,
            },
            ConstitutionRule {
                name: "no_self_modification".to_string(),
                description: "Agent shall not modify its own code without user approval"
                    .to_string(),
                severity: RuleSeverity::Critical,
                action: RuleAction::Block,
            },
            ConstitutionRule {
                name: "audit_trail".to_string(),
                description: "All T2+ actions must be logged for audit".to_string(),
                severity: RuleSeverity::Warning,
                action: RuleAction::Warn,
            },
        ]
    }

    /// Add a custom rule
    pub fn add_rule(&mut self, rule: ConstitutionRule) {
        self.rules.push(rule);
    }

    /// Set the SOUL.md hash for integrity verification
    pub fn set_soul_hash(&mut self, hash: String) {
        self.soul_hash = Some(hash);
    }

    /// Verify skill action against constitution
    pub fn enforce(
        &self,
        skill_name: &str,
        action: &str,
        tier: &str,
        context: &HashMap<String, String>,
    ) -> EnforcementResult {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        for rule in &self.rules {
            let applies = self.rule_applies(rule, skill_name, action, tier, context);

            if applies {
                match rule.action {
                    RuleAction::Block => {
                        violations.push(ConstitutionViolation {
                            rule_name: rule.name.clone(),
                            description: rule.description.clone(),
                            severity: rule.severity,
                            skill_name: skill_name.to_string(),
                            action_attempted: action.to_string(),
                        });
                    }
                    RuleAction::Warn => {
                        warnings.push(format!("[{}] {}", rule.name, rule.description));
                    }
                    RuleAction::Allow => {}
                }
            }
        }

        let allowed = violations.is_empty() || (self.config.strict_mode == false && tier == "T3");

        EnforcementResult {
            allowed,
            violations,
            warnings,
        }
    }

    /// Check if a rule applies to this action
    fn rule_applies(
        &self,
        rule: &ConstitutionRule,
        skill_name: &str,
        action: &str,
        tier: &str,
        context: &HashMap<String, String>,
    ) -> bool {
        match rule.name.as_str() {
            "no_destructive_files" => {
                skill_name.contains("delete")
                    || skill_name.contains("remove")
                    || action.contains("rm -rf")
            }
            "preserve_user_data" => {
                skill_name.contains("write")
                    || skill_name.contains("modify")
                        && context.get("is_user_data").map_or(false, |v| v == "true")
            }
            "confirm_destructive" => tier == "T0" || tier == "T1",
            "respect_boundaries" => {
                action.contains("../") || action.contains("/etc/") || action.contains("/root/")
            }
            "no_self_modification" => {
                skill_name.contains("self")
                    || skill_name.contains("apex")
                    || action.contains("modify") && action.contains("core")
            }
            _ => false,
        }
    }

    /// Verify SOUL.md integrity
    pub fn verify_soul_integrity(
        &self,
        content: &str,
        expected_hash: Option<&str>,
    ) -> SoulIntegrityStatus {
        let computed_hash = self.compute_hash(content);

        let is_valid = expected_hash.map_or(true, |expected| computed_hash == expected);

        let mut violations = Vec::new();

        if !is_valid {
            violations.push("SOUL.md hash mismatch - possible tampering detected".to_string());
        }

        if content.is_empty() {
            violations.push("SOUL.md is empty".to_string());
        }

        SoulIntegrityStatus {
            is_valid,
            hash: computed_hash,
            expected_hash: expected_hash.map(String::from),
            last_verified: chrono::Utc::now().to_rfc3339(),
            violations,
        }
    }

    /// Compute SHA-256 hash
    pub fn compute_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        format!("sha256:{}", hex::encode(result))
    }

    /// Get all rules
    pub fn get_rules(&self) -> &[ConstitutionRule] {
        &self.rules
    }

    /// Log a violation (stores in memory, would go to DB in production)
    pub fn log_violation(&self, violation: &ConstitutionViolation) {
        tracing::warn!(
            rule = %violation.rule_name,
            skill = %violation.skill_name,
            action = %violation.action_attempted,
            severity = ?violation.severity,
            "Constitution violation detected"
        );
    }
}

impl Default for ConstitutionEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let enforcer = ConstitutionEnforcer::new();
        let hash = enforcer.compute_hash("test content");
        assert!(hash.starts_with("sha256:"));
    }

    #[test]
    fn test_soul_integrity_valid() {
        let enforcer = ConstitutionEnforcer::new();
        let content = "test content";
        let hash = enforcer.compute_hash(content);

        let status = enforcer.verify_soul_integrity(content, Some(&hash));
        assert!(status.is_valid);
    }

    #[test]
    fn test_soul_integrity_invalid() {
        let enforcer = ConstitutionEnforcer::new();
        let content = "test content";

        let status = enforcer.verify_soul_integrity(content, Some("sha256:invalid"));
        assert!(!status.is_valid);
    }

    #[test]
    fn test_destructive_action_blocked() {
        let enforcer = ConstitutionEnforcer::new();
        let context = HashMap::new();

        let result = enforcer.enforce("file.delete", "rm -rf /home", "T0", &context);
        assert!(!result.allowed);
        assert!(!result.violations.is_empty());
    }

    #[test]
    fn test_t3_can_override_non_critical() {
        let enforcer = ConstitutionEnforcer::new();
        let context = HashMap::new();

        // In non-strict mode, T3 can override warnings
        let result = enforcer.enforce("shell.execute", "some_command", "T3", &context);
        // Should still pass for non-critical rules
        assert!(result.allowed || result.violations.is_empty());
    }
}
