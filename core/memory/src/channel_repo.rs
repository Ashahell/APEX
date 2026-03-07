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

#[derive(Debug)]
pub struct CreateChannel {
    pub name: String,
    pub description: Option<String>,
}

impl ChannelRepository {
    pub fn new(pool: &Pool<Sqlite>) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn find_all(&self) -> Result<Vec<Channel>, sqlx::Error> {
        sqlx::query_as::<_, Channel>(
            "SELECT id, name, description, created_at, updated_at FROM channels ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Channel>, sqlx::Error> {
        sqlx::query_as::<_, Channel>(
            "SELECT id, name, description, created_at, updated_at FROM channels WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Channel>, sqlx::Error> {
        sqlx::query_as::<_, Channel>(
            "SELECT id, name, description, created_at, updated_at FROM channels WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(&self, id: &str, channel: CreateChannel) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO channels (id, name, description) VALUES (?, ?, ?)"
        )
        .bind(id)
        .bind(&channel.name)
        .bind(&channel.description)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update(&self, id: &str, channel: CreateChannel) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE channels SET name = ?, description = ?, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(&channel.name)
        .bind(&channel.description)
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
