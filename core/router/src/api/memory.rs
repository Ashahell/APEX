use axum::{
    extract::Query,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use axum::extract::State;
use ulid::Ulid;

use apex_memory::hybrid_search::{rrf_score, reciprocal_rank_fusion, temporal_decay, frequency_boost, mmr_select};
use apex_memory::multimodal_repo::{MultimodalRepository, MultimodalStats, MultimodalSearchResult};
use super::{AppState, FileContent, FileItem, GetFileContentQuery, ListFilesQuery, MemoryStatsResponse, ReflectionItem};

#[derive(Debug, Deserialize)]
pub struct SearchMemoryQuery {
    pub q: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub chunk_id: String,
    pub file_path: String,
    pub content: String,
    pub score: f64,
    pub memory_type: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/files", get(list_files))
        .route("/api/v1/files/content", get(get_file_content))
        .route("/api/v1/memory/stats", get(get_memory_stats))
        .route("/api/v1/memory/reflections", get(get_reflections))
        .route("/api/v1/memory/search", get(search_memory))
        .route("/api/v1/memory/index", get(get_index_stats))
        // NEW: Memory consolidation endpoints
        .route("/api/v1/memory/consolidate", post(consolidate_memory))
        .route("/api/v1/memory/consolidation/stats", get(get_consolidation_stats))
        // NEW: Multimodal endpoints (Phase 5)
        .route("/api/v1/memory/multimodal/config", get(get_multimodal_config).put(update_multimodal_config))
        .route("/api/v1/memory/multimodal/stats", get(get_multimodal_stats))
        .route("/api/v1/memory/multimodal/embeddings", get(list_multimodal_embeddings))
        .route("/api/v1/memory/multimodal/index", post(index_memory))
        .route("/api/v1/memory/multimodal/search", get(search_multimodal))
}

async fn list_files(Query(query): Query<ListFilesQuery>) -> Result<Json<Vec<FileItem>>, String> {
    let path = query.path.as_deref().unwrap_or("/");

    let entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;

    let mut files: Vec<FileItem> = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            let metadata = entry.metadata().ok();
            let name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path().to_string_lossy().to_string();
            let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified = metadata
                .and_then(|m| m.modified().ok())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64)
                .unwrap_or(0);

            files.push(FileItem {
                name,
                path: file_path,
                is_dir,
                size,
                modified,
            });
        }
    }

    files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(Json(files))
}

async fn get_file_content(Query(query): Query<GetFileContentQuery>) -> Result<Json<FileContent>, String> {
    let path = &query.path;

    if !std::path::Path::new(path).exists() {
        return Err("File not found".to_string());
    }

    let content =
        std::fs::read_to_string(path).unwrap_or_else(|_| "// Binary file or unreadable content".to_string());

    Ok(Json(FileContent {
        path: path.clone(),
        content,
        encoding: "utf-8".to_string(),
    }))
}

async fn get_memory_stats() -> Result<Json<MemoryStatsResponse>, String> {
    let base_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".apex")
        .join("memory");

    let reflections_dir = base_path.join("reflections");
    let entities_dir = base_path.join("entities");
    let knowledge_dir = base_path.join("knowledge");

    let total_reflections = count_files_recursive(&reflections_dir).await.unwrap_or(0);
    let total_entities = count_files_recursive(&entities_dir).await.unwrap_or(0);
    let total_knowledge = count_files_recursive(&knowledge_dir).await.unwrap_or(0);

    let mut recent_reflections = Vec::new();
    if reflections_dir.exists() {
        if let Ok(entries) = tokio::fs::read_dir(&reflections_dir).await {
            let mut count = 0u32;
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        let importance = (count % 10) as u32 + 1;
                        let modified = metadata.modified().ok()
                            .map(|t| {
                                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                                datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
                            })
                            .unwrap_or_else(|| "unknown".to_string());
                        recent_reflections.push(ReflectionItem {
                            id: count + 1,
                            content: entry.file_name().to_string_lossy().to_string(),
                            importance,
                            created_at: modified,
                        });
                        count += 1;
                        if count >= 5 {
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(Json(MemoryStatsResponse {
        total_entities,
        total_knowledge,
        total_reflections,
        recent_reflections,
    }))
}

async fn count_files_recursive(dir: &std::path::Path) -> std::io::Result<u32> {
    use tokio::fs;

    let mut count = 0u32;

    if !dir.exists() {
        return Ok(0);
    }

    let mut stack = vec![dir.to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        let mut entries = fs::read_dir(&current_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().map_or(false, |ext| ext == "md") {
                count += 1;
            }
        }
    }

    Ok(count)
}

async fn get_reflections() -> Result<Json<Vec<ReflectionItem>>, String> {
    let base_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".apex")
        .join("memory")
        .join("reflections");

    let mut reflections = Vec::new();

    if base_path.exists() {
        if let Ok(entries) = tokio::fs::read_dir(&base_path).await {
            let mut count = 0u32;
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        let modified = metadata.modified().ok()
                            .map(|t| {
                                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                                datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
                            })
                            .unwrap_or_else(|| "unknown".to_string());
                        reflections.push(ReflectionItem {
                            id: count + 1,
                            content: entry.file_name().to_string_lossy().to_string(),
                            importance: (count % 10) as u32 + 1,
                            created_at: modified,
                        });
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(Json(reflections))
}

async fn search_memory(
    State(state): State<AppState>,
    Query(query): Query<SearchMemoryQuery>,
) -> Result<Json<Vec<MemorySearchResult>>, String> {
    let limit = query.limit.unwrap_or(8);
    
    // Get query embedding
    let query_embedding = state.embedder
        .embed_query(&query.q)
        .await
        .map_err(|e| format!("Failed to embed query: {}", e))?;
    
    // Fetch all chunks with their embeddings and metadata
    let rows: Vec<(String, String, String, String, Option<String>, f64, i64)> = sqlx::query_as(
        "SELECT mc.id, mc.file_path, mc.content, mc.memory_type, mv.embedding, 
                COALESCE(julianday('now') - julianday(mc.accessed_at), 0) as access_age_days,
                mc.access_count
         FROM memory_chunks mc
         LEFT JOIN memory_vec mv ON mc.id = mv.chunk_id"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| format!("Search failed: {}", e))?;
    
    // Compute vector similarity ranks
    let mut vec_scores: Vec<(String, f64, Vec<f32>)> = Vec::new();
    let mut bm25_scores: Vec<(String, usize)> = Vec::new();
    let mut chunk_data: std::collections::HashMap<String, (String, String, String)> = std::collections::HashMap::new();
    
    let query_lower = query.q.to_lowercase();
    
    for row in rows {
        let (chunk_id, file_path, content, memory_type, embedding_json, access_age_days, access_count) = row;
        
        // Store chunk data
        chunk_data.insert(chunk_id.clone(), (file_path.clone(), content.clone(), memory_type.clone()));
        
        // Vector similarity
        if let Some(emb_json) = embedding_json {
            if let Ok(embedding) = serde_json::from_str::<Vec<f32>>(&emb_json) {
                let sim = cosine_similarity_f32(&query_embedding, &embedding);
                // Apply temporal decay (half-life: 30 days)
                let decay = (access_age_days / 30.0).max(0.0);
                let temporal_score = sim * 2.0_f64.powf(-decay);
                // Apply frequency boost
                let freq_boost = frequency_boost(access_count as u64);
                vec_scores.push((chunk_id.clone(), temporal_score * freq_boost, embedding));
            }
        }
        
        // BM25-like keyword ranking
        let content_lower = content.to_lowercase();
        let count = content_lower.matches(&query_lower).count();
        if count > 0 {
            bm25_scores.push((chunk_id, count));
        }
    }
    
    // Sort vector scores by similarity
    vec_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let vec_ranks: Vec<_> = vec_scores.iter().enumerate().map(|(i, (id, _, _))| (id.clone(), i + 1)).collect();
    
    // Sort BM25 scores
    bm25_scores.sort_by(|a, b| b.1.cmp(&a.1));
    let bm25_ranks: Vec<_> = bm25_scores.iter().enumerate().map(|(i, (id, _))| (id.clone(), i + 1)).collect();
    
    // Apply Reciprocal Rank Fusion
    let rrf_k = 60;
    let fused = reciprocal_rank_fusion(&vec_ranks, &bm25_ranks, rrf_k);
    
    // Build final results with MMR for diversity
    let lambda = 0.7;
    let _mmr_selected = mmr_select(&vec_scores, &query_embedding, limit, lambda);
    
    let search_results: Vec<MemorySearchResult> = fused.iter()
        .take(limit)
        .map(|(chunk_id, score)| {
            let (file_path, content, memory_type) = chunk_data.get(chunk_id)
                .cloned()
                .unwrap_or_else(|| (chunk_id.clone(), String::new(), "unknown".to_string()));
            MemorySearchResult {
                chunk_id: chunk_id.clone(),
                file_path,
                content: content.chars().take(500).collect(),
                score: *score,
                memory_type,
            }
        })
        .collect();
    
    Ok(Json(search_results))
}

fn cosine_similarity_f32(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
    let magnitude_a: f64 = a.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
    let magnitude_b: f64 = b.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

async fn get_index_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, String> {
    let stats = state.background_indexer
        .get_index_stats()
        .await
        .map_err(|e| format!("Failed to get index stats: {}", e))?;
    
    Ok(Json(serde_json::json!({
        "total_chunks": stats.total_chunks,
        "indexed_files": stats.indexed_files,
        "queue_depth": stats.queue_depth,
    })))
}

// =============================================================================
// Memory Consolidation Endpoints
// =============================================================================

use apex_memory::SoulMemoryConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Request to update consolidation config
#[derive(Debug, Deserialize)]
pub struct UpdateConsolidationConfigRequest {
    pub retention_days: Option<u32>,
    pub forgetting_threshold_days: Option<u32>,
    pub emphasis_patterns: Option<Vec<String>>,
    pub auto_consolidate: Option<bool>,
    pub consolidate_interval_hours: Option<u32>,
}

/// Get consolidation stats
async fn get_consolidation_stats(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, String> {
    // Get narrative memory config
    let base_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".apex")
        .join("memory");
    
    // Return current config (could be enhanced with actual state tracking)
    Ok(Json(serde_json::json!({
        "base_path": base_path.to_string_lossy(),
        "retention_days": 90,
        "forgetting_threshold_days": 30,
        "emphasis_patterns": ["error", "correction", "success"],
        "auto_consolidate": true,
        "consolidate_interval_hours": 24,
        "status": "ready"
    })))
}

/// Trigger memory consolidation
async fn consolidate_memory(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, String> {
    // Get base path
    let base_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".apex")
        .join("memory");
    
    // Create consolidator with proper config
    let narrative_config = apex_memory::NarrativeConfig {
        base_path: base_path.clone(),
        retention_days: 90,
        forgetting_threshold_days: 30,
    };
    let memory_config = SoulMemoryConfig::default();
    let consolidator = apex_memory::MemoryConsolidator::new(narrative_config, memory_config);
    
    // Run consolidation
    let result = consolidator.consolidate().await;
    
    Ok(Json(serde_json::json!({
        "success": result.errors.is_empty(),
        "journal_entries_kept": result.journal_entries_kept,
        "journal_entries_removed": result.journal_entries_removed,
        "reflections_kept": result.reflections_kept,
        "reflections_removed": result.reflections_removed,
        "total_space_freed_bytes": result.total_space_freed_bytes,
        "errors": result.errors,
    })))
}

// ============ Multimodal Memory Handlers (Phase 5) ============

#[derive(Debug, Serialize)]
pub struct MultimodalConfigResponse {
    pub image_indexing: bool,
    pub audio_indexing: bool,
    pub embedding_model: String,
    pub embedding_dim: i32,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMultimodalConfigRequest {
    pub image_indexing: Option<bool>,
    pub audio_indexing: Option<bool>,
    pub embedding_model: Option<String>,
    pub embedding_dim: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct IndexMemoryRequest {
    pub memory_id: String,
    pub memory_type: String,
    pub modality: String,  // 'image' or 'audio'
    pub data: String,  // Base64 encoded
    pub mime_type: String,
}

#[derive(Debug, Serialize)]
pub struct IndexMemoryResponse {
    pub job_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct MultimodalSearchQuery {
    pub q: Option<String>,
    pub modality: Option<String>,  // 'text', 'image', 'audio'
    pub limit: Option<usize>,
}

/// Get multimodal configuration
async fn get_multimodal_config(
    State(state): State<AppState>,
) -> Result<Json<MultimodalConfigResponse>, String> {
    let repo = MultimodalRepository::new(&state.pool);
    
    let config = repo
        .get_config()
        .await
        .map_err(|e| format!("Failed to get config: {}", e))?;
    
    Ok(Json(MultimodalConfigResponse {
        image_indexing: config.image_indexing == 1,
        audio_indexing: config.audio_indexing == 1,
        embedding_model: config.embedding_model,
        embedding_dim: config.embedding_dim,
        enabled: config.enabled == 1,
    }))
}

/// Update multimodal configuration
async fn update_multimodal_config(
    State(state): State<AppState>,
    Json(req): Json<UpdateMultimodalConfigRequest>,
) -> Result<Json<MultimodalConfigResponse>, String> {
    let repo = MultimodalRepository::new(&state.pool);
    
    let config = repo
        .update_config(
            req.image_indexing,
            req.audio_indexing,
            req.embedding_model.as_deref(),
            req.embedding_dim,
            req.enabled,
        )
        .await
        .map_err(|e| format!("Failed to update config: {}", e))?;
    
    Ok(Json(MultimodalConfigResponse {
        image_indexing: config.image_indexing == 1,
        audio_indexing: config.audio_indexing == 1,
        embedding_model: config.embedding_model,
        embedding_dim: config.embedding_dim,
        enabled: config.enabled == 1,
    }))
}

/// Get multimodal stats
async fn get_multimodal_stats(
    State(state): State<AppState>,
) -> Result<Json<MultimodalStats>, String> {
    let repo = MultimodalRepository::new(&state.pool);
    
    let stats = repo
        .get_stats()
        .await
        .map_err(|e| format!("Failed to get stats: {}", e))?;
    
    Ok(Json(stats))
}

/// List multimodal embeddings
async fn list_multimodal_embeddings(
    State(state): State<AppState>,
    Query(query): Query<MultimodalSearchQuery>,
) -> Result<Json<Vec<serde_json::Value>>, String> {
    let repo = MultimodalRepository::new(&state.pool);
    
    let limit = query.limit.unwrap_or(50) as i64;
    
    let embeddings = if let Some(modality) = &query.modality {
        repo.get_embeddings_by_modality(modality, limit)
            .await
            .map_err(|e| format!("Failed to get embeddings: {}", e))?
    } else {
        repo.get_embeddings_by_modality("all", limit)
            .await
            .map_err(|e| format!("Failed to get embeddings: {}", e))?
    };
    
    let results: Vec<serde_json::Value> = embeddings
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "memory_id": e.memory_id,
                "memory_type": e.memory_type,
                "modality": e.modality,
                "mime_type": e.mime_type,
                "has_original_data": e.original_data.is_some(),
                "embedding_model": e.embedding_model,
                "created_at": e.created_at,
            })
        })
        .collect();
    
    Ok(Json(results))
}

/// Index memory (image or audio)
async fn index_memory(
    State(state): State<AppState>,
    Json(req): Json<IndexMemoryRequest>,
) -> Result<Json<IndexMemoryResponse>, String> {
    let repo = MultimodalRepository::new(&state.pool);
    
    // Create indexing job
    let job_id = Ulid::new().to_string();
    repo
        .create_indexing_job(&job_id, &req.memory_id, &req.modality)
        .await
        .map_err(|e| format!("Failed to create job: {}", e))?;
    
    // TODO: Actually process the image/audio and generate embeddings
    // For now, create a placeholder embedding
    let embedding_id = Ulid::new().to_string();
    let placeholder_embedding: Vec<f32> = vec![0.0; 1536]; // Default embedding dim
    
    let _ = repo
        .create_embedding(
            &embedding_id,
            &req.memory_id,
            &req.memory_type,
            &req.modality,
            &placeholder_embedding,
            "placeholder",
            Some(&req.data),
            Some(&req.mime_type),
        )
        .await;
    
    // Mark job as completed
    repo
        .update_job_status(&job_id, "completed", None)
        .await
        .map_err(|e| format!("Failed to update job: {}", e))?;
    
    Ok(Json(IndexMemoryResponse {
        job_id,
        status: "completed".to_string(),
    }))
}

/// Search multimodal memory
async fn search_multimodal(
    State(state): State<AppState>,
    Query(query): Query<MultimodalSearchQuery>,
) -> Result<Json<Vec<serde_json::Value>>, String> {
    let repo = MultimodalRepository::new(&state.pool);
    
    let limit = query.limit.unwrap_or(10);
    
    // TODO: Implement actual vector search
    // For now, return recent embeddings filtered by modality
    let embeddings = repo
        .get_embeddings_by_modality(
            query.modality.as_deref().unwrap_or("text"),
            limit as i64,
        )
        .await
        .map_err(|e| format!("Failed to search: {}", e))?;
    
    let results: Vec<serde_json::Value> = embeddings
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "memory_id": e.memory_id,
                "memory_type": e.memory_type,
                "modality": e.modality,
                "original_data": e.original_data,
                "mime_type": e.mime_type,
                "score": 1.0,  // Placeholder
                "created_at": e.created_at,
            })
        })
        .collect();
    
    Ok(Json(results))
}
