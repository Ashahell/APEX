use apex_memory::task_repo::TaskRepository;
use apex_memory::tasks::TaskStatus;
use apex_memory::decision_journal::{DecisionJournalRepository, CreateDecisionEntry};
use tokio::sync::broadcast;

use crate::agent_loop::{AgentConfig, AgentLoop};
use crate::circuit_breaker::CircuitBreakerRegistry;
use crate::execution_stream::{ExecutionEvent, ExecutionStreamManager};
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
}

impl DeepTaskWorker {
    pub fn new(
        pool: sqlx::Pool<sqlx::Sqlite>,
        message_bus: MessageBus,
        vm_pool: VmPool,
        circuit_breakers: CircuitBreakerRegistry,
        execution_streams: ExecutionStreamManager,
        ws_manager: WebSocketManager,
    ) -> Self {
        Self {
            pool,
            message_bus,
            vm_pool,
            circuit_breakers,
            execution_streams,
            ws_manager,
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

    async fn process_deep_task(&self, message: DeepTaskMessage) {
        let start_time = std::time::Instant::now();
        tracing::info!(
            task_id = %message.task_id,
            max_steps = message.max_steps,
            "Processing deep task"
        );

        let repo = TaskRepository::new(&self.pool);

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
        let result = self.execute_in_vm(&message, &vm_id).await;
        tracing::info!(task_id = %message.task_id, execute_ms = execute_start.elapsed().as_millis(), "Execution complete");

        if let Err(release_err) = self.vm_pool.release(&vm_id).await {
            tracing::warn!(vm_id = %vm_id, error = %release_err, "Failed to release VM");
        }

        tracing::info!(task_id = %message.task_id, total_ms = start_time.elapsed().as_millis(), "Deep task finished");

        match result {
            Ok(output) => {
                if let Err(e) = repo
                    .update_completed(
                        &message.task_id,
                        TaskStatus::Completed,
                        Some(output.clone()),
                        Some(10),  // 10 cents
                    )
                    .await
                {
                    tracing::error!(task_id = %message.task_id, error = %e, "Failed to update completed task");
                }

                if let Err(e) = self.write_decision_journal(&message.task_id, &output).await {
                    tracing::warn!(task_id = %message.task_id, error = %e, "Failed to write decision journal");
                }
            }
            Err(error) => {
                self.circuit_breakers.record_failure("deep_execution").await;

                tracing::error!(
                    task_id = %message.task_id,
                    error = %error,
                    "Deep task execution failed"
                );

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
    ) -> Result<String, String> {
        tracing::debug!(vm_id = %vm_id, "Executing deep task in VM");

        let stream = self.execution_streams.create_stream(&message.task_id);
        let task_id = message.task_id.clone();
        let execution_streams = self.execution_streams.clone();
        let ws_manager = self.ws_manager.clone();
        
        tokio::spawn(async move {
            if let Some(mut rx) = execution_streams.subscribe(&task_id) {
                while let Ok(event) = rx.recv().await {
                    if let Ok(json) = serde_json::to_string(&event) {
                        ws_manager.broadcast_task_update(&task_id, &json).await;
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

        let agent = AgentLoop::new(config)
            .with_execution_stream(stream);
        let result = agent
            .run(message.task_id.clone(), message.content.clone())
            .await;

        self.execution_streams.remove_stream(&message.task_id);

        let output = serde_json::json!({
            "vm_id": vm_id,
            "task_id": result.task_id,
            "success": result.success,
            "steps_executed": result.steps_executed,
            "total_cost_usd": result.total_cost_usd,
            "output": result.output,
            "history": result.history,
        });

        Ok(output.to_string())
    }
}
