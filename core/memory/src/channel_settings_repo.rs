use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

/// Extended channel settings for additional messaging platforms
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChannelSettings {
    pub id: String,
    pub channel_type: String,
    pub channel_id: String,
    pub settings: String, // JSON
    pub credentials_encrypted: Option<String>,
    pub is_enabled: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Channel message template
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChannelTemplate {
    pub id: String,
    pub channel_type: String,
    pub template_name: String,
    pub template_content: String, // JSON
    pub is_default: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Channel webhook
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChannelWebhook {
    pub id: String,
    pub channel_type: String,
    pub channel_id: String,
    pub webhook_url: String,
    pub events: String, // JSON array
    pub is_enabled: i32,
    pub created_at: String,
}

/// Channel settings repository
pub struct ChannelSettingsRepository {
    pool: Pool<Sqlite>,
}

impl ChannelSettingsRepository {
    pub fn new(pool: &Pool<Sqlite>) -> Self {
        Self { pool: pool.clone() }
    }

    // ============ Channel Settings ============

    /// Create channel settings
    pub async fn create_settings(
        &self,
        id: &str,
        channel_type: &str,
        channel_id: &str,
        settings: &str,
    ) -> Result<ChannelSettings, sqlx::Error> {
        sqlx::query_as::<_, ChannelSettings>(
            r#"
            INSERT INTO channel_settings (id, channel_type, channel_id, settings)
            VALUES (?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(channel_type)
        .bind(channel_id)
        .bind(settings)
        .fetch_one(&self.pool)
        .await
    }

    /// Get settings by ID
    pub async fn get_settings(&self, id: &str) -> Result<ChannelSettings, sqlx::Error> {
        sqlx::query_as::<_, ChannelSettings>("SELECT * FROM channel_settings WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    /// Get settings by channel type
    pub async fn get_by_type(
        &self,
        channel_type: &str,
    ) -> Result<Vec<ChannelSettings>, sqlx::Error> {
        sqlx::query_as::<_, ChannelSettings>(
            "SELECT * FROM channel_settings WHERE channel_type = ? ORDER BY created_at DESC",
        )
        .bind(channel_type)
        .fetch_all(&self.pool)
        .await
    }

    /// Get all enabled settings
    pub async fn get_enabled(&self) -> Result<Vec<ChannelSettings>, sqlx::Error> {
        sqlx::query_as::<_, ChannelSettings>(
            "SELECT * FROM channel_settings WHERE is_enabled = 1 ORDER BY channel_type",
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Update settings
    pub async fn update_settings(
        &self,
        id: &str,
        settings: &str,
        credentials_encrypted: Option<&str>,
    ) -> Result<ChannelSettings, sqlx::Error> {
        sqlx::query_as::<_, ChannelSettings>(
            r#"
            UPDATE channel_settings
            SET settings = ?, credentials_encrypted = ?, updated_at = datetime('now')
            WHERE id = ?
            RETURNING *
            "#,
        )
        .bind(settings)
        .bind(credentials_encrypted)
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    /// Toggle enabled status
    pub async fn toggle_enabled(
        &self,
        id: &str,
        enabled: bool,
    ) -> Result<ChannelSettings, sqlx::Error> {
        sqlx::query_as::<_, ChannelSettings>(
            r#"
            UPDATE channel_settings
            SET is_enabled = ?, updated_at = datetime('now')
            WHERE id = ?
            RETURNING *
            "#,
        )
        .bind(if enabled { 1 } else { 0 })
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    /// Delete settings
    pub async fn delete_settings(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM channel_settings WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// List all channel types available
    pub async fn list_channel_types(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT channel_type FROM channel_settings ORDER BY channel_type",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(t,)| t).collect())
    }

    // ============ Templates ============

    /// Create template
    pub async fn create_template(
        &self,
        id: &str,
        channel_type: &str,
        template_name: &str,
        template_content: &str,
    ) -> Result<ChannelTemplate, sqlx::Error> {
        sqlx::query_as::<_, ChannelTemplate>(
            r#"
            INSERT INTO channel_templates (id, channel_type, template_name, template_content)
            VALUES (?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(channel_type)
        .bind(template_name)
        .bind(template_content)
        .fetch_one(&self.pool)
        .await
    }

    /// Get templates by channel type
    pub async fn get_templates(
        &self,
        channel_type: &str,
    ) -> Result<Vec<ChannelTemplate>, sqlx::Error> {
        sqlx::query_as::<_, ChannelTemplate>(
            "SELECT * FROM channel_templates WHERE channel_type = ? ORDER BY template_name",
        )
        .bind(channel_type)
        .fetch_all(&self.pool)
        .await
    }

    /// Delete template
    pub async fn delete_template(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM channel_templates WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ============ Webhooks ============

    /// Create webhook
    pub async fn create_webhook(
        &self,
        id: &str,
        channel_type: &str,
        channel_id: &str,
        webhook_url: &str,
        events: &str,
    ) -> Result<ChannelWebhook, sqlx::Error> {
        sqlx::query_as::<_, ChannelWebhook>(
            r#"
            INSERT INTO channel_webhooks (id, channel_type, channel_id, webhook_url, events)
            VALUES (?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(channel_type)
        .bind(channel_id)
        .bind(webhook_url)
        .bind(events)
        .fetch_one(&self.pool)
        .await
    }

    /// Get webhooks by channel type
    pub async fn get_webhooks(
        &self,
        channel_type: &str,
    ) -> Result<Vec<ChannelWebhook>, sqlx::Error> {
        sqlx::query_as::<_, ChannelWebhook>(
            "SELECT * FROM channel_webhooks WHERE channel_type = ? ORDER BY created_at DESC",
        )
        .bind(channel_type)
        .fetch_all(&self.pool)
        .await
    }

    /// Toggle webhook
    pub async fn toggle_webhook(
        &self,
        id: &str,
        enabled: bool,
    ) -> Result<ChannelWebhook, sqlx::Error> {
        sqlx::query_as::<_, ChannelWebhook>(
            r#"
            UPDATE channel_webhooks
            SET is_enabled = ?
            WHERE id = ?
            RETURNING *
            "#,
        )
        .bind(if enabled { 1 } else { 0 })
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    /// Delete webhook
    pub async fn delete_webhook(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM channel_webhooks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// Supported channel types
pub const SUPPORTED_CHANNEL_TYPES: &[&str] = &[
    "signal",
    "irc",
    "matrix",
    "teams",
    "feishu",
    "line",
    "mattermost",
    "nostr",
    "synology",
    "webchat",
];

/// Get channel type display name
pub fn get_channel_display_name(channel_type: &str) -> &str {
    match channel_type {
        "signal" => "Signal",
        "irc" => "IRC",
        "matrix" => "Matrix",
        "teams" => "Microsoft Teams",
        "feishu" => "Feishu",
        "line" => "LINE",
        "mattermost" => "Mattermost",
        "nostr" => "Nostr",
        "synology" => "Synology Chat",
        "webchat" => "WebChat",
        _ => "Unknown",
    }
}

/// Get channel icon
pub fn get_channel_icon(channel_type: &str) -> &str {
    match channel_type {
        "signal" => "📱",
        "irc" => "💬",
        "matrix" => "🔷",
        "teams" => "📊",
        "feishu" => "🐦",
        "line" => "💚",
        "mattermost" => "💬",
        "nostr" => "⚡",
        "synology" => "🖥️",
        "webchat" => "🌐",
        _ => "❓",
    }
}
