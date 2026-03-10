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

    pub async fn execute(
        &self,
        name: &str,
        parameters: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let tool = self.get(name).await.ok_or("Tool not found")?;
        let result = execute_dynamic_tool(&tool, &parameters, &ToolExecutionContext {
            tool_name: name.to_string(),
            parameters: parameters.clone(),
            task_id: "dynamic".to_string(),
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
    _context: &ToolExecutionContext,
) -> Result<String, String> {
    tracing::info!(tool = %tool.name, "Executing dynamic tool");
    
    Ok(format!(
        "Dynamic tool '{}' would execute with params: {}\nTool code:\n{}",
        tool.name,
        parameters,
        tool.code
    ))
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
}
