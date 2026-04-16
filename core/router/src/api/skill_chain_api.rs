//! Skill Chain API Endpoints
//! 
//! v1.10.0: Skill Chaining API

use tokio::sync::RwLock;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use chrono::Utc;

use crate::api::api_error::ApiError;
use crate::api::AppState;
use crate::skill_chain::{
    SkillChain, ChainExecution, ChainExecutionStatus, ChainStatus,
    CreateSkillChainRequest, UpdateSkillChainRequest,
    ExecuteChainRequest, ExecuteChainResponse, ChainStep,
};

/// Shared skill chain state
pub struct SkillChainState {
    pub chains: RwLock<Vec<SkillChain>>,
    pub executions: RwLock<Vec<ChainExecution>>,
}

impl SkillChainState {
    pub fn new() -> Self {
        Self {
            chains: RwLock::new(Vec::new()),
            executions: RwLock::new(Vec::new()),
        }
    }

    pub async fn list_chains(&self) -> Vec<SkillChain> {
        self.chains.read().await.clone()
    }

    pub async fn get_chain(&self, id: &str) -> Option<SkillChain> {
        self.chains.read().await.iter()
            .find(|c| c.id == id)
            .cloned()
    }

    pub async fn create_chain(&self, mut chain: SkillChain) -> SkillChain {
        chain.id = ulid::Ulid::new().to_string();
        chain.created_at = Utc::now();
        chain.updated_at = Utc::now();
        self.chains.write().await.push(chain.clone());
        chain
    }

    pub async fn update_chain(&self, id: &str, update: UpdateSkillChainRequest) -> Option<SkillChain> {
        let mut chains = self.chains.write().await;
        if let Some(chain) = chains.iter_mut().find(|c| c.id == id) {
            if let Some(name) = update.name {
                chain.name = name;
            }
            if let Some(desc) = update.description {
                chain.description = Some(desc);
            }
            if let Some(steps) = update.steps {
                chain.steps = steps.into_iter().map(|s| {
                    let mut step = ChainStep::new(s.skill_name);
                    step.input_template = s.input_template.unwrap_or_default();
                    step.output_variable = s.output_variable;
                    step.conditions = s.conditions.unwrap_or_default();
                    step.on_success = s.on_success.unwrap_or(crate::skill_chain::NextStep::Next);
                    step.on_failure = s.on_failure.unwrap_or(crate::skill_chain::NextStep::End);
                    step.timeout_secs = s.timeout_secs.unwrap_or(300);
                    step.retry_on_failure = s.retry_on_failure.unwrap_or(true);
                    step
                }).collect();
            }
            if let Some(vars) = update.variables {
                chain.variables = vars;
            }
            if let Some(enabled) = update.enabled {
                chain.enabled = enabled;
            }
            chain.updated_at = Utc::now();
            return Some(chain.clone());
        }
        None
    }

    pub async fn delete_chain(&self, id: &str) -> bool {
        let mut chains = self.chains.write().await;
        let pos = chains.iter().position(|c| c.id == id);
        if let Some(pos) = pos {
            chains.remove(pos);
            true
        } else {
            false
        }
    }

    pub async fn execute_chain(&self, chain_id: &str, input_vars: Option<std::collections::HashMap<String, String>>, start_at: Option<usize>) -> Option<ChainExecution> {
        let chain = self.get_chain(chain_id).await?;
        if !chain.enabled {
            return None;
        }

        let mut execution = ChainExecution::new(chain_id.to_string());
        
        // Initialize variables
        execution.variables = chain.variables.clone();
        if let Some(vars) = input_vars {
            for (k, v) in vars {
                execution.variables.insert(k, v);
            }
        }

        execution.start();

        // Execute steps
        let start_step = start_at.unwrap_or(0);
        for i in start_step..chain.steps.len() {
            let step = &chain.steps[i];
            
            // Check conditions
            let conditions_met = step.conditions.is_empty() || 
                step.conditions.iter().all(|c| c.evaluate(&execution.variables));

            if !conditions_met {
                continue;
            }

            // Record step execution (in real impl, would call the skill)
            let result = crate::skill_chain::StepResult {
                step_id: step.id.clone(),
                skill_name: step.skill_name.clone(),
                success: true,
                output: format!("Executed {} successfully", step.skill_name),
                error: None,
                duration_ms: 100,
                output_variable: step.output_variable.clone().unwrap_or_else(|| format!("output_{}", i)),
            };

            let step_success = result.success;
            execution.record_step(result);
            execution.current_step = i + 1;

            if !step_success {
                match &step.on_failure {
                    crate::skill_chain::NextStep::End => break,
                    crate::skill_chain::NextStep::JumpTo(target_id) => {
                        if let Some(idx) = chain.steps.iter().position(|s| s.id == *target_id) {
                            execution.current_step = idx;
                        }
                    }
                    crate::skill_chain::NextStep::Next => continue,
                }
            }
        }

        execution.complete();
        self.executions.write().await.push(execution.clone());
        Some(execution)
    }

    pub async fn get_execution(&self, id: &str) -> Option<ChainExecution> {
        self.executions.read().await.iter()
            .find(|e| e.id == id)
            .cloned()
    }

    pub async fn list_executions(&self, chain_id: Option<&str>) -> Vec<ChainExecution> {
        let executions = self.executions.read().await;
        match chain_id {
            Some(id) => executions.iter().filter(|e| e.chain_id == id).cloned().collect(),
            None => executions.clone(),
        }
    }

    pub async fn cancel_execution(&self, id: &str) -> Option<ChainExecution> {
        let mut executions = self.executions.write().await;
        if let Some(exec) = executions.iter_mut().find(|e| e.id == id) {
            exec.cancel();
            return Some(exec.clone());
        }
        None
    }
}

impl Default for SkillChainState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the skill chain router
pub fn create_skill_chain_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/skill-chains", get(list_chains))
        .route("/api/v1/skill-chains/:id", get(get_chain))
        .route("/api/v1/skill-chains", post(create_chain))
        .route("/api/v1/skill-chains/:id", put(update_chain))
        .route("/api/v1/skill-chains/:id", delete(delete_chain))
        .route("/api/v1/skill-chains/:id/execute", post(execute_chain))
        .route("/api/v1/skill-chain-executions", get(list_executions))
        .route("/api/v1/skill-chain-executions/:id", get(get_execution))
        .route("/api/v1/skill-chain-executions/:id/cancel", post(cancel_execution))
}

/// GET /api/v1/skill-chains - List all chains
async fn list_chains(
    State(state): State<AppState>,
) -> Result<Json<Vec<SkillChain>>, (axum::http::StatusCode, String)> {
    let chains = state.skill_chain_state.list_chains().await;
    Ok(Json(chains))
}

/// GET /api/v1/skill-chains/:id - Get a specific chain
async fn get_chain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SkillChain>, (axum::http::StatusCode, String)> {
    state.skill_chain_state.get_chain(&id).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Chain {} not found", id)))
}

/// POST /api/v1/skill-chains - Create a new chain
async fn create_chain(
    State(state): State<AppState>,
    Json(req): Json<CreateSkillChainRequest>,
) -> Result<Json<SkillChain>, (axum::http::StatusCode, String)> {
    let mut chain = SkillChain::new(req.name);
    chain.description = req.description;
    if let Some(vars) = req.variables {
        chain.variables = vars;
    }

    let created = state.skill_chain_state.create_chain(chain).await;
    Ok(Json(created))
}

/// PUT /api/v1/skill-chains/:id - Update a chain
async fn update_chain(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSkillChainRequest>,
) -> Result<Json<SkillChain>, (axum::http::StatusCode, String)> {
    state.skill_chain_state.update_chain(&id, req).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Chain {} not found", id)))
}

/// DELETE /api/v1/skill-chains/:id - Delete a chain
async fn delete_chain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    if state.skill_chain_state.delete_chain(&id).await {
        Ok(Json(serde_json::json!({ "success": true })))
    } else {
        Err(ApiError::not_found(format!("Chain {} not found", id)))
    }
}

/// POST /api/v1/skill-chains/:id/execute - Execute a chain
async fn execute_chain(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ExecuteChainRequest>,
) -> Result<Json<ExecuteChainResponse>, (axum::http::StatusCode, String)> {
    match state.skill_chain_state.execute_chain(&id, req.input_variables, req.start_at_step).await {
        Some(execution) => Ok(Json(ExecuteChainResponse {
            execution_id: execution.id,
            status: execution.status,
            message: "Chain execution started".to_string(),
        })),
        None => Ok(Json(ExecuteChainResponse {
            execution_id: String::new(),
            status: ChainStatus::Failed,
            message: "Failed to start chain execution".to_string(),
        })),
    }
}

/// GET /api/v1/skill-chain-executions - List executions
#[derive(Debug, serde::Deserialize)]
pub struct ListExecutionsQuery {
    pub chain_id: Option<String>,
}

async fn list_executions(
    State(state): State<AppState>,
    Query(query): Query<ListExecutionsQuery>,
) -> Result<Json<Vec<ChainExecution>>, (axum::http::StatusCode, String)> {
    let executions = state.skill_chain_state.list_executions(query.chain_id.as_deref()).await;
    Ok(Json(executions))
}

/// GET /api/v1/skill-chain-executions/:id - Get execution status
async fn get_execution(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ChainExecution>, (axum::http::StatusCode, String)> {
    state.skill_chain_state.get_execution(&id).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Execution {} not found", id)))
}

/// POST /api/v1/skill-chain-executions/:id/cancel - Cancel execution
async fn cancel_execution(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ChainExecution>, (axum::http::StatusCode, String)> {
    state.skill_chain_state.cancel_execution(&id).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Execution {} not found", id)))
}
