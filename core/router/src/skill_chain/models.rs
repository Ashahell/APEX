//! Skill Chain Data Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill chain execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

impl Default for ChainStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Skill step in a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStep {
    pub id: String,
    pub skill_name: String,
    pub input_template: String,
    pub output_variable: Option<String>,
    pub conditions: Vec<StepCondition>,
    pub on_success: NextStep,
    pub on_failure: NextStep,
    pub timeout_secs: u64,
    pub retry_on_failure: bool,
}

impl ChainStep {
    pub fn new(skill_name: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            skill_name,
            input_template: String::new(),
            output_variable: None,
            conditions: Vec::new(),
            on_success: NextStep::Next,
            on_failure: NextStep::End,
            timeout_secs: 300,
            retry_on_failure: true,
        }
    }
}

/// Condition for step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCondition {
    pub variable: String,
    pub operator: ConditionOperator,
    pub value: String,
}

impl StepCondition {
    pub fn evaluate(&self, variables: &HashMap<String, String>) -> bool {
        if let Some(var_value) = variables.get(&self.variable) {
            self.operator.evaluate(var_value, &self.value)
        } else {
            false
        }
    }
}

/// Condition operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
    IsSet,
    IsNotSet,
}

impl ConditionOperator {
    pub fn evaluate(&self, var_value: &str, compare_value: &str) -> bool {
        match self {
            ConditionOperator::Equals => var_value == compare_value,
            ConditionOperator::NotEquals => var_value != compare_value,
            ConditionOperator::Contains => var_value.contains(compare_value),
            ConditionOperator::GreaterThan => var_value
                .parse::<f64>()
                .map(|f| f > compare_value.parse().unwrap_or(f))
                .unwrap_or(false),
            ConditionOperator::LessThan => var_value
                .parse::<f64>()
                .map(|f| f < compare_value.parse().unwrap_or(f))
                .unwrap_or(false),
            ConditionOperator::IsSet => !var_value.is_empty(),
            ConditionOperator::IsNotSet => var_value.is_empty(),
        }
    }
}

/// What to do after a step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NextStep {
    Next,
    End,
    JumpTo(String),
}

/// Skill chain definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillChain {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<ChainStep>,
    pub variables: HashMap<String, String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl SkillChain {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: ulid::Ulid::new().to_string(),
            name,
            description: None,
            steps: Vec::new(),
            variables: HashMap::new(),
            enabled: true,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn add_step(&mut self, skill_name: String) -> &ChainStep {
        let step = ChainStep::new(skill_name);
        self.steps.push(step);
        self.steps.last().unwrap()
    }
}

/// Skill chain execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExecution {
    pub id: String,
    pub chain_id: String,
    pub status: ChainStatus,
    pub current_step: usize,
    pub variables: HashMap<String, String>,
    pub step_results: Vec<StepResult>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl ChainExecution {
    pub fn new(chain_id: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            chain_id,
            status: ChainStatus::Pending,
            current_step: 0,
            variables: HashMap::new(),
            step_results: Vec::new(),
            started_at: Utc::now(),
            completed_at: None,
            error: None,
        }
    }

    pub fn start(&mut self) {
        self.status = ChainStatus::Running;
    }

    pub fn record_step(&mut self, result: StepResult) {
        if result.success {
            self.variables
                .insert(result.output_variable.clone(), result.output.clone());
        }
        self.step_results.push(result);
    }

    pub fn complete(&mut self) {
        self.status = ChainStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    pub fn fail(&mut self, error: String) {
        self.status = ChainStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
    }

    pub fn cancel(&mut self) {
        self.status = ChainStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }
}

/// Result of a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub skill_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub output_variable: String,
}

/// Request to create a skill chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSkillChainRequest {
    pub name: String,
    pub description: Option<String>,
    pub steps: Option<Vec<CreateChainStepRequest>>,
    pub variables: Option<HashMap<String, String>>,
}

/// Request to create a chain step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChainStepRequest {
    pub skill_name: String,
    pub input_template: Option<String>,
    pub output_variable: Option<String>,
    pub conditions: Option<Vec<StepCondition>>,
    pub on_success: Option<NextStep>,
    pub on_failure: Option<NextStep>,
    pub timeout_secs: Option<u64>,
    pub retry_on_failure: Option<bool>,
}

/// Request to update a skill chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSkillChainRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub steps: Option<Vec<CreateChainStepRequest>>,
    pub variables: Option<HashMap<String, String>>,
    pub enabled: Option<bool>,
}

/// Request to execute a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteChainRequest {
    pub input_variables: Option<HashMap<String, String>>,
    pub start_at_step: Option<usize>,
}

/// Response for chain execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteChainResponse {
    pub execution_id: String,
    pub status: ChainStatus,
    pub message: String,
}

/// Chain execution status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExecutionStatus {
    pub execution_id: String,
    pub chain_id: String,
    pub chain_name: String,
    pub status: ChainStatus,
    pub current_step: usize,
    pub total_steps: usize,
    pub current_step_name: Option<String>,
    pub progress_percent: f64,
    pub variables: HashMap<String, String>,
    pub step_results: Vec<StepResult>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_evaluate() {
        let cond = StepCondition {
            variable: "status".to_string(),
            operator: ConditionOperator::Equals,
            value: "success".to_string(),
        };

        let mut vars = HashMap::new();
        vars.insert("status".to_string(), "success".to_string());
        assert!(cond.evaluate(&vars));

        vars.insert("status".to_string(), "failed".to_string());
        assert!(!cond.evaluate(&vars));
    }

    #[test]
    fn test_chain_add_step() {
        let mut chain = SkillChain::new("Test Chain".to_string());
        chain.add_step("code.generate".to_string());
        chain.add_step("git.commit".to_string());

        assert_eq!(chain.steps.len(), 2);
        assert_eq!(chain.steps[0].skill_name, "code.generate");
        assert_eq!(chain.steps[1].skill_name, "git.commit");
    }

    #[test]
    fn test_execution_flow() {
        let mut execution = ChainExecution::new("chain-1".to_string());
        assert_eq!(execution.status, ChainStatus::Pending);

        execution.start();
        assert_eq!(execution.status, ChainStatus::Running);

        execution.record_step(StepResult {
            step_id: "step-1".to_string(),
            skill_name: "code.generate".to_string(),
            success: true,
            output: "generated code".to_string(),
            error: None,
            duration_ms: 100,
            output_variable: "code".to_string(),
        });

        assert_eq!(
            execution.variables.get("code"),
            Some(&"generated code".to_string())
        );

        execution.complete();
        assert_eq!(execution.status, ChainStatus::Completed);
        assert!(execution.completed_at.is_some());
    }
}

impl Default for SkillChain {
    fn default() -> Self {
        Self::new("Default Chain".to_string())
    }
}
