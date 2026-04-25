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
                // FIXED: Archive instead of delete to preserve hash chain
                "audit_log" => self.archive_old_audit_logs(config.retention_days).await?,
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
            "SELECT entity_type, retention_days, enabled FROM ttl_config WHERE enabled = 1"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get TTL configs: {}", e))?;

        Ok(rows.into_iter().map(|(t, d, e)| TtlConfig {
            entity_type: t,
            retention_days: d,
            enabled: e,
        }).collect())
    }

    async fn delete_old_tasks(&self, days: i32) -> Result<i64, String> {
        let result = sqlx::query(
            "DELETE FROM tasks WHERE created_at < datetime('now', ?)"
        )
        .bind(format!("-{} days", days))
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete old tasks: {}", e))?;

        Ok(result.rows_affected() as i64)
    }

    async fn delete_old_messages(&self, days: i32) -> Result<i64, String> {
        let result = sqlx::query(
            "DELETE FROM messages WHERE created_at < datetime('now', ?)"
        )
        .bind(format!("-{} days", days))
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete old messages: {}", e))?;

        Ok(result.rows_affected() as i64)
    }

    async fn archive_old_audit_logs(&self, days: i32) -> Result<i64, String> {
        // Get entries to archive (batch of 1000)
        let entries: Vec<(i64, String, String, String, String, String, String, Option<String>)> = 
            sqlx::query_as(
                "SELECT id, prev_hash, hash, timestamp, action, entity_type, entity_id, details 
                 FROM audit_log WHERE timestamp < datetime('now', ?) LIMIT 1000"
            )
            .bind(format!("-{} days", days))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to fetch audit logs to archive: {}", e))?;

        if entries.is_empty() {
            return Ok(0);
        }

        // Archive each entry
        for entry in &entries {
            sqlx::query(
                "INSERT INTO audit_archive (id, prev_hash, hash, timestamp, action, entity_type, entity_id, details, archived_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))"
            )
            .bind(entry.0)
            .bind(&entry.1)
            .bind(&entry.2)
            .bind(&entry.3)
            .bind(&entry.4)
            .bind(&entry.5)
            .bind(&entry.6)
            .bind(&entry.7)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to archive audit log: {}", e))?;
        }

        // Delete archived entries from audit_log
        let ids: Vec<i64> = entries.iter().map(|e| e.0).collect();
        let placeholders: String = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        
        sqlx::query(&format!(
            "DELETE FROM audit_log WHERE id IN ({})",
            placeholders
        ))
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete archived audit logs: {}", e))?;

        tracing::info!("Archived {} audit log entries", entries.len());
        
        Ok(entries.len() as i64)
    }

    async fn delete_old_vector_store(&self, days: i32) -> Result<i64, String> {
        let result = sqlx::query(
            "DELETE FROM vector_store WHERE created_at < datetime('now', ?)"
        )
        .bind(format!("-{} days", days))
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete old vector store: {}", e))?;

        Ok(result.rows_affected() as i64)
    }

    async fn update_last_cleanup(&self, entity_type: &str) -> Result<(), String> {
        sqlx::query(
            "UPDATE ttl_config SET last_cleanup = datetime('now') WHERE entity_type = ?"
        )
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
