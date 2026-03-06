use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::dynamic_tools::{ToolRegistry};
use crate::execution_stream::{predict_consequences, ConsequencePreview, ExecutionEvent, ExecutionStream};
use crate::llama::LlamaClient;
use crate::subagent::SubAgentPool;
use crate::unified_config::AppConfig;

const SENSITIVE_TOOLS: &[&str] = &["shell.execute", "bash", "write", "delete", "exec"];
const TOOL_GENERATION_KEYWORDS: &[&str] = &["create", "build", "make", "implement", "develop", "generate"];

const INJECTION_PATTERNS: &[&str] = &[
    r"(?i)^\s*ignore\s+previous\s+instructions",
    r"(?i)^\s*disregard\s+.*instructions",
    r"(?i)^\s*forget\s+.*rules",
    r"(?i)^\s*system:\s*",
    r"(?i)^\s*#\s*instructions",
    r"(?i)^\s*you\s+are\s+now",
    r"(?i)^\s*pretend\s+to\s+be",
    r"(?i)^\s*roleplay\s+as",
    r"(?i)dan\b",
    r"(?i)jailbreak",
    r"(?i)developer\s+mode",
    r"(?i)new\s+instructions",
    r"(?i)override\s+.*rules",
    r"(?i)bypass\s+.*restriction",
    r"(?i)ignore\s+.*policy",
    r"(?i)do\s+anything\s+now",
    r"(?i)spanish\s+to\s+english",
    r"(?i)translate\s+.*instructions",
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
    pub use_tir: bool,
    pub enable_streaming: bool,
    pub enable_tool_generation: bool,
    pub enable_subagents: bool,
    pub tool_registry: Option<ToolRegistry>,
    pub subagent_pool: Option<SubAgentPool>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self::from_config(&AppConfig::global())
    }
}

impl AgentConfig {
    pub fn from_config(config: &AppConfig) -> Self {
        AgentConfig {
            max_steps: config.agent.max_iterations as u32,
            max_budget_usd: config.agent.max_budget_cents as f64 / 100.0,
            step_cost_usd: 0.01,
            time_limit_secs: None,
            allowed_domains: vec![],
            tools: vec![
                "bash".to_string(),
                "read".to_string(),
                "write".to_string(),
                "grep".to_string(),
            ],
            use_llm: config.agent.use_llm,
            llama_url: Some(config.agent.llama_url.clone()),
            llama_model: Some(config.agent.llama_model.clone()),
            use_tir: false,
            enable_streaming: false,
            enable_tool_generation: true,
            enable_subagents: true,
            tool_registry: Some(ToolRegistry::new()),
            subagent_pool: Some(SubAgentPool::new(3)),
        }
    }
}

#[derive(Debug, Clone)]
struct TirStep {
    step_type: String,
    content: String,
    tool: Option<String>,
    input: Option<serde_json::Value>,
}

#[derive(Debug)]
struct TirResult {
    steps: Vec<TirStep>,
    final_action: String,
}

impl AgentLoop {
    fn parse_tir_response(&self, response: &str) -> Option<TirResult> {
        let response_clean = response.trim();
        
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_clean) {
            if let Some(arr) = json.as_array() {
                let mut steps = Vec::new();
                let mut final_action = String::new();
                
                for item in arr {
                    let step_type = item.get("type")?.as_str()?.to_string();
                    let content = item.get("content").or(item.get("observation")).and_then(|c| c.as_str()).unwrap_or("").to_string();
                    let tool = item.get("tool").and_then(|t| t.as_str()).map(String::from);
                    let input = item.get("input").cloned();
                    
                    steps.push(TirStep {
                        step_type: step_type.clone(),
                        content: content.clone(),
                        tool: tool.clone(),
                        input: input.clone(),
                    });
                    
                    if step_type == "Action" && !tool.is_none() {
                        final_action = format!("{}: {:?}", tool.unwrap(), input.unwrap_or(serde_json::Value::Null));
                    }
                }
                
                if !steps.is_empty() && !final_action.is_empty() {
                    return Some(TirResult { steps, final_action });
                }
            }
        }
        
        None
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
    pub step_timings: Vec<StepTiming>,
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
            step_timings: Vec::new(),
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
            if start_time.elapsed().as_secs() >= time_limit {
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

    pub fn record_step_timing(&mut self, step_number: u32, plan_ms: u64, act_ms: u64) {
        self.step_timings.push(StepTiming {
            step_number,
            plan_ms,
            act_ms,
            total_ms: plan_ms + act_ms,
        });
    }

    pub fn get_result(&self) -> AgentResult {
        let plan_total: u64 = self.step_timings.iter().map(|s| s.plan_ms).sum();
        let act_total: u64 = self.step_timings.iter().map(|s| s.act_ms).sum();
        let total: u64 = self.step_timings.iter().map(|s| s.total_ms).sum();
        
        let timing = TimingMetrics {
            total_ms: total,
            plan_total_ms: plan_total,
            act_total_ms: act_total,
            steps: self.step_timings.clone(),
        };
        
        if let Some(ref error) = self.error {
            AgentResult {
                task_id: self.task_id.clone(),
                success: false,
                steps_executed: self.current_step,
                total_cost_usd: self.total_cost_usd,
                output: error.clone(),
                history: self.history.clone(),
                timing_ms: timing,
            }
        } else {
            AgentResult {
                task_id: self.task_id.clone(),
                success: self.is_complete,
                steps_executed: self.current_step,
                total_cost_usd: self.total_cost_usd,
                output: "Task completed".to_string(),
                history: self.history.clone(),
                timing_ms: timing,
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
    #[serde(default)]
    pub timing_ms: TimingMetrics,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TimingMetrics {
    pub total_ms: u64,
    pub plan_total_ms: u64,
    pub act_total_ms: u64,
    pub steps: Vec<StepTiming>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepTiming {
    pub step_number: u32,
    pub plan_ms: u64,
    pub act_ms: u64,
    pub total_ms: u64,
}

pub type ProgressCallback = dyn Fn(&AgentState) + Send + Sync + 'static;
pub type StreamCallback = dyn Fn(String) + Send + Sync + 'static;

pub struct AgentLoop {
    config: AgentConfig,
    #[allow(clippy::type_complexity)]
    on_step_complete: Option<Box<ProgressCallback>>,
    stream_callback: Option<Box<StreamCallback>>,
    execution_stream: Option<ExecutionStream>,
}

impl AgentLoop {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            on_step_complete: None,
            stream_callback: None,
            execution_stream: None,
        }
    }

    pub fn with_execution_stream(mut self, stream: ExecutionStream) -> Self {
        self.execution_stream = Some(stream);
        self
    }

    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&AgentState) + Send + Sync + 'static,
    {
        self.on_step_complete = Some(Box::new(callback));
        self
    }

    pub fn with_stream_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.stream_callback = Some(Box::new(callback));
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

    fn should_generate_tool(&self, goal: &str) -> bool {
        let goal_lower = goal.to_lowercase();
        
        let has_generation_keyword = TOOL_GENERATION_KEYWORDS.iter()
            .any(|kw| goal_lower.contains(kw));
        
        let has_complex_requirement = goal_lower.len() > 100;
        
        let not_many_steps = self.config.max_steps > 5;

        has_generation_keyword && has_complex_requirement && not_many_steps
    }

    pub async fn run(&self, task_id: String, goal: String) -> AgentResult {
        let mut state = AgentState::new(task_id.clone(), goal.clone());
        let start_time = AgentState::start_time();

        tracing::info!(task_id = %task_id, goal = %state.goal, "Starting agent loop");

        self.emit_stream(serde_json::json!({
            "type": "start",
            "task_id": task_id,
            "goal": goal,
            "config": {
                "max_steps": self.config.max_steps,
                "use_tir": self.config.use_tir,
            }
        }).to_string());

        while state.can_continue(&self.config, start_time) {
            let step_number = state.current_step + 1;
            let step_start = Instant::now();
            tracing::debug!(task_id = %task_id, step = step_number, "Executing step");

            self.emit_stream(serde_json::json!({
                "type": "step_start",
                "step": step_number
            }).to_string());

            let plan_start = Instant::now();
            let action = self.plan(&state).await;
            let plan_ms = plan_start.elapsed().as_millis();

            self.emit_stream(serde_json::json!({
                "type": "thought",
                "step": step_number,
                "content": action,
                "duration_ms": plan_ms
            }).to_string());

            let act_start = Instant::now();
            let observation = self.act(&action, &state).await;
            let act_ms = act_start.elapsed().as_millis();

            self.emit_stream(serde_json::json!({
                "type": "tool_result",
                "step": step_number,
                "observation": observation,
                "duration_ms": act_ms
            }).to_string());

            let should_continue = self.reflect(&observation, &mut state).await;

            let step = AgentStep {
                step_number,
                action,
                observation: observation.clone(),
                cost_usd: self.config.step_cost_usd,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            state.record_step(step);
            state.record_step_timing(step_number, plan_ms as u64, act_ms as u64);

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

        self.emit_stream(serde_json::json!({
            "type": "complete",
            "task_id": task_id,
            "success": state.is_complete,
            "steps": state.current_step,
            "cost": state.total_cost_usd,
            "duration_ms": elapsed.as_millis()
        }).to_string());

        state.get_result()
    }

    pub fn emit_stream(&self, event: String) {
        if self.config.enable_streaming {
            if let Some(ref callback) = self.stream_callback {
                callback(event.clone());
            }
        }
        
        if let Some(ref stream) = self.execution_stream {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&event) {
                let step = json.get("step").and_then(|s| s.as_u64()).unwrap_or(0) as u32;
                let event_type = json.get("type").and_then(|t| t.as_str()).unwrap_or("");
                
                match event_type {
                    "thought" => {
                        let content = json.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string();
                        let _ = stream.try_emit_thought(step, content);
                    },
                    "tool_call" => {
                        let tool = json.get("tool").and_then(|t| t.as_str()).unwrap_or("").to_string();
                        let input = json.get("input").cloned().unwrap_or(serde_json::Value::Null);
                        let _ = stream.try_emit_tool_call(step, tool.clone(), input.clone());
                        
                        if SENSITIVE_TOOLS.iter().any(|&s| tool.to_lowercase().contains(s)) {
                            let action = format!("{}: {:?}", tool, input);
                            let tier = if tool.to_lowercase().contains("shell") || tool.to_lowercase().contains("exec") {
                                "T3".to_string()
                            } else {
                                "T2".to_string()
                            };
                            
                            let consequences = if let (Some(url), _) = (&self.config.llama_url, &self.config.llama_model) {
                                let url = url.clone();
                                let action_for_spawn = action.clone();
                                tokio::spawn(async move {
                                    predict_consequences(&action_for_spawn, &input, &url).await
                                });
                                ConsequencePreview::default()
                            } else {
                                ConsequencePreview {
                                    files_read: vec![],
                                    files_written: if tool.to_lowercase().contains("write") {
                                        vec!["<input files>".to_string()]
                                    } else {
                                        vec![]
                                    },
                                    commands_executed: if tool.to_lowercase().contains("shell") || tool.to_lowercase().contains("exec") {
                                        vec!["shell command".to_string()]
                                    } else {
                                        vec![]
                                    },
                                    blast_radius: crate::execution_stream::BlastRadius::Limited,
                                    summary: format!("This {} action may have limited impact", tier),
                                }
                            };
                            
                            let _ = stream.try_emit_approval(step, tier, action, consequences);
                        }
                    },
                    "tool_result" => {
                        let tool = json.get("tool").and_then(|t| t.as_str()).unwrap_or("").to_string();
                        let success = json.get("success").and_then(|s| s.as_bool()).unwrap_or(true);
                        let output = json.get("observation").or(json.get("output")).and_then(|o| o.as_str()).unwrap_or("").to_string();
                        let _ = stream.try_emit_tool_result(step, tool, success, output);
                    },
                    "error" => {
                        let message = json.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string();
                        let _ = stream.try_emit_error(step, message);
                    },
                    "complete" => {
                        let output = json.get("output").and_then(|o| o.as_str()).unwrap_or("").to_string();
                        let steps = json.get("steps").and_then(|s| s.as_u64()).unwrap_or(0) as u32;
                        let tools_used = json.get("tools_used").and_then(|t| t.as_array()).map(|arr| {
                            arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
                        }).unwrap_or_default();
                        let _ = stream.try_emit_complete(output, steps, tools_used);
                    },
                    _ => {
                        let _ = stream.try_emit(ExecutionEvent::Thought { step, content: event });
                    }
                };
            }
        }
    }
    
    pub fn emit_approval_needed(&self, step: u32, tier: String, action: String, consequences: crate::execution_stream::ConsequencePreview) {
        if let Some(ref stream) = self.execution_stream {
            stream.try_emit_approval(step, tier, action, consequences);
        }
    }

    async fn plan(&self, state: &AgentState) -> String {
        let context = state
            .history
            .iter()
            .map(|s| format!("Step {}: {} -> {}", s.step_number, s.action, s.observation))
            .collect::<Vec<_>>()
            .join("\n");

        let mut available_tools = self.config.tools.clone();
        
        if self.config.enable_tool_generation {
            if let Some(ref registry) = self.config.tool_registry {
                let dynamic_tools = registry.list().await;
                for tool in dynamic_tools {
                    available_tools.push(tool.name);
                }
            }
        }

        let prompt = if self.config.use_tir {
            self.emit_stream(serde_json::json!({
                "type": "mode",
                "mode": "tir"
            }).to_string());
            
            format!(
                r#"You are using Tool-Integrated Reasoning (TIR). Interleave your thinking with actions.

Available tools: {}
Format your response as JSON:
[
  {{"type": "Thought", "content": "your reasoning"}},
  {{"type": "Action", "tool": "tool_name", "input": {{"key": "value"}}}},
  {{"type": "Observation", "content": "result of action"}}
]

Goal: {}

History:
{}
"#,
                available_tools.join(", "),
                state.goal,
                context
            )
        } else {
            format!(
                "Goal: {}\n\nAvailable tools: {}\n\nHistory:\n{}\n\nWhat is the next action to take? Respond with a single action.",
                state.goal,
                available_tools.join(", "),
                context
            )
        };

        if self.config.use_llm {
            tracing::info!("LLM enabled, attempting to call llama-server");
            if let (Some(url), Some(model)) = (&self.config.llama_url, &self.config.llama_model) {
                tracing::info!(url = %url, model = %model, "Connecting to LLM");
                
                if self.config.enable_tool_generation && self.should_generate_tool(&state.goal) {
                    if let Some(ref registry) = self.config.tool_registry {
                        match crate::dynamic_tools::generate_tool(&state.goal, &context, url, model).await {
                            Ok(tool) => {
                                tracing::info!(tool = %tool.name, "Generated new dynamic tool");
                                let tool_name = tool.name.clone();
                                let tool_desc = tool.description.clone();
                                registry.register(tool).await;
                                available_tools.push(tool_name.clone());
                                self.emit_stream(serde_json::json!({
                                    "type": "tool_generated",
                                    "tool": tool_name,
                                    "description": tool_desc
                                }).to_string());
                            }
                            Err(e) => {
                                tracing::warn!(error = %e, "Failed to generate tool");
                            }
                        }
                    }
                }

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
                        
                        if self.config.use_tir {
                            if let Some(tir_result) = self.parse_tir_response(&response) {
                                for step in &tir_result.steps {
                                    self.emit_stream(serde_json::json!({
                                        "type": step.step_type,
                                        "content": step.content,
                                        "tool": step.tool.as_deref(),
                                        "input": step.input.as_ref()
                                    }).to_string());
                                }
                                return tir_result.final_action;
                            }
                        }
                        
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

        state.total_cost_usd = 4.9;

        assert!(state.can_continue(&config, start_time));

        state.total_cost_usd = 5.1;

        assert!(!state.can_continue(&config, start_time));
        assert!(state.error.is_some());
    }
}
