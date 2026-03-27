use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

/// Dashboard layout configuration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DashboardLayout {
    pub id: String,
    pub user_id: String,
    pub layout_config: String,
    pub theme: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Pinned message
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PinnedMessage {
    pub id: String,
    pub message_id: String,
    pub channel: String,
    pub thread_id: Option<String>,
    pub task_id: Option<String>,
    pub pinned_by: String,
    pub pin_note: Option<String>,
    pub pinned_at: String,
}

/// Chat bookmark
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatBookmark {
    pub id: String,
    pub message_id: String,
    pub channel: String,
    pub thread_id: Option<String>,
    pub bookmark_note: Option<String>,
    pub created_at: String,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionMetadata {
    pub id: String,
    pub session_id: String,
    pub model: Option<String>,
    pub thinking_level: String,
    pub verbose_level: String,
    pub fast_mode: i32,
    pub send_policy: String,
    pub activation_mode: String,
    pub group_policy: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Dashboard chat history entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DashboardChatHistory {
    pub id: String,
    pub session_id: String,
    pub message_id: String,
    pub channel: String,
    pub thread_id: Option<String>,
    pub author: String,
    pub content: String,
    pub role: String,
    pub metadata: Option<String>,
    pub created_at: String,
}

/// Command palette history entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CommandPaletteHistory {
    pub id: String,
    pub command: String,
    pub command_type: String,
    pub frequency: i32,
    pub last_used: String,
}

/// Dashboard export record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DashboardExport {
    pub id: String,
    pub session_id: String,
    pub export_format: String,
    pub export_range: String,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub file_path: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Dashboard repository for managing dashboard-related data
pub struct DashboardRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> DashboardRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    // ============ Dashboard Layout ============

    pub async fn get_layout(&self, user_id: &str) -> Result<Option<DashboardLayout>, sqlx::Error> {
        sqlx::query_as::<_, DashboardLayout>(
            "SELECT * FROM dashboard_layout WHERE user_id = ? ORDER BY updated_at DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn upsert_layout(
        &self,
        id: &str,
        user_id: &str,
        layout_config: &str,
        theme: &str,
    ) -> Result<DashboardLayout, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO dashboard_layout (id, user_id, layout_config, theme, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                layout_config = excluded.layout_config,
                theme = excluded.theme,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(layout_config)
        .bind(theme)
        .bind(&now)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.find_layout_by_id(id).await
    }

    pub async fn find_layout_by_id(&self, id: &str) -> Result<DashboardLayout, sqlx::Error> {
        sqlx::query_as::<_, DashboardLayout>("SELECT * FROM dashboard_layout WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
    }

    // ============ Pinned Messages ============

    pub async fn pin_message(
        &self,
        id: &str,
        message_id: &str,
        channel: &str,
        thread_id: Option<&str>,
        task_id: Option<&str>,
        pinned_by: &str,
        pin_note: Option<&str>,
    ) -> Result<PinnedMessage, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO pinned_messages (id, message_id, channel, thread_id, task_id, pinned_by, pin_note, pinned_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(message_id)
        .bind(channel)
        .bind(thread_id)
        .bind(task_id)
        .bind(pinned_by)
        .bind(pin_note)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.find_pinned_by_id(id).await
    }

    pub async fn find_pinned_by_id(&self, id: &str) -> Result<PinnedMessage, sqlx::Error> {
        sqlx::query_as::<_, PinnedMessage>("SELECT * FROM pinned_messages WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn find_pinned_by_message(
        &self,
        message_id: &str,
    ) -> Result<Option<PinnedMessage>, sqlx::Error> {
        sqlx::query_as::<_, PinnedMessage>("SELECT * FROM pinned_messages WHERE message_id = ?")
            .bind(message_id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn list_pinned(
        &self,
        channel: Option<&str>,
        limit: i64,
    ) -> Result<Vec<PinnedMessage>, sqlx::Error> {
        let query = match channel {
            Some(_) => {
                "SELECT * FROM pinned_messages WHERE channel = ? ORDER BY pinned_at DESC LIMIT ?"
            }
            None => "SELECT * FROM pinned_messages ORDER BY pinned_at DESC LIMIT ?",
        };

        match channel {
            Some(ch) => {
                sqlx::query_as::<_, PinnedMessage>(query)
                    .bind(ch)
                    .bind(limit)
                    .fetch_all(self.pool)
                    .await
            }
            None => {
                sqlx::query_as::<_, PinnedMessage>(query)
                    .bind(limit)
                    .fetch_all(self.pool)
                    .await
            }
        }
    }

    pub async fn unpin_message(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM pinned_messages WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    // ============ Chat Bookmarks ============

    pub async fn add_bookmark(
        &self,
        id: &str,
        message_id: &str,
        channel: &str,
        thread_id: Option<&str>,
        bookmark_note: Option<&str>,
    ) -> Result<ChatBookmark, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO chat_bookmarks (id, message_id, channel, thread_id, bookmark_note, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(message_id)
        .bind(channel)
        .bind(thread_id)
        .bind(bookmark_note)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.find_bookmark_by_id(id).await
    }

    pub async fn find_bookmark_by_id(&self, id: &str) -> Result<ChatBookmark, sqlx::Error> {
        sqlx::query_as::<_, ChatBookmark>("SELECT * FROM chat_bookmarks WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn list_bookmarks(
        &self,
        channel: Option<&str>,
        thread_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ChatBookmark>, sqlx::Error> {
        let mut sql = "SELECT * FROM chat_bookmarks WHERE 1=1".to_string();
        let mut bindings: Vec<Box<dyn sqlx::Encode<sqlx::Sqlite>>> = Vec::new();

        if let Some(ch) = channel {
            sql.push_str(" AND channel = ?");
            bindings.push(Box::new(ch.to_string()));
        }
        if let Some(tid) = thread_id {
            sql.push_str(" AND thread_id = ?");
            bindings.push(Box::new(tid.to_string()));
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ?");
        bindings.push(Box::new(limit));

        // For simplicity, we'll use a basic query without dynamic bindings
        // In production, you'd want to use a query builder
        sqlx::query_as::<_, ChatBookmark>(&sql)
            .bind(limit)
            .fetch_all(self.pool)
            .await
    }

    pub async fn delete_bookmark(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM chat_bookmarks WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    // ============ Session Metadata ============

    pub async fn upsert_session_metadata(
        &self,
        id: &str,
        session_id: &str,
        model: Option<&str>,
        thinking_level: Option<&str>,
        verbose_level: Option<&str>,
        fast_mode: Option<i32>,
        send_policy: Option<&str>,
        activation_mode: Option<&str>,
        group_policy: Option<&str>,
    ) -> Result<SessionMetadata, sqlx::Error> {
        let now = Utc::now();
        let thinking = thinking_level.unwrap_or("medium");
        let verbose = verbose_level.unwrap_or("default");
        let fast = fast_mode.unwrap_or(0);
        let send = send_policy.unwrap_or("async");
        let activation = activation_mode.unwrap_or("always");

        sqlx::query(
            r#"
            INSERT INTO session_metadata (id, session_id, model, thinking_level, verbose_level, fast_mode, send_policy, activation_mode, group_policy, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(session_id) DO UPDATE SET
                model = COALESCE(excluded.model, session_metadata.model),
                thinking_level = excluded.thinking_level,
                verbose_level = excluded.verbose_level,
                fast_mode = excluded.fast_mode,
                send_policy = excluded.send_policy,
                activation_mode = excluded.activation_mode,
                group_policy = COALESCE(excluded.group_policy, session_metadata.group_policy),
                updated_at = excluded.updated_at
            "#,
        )
        .bind(id)
        .bind(session_id)
        .bind(model)
        .bind(thinking)
        .bind(verbose)
        .bind(fast)
        .bind(send)
        .bind(activation)
        .bind(group_policy)
        .bind(&now)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.find_session_metadata(session_id).await
    }

    pub async fn find_session_metadata(
        &self,
        session_id: &str,
    ) -> Result<SessionMetadata, sqlx::Error> {
        sqlx::query_as::<_, SessionMetadata>("SELECT * FROM session_metadata WHERE session_id = ?")
            .bind(session_id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn list_sessions_metadata(
        &self,
        limit: i64,
    ) -> Result<Vec<SessionMetadata>, sqlx::Error> {
        sqlx::query_as::<_, SessionMetadata>(
            "SELECT * FROM session_metadata ORDER BY updated_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(self.pool)
        .await
    }

    // ============ Chat History ============

    pub async fn add_chat_message(
        &self,
        id: &str,
        session_id: &str,
        message_id: &str,
        channel: &str,
        thread_id: Option<&str>,
        author: &str,
        content: &str,
        role: &str,
        metadata: Option<&str>,
    ) -> Result<DashboardChatHistory, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO dashboard_chat_history (id, session_id, message_id, channel, thread_id, author, content, role, metadata, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(session_id)
        .bind(message_id)
        .bind(channel)
        .bind(thread_id)
        .bind(author)
        .bind(content)
        .bind(role)
        .bind(metadata)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.find_chat_message(id).await
    }

    pub async fn find_chat_message(&self, id: &str) -> Result<DashboardChatHistory, sqlx::Error> {
        sqlx::query_as::<_, DashboardChatHistory>(
            "SELECT * FROM dashboard_chat_history WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn search_chat(
        &self,
        query: &str,
        channel: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DashboardChatHistory>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);

        let sql = match channel {
            Some(_) => {
                "SELECT * FROM dashboard_chat_history WHERE content LIKE ? AND channel = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
            }
            None => {
                "SELECT * FROM dashboard_chat_history WHERE content LIKE ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
            }
        };

        match channel {
            Some(ch) => {
                sqlx::query_as::<_, DashboardChatHistory>(sql)
                    .bind(&search_pattern)
                    .bind(ch)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(self.pool)
                    .await
            }
            None => {
                sqlx::query_as::<_, DashboardChatHistory>(sql)
                    .bind(&search_pattern)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(self.pool)
                    .await
            }
        }
    }

    pub async fn get_chat_history(
        &self,
        session_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DashboardChatHistory>, sqlx::Error> {
        sqlx::query_as::<_, DashboardChatHistory>(
            "SELECT * FROM dashboard_chat_history WHERE session_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(session_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
    }

    // ============ Command Palette History ============

    pub async fn record_command(
        &self,
        id: &str,
        command: &str,
        command_type: &str,
    ) -> Result<CommandPaletteHistory, sqlx::Error> {
        let now = Utc::now();

        // Try to update existing
        let result = sqlx::query(
            r#"
            UPDATE command_palette_history 
            SET frequency = frequency + 1, last_used = ?
            WHERE command = ? AND command_type = ?
            "#,
        )
        .bind(&now)
        .bind(command)
        .bind(command_type)
        .execute(self.pool)
        .await?;

        // If no row was updated, insert new
        if result.rows_affected() == 0 {
            sqlx::query(
                r#"
                INSERT INTO command_palette_history (id, command, command_type, frequency, last_used)
                VALUES (?, ?, ?, 1, ?)
                "#,
            )
            .bind(id)
            .bind(command)
            .bind(command_type)
            .bind(&now)
            .execute(self.pool)
            .await?;
        }

        self.find_command_by_id(id).await
    }

    pub async fn find_command_by_id(&self, id: &str) -> Result<CommandPaletteHistory, sqlx::Error> {
        sqlx::query_as::<_, CommandPaletteHistory>(
            "SELECT * FROM command_palette_history WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn list_commands(
        &self,
        command_type: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CommandPaletteHistory>, sqlx::Error> {
        match command_type {
            Some(t) => {
                sqlx::query_as::<_, CommandPaletteHistory>(
                    "SELECT * FROM command_palette_history WHERE command_type = ? ORDER BY frequency DESC LIMIT ?",
                )
                .bind(t)
                .bind(limit)
                .fetch_all(self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, CommandPaletteHistory>(
                    "SELECT * FROM command_palette_history ORDER BY frequency DESC LIMIT ?",
                )
                .bind(limit)
                .fetch_all(self.pool)
                .await
            }
        }
    }

    // ============ Exports ============

    pub async fn create_export(
        &self,
        id: &str,
        session_id: &str,
        export_format: &str,
        export_range: &str,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<DashboardExport, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO dashboard_exports (id, session_id, export_format, export_range, date_from, date_to, status, created_at)
            VALUES (?, ?, ?, ?, ?, ?, 'pending', ?)
            "#,
        )
        .bind(id)
        .bind(session_id)
        .bind(export_format)
        .bind(export_range)
        .bind(date_from)
        .bind(date_to)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.find_export(id).await
    }

    pub async fn find_export(&self, id: &str) -> Result<DashboardExport, sqlx::Error> {
        sqlx::query_as::<_, DashboardExport>("SELECT * FROM dashboard_exports WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn update_export_status(
        &self,
        id: &str,
        status: &str,
        file_path: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE dashboard_exports 
            SET status = ?, file_path = ?, error_message = ?, completed_at = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(file_path)
        .bind(error_message)
        .bind(&now)
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_exports(
        &self,
        session_id: &str,
        limit: i64,
    ) -> Result<Vec<DashboardExport>, sqlx::Error> {
        sqlx::query_as::<_, DashboardExport>(
            "SELECT * FROM dashboard_exports WHERE session_id = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(session_id)
        .bind(limit)
        .fetch_all(self.pool)
        .await
    }
}
