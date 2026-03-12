use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicTool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub code: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionContext {
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub task_id: String,
    pub sandbox_config: Option<SandboxConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub memory_limit_mb: u64,
    pub timeout_secs: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolRegistry {
    #[serde(skip)]
    tools: Arc<RwLock<HashMap<String, DynamicTool>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, tool: DynamicTool) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name.clone(), tool);
    }

    pub async fn get(&self, name: &str) -> Option<DynamicTool> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }

    pub async fn list(&self) -> Vec<DynamicTool> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }

    pub async fn remove(&self, name: &str) -> bool {
        let mut tools = self.tools.write().await;
        tools.remove(name).is_some()
    }

    pub async fn count(&self) -> usize {
        let tools = self.tools.read().await;
        tools.len()
    }

    /// Remove tools older than specified hours
    /// Returns number of tools removed
    pub async fn cleanup_expired(&self, max_age_hours: i64) -> usize {
        let now = chrono::Utc::now().timestamp();
        let max_age_secs = max_age_hours * 3600;
        
        let mut tools = self.tools.write().await;
        let mut removed_count = 0;
        
        tools.retain(|_name, tool| {
            let age = now - tool.created_at;
            if age > max_age_secs {
                removed_count += 1;
                false  // Remove this tool
            } else {
                true  // Keep this tool
            }
        });
        
        removed_count
    }

    pub async fn execute(
        &self,
        name: &str,
        parameters: serde_json::Value,
        config: Option<SandboxConfig>,
    ) -> Result<serde_json::Value, String> {
        let tool = self.get(name).await.ok_or("Tool not found")?;
        let result = execute_dynamic_tool(&tool, &parameters, &ToolExecutionContext {
            tool_name: name.to_string(),
            parameters: parameters.clone(),
            task_id: "dynamic".to_string(),
            sandbox_config: config,
        }).await
        .map_err(|e| e.to_string())?;
        
        Ok(serde_json::json!({
            "success": true,
            "output": result,
            "error": serde_json::Value::Null
        }))
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn generate_tool(
    goal: &str,
    task_context: &str,
    llm_url: &str,
    model: &str,
) -> Result<DynamicTool, String> {
    use reqwest::Client;

    let prompt = format!(
        r#"You are a tool generator. Create a custom tool for the following goal.
Generate a tool that can help accomplish this task efficiently.

Goal: {}
Context: {}

Respond with a JSON object describing the tool:
{{
  "name": "tool_name_in_snake_case",
  "description": "What the tool does",
  "parameters": [
    {{"name": "param1", "param_type": "string", "description": "param description", "required": true}}
  ],
  "code": "Python code that implements the tool using 'parameters' dict"
}}

Only respond with valid JSON, no explanation."#,
        goal, task_context
    );

    let client = Client::new();
    let request_body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 1024,
    });

    let response = client
        .post(format!("{}/v1/chat/completions", llm_url))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to call LLM: {}", e))?;

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let content = data["choices"]
        .as_array()
        .and_then(|c| c.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or("No content in LLM response")?;

    let tool: DynamicTool = serde_json::from_str(content)
        .or_else(|_| {
            if let Some(start) = content.find('{') {
                if let Some(end) = content.rfind('}') {
                    serde_json::from_str(&content[start..=end])
                } else {
                    Err(serde_json::Error::io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "No JSON found",
                    )))
                }
            } else {
                Err(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "No JSON found",
                )))
            }
        })
        .map_err(|e| format!("Failed to parse tool: {}", e))?;

    Ok(tool)
}

pub async fn execute_dynamic_tool(
    tool: &DynamicTool,
    parameters: &serde_json::Value,
    context: &ToolExecutionContext,
) -> Result<String, String> {
    use std::process::Command;
    use std::fs;
    use std::io::Write;
    
    tracing::info!(tool = %tool.name, "Executing dynamic tool in sandbox");
    
    // Convert parameters to JSON string for Python
    let params_json = parameters.to_string();
    
    // Get sandbox config with defaults
    let (memory_limit_mb, timeout_secs) = context.sandbox_config
        .as_ref()
        .map(|c| (c.memory_limit_mb, c.timeout_secs))
        .unwrap_or((512, 30));
    
    // Find the sandbox.py path - try multiple locations
    let sandbox_paths = vec![
        // Development: execution/src/apex_agent/sandbox.py
        std::path::Path::new("execution/src/apex_agent/sandbox.py"),
        // Alternative: from project root
        std::path::Path::new("execution/src/apex_agent/sandbox.py"),
    ];
    
    let sandbox_path = sandbox_paths
        .iter()
        .find(|p| p.exists())
        .map(|p| p.to_path_buf())
        .ok_or_else(|| {
            tracing::error!("Sandbox not found at any expected location");
            "Sandbox not found. Expected at execution/src/apex_agent/sandbox.py".to_string()
        })?;
    
    // Get Python executable
    let python_cmd = if cfg!(windows) { "python" } else { "python3" };
    
    // Escape single quotes in the tool code and params for shell
    let escaped_code = tool.code.replace('\'', "''");
    let escaped_params = params_json.replace('\'', "''");
    
    // Run the sandbox with the tool code inline
    let output = Command::new(python_cmd)
        .arg("-c")
        .arg(format!(
            r##"
import sys
sys.path.insert(0, r'{}')
from sandbox import PythonSandbox, SandboxConfig

# Tool code
tool_code = '''{}'''

# Parse parameters
import json
try:
    params = json.loads('''{}''')
except:
    params = {{}}

# Execute in sandbox with config
config = SandboxConfig(
    memory_limit_mb={},
    timeout_seconds={}
)
sandbox = PythonSandbox(config)
result = sandbox.execute(tool_code, params, timeout_seconds={})

# Output as JSON
print(json.dumps(result.to_dict()))
"##,
            sandbox_path.display(),
            escaped_code,
            escaped_params,
            memory_limit_mb,
            timeout_secs,
            timeout_secs
        ))
        .output()
        .map_err(|e| format!("Failed to execute sandbox: {}", e))?;
    
    // Parse the result
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !output.status.success() {
        tracing::error!(tool = %tool.name, stderr = %stderr, "Sandbox execution failed");
        return Err(format!("Sandbox execution failed: {}", stderr));
    }
    
    // Parse JSON result
    let result_json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse result: {}. Output was: {}", e, stdout))?;
    
    let success = result_json["success"].as_bool().unwrap_or(false);
    let sandbox_output = result_json["output"].as_str().unwrap_or("");
    let error = result_json["error"].as_str();
    let exec_time = result_json["execution_time_ms"].as_i64().unwrap_or(0);
    
    if success {
        tracing::info!(tool = %tool.name, exec_time_ms = exec_time, "Tool executed successfully");
        Ok(format!(
            "Tool '{}' executed successfully ({}ms):\n{}",
            tool.name, exec_time, sandbox_output
        ))
    } else {
        let error_msg = error.unwrap_or("Unknown error");
        tracing::error!(tool = %tool.name, error = %error_msg, "Tool execution failed");
        Err(format!("Tool execution failed: {}", error_msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_new() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.tools.try_read().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_tool_registry_register() {
        let registry = ToolRegistry::new();
        let tool = DynamicTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: vec![],
            code: "return 'test'".to_string(),
            created_at: 0,
        };
        
        registry.register(tool.clone()).await;
        assert_eq!(registry.count().await, 1);
        
        let retrieved = registry.get("test_tool").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_tool");
    }

    #[tokio::test]
    async fn test_tool_registry_remove() {
        let registry = ToolRegistry::new();
        let tool = DynamicTool {
            name: "temp_tool".to_string(),
            description: "Temp tool".to_string(),
            parameters: vec![],
            code: "pass".to_string(),
            created_at: 0,
        };
        
        registry.register(tool).await;
        assert!(registry.remove("temp_tool").await);
        assert!(!registry.remove("nonexistent").await);
    }

    #[tokio::test]
    async fn test_tool_cleanup_expired() {
        let registry = ToolRegistry::new();
        let now = chrono::Utc::now().timestamp();
        
        // Create tools with different ages
        let old_tool = DynamicTool {
            name: "old_tool".to_string(),
            description: "Tool created 25 hours ago".to_string(),
            parameters: vec![],
            code: "pass".to_string(),
            created_at: now - (25 * 3600),  // 25 hours ago
        };
        
        let new_tool = DynamicTool {
            name: "new_tool".to_string(),
            description: "Tool created 1 hour ago".to_string(),
            parameters: vec![],
            code: "pass".to_string(),
            created_at: now - (1 * 3600),  // 1 hour ago
        };
        
        registry.register(old_tool).await;
        registry.register(new_tool).await;
        
        assert_eq!(registry.count().await, 2);
        
        // Cleanup tools older than 24 hours
        let removed = registry.cleanup_expired(24).await;
        assert_eq!(removed, 1);
        assert_eq!(registry.count().await, 1);
        
        // Verify old tool was removed
        assert!(registry.get("old_tool").await.is_none());
        assert!(registry.get("new_tool").await.is_some());
    }
}
