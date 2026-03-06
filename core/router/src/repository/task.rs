use async_trait::async_trait;
use sqlx::{Row, SqlitePool};

use super::{CreateTaskInput, RepositoryError, RepositoryResult, TaskDetail, TaskFilter, TaskRepository as TaskRepo, TaskSummary, UpdateTaskInput};

pub struct SqliteTaskRepository {
    pool: SqlitePool,
}

impl SqliteTaskRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaskRepo for SqliteTaskRepository {
    async fn get(&self, id: &str) -> RepositoryResult<Option<TaskDetail>> {
        let row = sqlx::query(
            r#"
            SELECT id, status, tier, input_content, output, error, cost_usd,
                   project, priority, category, created_at, updated_at,
                   started_at, completed_at
            FROM tasks WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        Ok(row.map(|r| TaskDetail {
            id: r.get("id"),
            status: r.get("status"),
            tier: r.get("tier"),
            input_content: r.get("input_content"),
            output: r.get("output"),
            error: r.get("error"),
            cost_usd: r.get("cost_usd"),
            project: r.get("project"),
            priority: r.get("priority"),
            category: r.get("category"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            started_at: r.get("started_at"),
            completed_at: r.get("completed_at"),
        }))
    }

    async fn list(&self, filter: TaskFilter) -> RepositoryResult<(Vec<TaskSummary>, i64)> {
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn sqlx::Encode<sqlx::Sqlite> + Send>> = Vec::new();

        if let Some(ref project) = filter.project {
            conditions.push("project = ?");
            params.push(Box::new(project.clone()));
        }
        if let Some(ref status) = filter.status {
            conditions.push("status = ?");
            params.push(Box::new(status.clone()));
        }
        if let Some(ref priority) = filter.priority {
            conditions.push("priority = ?");
            params.push(Box::new(priority.clone()));
        }
        if let Some(ref category) = filter.category {
            conditions.push("category = ?");
            params.push(Box::new(category.clone()));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Get total count
        let count_sql = format!("SELECT COUNT(*) as count FROM tasks {}", where_clause);
        let count_row = sqlx::query(&count_sql)
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::from)?;
        let total: i64 = count_row.get("count");

        // Get paginated results (only select needed columns)
        let sql = format!(
            r#"
            SELECT id, status, tier, project, priority, category, created_at, output
            FROM tasks {}
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            where_clause
        );

        let mut query = sqlx::query(&sql);
        for param in &params {
            query = query.bind(param.as_ref().encode(&self.pool));
        }
        query = query.bind(filter.limit).bind(filter.offset);

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(RepositoryError::from)?;

        let tasks: Vec<TaskSummary> = rows
            .into_iter()
            .map(|r| TaskSummary {
                id: r.get("id"),
                status: r.get("status"),
                tier: r.get("tier"),
                project: r.get("project"),
                priority: r.get("priority"),
                category: r.get("category"),
                created_at: r.get("created_at"),
                output: r.get("output"),
            })
            .collect();

        Ok((tasks, total))
    }

    async fn create(&self, input: CreateTaskInput) -> RepositoryResult<TaskDetail> {
        let id = ulid::Ulid::new().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let priority = input.priority.as_deref().unwrap_or("medium");

        sqlx::query(
            r#"
            INSERT INTO tasks (id, status, tier, input_content, channel, thread_id, author,
                             skill_name, project, priority, category, created_at, updated_at)
            VALUES (?, 'Pending', 'T1', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&input.input_content)
        .bind(&input.channel)
        .bind(&input.thread_id)
        .bind(&input.author)
        .bind(&input.skill_name)
        .bind(&input.project)
        .bind(priority)
        .bind(&input.category)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        self.get(&id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound("Failed to create task".to_string()))
    }

    async fn update(&self, id: &str, update: UpdateTaskInput) -> RepositoryResult<TaskDetail> {
        let mut updates = Vec::new();
        let mut params: Vec<Box<dyn sqlx::Encode<sqlx::Sqlite> + Send>> = Vec::new();

        if let Some(ref status) = update.status {
            updates.push("status = ?");
            params.push(Box::new(status.clone()));
        }
        if let Some(ref output) = update.output {
            updates.push("output = ?");
            params.push(Box::new(output.clone()));
        }
        if let Some(ref error) = update.error {
            updates.push("error = ?");
            params.push(Box::new(error.clone()));
        }
        if let Some(cost) = update.cost_usd {
            updates.push("cost_usd = ?");
            params.push(Box::new(cost));
        }
        if let Some(ref project) = update.project {
            updates.push("project = ?");
            params.push(Box::new(project.clone()));
        }
        if let Some(ref priority) = update.priority {
            updates.push("priority = ?");
            params.push(Box::new(priority.clone()));
        }
        if let Some(ref category) = update.category {
            updates.push("category = ?");
            params.push(Box::new(category.clone()));
        }

        if !updates.is_empty() {
            updates.push("updated_at = ?");
            params.push(Box::new(chrono::Utc::now().to_rfc3339()));

            let sql = format!("UPDATE tasks SET {} WHERE id = ?", updates.join(", "));
            params.push(Box::new(id.to_string()));

            let mut query = sqlx::query(&sql);
            for param in &params {
                query = query.bind(param.as_ref().encode(&self.pool));
            }

            query
                .execute(&self.pool)
                .await
                .map_err(RepositoryError::from)?;
        }

        self.get(id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Task not found: {}", id)))
    }

    async fn delete(&self, id: &str) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::from)?;
        Ok(())
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM tasks")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::from)?;
        Ok(row.get("count"))
    }

    async fn count_by_status(&self, status: &str) -> RepositoryResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM tasks WHERE status = ?")
            .bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::from)?;
        Ok(row.get("count"))
    }

    async fn get_unique_projects(&self) -> RepositoryResult<Vec<String>> {
        let rows = sqlx::query("SELECT DISTINCT project FROM tasks WHERE project IS NOT NULL ORDER BY project")
            .fetch_all(&self.pool)
            .await
            .map_err(RepositoryError::from)?;
        Ok(rows.into_iter().map(|r| r.get("project")).collect())
    }

    async fn get_unique_priorities(&self) -> RepositoryResult<Vec<String>> {
        let rows = sqlx::query("SELECT DISTINCT priority FROM tasks WHERE priority IS NOT NULL ORDER BY priority")
            .fetch_all(&self.pool)
            .await
            .map_err(RepositoryError::from)?;
        Ok(rows.into_iter().map(|r| r.get("priority")).collect())
    }

    async fn get_unique_categories(&self) -> RepositoryResult<Vec<String>> {
        let rows = sqlx::query("SELECT DISTINCT category FROM tasks WHERE category IS NOT NULL ORDER BY category")
            .fetch_all(&self.pool)
            .await
            .map_err(RepositoryError::from)?;
        Ok(rows.into_iter().map(|r| r.get("category")).collect())
    }
}
