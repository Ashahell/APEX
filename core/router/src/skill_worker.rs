use apex_memory::task_repo::TaskRepository;
use apex_memory::tasks::TaskStatus;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;
use std::time::Instant;
use std::collections::HashMap;

use crate::circuit_breaker::CircuitBreakerRegistry;
use crate::message_bus::{MessageBus, SkillExecutionMessage};
use crate::security::{AnomalyDetector, InjectionClassifier, ThreatLevel};
use crate::skill_pool::SkillPool;
use crate::vm_pool::VmPool;

/// Global anomaly detector instance
static ANOMALY_DETECTOR: std::sync::OnceLock<AnomalyDetector> = std::sync::OnceLock::new();

/// Initialize the global anomaly detector
pub fn init_anomaly_detector() -> &'static AnomalyDetector {
    ANOMALY_DETECTOR.get_or_init(AnomalyDetector::new)
}

/// Get the global anomaly detector
pub fn get_anomaly_detector() -> Option<&'static AnomalyDetector> {
    ANOMALY_DETECTOR.get()
}

/// Load environment variables from workspace .env file
fn load_workspace_env(workspace_path: &std::path::Path) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();
    
    let env_file = workspace_path.join(".env");
    if env_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&env_file) {
            for line in content.lines() {
                let line = line.trim();
                // Skip comments and empty lines
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                
                // Parse KEY=VALUE format
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim().to_string();
                    let value = value.trim().to_string();
                    // Remove surrounding quotes if present
                    let value = if (value.starts_with('"') && value.ends_with('"'))
                        || (value.starts_with('\'') && value.ends_with('\'')) {
                        value[1..value.len()-1].to_string()
                    } else {
                        value
                    };
                    env_vars.insert(key, value);
                }
            }
            tracing::info!("Loaded {} env vars from {:?}", env_vars.len(), env_file);
        }
    }
    
    env_vars
}

/// SkillWorker executes skills in either the Bun pool (T0-T2) or VM pool (T3)
/// 
/// Tier routing:
/// - T0, T1, T2 → SkillPool (Bun workers) - fast, for non-destructive tasks
/// - T3 → VmPool (Firecracker/Linux VM) - isolated, for shell.execute and destructive tasks
pub struct SkillWorker {
    pool: sqlx::Pool<sqlx::Sqlite>,
    skill_pool: Option<Arc<SkillPool>>,
    /// VM pool for T3 tasks - provides kernel-level isolation
    vm_pool: Option<Arc<VmPool>>,
    message_bus: MessageBus,
    circuit_breakers: CircuitBreakerRegistry,
}

impl SkillWorker {
    pub fn new(
        pool: sqlx::Pool<sqlx::Sqlite>,
        skill_pool: Option<Arc<SkillPool>>,
        vm_pool: Option<Arc<VmPool>>,
        message_bus: MessageBus,
        circuit_breakers: CircuitBreakerRegistry,
    ) -> Self {
        Self {
            pool,
            skill_pool,
            vm_pool,
            message_bus,
            circuit_breakers,
        }
    }

    pub async fn run(self) {
        let mut rx = self.message_bus.subscribe_skills();
        let pool = self.pool.clone();
        let skill_pool = self.skill_pool.clone();
        let vm_pool = self.vm_pool.clone();
        let circuit_breakers = self.circuit_breakers.clone();

        loop {
            match rx.recv().await {
                Ok(message) => {
                    Self::process_skill_execution(&pool, skill_pool.as_ref(), vm_pool.as_ref(), &circuit_breakers, message).await;
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
            let vm_pool = self.vm_pool.clone();
            let circuit_breakers = self.circuit_breakers.clone();
            let rx = self.message_bus.subscribe_skills();
            
            let result = Self::run_inner(pool, skill_pool, vm_pool, circuit_breakers, rx).await;
            
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
        vm_pool: Option<Arc<VmPool>>,
        circuit_breakers: CircuitBreakerRegistry,
        mut rx: broadcast::Receiver<SkillExecutionMessage>,
    ) -> Result<(), String> {
        loop {
            match rx.recv().await {
                Ok(message) => {
                    Self::process_skill_execution(&pool, skill_pool.as_ref(), vm_pool.as_ref(), &circuit_breakers, message).await;
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
        vm_pool: Option<&Arc<VmPool>>,
        circuit_breakers: &CircuitBreakerRegistry,
        message: SkillExecutionMessage,
    ) {
        let tier = &message.permission_tier;
        
        tracing::info!(
            task_id = %message.task_id,
            skill = %message.skill_name,
            tier = %tier,
            "Processing skill execution"
        );

        // SECURITY: Run injection detection before execution
        let input_str = serde_json::to_string(&message.input).unwrap_or_default();
        let injection_result = InjectionClassifier::analyze_skill_input(&message.skill_name, &input_str);
        
        if !injection_result.is_safe {
            tracing::warn!(
                task_id = %message.task_id,
                skill = %message.skill_name,
                threat_level = %injection_result.threat_level.as_str(),
                injection_type = ?injection_result.injection_type,
                "Blocked potentially malicious input"
            );
            
            // Block high/critical threats, warn on others
            if injection_result.should_block {
                let repo = TaskRepository::new(pool);
                let error_msg = format!("Security block: {}", injection_result.message);
                if let Err(e) = repo.update_failed(&message.task_id, &error_msg).await {
                    tracing::error!(task_id = %message.task_id, error = %e, "Failed to update failed task");
                }
                return;
            }
        }

        let repo = TaskRepository::new(pool);
        
        // Track execution time for anomaly detection
        let execution_start = Instant::now();

        // Route based on tier:
        // - T0, T1, T2 → SkillPool (Bun workers) - fast execution
        // - T3 → VmPool (Firecracker/Linux VM) - true isolation
        let result = match tier.as_str() {
            "T3" => {
                // T3 tasks get full VM isolation
                if let Some(vm) = vm_pool {
                    tracing::info!(task_id = %message.task_id, "Routing T3 task to VM pool");
                    Self::execute_in_vm(vm, &message).await
                } else {
                    // Fallback to skill pool if VM not available (warn but proceed)
                    tracing::warn!(task_id = %message.task_id, "VM pool unavailable, falling back to skill pool for T3");
                    Self::execute_in_skill_pool(skill_pool, &message).await
                }
            }
            _ => {
                // T0, T1, T2 → Bun pool
                Self::execute_in_skill_pool(skill_pool, &message).await
            }
        };

        // Determine success for anomaly detection before consuming result
        let success = result.is_ok();
        
        // Clone result for anomaly detection before consuming in match
        let result_clone = result.clone();
        
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
        
        // Record execution for anomaly detection
        let duration_ms = execution_start.elapsed().as_millis() as u64;
        let input_size = input_str.len();
        
        // Extract death spiral detection data from result
        let files_created = result_clone.as_ref().ok().and_then(|r| {
            serde_json::from_str::<serde_json::Value>(r)
                .ok()
                .and_then(|v| v.get("files_created").and_then(|f| f.as_u64()).map(|f| f as u32))
        });
        
        let tool_calls: Option<Vec<String>> = result_clone.as_ref().ok().and_then(|r| {
            serde_json::from_str::<serde_json::Value>(r)
                .ok()
                .and_then(|v| v.get("tool_calls").and_then(|t| t.as_array().cloned()))
                .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect::<Vec<_>>())
        });
        
        let had_side_effects = result_clone.as_ref().ok().map(|_| true); // Assume side effects unless explicitly marked otherwise
        
        if let Some(detector) = get_anomaly_detector() {
            if let Some(anomaly) = detector.record_execution(
                &message.skill_name,
                &message.task_id,
                duration_ms,
                success,
                input_size,
                files_created,
                tool_calls,
                had_side_effects,
            ).await {
                tracing::warn!(
                    task_id = %message.task_id,
                    skill = %message.skill_name,
                    anomaly_type = %anomaly.anomaly_type.as_str(),
                    severity = %anomaly.severity.as_str(),
                    "Anomaly detected: {}",
                    anomaly.description
                );
            }
        }
    }

    /// Execute in Bun skill pool (T0, T1, T2)
    async fn execute_in_skill_pool(
        skill_pool: Option<&Arc<SkillPool>>,
        message: &SkillExecutionMessage,
    ) -> Result<String, String> {
        if let Some(pool) = skill_pool {
            pool.execute(&message.skill_name, message.input.clone(), Some(message.permission_tier.clone()))
                .await
                .map_err(|e| e.to_string())
                .and_then(|resp| {
                    if resp.ok {
                        Ok(resp.output.unwrap_or_default())
                    } else {
                        Err(resp.error.unwrap_or_else(|| "Unknown error".to_string()))
                    }
                })
        } else {
            // Fallback to direct execution
            Self::execute_skill(message).await
        }
    }

    /// Execute in VM pool (T3) - provides kernel-level isolation
    async fn execute_in_vm(
        vm_pool: &Arc<VmPool>,
        message: &SkillExecutionMessage,
    ) -> Result<String, String> {
        // Execute T3 task in isolated VM
        let skill_name = &message.skill_name;
        let input = &message.input;
        
        tracing::info!(
            skill = %skill_name,
            tier = "T3",
            task_id = %message.task_id,
            "Executing in VM pool (T3 isolation)"
        );

        // Acquire a VM from the pool
        let vm_id = vm_pool.acquire().await.map_err(|e| format!("Failed to acquire VM: {}", e))?;
        
        // Prepare the skill execution script
        // The script will run the skill with the provided input
        let input_json = serde_json::to_string(input).map_err(|e| format!("Failed to serialize input: {}", e))?;
        
        // Build the execution script
        // This runs the skill with input via bun in the isolated environment
        let script = format!(
            r#"bun run /opt/apex/skills/{}/src/index.ts --input '{}' --task-id '{}'"#,
            skill_name,
            input_json.replace("'", "'\\''"),
            message.task_id
        );

        // Execute in the VM with timeout
        let result = vm_pool.execute_isolated(&vm_id, &script, Some(60)).await;
        
        // Always release the VM back to the pool
        if let Err(e) = vm_pool.release(&vm_id).await {
            tracing::warn!(vm_id = %vm_id, error = %e, "Failed to release VM");
        }

        // Process the result
        match result {
            Ok(exec_result) => {
                if exec_result.success {
                    Ok(exec_result.output)
                } else {
                    let err_msg = if exec_result.stderr.is_empty() {
                        exec_result.output
                    } else {
                        exec_result.stderr
                    };
                    Err(err_msg)
                }
            }
            Err(e) => Err(format!("VM execution failed: {}", e))
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

        // Load workspace .env file
        let workspace_path = std::path::Path::new(&skills_dir);
        let workspace_env = load_workspace_env(workspace_path);

        // Build command with environment variables
        let mut cmd = tokio::process::Command::new("pnpm");
        cmd.arg("tsx")
            .arg(&cli_path)
            .arg("--skill")
            .arg(skill_name)
            .arg("--input")
            .arg(&input_json)
            .arg("--task-id")
            .arg(task_id)
            .current_dir(&skills_dir);

        // Inject workspace environment variables
        for (key, value) in workspace_env {
            cmd.env(&key, &value);
        }

        let output = cmd.output()
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
