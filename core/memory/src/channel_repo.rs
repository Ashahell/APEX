use sqlx::{Pool, Sqlite};

pub struct ChannelRepository {
    pool: Pool<Sqlite>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl ChannelRepository {
    pub fn new(pool: &Pool<Sqlite>) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn find_all(&self) -> Result<Vec<Channel>, sqlx::Error> {
        sqlx::query_as::<_, Channel>("SELECT id, name, description, created_at, updated_at FROM channels ORDER BY name")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Channel>, sqlx::Error> {
        sqlx::query_as::<_, Channel>("SELECT id, name, description, created_at, updated_at FROM channels WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Channel>, sqlx::Error> {
        sqlx::query_as::<_, Channel>("SELECT id, name, description, created_at, updated_at FROM channels WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create(&self, id: &str, name: &str, description: Option<&str>) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO channels (id, name, description) VALUES (?, ?, ?)")
            .bind(id)
            .bind(name)
            .bind(description)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update(&self, id: &str, name: &str, description: Option<&str>) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE channels SET name = ?, description = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(name)
            .bind(description)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM channels WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_channel_create_and_find() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE channels (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, description TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')), updated_at TEXT NOT NULL DEFAULT (datetime('now')))")
            .execute(&pool)
            .await
            .unwrap();

        let repo = ChannelRepository::new(&pool);
        repo.create("test-1", "general", Some("General discussions"))
            .await
            .unwrap();

        let channels = repo.find_all().await.unwrap();
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].name, "general");
    }

    #[tokio::test]
    async fn test_channel_update() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE channels (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, description TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')), updated_at TEXT NOT NULL DEFAULT (datetime('now')))")
            .execute(&pool)
            .await
            .unwrap();

        let repo = ChannelRepository::new(&pool);
        repo.create("test-1", "general", Some("General"))
            .await
            .unwrap();

        repo.update("test-1", "general-updated", Some("Updated description"))
            .await
            .unwrap();

        let channel = repo.find_by_id("test-1").await.unwrap().unwrap();
        assert_eq!(channel.name, "general-updated");
    }

    #[tokio::test]
    async fn test_channel_delete() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE channels (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, description TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')), updated_at TEXT NOT NULL DEFAULT (datetime('now')))")
            .execute(&pool)
            .await
            .unwrap();

        let repo = ChannelRepository::new(&pool);
        repo.create("test-1", "general", Some("General"))
            .await
            .unwrap();

        repo.delete("test-1").await.unwrap();

        let channel = repo.find_by_id("test-1").await.unwrap();
        assert!(channel.is_none());
    }
}
