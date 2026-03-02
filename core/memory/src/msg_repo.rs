use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: String,
    pub task_id: Option<String>,
    pub channel: String,
    pub thread_id: Option<String>,
    pub author: String,
    pub content: String,
    pub role: String,
    pub attachments: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessage {
    pub task_id: Option<String>,
    pub channel: String,
    pub thread_id: Option<String>,
    pub author: String,
    pub content: String,
    pub role: String,
    pub attachments: Option<String>,
}

pub struct MessageRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> MessageRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, id: &str, msg: CreateMessage) -> Result<Message, sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO messages (id, task_id, channel, thread_id, author, content, role, attachments, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(&msg.task_id)
        .bind(&msg.channel)
        .bind(&msg.thread_id)
        .bind(&msg.author)
        .bind(&msg.content)
        .bind(&msg.role)
        .bind(&msg.attachments)
        .bind(now)
        .execute(self.pool)
        .await?;

        self.find_by_id(id).await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Message, sqlx::Error> {
        sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn find_by_channel(&self, channel: &str, limit: i64, offset: i64) -> Result<Vec<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE channel = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(channel)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
    }

    pub async fn find_by_task(&self, task_id: &str) -> Result<Vec<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE task_id = ? ORDER BY created_at ASC"
        )
        .bind(task_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn find_recent(&self, limit: i64) -> Result<Vec<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            "SELECT * FROM messages ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(self.pool)
        .await
    }
}
