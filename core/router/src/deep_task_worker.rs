use apex_memory::task_repo::TaskRepository;
use apex_memory::tasks::TaskStatus;
use apex_memory::decision_journal::{DecisionJournalRepository, CreateDecisionEntry};
use apex_memory::working_memory::WorkingMemory;
use apex_memory::narrative::NarrativeMemory;
use tokio::sync::broadcast;

use crate::agent_loop::{AgentConfig, AgentLoop};
use crate::circuit_breaker::CircuitBreakerRegistry;
use crate::execution_stream::ExecutionStreamManager;
use crate::message_bus::{DeepTaskMessage, MessageBus};
use crate::skill_manager::SkillManager;
use crate::unified_config::skill_constants::*;
use crate::vm_pool::VmPool;
use crate::websocket::WebSocketManager;

pub struct DeepTaskWorker {
    pool: sqlx::Pool<sqlx::Sqlite>,
    message_bus: MessageBus,
    vm_pool: VmPool,
    circuit_breakers: CircuitBreakerRegistry,
    execution_streams: ExecutionStreamManager,
    ws_manager: WebSocketManager,
    narrative_memory: std::sync::Arc<NarrativeMemory>,
    skill_manager: std::sync::Arc<tokio::sync::Mutex<SkillManager>>,
}

impl DeepTaskWorker {
    pub fn new(
        pool: sqlx::Pool<sqlx::Sqlite>,
        message_bus: MessageBus,
        vm_pool: VmPool,
        circuit_breakers: CircuitBreakerRegistry,
        execution_streams: ExecutionStreamManager,
        ws_manager: WebSocketManager,
        narrative_memory: std::sync::Arc<NarrativeMemory>,
        skill_manager: std::sync::Arc<tokio::sync::Mutex<SkillManager>>,
    ) -> Self {
        Self {
            pool,
            message_bus,
            vm_pool,
            circuit_breakers,
            execution_streams,
            ws_manager,
            narrative_memory,
            skill_manager,
        }
    }

    pub async fn run(self) {
        let mut rx = self.message_bus.subscribe_deep_tasks();

        loop {
            match rx.recv().await {
                Ok(message) => {
                    self.process_deep_task(message).await;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("Deep task worker: message bus closed");
                    break;
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("Deep task worker: lagged behind, skipping message");
                }
            }
        }
    }

    pub async fn run_supervised(mut self, worker_name: &str) {
        loop {
            tracing::info!(worker = %worker_name, "Starting supervised deep task worker");
            
            let result = self.run_inner().await;
            
            match result {
                Ok(()) => {
                    tracing::info!(worker = %worker_name, "Deep task worker exited normally");
                    break;
                }
                Err(e) => {
                    tracing::error!(worker = %worker_name, error = %e, "Deep task worker crashed, restarting in 1 second...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn run_inner(&mut self) -> Result<(), String> {
        let mut rx = self.message_bus.subscribe_deep_tasks();
        
        loop {
            match rx.recv().await {
                Ok(message) => {
                    self.process_deep_task(message).await;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("Deep task worker: message bus closed");
                    return Ok(());
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("Deep task worker: lagged behind, skipping message");
                }
            }
        }
    }

    async fn process_deep_task(&self, message: DeepTaskMessage) {
        let start_time = std::time::Instant::now();
        tracing::info!(
            task_id = %message.task_id,
            max_steps = message.max_steps,
            "Processing deep task"
        );

        let repo = TaskRepository::new(&self.pool);

        // Create or restore working memory for this task
        let working_memory = match WorkingMemory::new(&message.task_id, self.pool.clone()).await {
            Ok(wm) => {
                tracing::debug!(task_id = %message.task_id, "Working memory initialized");
                Some(wm)
            }
            Err(e) => {
                tracing::warn!(task_id = %message.task_id, error = %e, "Failed to create working memory, continuing without it");
                None
            }
        };

        let vm_acquire_start = std::time::Instant::now();
        let vm_id = match self.vm_pool.acquire().await {
            Ok(id) => id,
            Err(e) => {
                tracing::error!(task_id = %message.task_id, error = %e, "Failed to acquire VM");
                let _ = repo
                    .update_failed(&message.task_id, &format!("Failed to acquire VM: {}", e))
                    .await;
                return;
            }
        };
        tracing::info!(task_id = %message.task_id, vm_acquire_ms = vm_acquire_start.elapsed().as_millis(), "VM acquired");

        tracing::debug!(vm_id = %vm_id, "Acquired VM for deep task");

        let execute_start = std::time::Instant::now();
        let result = self.execute_in_vm(&message, &vm_id, working_memory).await;
        let execute_ms = execute_start.elapsed().as_millis();
        tracing::info!(task_id = %message.task_id, execute_ms = execute_ms, "Execution complete");

        let vm_acquire_ms = vm_acquire_start.elapsed().as_millis() as u64;
        let total_ms = start_time.elapsed().as_millis() as u64;

        if let Err(release_err) = self.vm_pool.release(&vm_id).await {
            tracing::warn!(vm_id = %vm_id, error = %release_err, "Failed to release VM");
        }

        tracing::info!(task_id = %message.task_id, total_ms = total_ms, "Deep task finished");

        match result {
            Ok(output_json) => {
                let output_val: serde_json::Value = serde_json::from_str(&output_json).unwrap_or_default();
                let agent_timing = output_val.get("timing").cloned().unwrap_or_default();
                
                let final_output = serde_json::json!({
                    "vm_id": vm_id,
                    "task_id": message.task_id,
                    "success": output_val.get("success").unwrap_or(&serde_json::Value::Bool(false)).clone(),
                    "steps_executed": output_val.get("steps_executed").unwrap_or(&serde_json::Value::Number(0.into())).clone(),
                    "total_cost_usd": output_val.get("total_cost_usd").unwrap_or(&serde_json::Value::Number(0.into())).clone(),
                    "output": output_val.get("output").unwrap_or(&serde_json::Value::Null).clone(),
                    "history": output_val.get("history").unwrap_or(&serde_json::Value::Array(vec![])).clone(),
                    "timing": {
                        "agent": agent_timing,
                        "system": {
                            "vm_acquire_ms": vm_acquire_ms,
                            "execute_ms": execute_ms,
                            "total_ms": total_ms,
                        }
                    },
                }).to_string();
                
                let journal_entries = Self::extract_journal_entries(&message.task_id, &final_output);
                
                // Write narrative to long-term memory
                let tools_used: Vec<String> = output_val.get("history")
                    .and_then(|h| h.as_array())
                    .map(|arr| arr.iter().filter_map(|s| s.get("action").and_then(|a| a.as_str()).map(String::from)).collect())
                    .unwrap_or_default();
                
                if let Err(e) = self.narrative_memory.narrativize_task(
                    &message.task_id,
                    &message.content,
                    Some(&final_output),
                    "completed",
                    &tools_used,
                    &[],
                ).await {
                    tracing::warn!(task_id = %message.task_id, error = %e, "Failed to write narrative");
                }
                
                // Check if this task should suggest a skill (Hermes-style agent learning)
                self.check_and_suggest_skill(&message.task_id, &message.content, &tools_used, &final_output).await;
                
                let mut tx = match self.pool.begin().await {
                    Ok(tx) => tx,
                    Err(e) => {
                        tracing::error!(task_id = %message.task_id, error = %e, "Failed to begin transaction");
                        return;
                    }
                };
                
                let now = chrono::Utc::now();
                if let Err(e) = sqlx::query(
                    "UPDATE tasks SET status = ?, output_content = ?, actual_cost_cents = ?, completed_at = ?, updated_at = ? WHERE id = ?"
                )
                .bind(TaskStatus::Completed.as_str())
                .bind(Some(&final_output))
                .bind(Some(10))
                .bind(now)
                .bind(now)
                .bind(&message.task_id)
                .execute(&mut *tx).await {
                    tracing::error!(task_id = %message.task_id, error = %e, "Failed to update task in transaction");
                    tx.rollback().await.ok();
                    return;
                }
                
                for (entry_id, title, decision, context) in journal_entries {
                    if let Err(e) = sqlx::query(
                        "INSERT INTO decision_journal (id, task_id, title, context, decision) VALUES (?, ?, ?, ?, ?)"
                    )
                    .bind(&entry_id)
                    .bind(&message.task_id)
                    .bind(&title)
                    .bind(&context)
                    .bind(&decision)
                    .execute(&mut *tx).await {
                        tracing::error!(error = %e, "Failed to insert journal entry in transaction");
                        tx.rollback().await.ok();
                        return;
                    }
                }
                
                if let Err(e) = tx.commit().await {
                    tracing::error!(task_id = %message.task_id, error = %e, "Failed to commit transaction");
                }
            }
            Err(error) => {
                self.circuit_breakers.record_failure("deep_execution").await;

                tracing::error!(
                    task_id = %message.task_id,
                    error = %error,
                    "Deep task execution failed"
                );

                // Write failure narrative
                if let Err(e) = self.narrative_memory.narrativize_task(
                    &message.task_id,
                    &message.content,
                    Some(&error.to_string()),
                    "failed",
                    &[],
                    &[error.to_string()],
                ).await {
                    tracing::warn!(task_id = %message.task_id, error = %e, "Failed to write narrative");
                }

                if let Err(e) = repo
                    .update_failed(&message.task_id, &error.to_string())
                    .await
                {
                    tracing::error!(task_id = %message.task_id, error = %e, "Failed to update failed task");
                }
            }
        }
    }

    async fn write_decision_journal(&self, task_id: &str, output_json: &str) -> Result<(), String> {
        let output: serde_json::Value = serde_json::from_str(output_json)
            .map_err(|e| format!("Failed to parse output: {}", e))?;

        let history = output.get("history")
            .and_then(|h| h.as_array())
            .ok_or("No history in output")?;

        let journal_repo = DecisionJournalRepository::new(&self.pool);

        for step in history {
            let step_number = step.get("step_number")
                .and_then(|s| s.as_u64())
                .unwrap_or(0) as i64;
            
            let action = step.get("action")
                .and_then(|a| a.as_str())
                .unwrap_or("unknown");
            
            let observation = step.get("observation")
                .and_then(|o| o.as_str())
                .unwrap_or("");
            
            let decision = format!("Step {}: {}", step_number, action);
            let context = format!("Observation: {}", observation);

            let entry = CreateDecisionEntry {
                task_id: Some(task_id.to_string()),
                title: format!("Step {}", step_number),
                decision,
                context: Some(context),
                rationale: None,
                outcome: None,
                tags: Some(vec![format!("step{}", step_number), "agent".to_string()]),
            };

            let entry_id = format!("{}-step-{}", task_id, step_number);
            journal_repo.create(&entry_id, entry).await
                .map_err(|e| format!("Failed to create journal entry: {}", e))?;
        }

        Ok(())
    }

    async fn execute_in_vm(
        &self,
        message: &DeepTaskMessage,
        vm_id: &str,
        mut working_memory: Option<WorkingMemory>,
    ) -> Result<String, String> {
        tracing::debug!(vm_id = %vm_id, "Executing deep task in VM");

        let stream = self.execution_streams.create_stream(&message.task_id);
        let task_id = message.task_id.clone();
        let task_id_for_spawn = task_id.clone();
        let execution_streams = self.execution_streams.clone();
        let ws_manager = self.ws_manager.clone();
        
        tokio::spawn(async move {
            if let Some(mut rx) = execution_streams.subscribe(&task_id_for_spawn) {
                while let Ok(event) = rx.recv().await {
                    if let Ok(json) = serde_json::to_string(&event) {
                        ws_manager.broadcast_task_update(&task_id_for_spawn, &json).await;
                    }
                }
            }
        });
        
        let config = AgentConfig {
            max_steps: message.max_steps,
            max_budget_usd: message.budget_usd,
            step_cost_usd: message.budget_usd / message.max_steps as f64,
            time_limit_secs: message.time_limit_secs,
            use_tir: message.use_tir,
            enable_subagents: message.enable_subagents,
            ..Default::default()
        };

        tracing::info!(use_llm = config.use_llm, llama_url = ?config.llama_url, llama_model = ?config.llama_model, "Agent config");
        
        // Debug: also log what AppConfig::global() returns
        let global_config = crate::unified_config::AppConfig::global();
        tracing::info!(global_use_llm = global_config.agent.use_llm, global_llama_url = %global_config.agent.llama_url, "Global config");

        let mut agent_builder = AgentLoop::new(config)
            .with_execution_stream(stream);
        
        // Pass working memory to agent if available
        if let Some(wm) = working_memory.take() {
            agent_builder = agent_builder.with_working_memory(wm);
        }
        
        let mut agent = agent_builder;
        let result = agent
            .run(message.task_id.clone(), message.content.clone())
            .await;

        self.execution_streams.remove_stream(&message.task_id);

        // Flush working memory if it was used
        if let Some(ref wm) = working_memory {
            if let Err(e) = wm.flush_to_longterm(&format!("{}/working_memory.md", dirs::data_local_dir().unwrap_or_default().join("apex").display())).await {
                tracing::warn!(task_id = %task_id, error = %e, "Failed to flush working memory");
            }
        }

        let output = serde_json::json!({
            "vm_id": vm_id,
            "task_id": result.task_id,
            "success": result.success,
            "steps_executed": result.steps_executed,
            "total_cost_usd": result.total_cost_usd,
            "output": result.output,
            "history": result.history,
            "timing": result.timing_ms,
        });

        Ok(output.to_string())
    }

    fn extract_journal_entries(task_id: &str, output_json: &str) -> Vec<(String, String, String, Option<String>)> {
        let output: serde_json::Value = match serde_json::from_str(output_json) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };

        let history = match output.get("history").and_then(|h| h.as_array()) {
            Some(h) => h,
            None => return Vec::new(),
        };

        history.iter().enumerate().map(|(i, step)| {
            let step_number = step.get("step_number").and_then(|s| s.as_u64()).unwrap_or(i as u64);
            let action = step.get("action").and_then(|a| a.as_str()).unwrap_or("unknown");
            let observation = step.get("observation").and_then(|o| o.as_str()).unwrap_or("");
            
            let entry_id = format!("{}-step-{}", task_id, step_number);
            let title = format!("Step {}", step_number);
            let decision = format!("Step {}: {}", step_number, action);
            let context = Some(format!("Observation: {}", observation));
            
            (entry_id, title, decision, context)
        }).collect()
    }
    
    /// Check if a task is complex enough to suggest a skill and suggest it if appropriate
    async fn check_and_suggest_skill(
        &self,
        task_id: &str,
        task_content: &str,
        tools_used: &[String],
        output: &str,
    ) {
        // Check if task is complex enough (minimum tool calls threshold)
        if tools_used.len() < MIN_TOOL_CALLS_FOR_SKILL as usize {
            tracing::debug!(task_id = %task_id, tools_used = tools_used.len(), "Task too simple for skill suggestion");
            return;
        }
        
        // Generate a skill name from the task content
        let skill_name = self.generate_skill_name(task_content);
        
        // Check if similar skill already exists
        let manager = self.skill_manager.lock().await;
        if manager.skill_exists(&skill_name) {
            tracing::debug!(task_id = %task_id, skill_name = %skill_name, "Similar skill already exists");
            return;
        }
        
        // Generate skill content from the task execution
        let skill_content = self.generate_skill_content(task_content, tools_used, output);
        
        // Log skill suggestion for later review (actual creation would require LLM)
        tracing::info!(
            task_id = %task_id,
            skill_name = %skill_name,
            tools_used = ?tools_used,
            "Task qualifies for skill creation"
        );
        
        // In a full implementation, we would:
        // 1. Call LLM to generate skill content
        // 2. Ask user for confirmation (T1/T2 tier)
        // 3. Create the skill via skill_manager.create_skill()
        
        // For now, we log it - the user can review and create skills via the UI
        let suggestion = serde_json::json!({
            "task_id": task_id,
            "skill_name": skill_name,
            "task_content": task_content,
            "skill_content": skill_content,
            "tools_used": tools_used,
            "suggested_at": chrono::Utc::now().to_rfc3339(),
        });
        
        // Store suggestion for UI review
        let suggestion_path = dirs::data_local_dir()
            .unwrap_or_default()
            .join("apex")
            .join("skill_suggestions");
        
        if let Err(e) = std::fs::create_dir_all(&suggestion_path) {
            tracing::warn!(error = %e, "Failed to create skill suggestions directory");
            return;
        }
        
        let suggestion_file = suggestion_path.join(format!("{}.json", task_id));
        if let Err(e) = std::fs::write(&suggestion_file, suggestion.to_string()) {
            tracing::warn!(error = %e, "Failed to write skill suggestion");
        } else {
            tracing::info!(task_id = %task_id, "Skill suggestion saved for review");
        }
    }
    
    /// Generate a skill name from task content
    fn generate_skill_name(&self, task_content: &str) -> String {
        // Extract key words and create slug
        let words: Vec<&str> = task_content
            .split(|c: char| c.is_whitespace() || c == '-' || c == '_' || c == '.' || c == '/' || c == '\\')
            .filter(|w| w.len() > 3)
            .filter(|w| !Self::is_common_word(w))
            .take(4)
            .collect();
        
        if words.is_empty() {
            format!("task-{}", &task_content[..task_content.len().min(20)].to_lowercase())
        } else {
            words.join("-").to_lowercase()
        }
    }
    
    /// Check if word is a common word that shouldn't be in skill name
    fn is_common_word(word: &str) -> bool {
        let common = ["the", "and", "for", "with", "this", "that", "from", "have", "been", "were", "they", "what", "when", "where", "which", "about", "into", "make", "just", "also", "very", "some", "could", "would", "should"];
        common.contains(&word.to_lowercase().as_str())
    }
    
    /// Generate skill content from task execution
    fn generate_skill_content(&self, task_content: &str, tools_used: &[String], _output: &str) -> String {
        // Extract key actions from the task
        let action_summary = task_content
            .lines()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");
        
        format!(
            "This skill was auto-generated from task execution.\n\n\
            ## Task\n\
            {}\n\n\
            ## Tools Used\n\
            {}\n\n\
            ## Notes\n\
            - Successfully completed\n\
            - Can be reused for similar tasks\n",
            action_summary,
            tools_used.join(", ")
        )
    }
}
