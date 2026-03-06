use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use apex_memory::MemoryResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilter {
    pub project: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

impl Default for TaskFilter {
    fn default() -> Self {
        Self {
            project: None,
            status: None,
            priority: None,
            category: None,
            limit: 50,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: String,
    pub status: String,
    pub tier: String,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub created_at: String,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDetail {
    pub id: String,
    pub status: String,
    pub tier: String,
    pub input_content: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub cost_usd: Option<f64>,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn get(&self, id: &str) -> MemoryResult<Option<TaskDetail>>;
    
    async fn list(&self, filter: TaskFilter) -> MemoryResult<(Vec<TaskSummary>, i64)>;
    
    async fn create(&self, input: CreateTaskInput) -> MemoryResult<TaskDetail>;
    
    async fn update(&self, id: &str, update: UpdateTaskInput) -> MemoryResult<TaskDetail>;
    
    async fn delete(&self, id: &str) -> MemoryResult<()>;
    
    async fn count(&self) -> MemoryResult<i64>;
    
    async fn count_by_status(&self, status: &str) -> MemoryResult<i64>;
    
    async fn get_unique_projects(&self) -> MemoryResult<Vec<String>>;
    
    async fn get_unique_priorities(&self) -> MemoryResult<Vec<String>>;
    
    async fn get_unique_categories(&self) -> MemoryResult<Vec<String>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskInput {
    pub input_content: String,
    pub channel: Option<String>,
    pub thread_id: Option<String>,
    pub author: Option<String>,
    pub skill_name: Option<String>,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskInput {
    pub status: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub cost_usd: Option<f64>,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
}
