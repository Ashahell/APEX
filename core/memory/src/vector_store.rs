use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VectorEntry {
    pub id: i64,
    pub key: String,
    pub embedding: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub key: String,
    pub content: String,
    pub similarity: f64,
}

pub struct VectorStore<'a> {
    pool: &'a SqlitePool,
}

impl<'a> VectorStore<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn init(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS vector_store (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,
                embedding TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_vector_key ON vector_store(key);
            "#
        )
        .execute(self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn add(&self, key: &str, embedding: &[f64], content: &str) -> Result<(), sqlx::Error> {
        let embedding_json = serde_json::to_string(embedding).unwrap_or_default();
        
        sqlx::query(
            r#"
            INSERT INTO vector_store (key, embedding, content)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET embedding = ?, content = ?
            "#
        )
        .bind(key)
        .bind(&embedding_json)
        .bind(content)
        .bind(&embedding_json)
        .bind(content)
        .execute(self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn search(&self, query_embedding: &[f64], limit: usize) -> Result<Vec<SearchResult>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT key, embedding, content FROM vector_store"
        )
        .fetch_all(self.pool)
        .await?;

        let mut results: Vec<SearchResult> = rows
            .into_iter()
            .filter_map(|(key, embedding_json, content)| {
                let embedding: Vec<f64> = serde_json::from_str(&embedding_json).ok()?;
                let similarity = Self::cosine_similarity(query_embedding, &embedding);
                Some(SearchResult { key, content, similarity })
            })
            .collect();

        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        results.truncate(limit);

        Ok(results)
    }

    fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let magnitude_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
    }

    pub async fn get(&self, key: &str) -> Result<Option<VectorEntry>, sqlx::Error> {
        sqlx::query_as::<_, VectorEntry>(
            "SELECT * FROM vector_store WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn delete(&self, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM vector_store WHERE key = ?")
            .bind(key)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
