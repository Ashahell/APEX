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
    ) -> Self {
        Self {
            pool,
            message_bus,
            vm_pool,
            circuit_breakers,
            execution_streams,
            ws_manager,
            narrative_memory,
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
            ..Default::default()
        };

        tracing::info!(use_llm = config.use_llm, llama_url = ?config.llama_url, "Agent config");

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
}
