use sqlx::SqlitePool;

pub struct TtlCleanup {
    pool: SqlitePool,
}

impl TtlCleanup {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn cleanup_old_records(&self) -> Result<CleanupReport, String> {
        let mut report = CleanupReport::default();

        let configs = self.get_ttl_configs().await?;

        for config in configs {
            if !config.enabled {
                continue;
            }

            let deleted = match config.entity_type.as_str() {
                "tasks" => self.delete_old_tasks(config.retention_days).await?,
                "messages" => self.delete_old_messages(config.retention_days).await?,
                "audit_log" => self.delete_old_audit_logs(config.retention_days).await?,
                "vector_store" => self.delete_old_vector_store(config.retention_days).await?,
                _ => 0,
            };

            report.add(&config.entity_type, deleted);
            self.update_last_cleanup(&config.entity_type).await?;
        }

        Ok(report)
    }

    async fn get_ttl_configs(&self) -> Result<Vec<TtlConfig>, String> {
        let rows: Vec<(String, i32, bool)> = sqlx::query_as(
            "SELECT entity_type, retention_days, enabled FROM ttl_config WHERE enabled = 1",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get TTL configs: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|(t, d, e)| TtlConfig {
                entity_type: t,
                retention_days: d,
                enabled: e,
            })
            .collect())
    }

    async fn delete_old_tasks(&self, days: i32) -> Result<i64, String> {
        let result = sqlx::query("DELETE FROM tasks WHERE created_at < datetime('now', ?)")
            .bind(format!("-{} days", days))
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete old tasks: {}", e))?;

        Ok(result.rows_affected() as i64)
    }

    async fn delete_old_messages(&self, days: i32) -> Result<i64, String> {
        let result = sqlx::query("DELETE FROM messages WHERE created_at < datetime('now', ?)")
            .bind(format!("-{} days", days))
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete old messages: {}", e))?;

        Ok(result.rows_affected() as i64)
    }

    async fn delete_old_audit_logs(&self, days: i32) -> Result<i64, String> {
        let result = sqlx::query("DELETE FROM audit_log WHERE timestamp < datetime('now', ?)")
            .bind(format!("-{} days", days))
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete old audit logs: {}", e))?;

        Ok(result.rows_affected() as i64)
    }

    async fn delete_old_vector_store(&self, days: i32) -> Result<i64, String> {
        let result = sqlx::query("DELETE FROM vector_store WHERE created_at < datetime('now', ?)")
            .bind(format!("-{} days", days))
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete old vector store: {}", e))?;

        Ok(result.rows_affected() as i64)
    }

    async fn update_last_cleanup(&self, entity_type: &str) -> Result<(), String> {
        sqlx::query("UPDATE ttl_config SET last_cleanup = datetime('now') WHERE entity_type = ?")
            .bind(entity_type)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update last cleanup: {}", e))?;

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct CleanupReport {
    pub total_deleted: i64,
    pub by_type: Vec<(String, i64)>,
}

impl CleanupReport {
    pub fn add(&mut self, entity_type: &str, count: i64) {
        self.total_deleted += count;
        self.by_type.push((entity_type.to_string(), count));
    }
}

#[derive(Debug)]
struct TtlConfig {
    entity_type: String,
    retention_days: i32,
    enabled: bool,
}
