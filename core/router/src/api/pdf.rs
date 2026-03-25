use axum::{
    extract::{State, Path, Multipart},
    extract::Query,
    routing::{get, post, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use std::sync::Arc;

use apex_memory::pdf_repo::{PdfDocument, PdfExtractionJob, PdfRepository};

use crate::api::AppState;

// ============ Router ============

pub fn router() -> Router<AppState> {
    Router::new()
        // PDF documents
        .route("/api/v1/pdf/documents", get(list_documents))
        .route("/api/v1/pdf/upload", post(upload_pdf))
        .route("/api/v1/pdf/:id", get(get_pdf))
        .route("/api/v1/pdf/:id", delete(delete_pdf))
        
        // Text extraction
        .route("/api/v1/pdf/:id/extract", post(extract_text))
        
        // Analysis
        .route("/api/v1/pdf/:id/analyze", post(analyze_pdf))
        
        // Jobs
        .route("/api/v1/pdf/jobs/:job_id", get(get_job_status))
}

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ExtractTextRequest {
    pub provider: Option<String>,  // 'anthropic', 'google', 'fallback'
}

#[derive(Debug, Deserialize)]
pub struct AnalyzePdfRequest {
    pub prompt: String,
    pub provider: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PdfDocumentResponse {
    pub id: String,
    pub file_name: String,
    pub file_hash: String,
    pub file_size: i64,
    pub page_count: Option<i32>,
    pub extracted_text: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub provider: String,
    pub model_used: Option<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
}

impl From<PdfDocument> for PdfDocumentResponse {
    fn from(doc: PdfDocument) -> Self {
        let metadata: Option<serde_json::Value> = doc.metadata
            .as_ref()
            .and_then(|m| serde_json::from_str(m).ok());
        
        Self {
            id: doc.id,
            file_name: doc.file_name,
            file_hash: doc.file_hash,
            file_size: doc.file_size,
            page_count: doc.page_count,
            extracted_text: doc.extracted_text,
            metadata,
            provider: doc.provider,
            model_used: doc.model_used,
            created_at: doc.created_at,
            expires_at: doc.expires_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PdfExtractResponse {
    pub document_id: String,
    pub text: String,
    pub page_count: i32,
    pub provider: String,
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PdfAnalyzeResponse {
    pub analysis: String,
    pub provider: String,
    pub model: String,
    pub tokens_used: i32,
}

#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
    pub id: String,
    pub document_id: String,
    pub status: String,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

impl From<PdfExtractionJob> for JobStatusResponse {
    fn from(job: PdfExtractionJob) -> Self {
        Self {
            id: job.id,
            document_id: job.document_id,
            status: job.status,
            error_message: job.error_message,
            started_at: job.started_at,
            completed_at: job.completed_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub document: PdfDocumentResponse,
    pub cached: bool,  // True if document was retrieved from cache
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ============ Handlers ============

// List all PDF documents
async fn list_documents(
    State(state): State<AppState>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<Vec<PdfDocumentResponse>>, String> {
    let repo = PdfRepository::new(&state.pool);
    
    let docs = repo
        .list_documents(query.limit, query.offset)
        .await
        .map_err(|e| format!("Failed to list documents: {}", e))?;
    
    Ok(Json(docs.into_iter().map(|d| d.into()).collect()))
}

// Upload a PDF
async fn upload_pdf(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, String> {
    let repo = PdfRepository::new(&state.pool);
    
    // Get the file from multipart
    let field = multipart
        .next_field()
        .await
        .map_err(|e| format!("Failed to read multipart: {}", e))?
        .ok_or("No file in request")?;
    
    let file_name = field
        .file_name()
        .ok_or("Missing file name")?
        .to_string();
    
    let data = field
        .bytes()
        .await
        .map_err(|e| format!("Failed to read file data: {}", e))?;
    
    let file_size = data.len() as i64;
    
    // Calculate SHA256 hash
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let file_hash = format!("{:x}", hasher.finish());
    
    // Check for cached document
    let cached = repo
        .get_document_by_hash(&file_hash)
        .await
        .map_err(|e| format!("Failed to check cache: {}", e))?;
    
    if let Some(cached_doc) = cached {
        return Ok(Json(UploadResponse {
            document: cached_doc.into(),
            cached: true,
        }));
    }
    
    // Create new document
    let id = Ulid::new().to_string();
    let document = repo
        .create_document(&id, &file_name, &file_hash, file_size, "fallback")
        .await
        .map_err(|e| format!("Failed to create document: {}", e))?;
    
    // NOTE: Async text extraction would be handled by background job queue
    // For now, text extraction happens on-demand in get_pdf
    
    Ok(Json(UploadResponse {
        document: document.into(),
        cached: false,
    }))
}

// Get a PDF document
async fn get_pdf(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PdfDocumentResponse>, String> {
    let repo = PdfRepository::new(&state.pool);
    
    let doc = repo
        .get_document(&id)
        .await
        .map_err(|e| format!("Failed to get document: {}", e))?;
    
    Ok(Json(doc.into()))
}

// Delete a PDF document
async fn delete_pdf(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = PdfRepository::new(&state.pool);
    
    repo
        .delete_document(&id)
        .await
        .map_err(|e| format!("Failed to delete document: {}", e))?;
    
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// Extract text from PDF
async fn extract_text(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ExtractTextRequest>,
) -> Result<Json<PdfExtractResponse>, String> {
    let repo = PdfRepository::new(&state.pool);
    
    // Get the document
    let doc = repo
        .get_document(&id)
        .await
        .map_err(|e| format!("Failed to get document: {}", e))?;
    
    // If already extracted, return cached
    if let Some(text) = &doc.extracted_text {
        return Ok(Json(PdfExtractResponse {
            document_id: doc.id,
            text: text.clone(),
            page_count: doc.page_count.unwrap_or(0),
            provider: doc.provider,
            model: doc.model_used,
        }));
    }
    
    // NOTE: Full PDF text extraction requires external OCR/service integration
    // For now, return placeholder
    let extracted_text = "PDF text extraction not yet implemented".to_string();
    let page_count = 0;
    
    // Update document with extracted text
    let _updated = repo
        .update_extracted_text(&id, Some(page_count), &extracted_text, None, Some("fallback"))
        .await
        .map_err(|e| format!("Failed to update document: {}", e))?;
    
    Ok(Json(PdfExtractResponse {
        document_id: id,
        text: extracted_text,
        page_count,
        provider: req.provider.unwrap_or_else(|| "fallback".to_string()),
        model: None,
    }))
}

// Analyze PDF with LLM
async fn analyze_pdf(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AnalyzePdfRequest>,
) -> Result<Json<PdfAnalyzeResponse>, String> {
    let repo = PdfRepository::new(&state.pool);
    
    // Get the document
    let doc = repo
        .get_document(&id)
        .await
        .map_err(|e| format!("Failed to get document: {}", e))?;
    
    // Get extracted text
    let text = doc.extracted_text
        .ok_or("PDF text not extracted yet. Call /extract first.")?;
    
    // NOTE: Full LLM analysis requires llama.rs integration
    // For now, return placeholder
    let analysis = format!(
        "Analysis of '{}':\n\nThis is a placeholder analysis. The PDF contains {} pages with {} characters.",
        doc.file_name,
        doc.page_count.unwrap_or(0),
        text.len()
    );
    
    let tokens_used = (analysis.len() / 4) as i32;
    
    Ok(Json(PdfAnalyzeResponse {
        analysis,
        provider: req.provider.unwrap_or_else(|| "fallback".to_string()),
        model: "placeholder".to_string(),
        tokens_used,
    }))
}

// Get job status
async fn get_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<JobStatusResponse>, String> {
    let repo = PdfRepository::new(&state.pool);
    
    let job = repo
        .get_job(&job_id)
        .await
        .map_err(|e| format!("Failed to get job: {}", e))?;
    
    Ok(Json(job.into()))
}
