use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

/// Secret reference for runtime resolution
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecretRef {
    pub id: String,
    pub ref_key: String,
    pub secret_name: String,
    pub env_var: Option<String>,
    pub description: Option<String>,
    pub targets: String, // JSON array
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Secret rotation log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecretRotationLog {
    pub id: String,
    pub secret_name: String,
    pub rotated_at: String,
    pub rotated_by: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub old_value_hash: Option<String>,
    pub new_value_hash: Option<String>,
}

/// Secret access log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecretAccessLog {
    pub id: String,
    pub secret_ref_id: String,
    pub accessed_at: String,
    pub access_type: String,
    pub accessed_by: Option<String>,
    pub success: i32,
    pub error_message: Option<String>,
}

/// Secrets repository
pub struct SecretsRepository {
    pool: Pool<Sqlite>,
}

impl SecretsRepository {
    pub fn new(pool: &Pool<Sqlite>) -> Self {
        Self { pool: pool.clone() }
    }

    // ============ Secret References ============

    /// Get all secret references
    pub async fn list_secrets(&self) -> Result<Vec<SecretRef>, sqlx::Error> {
        sqlx::query_as::<_, SecretRef>("SELECT * FROM secret_refs ORDER BY category, secret_name")
            .fetch_all(&self.pool)
            .await
    }

    /// Get secret reference by ID
    pub async fn get_secret(&self, id: &str) -> Result<SecretRef, sqlx::Error> {
        sqlx::query_as::<_, SecretRef>("SELECT * FROM secret_refs WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    /// Get secret reference by ref_key
    pub async fn get_by_ref_key(&self, ref_key: &str) -> Result<SecretRef, sqlx::Error> {
        sqlx::query_as::<_, SecretRef>("SELECT * FROM secret_refs WHERE ref_key = ?")
            .bind(ref_key)
            .fetch_one(&self.pool)
            .await
    }

    /// Get secrets by category
    pub async fn get_by_category(&self, category: &str) -> Result<Vec<SecretRef>, sqlx::Error> {
        sqlx::query_as::<_, SecretRef>(
            "SELECT * FROM secret_refs WHERE category = ? ORDER BY secret_name",
        )
        .bind(category)
        .fetch_all(&self.pool)
        .await
    }

    /// Get secrets by target
    pub async fn get_by_target(&self, target: &str) -> Result<Vec<SecretRef>, sqlx::Error> {
        // Search in JSON targets array - SQLite doesn't have great JSON query support
        // so we do a simple LIKE search
        sqlx::query_as::<_, SecretRef>(
            "SELECT * FROM secret_refs WHERE targets LIKE ? ORDER BY secret_name",
        )
        .bind(format!("%{}%", target))
        .fetch_all(&self.pool)
        .await
    }

    /// Update secret reference description
    pub async fn update_description(
        &self,
        id: &str,
        description: &str,
    ) -> Result<SecretRef, sqlx::Error> {
        sqlx::query_as::<_, SecretRef>(
            r#"
            UPDATE secret_refs
            SET description = ?, updated_at = datetime('now')
            WHERE id = ?
            RETURNING *
            "#,
        )
        .bind(description)
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    /// Delete secret reference (only custom ones)
    pub async fn delete_secret(&self, id: &str) -> Result<(), sqlx::Error> {
        // Only allow deleting custom secrets (ids starting with 'custom_')
        if id.starts_with("custom_") {
            sqlx::query("DELETE FROM secret_refs WHERE id = ?")
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    /// List all categories
    pub async fn list_categories(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT DISTINCT category FROM secret_refs ORDER BY category")
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.into_iter().map(|(c,)| c).collect())
    }

    // ============ Secret Rotation Log ============

    /// Log a secret rotation
    pub async fn log_rotation(
        &self,
        id: &str,
        secret_name: &str,
        rotated_by: Option<&str>,
        status: &str,
        error_message: Option<&str>,
        old_value_hash: Option<&str>,
        new_value_hash: Option<&str>,
    ) -> Result<SecretRotationLog, sqlx::Error> {
        sqlx::query_as::<_, SecretRotationLog>(
            r#"
            INSERT INTO secret_rotation_log 
            (id, secret_name, rotated_by, status, error_message, old_value_hash, new_value_hash)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(secret_name)
        .bind(rotated_by)
        .bind(status)
        .bind(error_message)
        .bind(old_value_hash)
        .bind(new_value_hash)
        .fetch_one(&self.pool)
        .await
    }

    /// Get rotation history for a secret
    pub async fn get_rotation_history(
        &self,
        secret_name: &str,
        limit: Option<i64>,
    ) -> Result<Vec<SecretRotationLog>, sqlx::Error> {
        let limit = limit.unwrap_or(50);
        sqlx::query_as::<_, SecretRotationLog>(
            "SELECT * FROM secret_rotation_log WHERE secret_name = ? ORDER BY rotated_at DESC LIMIT ?"
        )
        .bind(secret_name)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Get recent rotations
    pub async fn get_recent_rotations(
        &self,
        limit: i64,
    ) -> Result<Vec<SecretRotationLog>, sqlx::Error> {
        sqlx::query_as::<_, SecretRotationLog>(
            "SELECT * FROM secret_rotation_log ORDER BY rotated_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // ============ Secret Access Log ============

    /// Log secret access
    pub async fn log_access(
        &self,
        id: &str,
        secret_ref_id: &str,
        access_type: &str,
        accessed_by: Option<&str>,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<SecretAccessLog, sqlx::Error> {
        sqlx::query_as::<_, SecretAccessLog>(
            r#"
            INSERT INTO secret_access_log 
            (id, secret_ref_id, access_type, accessed_by, success, error_message)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(secret_ref_id)
        .bind(access_type)
        .bind(accessed_by)
        .bind(if success { 1 } else { 0 })
        .bind(error_message)
        .fetch_one(&self.pool)
        .await
    }

    /// Get access history for a secret
    pub async fn get_access_history(
        &self,
        secret_ref_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<SecretAccessLog>, sqlx::Error> {
        let limit = limit.unwrap_or(50);
        sqlx::query_as::<_, SecretAccessLog>(
            "SELECT * FROM secret_access_log WHERE secret_ref_id = ? ORDER BY accessed_at DESC LIMIT ?"
        )
        .bind(secret_ref_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Get recent accesses
    pub async fn get_recent_accesses(
        &self,
        limit: i64,
    ) -> Result<Vec<SecretAccessLog>, sqlx::Error> {
        sqlx::query_as::<_, SecretAccessLog>(
            "SELECT * FROM secret_access_log ORDER BY accessed_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Get failed accesses
    pub async fn get_failed_accesses(
        &self,
        limit: i64,
    ) -> Result<Vec<SecretAccessLog>, sqlx::Error> {
        sqlx::query_as::<_, SecretAccessLog>(
            "SELECT * FROM secret_access_log WHERE success = 0 ORDER BY accessed_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }
}

/// Secret categories with metadata
pub fn get_category_info(category: &str) -> (&str, &str) {
    match category {
        "api_key" => ("API Key", "🔑"),
        "token" => ("Token", "🎫"),
        "credential" => ("Credential", "🔐"),
        "certificate" => ("Certificate", "📜"),
        "generic" => ("Generic", "⚙️"),
        _ => ("Unknown", "❓"),
    }
}

/// Get all predefined secret IDs (for validation)
pub fn get_predefined_secret_ids() -> Vec<&'static str> {
    vec![
        // API Keys
        "openai_api_key",
        "anthropic_api_key",
        "google_api_key",
        // OAuth Tokens
        "slack_token",
        "discord_token",
        "telegram_token",
        // Database
        "db_password",
        "db_api_key",
        // Cloud Providers
        "aws_access_key",
        "aws_secret_key",
        "gcp_key",
        "azure_key",
        // Webhooks
        "webhook_secret",
        // SSH Keys
        "ssh_private_key",
        "ssh_public_key",
        // Encryption
        "encryption_key",
        "hmac_secret",
        // LLM Providers
        "ollama_url",
        "ollama_api_key",
        "vllm_url",
        "vllm_api_key",
        // Additional Channels
        "matrix_token",
        "signal_cli_path",
        "irc_password",
        "teams_webhook",
        "feishu_app_id",
        "feishu_app_secret",
        "line_channel_secret",
        "line_channel_token",
        "mattermost_token",
        "nostr_private_key",
        // Storage
        "s3_bucket",
        "s3_region",
        "gcs_bucket",
        "azure_blob",
        // Email/SMTP
        "smtp_host",
        "smtp_user",
        "smtp_password",
        // Misc Services
        "github_token",
        "gitlab_token",
        "jira_api_key",
        "notion_token",
        "airtable_key",
        "sendgrid_key",
        "twilio_sid",
        "twilio_token",
        "stripe_key",
    ]
}

/// Check if a secret ID is predefined (cannot be deleted)
pub fn is_predefined_secret(id: &str) -> bool {
    get_predefined_secret_ids().contains(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn create_tables(pool: &SqlitePool) {
        // Create secret_refs table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS secret_refs (
            id TEXT PRIMARY KEY,
            ref_key TEXT NOT NULL UNIQUE,
            secret_name TEXT NOT NULL,
            env_var TEXT,
            description TEXT,
            targets TEXT NOT NULL,
            category TEXT DEFAULT 'generic',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        )
        .execute(pool)
        .await
        .unwrap();

        // Create secret_rotation_log table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS secret_rotation_log (
            id TEXT PRIMARY KEY,
            secret_name TEXT NOT NULL,
            rotated_at TEXT NOT NULL DEFAULT (datetime('now')),
            rotated_by TEXT,
            status TEXT NOT NULL,
            error_message TEXT,
            old_value_hash TEXT,
            new_value_hash TEXT
        )",
        )
        .execute(pool)
        .await
        .unwrap();

        // Create secret_access_log table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS secret_access_log (
            id TEXT PRIMARY KEY,
            secret_ref_id TEXT NOT NULL,
            accessed_at TEXT NOT NULL DEFAULT (datetime('now')),
            access_type TEXT NOT NULL,
            accessed_by TEXT,
            success INTEGER DEFAULT 1,
            error_message TEXT
        )",
        )
        .execute(pool)
        .await
        .unwrap();

        // Insert predefined secrets
        sqlx::query("INSERT OR IGNORE INTO secret_refs (id, ref_key, secret_name, targets, category) VALUES (?, ?, ?, ?, ?)")
            .bind("openai_api_key")
            .bind("OPENAI_API_KEY")
            .bind("OpenAI API Key")
            .bind(r#"["llm.openai"]"#)
            .bind("api_key")
            .execute(pool).await.unwrap();

        sqlx::query("INSERT OR IGNORE INTO secret_refs (id, ref_key, secret_name, targets, category) VALUES (?, ?, ?, ?, ?)")
            .bind("custom_1")
            .bind("CUSTOM_SECRET_1")
            .bind("Custom Secret 1")
            .bind(r#"["custom"]"#)
            .bind("generic")
            .execute(pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_secrets_list() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SecretsRepository::new(&pool);
        let secrets = repo.list_secrets().await.unwrap();

        assert!(secrets.len() >= 2);
        assert!(secrets.iter().any(|s| s.id == "openai_api_key"));
        assert!(secrets.iter().any(|s| s.id == "custom_1"));
    }

    #[tokio::test]
    async fn test_secrets_get_by_id() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SecretsRepository::new(&pool);
        let secret = repo.get_secret("openai_api_key").await.unwrap();

        assert_eq!(secret.ref_key, "OPENAI_API_KEY");
        assert_eq!(secret.secret_name, "OpenAI API Key");
    }

    #[tokio::test]
    async fn test_secrets_get_by_category() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SecretsRepository::new(&pool);
        let secrets = repo.get_by_category("api_key").await.unwrap();

        assert!(!secrets.is_empty());
        assert!(secrets.iter().all(|s| s.category == "api_key"));
    }

    #[tokio::test]
    async fn test_secrets_update_description() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SecretsRepository::new(&pool);
        let updated = repo
            .update_description("custom_1", "My custom secret")
            .await
            .unwrap();

        assert_eq!(updated.description, Some("My custom secret".to_string()));
    }

    #[tokio::test]
    async fn test_secrets_list_categories() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SecretsRepository::new(&pool);
        let categories = repo.list_categories().await.unwrap();

        assert!(categories.contains(&"api_key".to_string()));
        assert!(categories.contains(&"generic".to_string()));
    }

    #[tokio::test]
    async fn test_log_rotation() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SecretsRepository::new(&pool);
        let log = repo
            .log_rotation(
                "rot-1",
                "OPENAI_API_KEY",
                Some("system"),
                "success",
                None,
                Some("old_hash"),
                Some("new_hash"),
            )
            .await
            .unwrap();

        assert_eq!(log.secret_name, "OPENAI_API_KEY");
        assert_eq!(log.status, "success");
    }

    #[tokio::test]
    async fn test_log_access() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SecretsRepository::new(&pool);
        let log = repo
            .log_access(
                "acc-1",
                "openai_api_key",
                "read",
                Some("skill_worker"),
                true,
                None,
            )
            .await
            .unwrap();

        assert_eq!(log.secret_ref_id, "openai_api_key");
        assert_eq!(log.access_type, "read");
        assert_eq!(log.success, 1);
    }

    #[tokio::test]
    async fn test_get_rotation_history() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SecretsRepository::new(&pool);

        // Log a rotation
        repo.log_rotation(
            "rot-1",
            "OPENAI_API_KEY",
            Some("system"),
            "success",
            None,
            None,
            None,
        )
        .await
        .unwrap();

        let history = repo
            .get_rotation_history("OPENAI_API_KEY", None)
            .await
            .unwrap();
        assert!(!history.is_empty());
    }

    #[tokio::test]
    async fn test_get_access_history() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SecretsRepository::new(&pool);

        // Log an access
        repo.log_access(
            "acc-1",
            "openai_api_key",
            "read",
            Some("skill_worker"),
            true,
            None,
        )
        .await
        .unwrap();

        let history = repo
            .get_access_history("openai_api_key", None)
            .await
            .unwrap();
        assert!(!history.is_empty());
    }

    #[tokio::test]
    async fn test_is_predefined_secret() {
        assert!(is_predefined_secret("openai_api_key"));
        assert!(is_predefined_secret("github_token"));
        assert!(!is_predefined_secret("custom_1"));
        assert!(!is_predefined_secret("nonexistent"));
    }

    #[tokio::test]
    async fn test_get_predefined_secret_ids() {
        let ids = get_predefined_secret_ids();

        assert!(ids.contains(&"openai_api_key"));
        assert!(ids.contains(&"anthropic_api_key"));
        assert!(ids.contains(&"github_token"));
        assert!(ids.len() >= 40); // Should have 40+ predefined secrets
    }

    #[tokio::test]
    async fn test_get_category_info() {
        let (label, icon) = get_category_info("api_key");
        assert_eq!(label, "API Key");
        assert_eq!(icon, "🔑");

        let (label, icon) = get_category_info("token");
        assert_eq!(label, "Token");
        assert_eq!(icon, "🎫");

        let (label, icon) = get_category_info("unknown");
        assert_eq!(label, "Unknown");
        assert_eq!(icon, "❓");
    }

    #[tokio::test]
    async fn test_get_failed_accesses() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SecretsRepository::new(&pool);

        // Log a failed access
        repo.log_access(
            "acc-fail",
            "openai_api_key",
            "read",
            Some("skill_worker"),
            false,
            Some("Permission denied"),
        )
        .await
        .unwrap();

        let failed = repo.get_failed_accesses(10).await.unwrap();
        assert!(!failed.is_empty());
        assert_eq!(failed[0].success, 0);
    }
}
