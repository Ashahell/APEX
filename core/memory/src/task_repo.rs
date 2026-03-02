use crate::tasks::{CreateTask, Task, TaskStatus, TaskTier};
use chrono::Utc;
use sqlx::{Row, SqlitePool};

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

        sqlx::query(
            r#"
            INSERT INTO tasks (id, status, tier, input_content, channel, thread_id, author, skill_name, project, priority, category, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .execute(self.pool)
        .await?;

        self.find_by_id(id).await
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
        cost: Option<f64>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE tasks 
            SET status = ?, output_content = ?, actual_cost_usd = ?, completed_at = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(status.as_str())
        .bind(&output_content)
        .bind(cost)
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

    pub async fn get_total_cost(&self) -> Result<f64, sqlx::Error> {
        let row = sqlx::query("SELECT COALESCE(SUM(actual_cost_usd), 0.0) FROM tasks WHERE status = 'completed' AND actual_cost_usd IS NOT NULL")
            .fetch_one(self.pool)
            .await?;

        Ok(row.get::<f64, _>(0))
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
        let mut conditions = Vec::new();
        let mut query = "SELECT * FROM tasks WHERE 1=1".to_string();

        if let Some(p) = project {
            conditions.push(format!(" AND project = '{}'", p));
        }
        if let Some(s) = status {
            conditions.push(format!(" AND status = '{}'", s));
        }
        if let Some(p) = priority {
            conditions.push(format!(" AND priority = '{}'", p));
        }
        if let Some(c) = category {
            conditions.push(format!(" AND category = '{}'", c));
        }

        for cond in conditions {
            query.push_str(&cond);
        }

        query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        sqlx::query_as::<_, Task>(&query)
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
}
