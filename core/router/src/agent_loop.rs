use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::llama::LlamaClient;

const INJECTION_PATTERNS: &[&str] = &[
    r"(?i)^\s*ignore\s+previous\s+instructions",
    r"(?i)^\s*disregard\s+.*instructions",
    r"(?i)^\s*forget\s+.*rules",
    r"(?i)^\s*system:\s*",
    r"(?i)^\s*#\s*instructions",
    r"(?i)^\s*you\s+are\s+now",
    r"(?i)^\s*pretend\s+to\s+be",
    r"(?i)^\s*roleplay\s+as",
];

fn sanitize_for_llm(input: &str) -> String {
    let mut result = input.to_string();
    
    for pattern in INJECTION_PATTERNS {
        if let Ok(regex) = Regex::new(pattern) {
            result = regex.replace_all(&result, "[FILTERED]").to_string();
        }
    }
    
    result
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_steps: u32,
    pub max_budget_usd: f64,
    pub step_cost_usd: f64,
    pub time_limit_secs: Option<u64>,
    pub allowed_domains: Vec<String>,
    pub tools: Vec<String>,
    pub use_llm: bool,
    pub llama_url: Option<String>,
    pub llama_model: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_steps: 3,
            max_budget_usd: 1.0,
            step_cost_usd: 0.01,
            time_limit_secs: None,
            allowed_domains: vec![],
            tools: vec![
                "bash".to_string(),
                "read".to_string(),
                "write".to_string(),
                "grep".to_string(),
            ],
            use_llm: std::env::var("APEX_USE_LLM").is_ok(),
            llama_url: std::env::var("LLAMA_SERVER_URL").ok(),
            llama_model: std::env::var("LLAMA_MODEL").ok(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentState {
    pub task_id: String,
    pub goal: String,
    pub current_step: u32,
    pub total_cost_usd: f64,
    pub history: Vec<AgentStep>,
    pub is_complete: bool,
    pub error: Option<String>,
}

impl AgentState {
    pub fn new(task_id: String, goal: String) -> Self {
        Self {
            task_id,
            goal,
            current_step: 0,
            total_cost_usd: 0.0,
            history: Vec::new(),
            is_complete: false,
            error: None,
        }
    }

    pub fn start_time() -> std::time::Instant {
        std::time::Instant::now()
    }

    pub fn can_continue(&mut self, config: &AgentConfig, start_time: std::time::Instant) -> bool {
        if self.is_complete {
            return false;
        }
        if self.current_step >= config.max_steps {
            self.error = Some("Max steps reached".to_string());
            return false;
        }
        if self.total_cost_usd >= config.max_budget_usd {
            self.error = Some("Budget exhausted".to_string());
            return false;
        }
        if let Some(time_limit) = config.time_limit_secs {
            if !config.use_llm && start_time.elapsed().as_secs() >= time_limit {
                self.error = Some("Time limit exceeded".to_string());
                return false;
            }
        }
        true
    }

    pub fn record_step(&mut self, step: AgentStep) {
        self.current_step += 1;
        self.total_cost_usd += step.cost_usd;
        self.history.push(step);
    }

    pub fn get_result(&self) -> AgentResult {
        if let Some(ref error) = self.error {
            AgentResult {
                task_id: self.task_id.clone(),
                success: false,
                steps_executed: self.current_step,
                total_cost_usd: self.total_cost_usd,
                output: error.clone(),
                history: self.history.clone(),
            }
        } else {
            AgentResult {
                task_id: self.task_id.clone(),
                success: self.is_complete,
                steps_executed: self.current_step,
                total_cost_usd: self.total_cost_usd,
                output: "Task completed".to_string(),
                history: self.history.clone(),
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentStep {
    pub step_number: u32,
    pub action: String,
    pub observation: String,
    pub cost_usd: f64,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentResult {
    pub task_id: String,
    pub success: bool,
    pub steps_executed: u32,
    pub total_cost_usd: f64,
    pub output: String,
    pub history: Vec<AgentStep>,
}

pub type ProgressCallback = dyn Fn(&AgentState) + Send + Sync + 'static;

pub struct AgentLoop {
    config: AgentConfig,
    #[allow(clippy::type_complexity)]
    on_step_complete: Option<Box<ProgressCallback>>,
}

impl AgentLoop {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            on_step_complete: None,
        }
    }

    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&AgentState) + Send + Sync + 'static,
    {
        self.on_step_complete = Some(Box::new(callback));
        self
    }

    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        if self.config.allowed_domains.is_empty() {
            return true;
        }

        for pattern in &self.config.allowed_domains {
            if let Ok(regex) = Regex::new(&format!(".*\\.{}|^{}", pattern, pattern)) {
                if regex.is_match(domain) {
                    return true;
                }
            }
        }

        false
    }

    pub async fn run(&self, task_id: String, goal: String) -> AgentResult {
        let mut state = AgentState::new(task_id.clone(), goal);
        let start_time = AgentState::start_time();

        tracing::info!(task_id = %task_id, goal = %state.goal, "Starting agent loop");

        while state.can_continue(&self.config, start_time) {
            let step_number = state.current_step + 1;
            let step_start = Instant::now();
            tracing::debug!(task_id = %task_id, step = step_number, "Executing step");

            let plan_start = Instant::now();
            let action = self.plan(&state).await;
            let plan_ms = plan_start.elapsed().as_millis();

            let act_start = Instant::now();
            let observation = self.act(&action, &state).await;
            let act_ms = act_start.elapsed().as_millis();

            let should_continue = self.reflect(&observation, &mut state).await;

            let step = AgentStep {
                step_number,
                action,
                observation: observation.clone(),
                cost_usd: self.config.step_cost_usd,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            state.record_step(step);

            tracing::info!(task_id = %task_id, step = step_number, plan_ms, act_ms, total_ms = step_start.elapsed().as_millis(), "Step complete");

            if let Some(ref callback) = self.on_step_complete {
                callback(&state);
            }

            if !should_continue {
                break;
            }
        }

        if state.current_step == 0 && state.error.is_none() {
            state.is_complete = true;
        }

        let elapsed = start_time.elapsed();
        tracing::info!(
            task_id = %task_id,
            steps = state.current_step,
            cost = state.total_cost_usd,
            duration_ms = elapsed.as_millis(),
            "Agent loop completed"
        );

        state.get_result()
    }

    async fn plan(&self, state: &AgentState) -> String {
        let context = state
            .history
            .iter()
            .map(|s| format!("Step {}: {} -> {}", s.step_number, s.action, s.observation))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Goal: {}\n\nHistory:\n{}\n\nWhat is the next action to take? Respond with a single action.",
            state.goal,
            context
        );

        if self.config.use_llm {
            tracing::info!("LLM enabled, attempting to call llama-server");
            if let (Some(url), Some(model)) = (&self.config.llama_url, &self.config.llama_model) {
                tracing::info!(url = %url, model = %model, "Connecting to LLM");
                let client = LlamaClient::new(url.clone(), model.clone());
                match client
                    .chat(
                        "You are an autonomous agent. Respond with a single action to take.",
                        &prompt,
                    )
                    .await
                {
                    Ok(response) => {
                        tracing::info!(response = %response, "LLM response received");
                        return response;
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "LLM call failed, using mock");
                    }
                }
            } else {
                tracing::warn!("LLM enabled but URL or model not configured");
            }
        }

        format!(
            "Plan: {} (simulated)",
            prompt.chars().take(50).collect::<String>()
        )
    }

    async fn act(&self, action: &str, state: &AgentState) -> String {
        let action_lower = action.to_lowercase();

        // Try to use LLM for generating response
        if self.config.use_llm {
            if let (Some(url), Some(model)) = (&self.config.llama_url, &self.config.llama_model) {
                let sanitized_goal = sanitize_for_llm(&state.goal);
                let sanitized_action = sanitize_for_llm(action);
                let prompt = format!(
                    "User's goal: {}\nAction taken: {}\n\nProvide a brief, helpful response to the user (1-2 sentences):",
                    sanitized_goal,
                    sanitized_action
                );

                let client = crate::llama::LlamaClient::new(url.clone(), model.clone());
                match client
                    .chat("You are a helpful AI assistant.", &prompt)
                    .await
                {
                    Ok(response) => {
                        return response;
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "LLM act call failed, using mock");
                    }
                }
            }
        }

        // Fallback to mock responses
        if action_lower.contains("greet") {
            "Hello! How can I help you today?".to_string()
        } else if action_lower.contains("build")
            || action_lower.contains("create")
            || action_lower.contains("write")
        {
            format!(
                "I'll help you with: {}. What specific details would you like?",
                state.goal.chars().take(30).collect::<String>()
            )
        } else if action_lower.contains("read")
            || action_lower.contains("search")
            || action_lower.contains("find")
        {
            format!("I found some information about: {}", state.goal)
        } else if action_lower.contains("error") || action_lower.contains("fail") {
            "I encountered an issue. Can you provide more details?".to_string()
        } else {
            format!(
                "I understand your goal: {}. How would you like to proceed?",
                state.goal.chars().take(30).collect::<String>()
            )
        }
    }

    async fn reflect(&self, observation: &str, state: &mut AgentState) -> bool {
        let obs_lower = observation.to_lowercase();

        let completion_keywords = [
            "completed",
            "done",
            "finished",
            "success",
            "built",
            "created",
            "how can i help",
            "how can i assist",
            "what would you like",
            "how would you like",
            "i understand your goal",
        ];
        let should_stop = completion_keywords.iter().any(|kw| obs_lower.contains(kw));

        if should_stop {
            state.is_complete = true;
            return false;
        }

        if state.current_step >= 1 {
            let is_repeating = state.history.iter()
                .skip(1)
                .all(|h| h.observation == observation);
            if is_repeating && state.history.len() >= 2 {
                state.is_complete = true;
                return false;
            }
        }

        let error_keywords = ["error", "failed", "exception", "cannot", "unable"];
        if error_keywords.iter().any(|kw| obs_lower.contains(kw)) && state.current_step >= 1 {
            return false;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_loop_completes() {
        let config = AgentConfig {
            max_steps: 5,
            max_budget_usd: 1.0,
            step_cost_usd: 0.1,
            ..Default::default()
        };

        let agent = AgentLoop::new(config);
        let result = agent
            .run("test-1".to_string(), "Build a website".to_string())
            .await;

        assert!(result.steps_executed > 0);
        assert!(result.total_cost_usd > 0.0);
    }

    #[test]
    fn test_agent_state_budget_check() {
        let config = AgentConfig::default();
        let mut state = AgentState::new("test".to_string(), "test goal".to_string());
        let start_time = AgentState::start_time();

        state.total_cost_usd = 0.9;

        assert!(state.can_continue(&config, start_time));

        state.total_cost_usd = 1.1;

        assert!(!state.can_continue(&config, start_time));
        assert!(state.error.is_some());
    }
}
