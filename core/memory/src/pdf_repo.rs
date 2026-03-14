use sqlx::SqlitePool;
use sqlx::FromRow;
use serde::{Deserialize, Serialize};

/// PDF document record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PdfDocument {
    pub id: String,
    pub file_name: String,
    pub file_hash: String,
    pub file_size: i64,
    pub page_count: Option<i32>,
    pub extracted_text: Option<String>,
    pub metadata: Option<String>,
    pub provider: String,
    pub model_used: Option<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
}

/// PDF extraction job record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PdfExtractionJob {
    pub id: String,
    pub document_id: String,
    pub status: String,
    pub provider: String,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// PDF Repository for managing PDF documents
pub struct PdfRepository {
    pool: SqlitePool,
}

impl PdfRepository {
    pub fn new(pool: &SqlitePool) -> Self {
        Self { pool: pool.clone() }
    }

    /// Create a new PDF document record
    pub async fn create_document(
        &self,
        id: &str,
        file_name: &str,
        file_hash: &str,
        file_size: i64,
        provider: &str,
    ) -> Result<PdfDocument, sqlx::Error> {
        sqlx::query_as::<_, PdfDocument>(
            r#"
            INSERT INTO pdf_documents (id, file_name, file_hash, file_size, provider)
            VALUES (?, ?, ?, ?, ?)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(file_name)
        .bind(file_hash)
        .bind(file_size)
        .bind(provider)
        .fetch_one(&self.pool)
        .await
    }

    /// Get a PDF document by ID
    pub async fn get_document(&self, id: &str) -> Result<PdfDocument, sqlx::Error> {
        sqlx::query_as::<_, PdfDocument>(
            "SELECT * FROM pdf_documents WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    /// Get a PDF document by file hash (for deduplication)
    pub async fn get_document_by_hash(&self, file_hash: &str) -> Result<Option<PdfDocument>, sqlx::Error> {
        sqlx::query_as::<_, PdfDocument>(
            "SELECT * FROM pdf_documents WHERE file_hash = ?"
        )
        .bind(file_hash)
        .fetch_optional(&self.pool)
        .await
    }

    /// Update PDF document with extracted text
    pub async fn update_extracted_text(
        &self,
        id: &str,
        page_count: Option<i32>,
        extracted_text: &str,
        metadata: Option<&str>,
        model_used: Option<&str>,
    ) -> Result<PdfDocument, sqlx::Error> {
        sqlx::query_as::<_, PdfDocument>(
            r#"
            UPDATE pdf_documents
            SET page_count = ?, extracted_text = ?, metadata = ?, model_used = ?
            WHERE id = ?
            RETURNING *
            "#
        )
        .bind(page_count)
        .bind(extracted_text)
        .bind(metadata)
        .bind(model_used)
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    /// Delete a PDF document
    pub async fn delete_document(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM pdf_documents WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// List all PDF documents
    pub async fn list_documents(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<PdfDocument>, sqlx::Error> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        sqlx::query_as::<_, PdfDocument>(
            "SELECT * FROM pdf_documents ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    /// Create a PDF extraction job
    pub async fn create_job(
        &self,
        id: &str,
        document_id: &str,
        provider: &str,
    ) -> Result<PdfExtractionJob, sqlx::Error> {
        sqlx::query_as::<_, PdfExtractionJob>(
            r#"
            INSERT INTO pdf_extraction_jobs (id, document_id, provider)
            VALUES (?, ?, ?)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(document_id)
        .bind(provider)
        .fetch_one(&self.pool)
        .await
    }

    /// Get a job by ID
    pub async fn get_job(&self, id: &str) -> Result<PdfExtractionJob, sqlx::Error> {
        sqlx::query_as::<_, PdfExtractionJob>(
            "SELECT * FROM pdf_extraction_jobs WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    /// Update job status
    pub async fn update_job_status(
        &self,
        id: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<PdfExtractionJob, sqlx::Error> {
        let completed_at = if status == "completed" || status == "failed" {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };

        sqlx::query_as::<_, PdfExtractionJob>(
            r#"
            UPDATE pdf_extraction_jobs
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

    /// Get jobs for a document
    pub async fn get_jobs_for_document(&self, document_id: &str) -> Result<Vec<PdfExtractionJob>, sqlx::Error> {
        sqlx::query_as::<_, PdfExtractionJob>(
            "SELECT * FROM pdf_extraction_jobs WHERE document_id = ? ORDER BY started_at DESC"
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Clean up expired PDF documents
    pub async fn cleanup_expired(&self) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM pdf_documents 
            WHERE expires_at IS NOT NULL AND expires_at < datetime('now')
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}
