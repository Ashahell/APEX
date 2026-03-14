use sqlx::SqlitePool;
use sqlx::FromRow;
use serde::{Deserialize, Serialize};

/// Memory embedding record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MemoryEmbedding {
    pub id: String,
    pub memory_id: String,
    pub memory_type: String,
    pub modality: String,
    pub embedding: String,
    pub embedding_model: String,
    pub original_data: Option<String>,
    pub mime_type: Option<String>,
    pub created_at: String,
}

/// Memory indexing job
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MemoryIndexingJob {
    pub id: String,
    pub memory_id: String,
    pub modality: String,
    pub status: String,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// Multimodal configuration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MultimodalConfig {
    pub id: String,
    pub image_indexing: i32,
    pub audio_indexing: i32,
    pub embedding_model: String,
    pub embedding_dim: i32,
    pub enabled: i32,
    pub updated_at: String,
}

/// Search result with modality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalSearchResult {
    pub memory_id: String,
    pub memory_type: String,
    pub modality: String,
    pub original_data: Option<String>,
    pub mime_type: Option<String>,
    pub score: f32,
    pub created_at: String,
}

/// Multimodal Repository
pub struct MultimodalRepository {
    pool: SqlitePool,
}

impl MultimodalRepository {
    pub fn new(pool: &SqlitePool) -> Self {
        Self { pool: pool.clone() }
    }

    // ============ Embeddings ============

    /// Create a memory embedding
    pub async fn create_embedding(
        &self,
        id: &str,
        memory_id: &str,
        memory_type: &str,
        modality: &str,
        embedding: &[f32],
        embedding_model: &str,
        original_data: Option<&str>,
        mime_type: Option<&str>,
    ) -> Result<MemoryEmbedding, sqlx::Error> {
        let embedding_json = serde_json::to_string(embedding).unwrap_or_default();
        
        sqlx::query_as::<_, MemoryEmbedding>(
            r#"
            INSERT INTO memory_embeddings (id, memory_id, memory_type, modality, embedding, embedding_model, original_data, mime_type)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(memory_id)
        .bind(memory_type)
        .bind(modality)
        .bind(embedding_json)
        .bind(embedding_model)
        .bind(original_data)
        .bind(mime_type)
        .fetch_one(&self.pool)
        .await
    }

    /// Get embeddings for a memory item
    pub async fn get_embeddings_for_memory(&self, memory_id: &str) -> Result<Vec<MemoryEmbedding>, sqlx::Error> {
        sqlx::query_as::<_, MemoryEmbedding>(
            "SELECT * FROM memory_embeddings WHERE memory_id = ? ORDER BY created_at DESC"
        )
        .bind(memory_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Get embeddings by modality
    pub async fn get_embeddings_by_modality(&self, modality: &str, limit: i64) -> Result<Vec<MemoryEmbedding>, sqlx::Error> {
        sqlx::query_as::<_, MemoryEmbedding>(
            "SELECT * FROM memory_embeddings WHERE modality = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(modality)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Count embeddings by modality
    pub async fn count_by_modality(&self) -> Result<Vec<(String, i64)>, sqlx::Error> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT modality, COUNT(*) as count FROM memory_embeddings GROUP BY modality"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows)
    }

    /// Delete embeddings for a memory item
    pub async fn delete_embeddings_for_memory(&self, memory_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM memory_embeddings WHERE memory_id = ?")
            .bind(memory_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ============ Indexing Jobs ============

    /// Create an indexing job
    pub async fn create_indexing_job(
        &self,
        id: &str,
        memory_id: &str,
        modality: &str,
    ) -> Result<MemoryIndexingJob, sqlx::Error> {
        sqlx::query_as::<_, MemoryIndexingJob>(
            r#"
            INSERT INTO memory_indexing_jobs (id, memory_id, modality)
            VALUES (?, ?, ?)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(memory_id)
        .bind(modality)
        .fetch_one(&self.pool)
        .await
    }

    /// Update job status
    pub async fn update_job_status(
        &self,
        id: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<MemoryIndexingJob, sqlx::Error> {
        let completed_at = if status == "completed" || status == "failed" {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };

        sqlx::query_as::<_, MemoryIndexingJob>(
            r#"
            UPDATE memory_indexing_jobs
            SET status = ?, error_message = ?, completed_at = ?
            WHERE id = ?
            RETURNING *
            "#
        )
        .bind(status)
        .bind(error_message)
        .bind(completed_at)
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    /// Get job by ID
    pub async fn get_job(&self, id: &str) -> Result<MemoryIndexingJob, sqlx::Error> {
        sqlx::query_as::<_, MemoryIndexingJob>(
            "SELECT * FROM memory_indexing_jobs WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    /// Get pending jobs
    pub async fn get_pending_jobs(&self, limit: i64) -> Result<Vec<MemoryIndexingJob>, sqlx::Error> {
        sqlx::query_as::<_, MemoryIndexingJob>(
            "SELECT * FROM memory_indexing_jobs WHERE status = 'pending' ORDER BY started_at ASC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // ============ Configuration ============

    /// Get multimodal config
    pub async fn get_config(&self) -> Result<MultimodalConfig, sqlx::Error> {
        sqlx::query_as::<_, MultimodalConfig>(
            "SELECT * FROM memory_multimodal_config WHERE id = 'default'"
        )
        .fetch_one(&self.pool)
        .await
    }

    /// Update multimodal config
    pub async fn update_config(
        &self,
        image_indexing: Option<bool>,
        audio_indexing: Option<bool>,
        embedding_model: Option<&str>,
        embedding_dim: Option<i32>,
        enabled: Option<bool>,
    ) -> Result<MultimodalConfig, sqlx::Error> {
        let current = self.get_config().await?;
        
        sqlx::query_as::<_, MultimodalConfig>(
            r#"
            UPDATE memory_multimodal_config
            SET image_indexing = ?, audio_indexing = ?, embedding_model = ?, embedding_dim = ?, enabled = ?, updated_at = datetime('now')
            WHERE id = 'default'
            RETURNING *
            "#
        )
        .bind(image_indexing.unwrap_or(current.image_indexing == 1) as i32)
        .bind(audio_indexing.unwrap_or(current.audio_indexing == 1) as i32)
        .bind(embedding_model.unwrap_or(&current.embedding_model))
        .bind(embedding_dim.unwrap_or(current.embedding_dim))
        .bind(enabled.unwrap_or(current.enabled == 1) as i32)
        .fetch_one(&self.pool)
        .await
    }

    // ============ Stats ============

    /// Get embedding stats
    pub async fn get_stats(&self) -> Result<MultimodalStats, sqlx::Error> {
        let total_embeddings: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM memory_embeddings"
        )
        .fetch_one(&self.pool)
        .await?;

        let image_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM memory_embeddings WHERE modality = 'image'"
        )
        .fetch_one(&self.pool)
        .await?;

        let audio_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM memory_embeddings WHERE modality = 'audio'"
        )
        .fetch_one(&self.pool)
        .await?;

        let text_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM memory_embeddings WHERE modality = 'text'"
        )
        .fetch_one(&self.pool)
        .await?;

        let pending_jobs: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM memory_indexing_jobs WHERE status = 'pending'"
        )
        .fetch_one(&self.pool)
        .await?;

        let processing_jobs: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM memory_indexing_jobs WHERE status = 'processing'"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(MultimodalStats {
            total_embeddings: total_embeddings.0,
            image_embeddings: image_count.0,
            audio_embeddings: audio_count.0,
            text_embeddings: text_count.0,
            pending_jobs: pending_jobs.0,
            processing_jobs: processing_jobs.0,
        })
    }
}

/// Stats for multimodal memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalStats {
    pub total_embeddings: i64,
    pub image_embeddings: i64,
    pub audio_embeddings: i64,
    pub text_embeddings: i64,
    pub pending_jobs: i64,
    pub processing_jobs: i64,
}
