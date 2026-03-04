use sqlx::{Pool, Sqlite};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub definition: String,
    pub category: Option<String>,
    pub version: i32,
    pub is_active: i32,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub last_executed_at_ms: Option<i64>,
    pub execution_count: i32,
    pub avg_duration_secs: Option<f64>,
    pub success_rate: Option<f64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_id: String,
    pub status: String,
    pub started_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub duration_secs: Option<f64>,
    pub input_data: Option<String>,
    pub output_data: Option<String>,
    pub error_message: Option<String>,
    pub triggered_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWorkflow {
    pub name: String,
    pub description: Option<String>,
    pub definition: String,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWorkflow {
    pub name: Option<String>,
    pub description: Option<String>,
    pub definition: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Clone)]
pub struct WorkflowRepository {
    pool: Pool<Sqlite>,
}

impl WorkflowRepository {
    pub fn new(pool: &Pool<Sqlite>) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn find_all(&self) -> Result<Vec<Workflow>, sqlx::Error> {
        sqlx::query_as::<_, Workflow>(
            "SELECT id, name, description, definition, category, version, is_active, 
             created_at_ms, updated_at_ms, last_executed_at_ms, execution_count, 
             avg_duration_secs, success_rate 
             FROM workflows ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_active(&self) -> Result<Vec<Workflow>, sqlx::Error> {
        sqlx::query_as::<_, Workflow>(
            "SELECT id, name, description, definition, category, version, is_active, 
             created_at_ms, updated_at_ms, last_executed_at_ms, execution_count, 
             avg_duration_secs, success_rate 
             FROM workflows WHERE is_active = 1 ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_category(&self, category: &str) -> Result<Vec<Workflow>, sqlx::Error> {
        sqlx::query_as::<_, Workflow>(
            "SELECT id, name, description, definition, category, version, is_active, 
             created_at_ms, updated_at_ms, last_executed_at_ms, execution_count, 
             avg_duration_secs, success_rate 
             FROM workflows WHERE category = ? ORDER BY name"
        )
        .bind(category)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Workflow>, sqlx::Error> {
        sqlx::query_as::<_, Workflow>(
            "SELECT id, name, description, definition, category, version, is_active, 
             created_at_ms, updated_at_ms, last_executed_at_ms, execution_count, 
             avg_duration_secs, success_rate 
             FROM workflows WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(&self, id: &str, workflow: &CreateWorkflow) -> Result<(), sqlx::Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        
        sqlx::query(
            "INSERT INTO workflows (id, name, description, definition, category, version, is_active, created_at_ms, updated_at_ms, execution_count)
             VALUES (?, ?, ?, ?, ?, 1, 1, ?, ?, 0)"
        )
        .bind(id)
        .bind(&workflow.name)
        .bind(&workflow.description)
        .bind(&workflow.definition)
        .bind(&workflow.category)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update(&self, id: &str, update: &UpdateWorkflow) -> Result<(), sqlx::Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let workflow = self.find_by_id(id).await?;
        if workflow.is_none() {
            return Err(sqlx::Error::RowNotFound);
        }
        let workflow = workflow.unwrap();

        let desc = update.description.as_ref()
            .or(workflow.description.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let cat = update.category.as_ref()
            .or(workflow.category.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        sqlx::query(
            "UPDATE workflows SET 
             name = ?, description = ?, definition = ?, category = ?, 
             is_active = ?, version = version + 1, updated_at_ms = ?
             WHERE id = ?"
        )
        .bind(update.name.as_ref().unwrap_or(&workflow.name))
        .bind(&desc)
        .bind(update.definition.as_ref().unwrap_or(&workflow.definition))
        .bind(&cat)
        .bind(update.is_active.map(|v| if v { 1 } else { 0 }).unwrap_or(workflow.is_active))
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM workflows WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn record_execution(&self, execution: &WorkflowExecution) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO workflow_execution_logs (id, workflow_id, status, started_at_ms, completed_at_ms, duration_secs, input_data, output_data, error_message, triggered_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&execution.id)
        .bind(&execution.workflow_id)
        .bind(&execution.status)
        .bind(execution.started_at_ms)
        .bind(execution.completed_at_ms)
        .bind(execution.duration_secs)
        .bind(&execution.input_data)
        .bind(&execution.output_data)
        .bind(&execution.error_message)
        .bind(&execution.triggered_by)
        .execute(&self.pool)
        .await?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        sqlx::query(
            "UPDATE workflows SET 
             last_executed_at_ms = ?, 
             execution_count = execution_count + 1 
             WHERE id = ?"
        )
        .bind(now)
        .bind(&execution.workflow_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_executions(&self, workflow_id: &str, limit: i32) -> Result<Vec<WorkflowExecution>, sqlx::Error> {
        sqlx::query_as::<_, WorkflowExecution>(
            "SELECT id, workflow_id, status, started_at_ms, completed_at_ms, duration_secs, 
             input_data, output_data, error_message, triggered_by 
             FROM workflow_execution_logs 
             WHERE workflow_id = ? 
             ORDER BY started_at_ms DESC 
             LIMIT ?"
        )
        .bind(workflow_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_categories(&self) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT category FROM workflows WHERE category IS NOT NULL ORDER BY category"
        )
        .fetch_all(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn create_tables(pool: &SqlitePool) {
        sqlx::query("CREATE TABLE IF NOT EXISTS workflows (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            definition TEXT NOT NULL,
            category TEXT,
            version INTEGER DEFAULT 1,
            is_active INTEGER DEFAULT 1,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL,
            last_executed_at_ms INTEGER,
            execution_count INTEGER DEFAULT 0,
            avg_duration_secs REAL,
            success_rate REAL
        )").execute(pool).await.unwrap();

        sqlx::query("CREATE TABLE IF NOT EXISTS workflow_execution_logs (
            id TEXT PRIMARY KEY,
            workflow_id TEXT NOT NULL,
            status TEXT NOT NULL,
            started_at_ms INTEGER NOT NULL,
            completed_at_ms INTEGER,
            duration_secs REAL,
            input_data TEXT,
            output_data TEXT,
            error_message TEXT,
            triggered_by TEXT
        )").execute(pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_workflow_create_and_find() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = WorkflowRepository::new(&pool);
        let workflow = CreateWorkflow {
            name: "Test Workflow".to_string(),
            description: Some("A test workflow".to_string()),
            definition: "steps: []".to_string(),
            category: Some("testing".to_string()),
        };
        repo.create("wf-1", &workflow).await.unwrap();

        let workflows = repo.find_all().await.unwrap();
        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name, "Test Workflow");
    }

    #[tokio::test]
    async fn test_workflow_find_by_category() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = WorkflowRepository::new(&pool);
        let workflow = CreateWorkflow {
            name: "Test Workflow".to_string(),
            description: None,
            definition: "steps: []".to_string(),
            category: Some("dev".to_string()),
        };
        repo.create("wf-1", &workflow).await.unwrap();

        let workflows = repo.find_by_category("dev").await.unwrap();
        assert_eq!(workflows.len(), 1);
    }
}
