use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: String,
    pub description: String,
    pub status: SubTaskStatus,
    pub result: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubAgentPool {
    #[serde(skip)]
    tasks: Arc<RwLock<Vec<SubTask>>>,
    max_parallel: usize,
}

impl SubAgentPool {
    pub fn new(max_parallel: usize) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(Vec::new())),
            max_parallel,
        }
    }

    /// Get the max parallel setting
    pub fn max_parallel(&self) -> usize {
        self.max_parallel
    }

    pub async fn split_task(&self, goal: &str, context: &str, llm_url: &str, model: &str) -> Result<Vec<SubTask>, String> {
        use reqwest::Client;

        let prompt = format!(
            r#"Analyze this complex task and break it down into smaller, independent subtasks.
Return a JSON array of subtasks. Each subtask should be independent and able to run in parallel.

Task: {}
Context: {}

Format your response as JSON:
[
  {{"id": "subtask_1", "description": "what this subtask does", "dependencies": []}},
  {{"id": "subtask_2", "description": "what this subtask does", "dependencies": ["subtask_1"]}}
]

Only respond with valid JSON array, no explanation."#,
            goal, context
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

        let subtasks: Vec<SubTask> = serde_json::from_str(content)
            .or_else(|_| {
                if let Some(start) = content.find('[') {
                    if let Some(end) = content.rfind(']') {
                        serde_json::from_str(&content[start..=end])
                    } else {
                        Err(serde_json::Error::io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "No JSON array found",
                        )))
                    }
                } else {
                    Err(serde_json::Error::io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "No JSON found",
                    )))
                }
            })
            .map_err(|e| format!("Failed to parse subtasks: {}", e))?;

        let mut tasks = self.tasks.write().await;
        for task in &subtasks {
            tasks.push(task.clone());
        }

        Ok(subtasks)
    }

    pub async fn get_task(&self, id: &str) -> Option<SubTask> {
        let tasks = self.tasks.read().await;
        tasks.iter().find(|t| t.id == id).cloned()
    }

    pub async fn get_all_tasks(&self) -> Vec<SubTask> {
        let tasks = self.tasks.read().await;
        tasks.clone()
    }

    pub async fn update_status(&self, id: &str, status: SubTaskStatus, result: Option<String>) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            task.status = status;
            task.result = result;
            return true;
        }
        false
    }

    pub async fn get_ready_tasks(&self) -> Vec<SubTask> {
        let tasks = self.tasks.read().await;
        
        tasks
            .iter()
            .filter(|t| {
                if t.status != SubTaskStatus::Pending {
                    return false;
                }
                t.dependencies.iter().all(|dep_id| {
                    tasks
                        .iter()
                        .find(|t| &t.id == dep_id)
                        .map(|t| t.status == SubTaskStatus::Completed)
                        .unwrap_or(false)
                })
            })
            .cloned()
            .collect()
    }

    pub async fn is_complete(&self) -> bool {
        let tasks = self.tasks.read().await;
        !tasks.is_empty() && tasks.iter().all(|t| t.status == SubTaskStatus::Completed)
    }

    pub async fn has_failed(&self) -> bool {
        let tasks = self.tasks.read().await;
        tasks.iter().any(|t| t.status == SubTaskStatus::Failed)
    }

    pub async fn clear(&self) {
        let mut tasks = self.tasks.write().await;
        tasks.clear();
    }

    pub async fn summary(&self) -> String {
        let tasks = self.tasks.read().await;
        let pending = tasks.iter().filter(|t| t.status == SubTaskStatus::Pending).count();
        let running = tasks.iter().filter(|t| t.status == SubTaskStatus::Running).count();
        let completed = tasks.iter().filter(|t| t.status == SubTaskStatus::Completed).count();
        let failed = tasks.iter().filter(|t| t.status == SubTaskStatus::Failed).count();
        
        format!("Subtasks: {} pending, {} running, {} completed, {} failed", 
            pending, running, completed, failed)
    }
}

impl Default for SubAgentPool {
    fn default() -> Self {
        Self::new(3)
    }
}

pub fn should_split_task(goal: &str, current_step: u32) -> bool {
    let goal_len = goal.len();
    let complexity_indicators = [
        " and ",
        " then ",
        " after ",
        " before ",
        " multiple ",
        " several ",
        " different ",
    ];
    
    let has_parallel_indicators = complexity_indicators.iter()
        .any(|ind| goal.to_lowercase().contains(ind));
    
    let is_long_task = goal_len > 200;
    let is_early_step = current_step < 3;
    
    (has_parallel_indicators || is_long_task) && is_early_step
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_split_task() {
        assert!(!should_split_task("Simple task", 1));
        assert!(should_split_task("Do this and then do that", 1));
        assert!(should_split_task("Create a web app with user authentication and database and API endpoints", 1));
    }

    #[tokio::test]
    async fn test_subagent_pool_new() {
        let pool = SubAgentPool::new(3);
        assert_eq!(pool.get_ready_tasks().await.len(), 0);
    }

    #[tokio::test]
    async fn test_subagent_pool_update_status() {
        let pool = SubAgentPool::new(3);
        
        let tasks = vec![
            SubTask {
                id: "task1".to_string(),
                description: "First task".to_string(),
                status: SubTaskStatus::Pending,
                result: None,
                dependencies: vec![],
            },
        ];
        
        {
            let mut pool_tasks = pool.tasks.write().await;
            *pool_tasks = tasks;
        }
        
        assert!(pool.update_status("task1", SubTaskStatus::Running, None).await);
        
        let task = pool.get_task("task1").await;
        assert!(task.is_some());
        assert_eq!(task.unwrap().status, SubTaskStatus::Running);
    }

    #[tokio::test]
    async fn test_subagent_pool_ready_tasks() {
        let pool = SubAgentPool::new(3);
        
        let tasks = vec![
            SubTask {
                id: "task1".to_string(),
                description: "First task".to_string(),
                status: SubTaskStatus::Completed,
                result: Some("done".to_string()),
                dependencies: vec![],
            },
            SubTask {
                id: "task2".to_string(),
                description: "Second task".to_string(),
                status: SubTaskStatus::Pending,
                result: None,
                dependencies: vec!["task1".to_string()],
            },
        ];
        
        {
            let mut pool_tasks = pool.tasks.write().await;
            *pool_tasks = tasks;
        }
        
        let ready = pool.get_ready_tasks().await;
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "task2");
    }
}
