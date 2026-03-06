use axum::{
    extract::Query,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use axum::extract::State;

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
                        recent_reflections.push(ReflectionItem {
                            id: count + 1,
                            content: entry.file_name().to_string_lossy().to_string(),
                            importance,
                            created_at: format!("2026-03-{:02}T10:00:00Z", (count % 28) + 1),
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
                        reflections.push(ReflectionItem {
                            id: count + 1,
                            content: entry.file_name().to_string_lossy().to_string(),
                            importance: (count % 10) as u32 + 1,
                            created_at: format!("2026-03-{:02}T10:00:00Z", (count % 28) + 1),
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
    
    // Fetch all chunks with their embeddings
    let rows: Vec<(String, String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT mc.id, mc.file_path, mc.content, mc.memory_type, mv.embedding
         FROM memory_chunks mc
         LEFT JOIN memory_vec mv ON mc.id = mv.chunk_id"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| format!("Search failed: {}", e))?;
    
    // Compute similarity scores and sort
    let mut scored_results: Vec<(String, String, String, f64, String)> = rows
        .into_iter()
        .filter_map(|(chunk_id, file_path, content, memory_type, embedding_json)| {
            if let Some(emb_json) = embedding_json {
                if let Ok(embedding) = serde_json::from_str::<Vec<f32>>(&emb_json) {
                    let sim = cosine_similarity_f32(&query_embedding, &embedding);
                    return Some((chunk_id, file_path, content, sim, memory_type));
                }
            }
            // Fallback: keyword match for chunks without embeddings
            let query_lower = query.q.to_lowercase();
            let content_lower = content.to_lowercase();
            let count = content_lower.matches(&query_lower).count();
            let score = (count as f64).min(10.0) / 10.0;
            Some((chunk_id, file_path, content, score, memory_type))
        })
        .collect();
    
    // Sort by score descending
    scored_results.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
    scored_results.truncate(limit);
    
    let search_results: Vec<MemorySearchResult> = scored_results
        .into_iter()
        .map(|(chunk_id, file_path, content, score, memory_type)| {
            MemorySearchResult {
                chunk_id,
                file_path,
                content: content.chars().take(500).collect(),
                score,
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
