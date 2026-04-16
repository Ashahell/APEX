//! Runbook Executor - Executes runbook steps

use super::models::*;
use anyhow::{anyhow, Result};
use std::time::Instant;

/// RunbookExecutor - executes runbook steps sequentially
pub struct RunbookExecutor {
    // Add HTTP client for webhooks, task client for task operations
}

impl RunbookExecutor {
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a runbook and return the execution record
    pub async fn execute(&self, runbook: &Runbook, alert_id: Option<String>) -> Result<RunbookExecution> {
        let mut execution = RunbookExecution::new(
            runbook.id.clone(),
            runbook.name.clone(),
            alert_id,
        );
        
        execution.mark_running();
        
        tracing::info!("Starting runbook execution: {} ({})", runbook.name, execution.id);
        
        for (idx, step) in runbook.steps.iter().enumerate() {
            tracing::debug!("Executing step {}: {:?}", idx, step);
            
            let start = Instant::now();
            
            match self.execute_step(step).await {
                Ok(result) => {
                    let duration = start.elapsed().as_millis() as u64;
                    execution.add_step_result(StepResult::success(
                        idx as u32,
                        &step.type_name(),
                        result,
                        duration,
                    ));
                    
                    tracing::info!("Step {} completed successfully", idx);
                }
                Err(e) => {
                    let duration = start.elapsed().as_millis() as u64;
                    execution.add_step_result(StepResult::failure(
                        idx as u32,
                        &step.type_name(),
                        e.to_string(),
                        duration,
                    ));
                    
                    // Log but continue - some steps may be non-critical
                    tracing::warn!("Step {} failed: {}", idx, e);
                    
                    // If step is critical, fail the runbook
                    if step.is_critical() {
                        execution.mark_failed(format!("Critical step {} failed: {}", idx, e));
                        return Ok(execution);
                    }
                }
            }
        }
        
        execution.mark_completed();
        tracing::info!("Runbook {} completed in {}ms", runbook.name, execution.duration_ms());
        
        Ok(execution)
    }

    /// Execute a single step
    async fn execute_step(&self, step: &RunbookStep) -> Result<String> {
        match step {
            RunbookStep::CancelTask { task_id } => self.cancel_task(task_id).await,
            RunbookStep::CreateTask { input, priority, project } => {
                self.create_task(input, priority.as_deref(), project.as_deref()).await
            }
            RunbookStep::Notify { message } => self.notify(message).await,
            RunbookStep::ExecuteCommand { command } => self.execute_command(command).await,
            RunbookStep::Delay { ms } => self.delay(*ms).await,
            RunbookStep::Webhook { url, method, body, headers } => {
                self.webhook(url, method.as_deref(), body.as_deref(), headers.as_ref()).await
            }
            RunbookStep::RestartTask { task_id } => self.restart_task(task_id).await,
        }
    }

    async fn cancel_task(&self, task_id: &str) -> Result<String> {
        tracing::info!("Cancelling task: {}", task_id);
        // In a full implementation, this would call the task API
        Ok(format!("Task {} cancelled", task_id))
    }

    async fn create_task(&self, input: &str, priority: Option<&str>, _project: Option<&str>) -> Result<String> {
        tracing::info!("Creating task with input: {} (priority: {:?})", input, priority);
        // In a full implementation, this would call the task creation API
        let task_id = ulid::Ulid::new().to_string();
        Ok(format!("Task {} created", task_id))
    }

    async fn notify(&self, message: &str) -> Result<String> {
        tracing::info!("Notification: {}", message);
        // In a full implementation, this would send to notification system
        Ok(format!("Notified: {}", message))
    }

    async fn execute_command(&self, command: &str) -> Result<String> {
        tracing::info!("Executing command: {}", command);
        
        // For safety, only allow certain commands in production
        // This is a simplified implementation
        let output = std::process::Command::new("sh")
            .args(["-c", command])
            .output()
            .map_err(|e| anyhow!("Command execution failed: {}", e))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!("Command failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    async fn delay(&self, ms: u64) -> Result<String> {
        tracing::debug!("Delaying for {}ms", ms);
        tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
        Ok(format!("Delayed {}ms", ms))
    }

    async fn webhook(
        &self,
        url: &str,
        method: Option<&str>,
        body: Option<&str>,
        _headers: Option<&std::collections::HashMap<String, String>>,
    ) -> Result<String> {
        tracing::info!("Calling webhook: {} ({})", url, method.unwrap_or("GET"));
        
        let client = reqwest::Client::new();
        let method = method.unwrap_or("GET");
        
        let request = match method {
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "PATCH" => client.patch(url),
            "DELETE" => client.delete(url),
            _ => client.get(url),
        };
        
        let request = if let Some(body) = body {
            request.body(body.to_string())
        } else {
            request
        };
        
        let response = request
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow!("Webhook request failed: {}", e))?;
        
        if response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            Ok(format!("Webhook succeeded: {}", body))
        } else {
            Err(anyhow!("Webhook failed with status: {}", response.status()))
        }
    }

    async fn restart_task(&self, task_id: &str) -> Result<String> {
        tracing::info!("Restarting task: {}", task_id);
        // Cancel then create new task
        self.cancel_task(task_id).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        self.create_task(&format!("Restarted task: {}", task_id), Some("high"), None).await
    }
}

impl Default for RunbookExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Helper trait for step introspection
trait StepType {
    fn type_name(&self) -> String;
    fn is_critical(&self) -> bool;
}

impl StepType for RunbookStep {
    fn type_name(&self) -> String {
        match self {
            RunbookStep::CancelTask { .. } => "cancel_task",
            RunbookStep::CreateTask { .. } => "create_task",
            RunbookStep::Notify { .. } => "notify",
            RunbookStep::ExecuteCommand { .. } => "execute_command",
            RunbookStep::Delay { .. } => "delay",
            RunbookStep::Webhook { .. } => "webhook",
            RunbookStep::RestartTask { .. } => "restart_task",
        }.to_string()
    }
    
    fn is_critical(&self) -> bool {
        matches!(self, RunbookStep::CancelTask { .. } | RunbookStep::RestartTask { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_delay_step() {
        let executor = RunbookExecutor::new();
        let result = executor.delay(100).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("100ms"));
    }

    #[tokio::test]
    async fn test_execute_simple_runbook() {
        let executor = RunbookExecutor::new();
        let runbook = Runbook::new(
            "test".to_string(),
            "Test Runbook".to_string(),
            vec![
                RunbookStep::Delay { ms: 10 },
                RunbookStep::Notify { message: "Test".to_string() },
            ],
        );
        
        let execution = executor.execute(&runbook, None).await.unwrap();
        assert_eq!(execution.status, ExecutionStatus::Completed);
        assert_eq!(execution.step_results.len(), 2);
    }
}
