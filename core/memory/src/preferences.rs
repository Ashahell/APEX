use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Preference {
    pub key: String,
    pub value: String,
    pub encrypted: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub provider: String,
    pub key: String,
}

pub struct PreferencesRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> PreferencesRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, key: &str) -> Result<Option<Preference>, sqlx::Error> {
        sqlx::query_as::<_, Preference>("SELECT * FROM preferences WHERE key = ?")
            .bind(key)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn set(&self, key: &str, value: &str, encrypt: bool) -> Result<(), sqlx::Error> {
        let final_value = if encrypt {
            Self::encode(value)
        } else {
            value.to_string()
        };

        sqlx::query(
            r#"
            INSERT INTO preferences (key, value, encrypted, updated_at)
            VALUES (?, ?, ?, datetime('now'))
            ON CONFLICT(key) DO UPDATE SET value = ?, encrypted = ?, updated_at = datetime('now')
            "#
        )
        .bind(key)
        .bind(&final_value)
        .bind(encrypt)
        .bind(&final_value)
        .bind(encrypt)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM preferences WHERE key = ?")
            .bind(key)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    fn encode(plaintext: &str) -> String {
        use base64::{engine::general_purpose::STANDARD, Engine};
        STANDARD.encode(plaintext)
    }

    pub fn decode(encoded: &str) -> String {
        use base64::{engine::general_purpose::STANDARD, Engine};
        STANDARD.decode(encoded)
            .map(|d| String::from_utf8(d).unwrap_or_else(|_| encoded.to_string()))
            .unwrap_or_else(|_| encoded.to_string())
    }
}
