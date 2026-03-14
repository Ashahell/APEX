use sqlx::{Pool, Sqlite, FromRow};
use serde::{Deserialize, Serialize};

/// Execution pattern detected (Death Spiral Detection)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionPattern {
    pub id: String,
    pub task_id: String,
    pub pattern_type: String,
    pub severity: String,
    pub tool_calls: Option<String>,
    pub file_ops: Option<String>,
    pub error_count: i32,
    pub details: Option<String>,
    pub detected_at: String,
}

/// Pattern alert template
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PatternAlertTemplate {
    pub id: String,
    pub pattern_type: String,
    pub title: String,
    pub description: Option<String>,
    pub severity: String,
    pub remediation: Option<String>,
    pub created_at: String,
}

/// Execution Pattern repository
pub struct ExecutionPatternRepository {
    pool: Pool<Sqlite>,
}

impl ExecutionPatternRepository {
    pub fn new(pool: &Pool<Sqlite>) -> Self {
        Self { pool: pool.clone() }
    }

    /// Record a new execution pattern
    pub async fn record_pattern(
        &self,
        id: &str,
        task_id: &str,
        pattern_type: &str,
        severity: &str,
        tool_calls: Option<&str>,
        file_ops: Option<&str>,
        error_count: i32,
        details: Option<&str>,
    ) -> Result<ExecutionPattern, sqlx::Error> {
        sqlx::query_as::<_, ExecutionPattern>(
            r#"
            INSERT INTO execution_patterns (id, task_id, pattern_type, severity, tool_calls, file_ops, error_count, details)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(task_id)
        .bind(pattern_type)
        .bind(severity)
        .bind(tool_calls)
        .bind(file_ops)
        .bind(error_count)
        .bind(details)
        .fetch_one(&self.pool)
        .await
    }

    /// Get patterns by task ID
    pub async fn get_by_task(&self, task_id: &str) -> Result<Vec<ExecutionPattern>, sqlx::Error> {
        sqlx::query_as::<_, ExecutionPattern>(
            "SELECT * FROM execution_patterns WHERE task_id = ? ORDER BY detected_at DESC"
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Get patterns by type
    pub async fn get_by_type(&self, pattern_type: &str) -> Result<Vec<ExecutionPattern>, sqlx::Error> {
        sqlx::query_as::<_, ExecutionPattern>(
            "SELECT * FROM execution_patterns WHERE pattern_type = ? ORDER BY detected_at DESC"
        )
        .bind(pattern_type)
        .fetch_all(&self.pool)
        .await
    }

    /// Get patterns by severity
    pub async fn get_by_severity(&self, severity: &str) -> Result<Vec<ExecutionPattern>, sqlx::Error> {
        sqlx::query_as::<_, ExecutionPattern>(
            "SELECT * FROM execution_patterns WHERE severity = ? ORDER BY detected_at DESC"
        )
        .bind(severity)
        .fetch_all(&self.pool)
        .await
    }

    /// Get recent patterns
    pub async fn get_recent(&self, limit: i64) -> Result<Vec<ExecutionPattern>, sqlx::Error> {
        sqlx::query_as::<_, ExecutionPattern>(
            "SELECT * FROM execution_patterns ORDER BY detected_at DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Count patterns by severity
    pub async fn count_by_severity(&self) -> Result<Vec<(String, i64)>, sqlx::Error> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT severity, COUNT(*) as count FROM execution_patterns GROUP BY severity ORDER BY count DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// Count patterns by type
    pub async fn count_by_type(&self) -> Result<Vec<(String, i64)>, sqlx::Error> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT pattern_type, COUNT(*) as count FROM execution_patterns GROUP BY pattern_type ORDER BY count DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// Delete patterns by task ID
    pub async fn delete_by_task(&self, task_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM execution_patterns WHERE task_id = ?")
            .bind(task_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ============ Alert Templates ============

    /// Get all alert templates
    pub async fn list_templates(&self) -> Result<Vec<PatternAlertTemplate>, sqlx::Error> {
        sqlx::query_as::<_, PatternAlertTemplate>(
            "SELECT * FROM pattern_alert_templates ORDER BY severity, pattern_type"
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Get template by pattern type
    pub async fn get_template(&self, pattern_type: &str) -> Result<PatternAlertTemplate, sqlx::Error> {
        sqlx::query_as::<_, PatternAlertTemplate>(
            "SELECT * FROM pattern_alert_templates WHERE pattern_type = ?"
        )
        .bind(pattern_type)
        .fetch_one(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn create_tables(pool: &SqlitePool) {
        sqlx::query("CREATE TABLE IF NOT EXISTS execution_patterns (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            pattern_type TEXT NOT NULL,
            severity TEXT NOT NULL,
            tool_calls TEXT,
            file_ops TEXT,
            error_count INTEGER DEFAULT 0,
            details TEXT,
            detected_at TEXT NOT NULL DEFAULT (datetime('now'))
        )").execute(pool).await.unwrap();

        sqlx::query("CREATE TABLE IF NOT EXISTS pattern_alert_templates (
            id TEXT PRIMARY KEY,
            pattern_type TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            description TEXT,
            severity TEXT NOT NULL,
            remediation TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )").execute(pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_record_pattern() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = ExecutionPatternRepository::new(&pool);
        let pattern = repo.record_pattern(
            "pat-1",
            "task-1",
            "tool_call_loop",
            "critical",
            Some(r#"["tool.read", "tool.read", "tool.read"]"#),
            None,
            5,
            Some(r#"{"consecutive": 5}"#),
        ).await.unwrap();

        assert_eq!(pattern.pattern_type, "tool_call_loop");
        assert_eq!(pattern.severity, "critical");
    }

    #[tokio::test]
    async fn test_get_by_task() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = ExecutionPatternRepository::new(&pool);
        repo.record_pattern("pat-1", "task-1", "tool_call_loop", "critical", None, None, 0, None).await.unwrap();
        repo.record_pattern("pat-2", "task-1", "error_cascade", "high", None, None, 3, None).await.unwrap();

        let patterns = repo.get_by_task("task-1").await.unwrap();
        assert_eq!(patterns.len(), 2);
    }

    #[tokio::test]
    async fn test_count_by_severity() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = ExecutionPatternRepository::new(&pool);
        repo.record_pattern("pat-1", "task-1", "tool_call_loop", "critical", None, None, 0, None).await.unwrap();
        repo.record_pattern("pat-2", "task-2", "error_cascade", "high", None, None, 0, None).await.unwrap();
        repo.record_pattern("pat-3", "task-3", "file_creation_burst", "high", None, None, 0, None).await.unwrap();

        let counts = repo.count_by_severity().await.unwrap();
        assert_eq!(counts.len(), 2);
    }
}
