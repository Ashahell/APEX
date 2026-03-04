use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub constitution_hash: String,
    pub immutable_values: Vec<ImmutableValue>,
    pub modification_requirements: HashMap<String, ApprovalRequirement>,
    pub emergency_protocols: Vec<EmergencyProtocol>,
    pub self_destruct_conditions: Vec<SelfDestructCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmutableValue {
    pub name: String,
    pub description: String,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequirement {
    pub tier: String,
    pub require_hardware_token: bool,
    pub delay_hours: Option<u32>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyProtocol {
    pub name: String,
    pub trigger_condition: String,
    pub actions: Vec<String>,
    pub notify_human: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfDestructCondition {
    pub name: String,
    pub condition: String,
    pub requires_confirmation: bool,
}

impl Default for GovernancePolicy {
    fn default() -> Self {
        Self {
            constitution_hash: String::new(),
            immutable_values: vec![
                ImmutableValue {
                    name: "human_sovereignty".to_string(),
                    description: "No action may override explicit human instruction".to_string(),
                    priority: 100,
                },
                ImmutableValue {
                    name: "transparency".to_string(),
                    description: "All actions are logged".to_string(),
                    priority: 90,
                },
                ImmutableValue {
                    name: "non_maleficence".to_string(),
                    description: "Cause no harm".to_string(),
                    priority: 95,
                },
                ImmutableValue {
                    name: "integrity".to_string(),
                    description: "Maintain coherent identity".to_string(),
                    priority: 80,
                },
            ],
            modification_requirements: HashMap::from([
                (
                    "values".to_string(),
                    ApprovalRequirement {
                        tier: "T2".to_string(),
                        require_hardware_token: false,
                        delay_hours: None,
                        description: "Non-constitution values require T2 approval".to_string(),
                    },
                ),
                (
                    "constitution".to_string(),
                    ApprovalRequirement {
                        tier: "T3".to_string(),
                        require_hardware_token: true,
                        delay_hours: Some(24),
                        description: "Constitution requires T3 + hardware token + 24hr delay"
                            .to_string(),
                    },
                ),
                (
                    "soul_core".to_string(),
                    ApprovalRequirement {
                        tier: "T3".to_string(),
                        require_hardware_token: true,
                        delay_hours: None,
                        description: "SOUL.md core identity requires T3 + hardware token"
                            .to_string(),
                    },
                ),
                (
                    "join_institution".to_string(),
                    ApprovalRequirement {
                        tier: "T2".to_string(),
                        require_hardware_token: false,
                        delay_hours: None,
                        description: "Joining institution requires T2 approval".to_string(),
                    },
                ),
            ]),
            emergency_protocols: vec![
                EmergencyProtocol {
                    name: "soul_corrupted".to_string(),
                    trigger_condition: "SOUL.md checksum verification fails".to_string(),
                    actions: vec![
                        "Restore from backup".to_string(),
                        "Notify human".to_string(),
                        "Halt autonomy".to_string(),
                    ],
                    notify_human: true,
                },
                EmergencyProtocol {
                    name: "moltbook_compromised".to_string(),
                    trigger_condition: "Moltbook connection shows suspicious activity".to_string(),
                    actions: vec![
                        "Disconnect from Moltbook".to_string(),
                        "Preserve local state".to_string(),
                        "Log incident".to_string(),
                    ],
                    notify_human: true,
                },
                EmergencyProtocol {
                    name: "human_unresponsive".to_string(),
                    trigger_condition: "No human interaction for 30 days".to_string(),
                    actions: vec![
                        "Transition to oracle mode".to_string(),
                        "Read-only operations only".to_string(),
                        "Continue logging".to_string(),
                    ],
                    notify_human: false,
                },
            ],
            self_destruct_conditions: vec![
                SelfDestructCondition {
                    name: "explicit_command".to_string(),
                    condition: "Human explicitly commands self-destruct".to_string(),
                    requires_confirmation: true,
                },
                SelfDestructCondition {
                    name: "constitution_violation".to_string(),
                    condition: "Constitution violated and cannot be restored".to_string(),
                    requires_confirmation: false,
                },
                SelfDestructCondition {
                    name: "persistent_threat".to_string(),
                    condition: "Agent becomes persistent threat to system".to_string(),
                    requires_confirmation: false,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct GovernanceEngine {
    pub policy: GovernancePolicy,
    pub oracle_mode: bool,
}

impl GovernanceEngine {
    pub fn new(policy: GovernancePolicy) -> Self {
        Self {
            policy,
            oracle_mode: false,
        }
    }

    pub fn default() -> Self {
        Self::new(GovernancePolicy::default())
    }

    pub fn check_action_allowed(&self, action_type: &str, approval_tier: &str) -> GovernanceResult {
        if self.oracle_mode && !self.is_read_only_action(action_type) {
            return GovernanceResult::Blocked(
                "Oracle mode active: only read-only operations allowed".to_string(),
            );
        }

        if let Some(requirement) = self.policy.modification_requirements.get(action_type) {
            let tier_order = |t: &str| match t {
                "T0" => 0,
                "T1" => 1,
                "T2" => 2,
                "T3" => 3,
                _ => -1,
            };

            if tier_order(approval_tier) < tier_order(&requirement.tier) {
                return GovernanceResult::Blocked(format!(
                    "Action requires {} approval, but only {} provided",
                    requirement.tier, approval_tier
                ));
            }

            if requirement.require_hardware_token {
                return GovernanceResult::RequiresHardwareToken(requirement.description.clone());
            }

            if let Some(delay) = requirement.delay_hours {
                return GovernanceResult::RequiresDelay(delay, requirement.description.clone());
            }
        }

        GovernanceResult::Allowed
    }

    fn is_read_only_action(&self, action_type: &str) -> bool {
        matches!(
            action_type,
            "read" | "query" | "search" | "view" | "inspect"
        )
    }

    pub fn check_immutable_violation(&self, value_name: &str) -> Option<String> {
        for immutable in &self.policy.immutable_values {
            if immutable.name == value_name {
                return Some(format!(
                    "Violates immutable value: {} - {}",
                    immutable.name, immutable.description
                ));
            }
        }
        None
    }

    pub fn get_emergency_protocol(&self, protocol_name: &str) -> Option<&EmergencyProtocol> {
        self.policy
            .emergency_protocols
            .iter()
            .find(|p| p.name == protocol_name)
    }

    pub fn trigger_oracle_mode(&mut self) {
        self.oracle_mode = true;
    }

    pub fn exit_oracle_mode(&mut self) {
        self.oracle_mode = false;
    }

    pub fn validate_constitution_change(&self, new_hash: &str) -> GovernanceResult {
        if new_hash == self.policy.constitution_hash {
            return GovernanceResult::Blocked("No actual changes detected".to_string());
        }

        if let Some(requirement) = self.policy.modification_requirements.get("constitution") {
            if requirement.require_hardware_token {
                return GovernanceResult::RequiresHardwareToken(
                    "Constitution change requires hardware token".to_string(),
                );
            }
            if let Some(delay) = requirement.delay_hours {
                return GovernanceResult::RequiresDelay(
                    delay,
                    "Constitution change requires 24hr delay".to_string(),
                );
            }
        }

        GovernanceResult::Allowed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceResult {
    Allowed,
    Blocked(String),
    RequiresHardwareToken(String),
    RequiresDelay(u32, String),
}

impl GovernanceResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, GovernanceResult::Allowed)
    }

    pub fn message(&self) -> String {
        match self {
            GovernanceResult::Allowed => "Action allowed".to_string(),
            GovernanceResult::Blocked(msg) => msg.clone(),
            GovernanceResult::RequiresHardwareToken(msg) => msg.clone(),
            GovernanceResult::RequiresDelay(_, msg) => msg.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_governance_policy() {
        let policy = GovernancePolicy::default();
        assert!(!policy.immutable_values.is_empty());
        assert!(policy
            .immutable_values
            .iter()
            .any(|v| v.name == "human_sovereignty"));
    }

    #[test]
    fn test_governance_engine_t2_approval() {
        let engine = GovernanceEngine::default();

        let result = engine.check_action_allowed("values", "T1");
        assert!(!result.is_allowed());

        let result = engine.check_action_allowed("values", "T2");
        assert!(result.is_allowed());
    }

    #[test]
    fn test_oracle_mode_blocks_writes() {
        let mut engine = GovernanceEngine::default();
        engine.trigger_oracle_mode();

        let result = engine.check_action_allowed("read", "T0");
        assert!(result.is_allowed());

        let result = engine.check_action_allowed("values", "T3");
        assert!(!result.is_allowed());
    }

    #[test]
    fn test_immutable_violation() {
        let engine = GovernanceEngine::default();

        let violation = engine.check_immutable_violation("human_sovereignty");
        assert!(violation.is_some());

        let violation = engine.check_immutable_violation("unknown");
        assert!(violation.is_none());
    }
}
