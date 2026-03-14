use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Session yield log entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionYieldLog {
    pub id: String,
    pub parent_session: String,
    pub child_session: String,
    pub yield_reason: Option<String>,
    pub yield_payload: Option<String>,
    pub resumed_at: Option<String>,
    pub created_at: String,
}

/// Session resume history entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionResumeHistory {
    pub id: String,
    pub session_id: String,
    pub resumed_from: String,
    pub resume_type: String,
    pub context_summary: Option<String>,
    pub created_at: String,
}

/// Session attachment
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionAttachment {
    pub id: String,
    pub session_id: String,
    pub task_id: Option<String>,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub file_path: String,
    pub encoding: String,
    pub uploaded_by: String,
    pub created_at: String,
}

/// Session state for persistence
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionState {
    pub id: String,
    pub session_id: String,
    pub state_data: String,
    pub checkpoint_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Session checkpoint
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionCheckpoint {
    pub id: String,
    pub session_id: String,
    pub checkpoint_name: String,
    pub checkpoint_data: String,
    pub description: Option<String>,
    pub created_at: String,
}

/// Session control repository
pub struct SessionControlRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SessionControlRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    // ============ Session Yield ============

    pub async fn log_yield(
        &self,
        id: &str,
        parent_session: &str,
        child_session: &str,
        yield_reason: Option<&str>,
        yield_payload: Option<&str>,
    ) -> Result<SessionYieldLog, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO session_yield_log (id, parent_session, child_session, yield_reason, yield_payload, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(parent_session)
        .bind(child_session)
        .bind(yield_reason)
        .bind(yield_payload)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.get_yield_log(id).await
    }

    pub async fn get_yield_log(&self, id: &str) -> Result<SessionYieldLog, sqlx::Error> {
        sqlx::query_as::<_, SessionYieldLog>(
            "SELECT * FROM session_yield_log WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn get_session_yields(&self, session_id: &str) -> Result<Vec<SessionYieldLog>, sqlx::Error> {
        sqlx::query_as::<_, SessionYieldLog>(
            "SELECT * FROM session_yield_log WHERE parent_session = ? OR child_session = ? ORDER BY created_at DESC",
        )
        .bind(session_id)
        .bind(session_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn mark_yield_resumed(&self, id: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query("UPDATE session_yield_log SET resumed_at = ? WHERE id = ?")
            .bind(&now)
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    // ============ Session Resume ============

    pub async fn log_resume(
        &self,
        id: &str,
        session_id: &str,
        resumed_from: &str,
        resume_type: &str,
        context_summary: Option<&str>,
    ) -> Result<SessionResumeHistory, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO session_resume_history (id, session_id, resumed_from, resume_type, context_summary, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(session_id)
        .bind(resumed_from)
        .bind(resume_type)
        .bind(context_summary)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.get_resume_history(id).await
    }

    pub async fn get_resume_history(&self, id: &str) -> Result<SessionResumeHistory, sqlx::Error> {
        sqlx::query_as::<_, SessionResumeHistory>(
            "SELECT * FROM session_resume_history WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn get_session_resume_history(&self, session_id: &str) -> Result<Vec<SessionResumeHistory>, sqlx::Error> {
        sqlx::query_as::<_, SessionResumeHistory>(
            "SELECT * FROM session_resume_history WHERE session_id = ? ORDER BY created_at DESC",
        )
        .bind(session_id)
        .fetch_all(self.pool)
        .await
    }

    // ============ Session Attachments ============

    pub async fn add_attachment(
        &self,
        id: &str,
        session_id: &str,
        task_id: Option<&str>,
        file_name: &str,
        file_type: &str,
        file_size: i64,
        file_path: &str,
        encoding: &str,
        uploaded_by: &str,
    ) -> Result<SessionAttachment, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO session_attachments (id, session_id, task_id, file_name, file_type, file_size, file_path, encoding, uploaded_by, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(session_id)
        .bind(task_id)
        .bind(file_name)
        .bind(file_type)
        .bind(file_size)
        .bind(file_path)
        .bind(encoding)
        .bind(uploaded_by)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.get_attachment(id).await
    }

    pub async fn get_attachment(&self, id: &str) -> Result<SessionAttachment, sqlx::Error> {
        sqlx::query_as::<_, SessionAttachment>(
            "SELECT * FROM session_attachments WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn get_session_attachments(&self, session_id: &str) -> Result<Vec<SessionAttachment>, sqlx::Error> {
        sqlx::query_as::<_, SessionAttachment>(
            "SELECT * FROM session_attachments WHERE session_id = ? ORDER BY created_at DESC",
        )
        .bind(session_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn delete_attachment(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM session_attachments WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    // ============ Session State ============

    pub async fn save_session_state(
        &self,
        id: &str,
        session_id: &str,
        state_data: &str,
        checkpoint_id: Option<&str>,
    ) -> Result<SessionState, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO session_state (id, session_id, state_data, checkpoint_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(session_id) DO UPDATE SET
                state_data = excluded.state_data,
                checkpoint_id = COALESCE(excluded.checkpoint_id, session_state.checkpoint_id),
                updated_at = excluded.updated_at
            "#,
        )
        .bind(id)
        .bind(session_id)
        .bind(state_data)
        .bind(checkpoint_id)
        .bind(&now)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.get_session_state(session_id).await
    }

    pub async fn get_session_state(&self, session_id: &str) -> Result<SessionState, sqlx::Error> {
        sqlx::query_as::<_, SessionState>(
            "SELECT * FROM session_state WHERE session_id = ?",
        )
        .bind(session_id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete_session_state(&self, session_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM session_state WHERE session_id = ?")
            .bind(session_id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    // ============ Session Checkpoints ============

    pub async fn create_checkpoint(
        &self,
        id: &str,
        session_id: &str,
        checkpoint_name: &str,
        checkpoint_data: &str,
        description: Option<&str>,
    ) -> Result<SessionCheckpoint, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO session_checkpoints (id, session_id, checkpoint_name, checkpoint_data, description, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(session_id)
        .bind(checkpoint_name)
        .bind(checkpoint_data)
        .bind(description)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.get_checkpoint(id).await
    }

    pub async fn get_checkpoint(&self, id: &str) -> Result<SessionCheckpoint, sqlx::Error> {
        sqlx::query_as::<_, SessionCheckpoint>(
            "SELECT * FROM session_checkpoints WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn get_session_checkpoints(&self, session_id: &str) -> Result<Vec<SessionCheckpoint>, sqlx::Error> {
        sqlx::query_as::<_, SessionCheckpoint>(
            "SELECT * FROM session_checkpoints WHERE session_id = ? ORDER BY created_at DESC",
        )
        .bind(session_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn get_checkpoint_by_name(&self, session_id: &str, name: &str) -> Result<SessionCheckpoint, sqlx::Error> {
        sqlx::query_as::<_, SessionCheckpoint>(
            "SELECT * FROM session_checkpoints WHERE session_id = ? AND checkpoint_name = ?",
        )
        .bind(session_id)
        .bind(name)
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete_checkpoint(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM session_checkpoints WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
