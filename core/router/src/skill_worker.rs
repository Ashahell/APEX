use apex_memory::task_repo::TaskRepository;
use apex_memory::tasks::TaskStatus;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::circuit_breaker::CircuitBreakerRegistry;
use crate::message_bus::{MessageBus, SkillExecutionMessage};
use crate::skill_pool::SkillPool;

pub struct SkillWorker {
    pool: sqlx::Pool<sqlx::Sqlite>,
    skill_pool: Option<Arc<SkillPool>>,
    message_bus: MessageBus,
    circuit_breakers: CircuitBreakerRegistry,
}

impl SkillWorker {
    pub fn new(
        pool: sqlx::Pool<sqlx::Sqlite>,
        skill_pool: Option<Arc<SkillPool>>,
        message_bus: MessageBus,
        circuit_breakers: CircuitBreakerRegistry,
    ) -> Self {
        Self {
            pool,
            skill_pool,
            message_bus,
            circuit_breakers,
        }
    }

    pub async fn run(self) {
        let mut rx = self.message_bus.subscribe_skills();
        let pool = self.pool.clone();
        let skill_pool = self.skill_pool.clone();
        let circuit_breakers = self.circuit_breakers.clone();

        loop {
            match rx.recv().await {
                Ok(message) => {
                    Self::process_skill_execution(&pool, skill_pool.as_ref(), &circuit_breakers, message).await;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("Skill worker: message bus closed");
                    break;
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("Skill worker: lagged behind, skipping message");
                }
            }
        }
    }

    pub async fn run_supervised(self, worker_name: &str) {
        loop {
            tracing::info!(worker = %worker_name, "Starting supervised skill worker");
            
            let pool = self.pool.clone();
            let skill_pool = self.skill_pool.clone();
            let circuit_breakers = self.circuit_breakers.clone();
            let rx = self.message_bus.subscribe_skills();
            
            let result = Self::run_inner(pool, skill_pool, circuit_breakers, rx).await;
            
            match result {
                Ok(()) => {
                    tracing::info!(worker = %worker_name, "Skill worker exited normally");
                    break;
                }
                Err(e) => {
                    tracing::error!(worker = %worker_name, error = %e, "Skill worker crashed, restarting in 1 second...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn run_inner(
        pool: sqlx::Pool<sqlx::Sqlite>,
        skill_pool: Option<Arc<SkillPool>>,
        circuit_breakers: CircuitBreakerRegistry,
        mut rx: broadcast::Receiver<SkillExecutionMessage>,
    ) -> Result<(), String> {
        loop {
            match rx.recv().await {
                Ok(message) => {
                    Self::process_skill_execution(&pool, skill_pool.as_ref(), &circuit_breakers, message).await;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("Skill worker: message bus closed");
                    return Ok(());
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("Skill worker: lagged behind, skipping message");
                }
            }
        }
    }

    async fn process_skill_execution(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        skill_pool: Option<&Arc<SkillPool>>,
        circuit_breakers: &CircuitBreakerRegistry,
        message: SkillExecutionMessage,
    ) {
        tracing::info!(
            task_id = %message.task_id,
            skill = %message.skill_name,
            "Processing skill execution"
        );

        let repo = TaskRepository::new(pool);

        let result = if let Some(pool) = skill_pool {
            pool.execute(&message.skill_name, message.input.clone()).await
                .map_err(|e| e.to_string())
                .and_then(|resp| {
                    if resp.ok {
                        Ok(resp.output.unwrap_or_default())
                    } else {
                        Err(resp.error.unwrap_or_else(|| "Unknown error".to_string()))
                    }
                })
        } else {
            Self::execute_skill(&message).await
        };

        match result {
            Ok(result) => {
                circuit_breakers.record_success(&message.skill_name).await;

                if let Err(e) = repo
                    .update_completed(
                        &message.task_id,
                        TaskStatus::Completed,
                        Some(result),
                        Some(1),  // 1 cent
                    )
                    .await
                {
                    tracing::error!(task_id = %message.task_id, error = %e, "Failed to update completed task");
                }
            }
            Err(error) => {
                circuit_breakers.record_failure(&message.skill_name).await;

                tracing::error!(
                    task_id = %message.task_id,
                    skill = %message.skill_name,
                    error = %error,
                    "Skill execution failed"
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

    async fn execute_skill(message: &SkillExecutionMessage) -> Result<String, String> {
        let skill_name = &message.skill_name;
        let input = &message.input;

        Self::execute_typescript_skill(skill_name, input, &message.task_id).await
    }

    async fn execute_typescript_skill(
        skill_name: &str,
        input: &Value,
        task_id: &str,
    ) -> Result<String, String> {
        let input_json = serde_json::to_string(input).map_err(|e| e.to_string())?;

        // Use environment variable or fall back to discovery
        let cli_path = std::env::var("APEX_SKILLS_CLI")
            .ok()
            .map(std::path::PathBuf::from)
            .or_else(Self::find_cli_path);

        let cli_path = cli_path.ok_or_else(|| "Could not find skills CLI".to_string())?;

        let skills_dir = std::env::var("APEX_SKILLS_DIR")
            .unwrap_or_else(|_| "E:\\projects\\APEX\\skills".to_string());

        let output = tokio::process::Command::new("pnpm")
            .arg("tsx")
            .arg(&cli_path)
            .arg("--skill")
            .arg(skill_name)
            .arg("--input")
            .arg(&input_json)
            .arg("--task-id")
            .arg(task_id)
            .current_dir(&skills_dir)
            .output()
            .await
            .map_err(|e| format!("Failed to execute skill: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let result: serde_json::Value = serde_json::from_str(&stdout)
                .map_err(|e| format!("Invalid JSON from skill: {}", e))?;

            if result
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                Ok(result
                    .get("output")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string())
            } else {
                Err(result
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string())
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(format!("Skill execution failed: {}", stderr))
        }
    }

    fn find_cli_path() -> Option<std::path::PathBuf> {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| {
                let candidates = vec![
                    p.join("..").join("skills").join("src").join("cli.ts"),
                    p.join("..")
                        .join("..")
                        .join("skills")
                        .join("src")
                        .join("cli.ts"),
                ];
                for c in &candidates {
                    if c.exists() {
                        return c.clone();
                    }
                }
                candidates.first().cloned().unwrap_or_default()
            })
            .filter(|p| !p.as_os_str().is_empty())
    }
}
