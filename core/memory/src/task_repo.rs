use crate::tasks::{CreateTask, Task, TaskStatus, TaskTier};
use chrono::Utc;
use sqlx::{Row, SqlitePool};

fn sanitize_identifier(value: &str) -> Result<String, String> {
    // Validate that the value contains only safe characters (alphanumeric, dash, underscore, space)
    if value.is_empty() || value.len() > 100 {
        return Err("Invalid identifier length".to_string());
    }
    if !value.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ') {
        return Err("Invalid characters in identifier".to_string());
    }
    Ok(value.to_string())
}

pub struct TaskRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> TaskRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        id: &str,
        input: CreateTask,
        tier: TaskTier,
    ) -> Result<Task, sqlx::Error> {
        let now = Utc::now();
        let priority = input.priority.as_deref().unwrap_or("medium");

        // Use INSERT RETURNING to get the created task in a single query
        sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (id, status, tier, input_content, channel, thread_id, author, skill_name, project, priority, category, created_at, updated_at, cancellation_requested, cancellation_requested_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, NULL)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(TaskStatus::Pending.as_str())
        .bind(tier.as_str())
        .bind(&input.input_content)
        .bind(&input.channel)
        .bind(&input.thread_id)
        .bind(&input.author)
        .bind(&input.skill_name)
        .bind(&input.project)
        .bind(priority)
        .bind(&input.category)
        .bind(now)
        .bind(now)
        .fetch_one(self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Task, sqlx::Error> {
        sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>("SELECT * FROM tasks ORDER BY created_at DESC LIMIT ? OFFSET ?")
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await
    }

    pub async fn find_by_status(&self, status: &str, limit: i64) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE status = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(status)
        .bind(limit)
        .fetch_all(self.pool)
        .await
    }

    pub async fn update_status(&self, id: &str, status: TaskStatus) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query("UPDATE tasks SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(now)
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_completed(
        &self,
        id: &str,
        status: TaskStatus,
        output_content: Option<String>,
        cost_cents: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE tasks 
            SET status = ?, output_content = ?, actual_cost_cents = ?, completed_at = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(status.as_str())
        .bind(&output_content)
        .bind(cost_cents)
        .bind(now)
        .bind(now)
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_failed(&self, id: &str, error_message: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query("UPDATE tasks SET status = ?, error_message = ?, updated_at = ? WHERE id = ?")
            .bind(TaskStatus::Failed.as_str())
            .bind(error_message)
            .bind(now)
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(())
    }

    pub async fn count(&self) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT COUNT(*) FROM tasks")
            .fetch_one(self.pool)
            .await?;

        Ok(row.get::<i64, _>(0))
    }

    pub async fn count_by_status(&self, status: &str) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT COUNT(*) FROM tasks WHERE status = ?")
            .bind(status)
            .fetch_one(self.pool)
            .await?;

        Ok(row.get::<i64, _>(0))
    }

    pub async fn cleanup_old_completed(&self, days: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM tasks 
            WHERE status IN ('completed', 'failed', 'cancelled') 
            AND completed_at IS NOT NULL 
            AND completed_at < datetime('now', ?)
            "#
        )
        .bind(format!("-{} days", days))
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn get_total_cost_cents(&self) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT COALESCE(SUM(actual_cost_cents), 0) FROM tasks WHERE status = 'completed' AND actual_cost_cents IS NOT NULL")
            .fetch_one(self.pool)
            .await?;

        Ok(row.get::<i64, _>(0))
    }

    pub async fn get_total_cost(&self) -> Result<f64, sqlx::Error> {
        let cents = self.get_total_cost_cents().await?;
        Ok(cents as f64 / 100.0)
    }

    pub async fn find_by_project(&self, project: &str, limit: i64) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE project = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(project)
        .bind(limit)
        .fetch_all(self.pool)
        .await
    }

    pub async fn find_by_priority(&self, priority: &str, limit: i64) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE priority = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(priority)
        .bind(limit)
        .fetch_all(self.pool)
        .await
    }

    pub async fn find_by_category(&self, category: &str, limit: i64) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE category = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(category)
        .bind(limit)
        .fetch_all(self.pool)
        .await
    }

    pub async fn find_by_filter(
        &self,
        project: Option<&str>,
        status: Option<&str>,
        priority: Option<&str>,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Task>, sqlx::Error> {
        // Build query dynamically with validation to prevent SQL injection
        // Column names are hardcoded, values are validated and parameterized
        let mut conditions = Vec::new();
        let mut values: Vec<String> = Vec::new();

        if let Some(p) = project {
            let validated = sanitize_identifier(p).unwrap_or_else(|_| "".to_string());
            if !validated.is_empty() {
                conditions.push("project = ?");
                values.push(validated);
            }
        }
        if let Some(s) = status {
            let validated = sanitize_identifier(s).unwrap_or_else(|_| "".to_string());
            if !validated.is_empty() {
                conditions.push("status = ?");
                values.push(validated);
            }
        }
        if let Some(p) = priority {
            let validated = sanitize_identifier(p).unwrap_or_else(|_| "".to_string());
            if !validated.is_empty() {
                conditions.push("priority = ?");
                values.push(validated);
            }
        }
        if let Some(c) = category {
            let validated = sanitize_identifier(c).unwrap_or_else(|_| "".to_string());
            if !validated.is_empty() {
                conditions.push("category = ?");
                values.push(validated);
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let query = format!(
            "SELECT id, status, tier, input_content, output_content, error_message, channel, thread_id, author, skill_name, project, priority, category, cost_estimate_cents, actual_cost_cents, started_at, completed_at, created_at, updated_at FROM tasks{} ORDER BY created_at DESC",
            where_clause
        );

        // Use query! macro with validated values to prevent SQL injection
        let mut query_builder = sqlx::query_as::<_, Task>(&query);
        
        for value in &values {
            query_builder = query_builder.bind(value.as_str());
        }
        
        query_builder
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await
    }

    pub async fn update_task_fields(
        &self,
        id: &str,
        project: Option<&str>,
        priority: Option<&str>,
        category: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let mut updates = Vec::new();
        let mut query = "UPDATE tasks SET updated_at = ?".to_string();

        if project.is_some() {
            updates.push("project = ?");
        }
        if priority.is_some() {
            updates.push("priority = ?");
        }
        if category.is_some() {
            updates.push("category = ?");
        }

        if !updates.is_empty() {
            query.push_str(", ");
            query.push_str(&updates.join(", "));
        }

        query.push_str(" WHERE id = ?");

        let now = Utc::now();
        let mut q = sqlx::query(&query).bind(now);

        if let Some(p) = project {
            q = q.bind(p);
        }
        if let Some(p) = priority {
            q = q.bind(p);
        }
        if let Some(c) = category {
            q = q.bind(c);
        }

        q = q.bind(id);
        q.execute(self.pool).await?;

        Ok(())
    }

    pub async fn get_projects(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query("SELECT DISTINCT project FROM tasks WHERE project IS NOT NULL ORDER BY project")
            .fetch_all(self.pool)
            .await?;

        let mut projects = Vec::new();
        for row in rows {
            if let Ok(project) = row.try_get::<String, _>(0) {
                projects.push(project);
            }
        }
        Ok(projects)
    }

    pub async fn get_categories(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query("SELECT DISTINCT category FROM tasks WHERE category IS NOT NULL ORDER BY category")
            .fetch_all(self.pool)
            .await?;

        let mut categories = Vec::new();
        for row in rows {
            if let Ok(category) = row.try_get::<String, _>(0) {
                categories.push(category);
            }
        }
        Ok(categories)
    }

    pub async fn update_input_content(&self, id: &str, input_content: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query("UPDATE tasks SET input_content = ?, updated_at = ? WHERE id = ?")
            .bind(input_content)
            .bind(now)
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub fn begin(&self) -> impl sqlx::Executor<'a, Database = sqlx::Sqlite> + '_ {
        self.pool
    }

    pub async fn request_cancellation(&self, task_id: &str, source: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();

        // Insert cancellation request
        sqlx::query(
            r#"
            INSERT INTO cancellation_requests (id, task_id, requested_at, source)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(&id)
        .bind(task_id)
        .bind(now)
        .bind(source)
        .execute(self.pool)
        .await?;

        // Update task cancellation flag
        sqlx::query(
            r#"
            UPDATE tasks 
            SET cancellation_requested = 1, cancellation_requested_at = ?
            WHERE id = ?
            "#
        )
        .bind(now)
        .bind(task_id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn check_cancellation(&self, task_id: &str) -> Result<bool, sqlx::Error> {
        let row = sqlx::query(
            "SELECT cancellation_requested FROM tasks WHERE id = ?"
        )
        .bind(task_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(|r| r.get::<i32, _>(0) == 1).unwrap_or(false))
    }

    pub async fn clear_cancellation(&self, task_id: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        // Mark cancellation request as fulfilled
        sqlx::query(
            "UPDATE cancellation_requests SET fulfilled = 1, fulfilled_at = ? WHERE task_id = ? AND fulfilled = 0"
        )
        .bind(now)
        .bind(task_id)
        .execute(self.pool)
        .await?;

        // Clear task cancellation flag
        sqlx::query(
            "UPDATE tasks SET cancellation_requested = 0 WHERE id = ?"
        )
        .bind(task_id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_pending_cancellations(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT task_id FROM cancellation_requests 
            WHERE fulfilled = 0 
            AND requested_at > datetime('now', '-5 minutes')
            "#
        )
        .fetch_all(self.pool)
        .await?;

        let task_ids: Vec<String> = rows.iter().filter_map(|r| r.try_get(0).ok()).collect();
        Ok(task_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::tasks::{CreateTask, TaskTier};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_request_cancellation() {
        let db = Database::new(&PathBuf::from(":memory:")).await.unwrap();
        db.run_migrations().await.unwrap();
        let pool = db.pool().clone();
        let repo = TaskRepository::new(&pool);
        
        // Create a task first
        let task_id = "test-task-1";
        repo.create(
            task_id,
            CreateTask {
                input_content: "Test task".to_string(),
                channel: Some("test".to_string()),
                thread_id: None,
                author: None,
                skill_name: None,
                project: None,
                priority: None,
                category: None,
            },
            TaskTier::Deep,
        ).await.unwrap();

        // Request cancellation
        repo.request_cancellation(task_id, "user").await.unwrap();

        // Check cancellation is set
        let cancelled = repo.check_cancellation(task_id).await.unwrap();
        assert!(cancelled, "Task should be marked for cancellation");
    }

    #[tokio::test]
    async fn test_check_cancellation_no_request() {
        let db = Database::new(&PathBuf::from(":memory:")).await.unwrap();
        db.run_migrations().await.unwrap();
        let pool = db.pool().clone();
        let repo = TaskRepository::new(&pool);
        
        // Create a task without cancellation
        let task_id = "test-task-2";
        repo.create(
            task_id,
            CreateTask {
                input_content: "Test task".to_string(),
                channel: Some("test".to_string()),
                thread_id: None,
                author: None,
                skill_name: None,
                project: None,
                priority: None,
                category: None,
            },
            TaskTier::Deep,
        ).await.unwrap();

        // Check cancellation - should be false
        let cancelled = repo.check_cancellation(task_id).await.unwrap();
        assert!(!cancelled, "Task should not be marked for cancellation");
    }

    #[tokio::test]
    async fn test_clear_cancellation() {
        let db = Database::new(&PathBuf::from(":memory:")).await.unwrap();
        db.run_migrations().await.unwrap();
        let pool = db.pool().clone();
        let repo = TaskRepository::new(&pool);
        
        // Create a task
        let task_id = "test-task-3";
        repo.create(
            task_id,
            CreateTask {
                input_content: "Test task".to_string(),
                channel: Some("test".to_string()),
                thread_id: None,
                author: None,
                skill_name: None,
                project: None,
                priority: None,
                category: None,
            },
            TaskTier::Deep,
        ).await.unwrap();

        // Request cancellation
        repo.request_cancellation(task_id, "user").await.unwrap();
        
        // Verify cancellation is set
        let cancelled = repo.check_cancellation(task_id).await.unwrap();
        assert!(cancelled);

        // Clear cancellation
        repo.clear_cancellation(task_id).await.unwrap();

        // Verify cancellation is cleared
        let cancelled = repo.check_cancellation(task_id).await.unwrap();
        assert!(!cancelled, "Cancellation should be cleared");
    }

    #[tokio::test]
    async fn test_get_pending_cancellations() {
        let db = Database::new(&PathBuf::from(":memory:")).await.unwrap();
        db.run_migrations().await.unwrap();
        let pool = db.pool().clone();
        let repo = TaskRepository::new(&pool);
        
        // Create tasks
        let task_id_1 = "test-task-4";
        let task_id_2 = "test-task-5";
        let task_id_3 = "test-task-6";
        
        for task_id in [task_id_1, task_id_2, task_id_3] {
            repo.create(
                task_id,
                CreateTask {
                    input_content: "Test task".to_string(),
                    channel: Some("test".to_string()),
                    thread_id: None,
                    author: None,
                    skill_name: None,
                    project: None,
                    priority: None,
                    category: None,
                },
                TaskTier::Deep,
            ).await.unwrap();
        }

        // Request cancellation for two tasks
        repo.request_cancellation(task_id_1, "user").await.unwrap();
        repo.request_cancellation(task_id_2, "system").await.unwrap();

        // Get pending cancellations
        let pending = repo.get_pending_cancellations().await.unwrap();
        assert_eq!(pending.len(), 2, "Should have 2 pending cancellations");
        assert!(pending.contains(&task_id_1.to_string()));
        assert!(pending.contains(&task_id_2.to_string()));
        assert!(!pending.contains(&task_id_3.to_string()));
    }
}
