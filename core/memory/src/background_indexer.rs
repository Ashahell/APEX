use crate::chunker::{chunk_text, ChunkerConfig};
use crate::embedder::Embedder;
use crate::MemoryError;
use chrono::Utc;
use sqlx::{Pool, Sqlite};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct IndexerConfig {
    pub batch_size: usize,
    pub embed_rate_limit_ms: u64,
    pub chunk_config: ChunkerConfig,
    pub embedding_dim: usize,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            batch_size: 16,
            embed_rate_limit_ms: 50,
            chunk_config: ChunkerConfig::default(),
            embedding_dim: 768,
        }
    }
}

impl Clone for IndexerConfig {
    fn clone(&self) -> Self {
        Self {
            batch_size: self.batch_size,
            embed_rate_limit_ms: self.embed_rate_limit_ms,
            chunk_config: self.chunk_config.clone(),
            embedding_dim: self.embedding_dim,
        }
    }
}

enum IndexJob {
    File(PathBuf),
    Directory(PathBuf),
    Shutdown,
}

pub struct BackgroundIndexer {
    queue: mpsc::Sender<IndexJob>,
    embedder: Arc<Embedder>,
    pool: Pool<Sqlite>,
    config: IndexerConfig,
}

impl BackgroundIndexer {
    pub fn new(
        embedder: Arc<Embedder>,
        pool: Pool<Sqlite>,
        config: IndexerConfig,
    ) -> Self {
        let (tx, rx) = mpsc::channel(100);
        
        let embedder_clone = embedder.clone();
        let pool_clone = pool.clone();
        let config_clone = config.clone();

        tokio::spawn(async move {
            Self::run(rx, embedder_clone, pool_clone, config_clone).await;
        });

        Self {
            queue: tx,
            embedder,
            pool,
            config,
        }
    }

    pub async fn queue_file(&self, path: impl Into<PathBuf>) {
        let _ = self.queue.send(IndexJob::File(path.into())).await;
    }

    pub async fn initial_scan(&self, memory_dir: &Path) {
        let _ = self.queue.send(IndexJob::Directory(memory_dir.to_path_buf())).await;
    }

    async fn run(
        mut rx: mpsc::Receiver<IndexJob>,
        embedder: Arc<Embedder>,
        pool: Pool<Sqlite>,
        config: IndexerConfig,
    ) {
        while let Some(job) = rx.recv().await {
            match job {
                IndexJob::File(path) => {
                    Self::index_file(&path, &embedder, &pool, &config).await;
                }
                IndexJob::Directory(path) => {
                    Self::scan_directory(&path, &embedder, &pool, &config).await;
                }
                IndexJob::Shutdown => break,
            }
        }
    }

    async fn scan_directory(
        path: &Path,
        embedder: &Arc<Embedder>,
        pool: &Pool<Sqlite>,
        config: &IndexerConfig,
    ) {
        if !path.exists() {
            tracing::warn!("Indexer: directory does not exist: {:?}", path);
            return;
        }

        let mut dirs = vec![path.to_path_buf()];

        while let Some(dir) = dirs.pop() {
            let mut entries = match tokio::fs::read_dir(&dir).await {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("Indexer: failed to read dir {:?}: {}", dir, e);
                    continue;
                }
            };

            while let Some(entry_result) = entries.next_entry().await.ok().flatten() {
                let file_path = entry_result.path();
                if file_path.is_dir() {
                    dirs.push(file_path);
                } else if file_path.extension().map(|e| e == "md").unwrap_or(false) {
                    Self::index_file(&file_path, embedder, pool, config).await;
                }
            }
        }
    }

    async fn index_file(
        path: &Path,
        embedder: &Arc<Embedder>,
        pool: &Pool<Sqlite>,
        config: &IndexerConfig,
    ) {
        let mtime = tokio::fs::metadata(path)
            .await
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);

        if let Some(mtime) = mtime {
            if Self::is_up_to_date(path, mtime, pool).await {
                return;
            }
        }

        let text = match tokio::fs::read_to_string(path).await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("Indexer: failed to read {:?}: {}", path, e);
                return;
            }
        };

        let chunks = chunk_text(&text, &config.chunk_config);
        let chunk_count = chunks.len();

        Self::delete_chunks_for_file(path, pool).await;

        for (idx, chunk_text_content) in chunks {
            tokio::time::sleep(tokio::time::Duration::from_millis(config.embed_rate_limit_ms)).await;

            let embedding = match embedder.embed(&chunk_text_content).await {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("Indexer: embed failed for chunk {}: {}", idx, e);
                    continue;
                }
            };

            let chunk_id = Uuid::new_v4().to_string();
            let memory_type = Self::classify_path(path);

            if let Err(e) = sqlx::query(
                "INSERT OR REPLACE INTO memory_chunks
                 (id, file_path, chunk_index, content, word_count, memory_type)
                 VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(&chunk_id)
            .bind(path.to_string_lossy().to_string())
            .bind(idx as i64)
            .bind(&chunk_text_content)
            .bind(chunk_text_content.split_whitespace().count() as i64)
            .bind(&memory_type)
            .execute(pool)
            .await
            {
                tracing::warn!("Indexer: failed to insert chunk: {}", e);
                continue;
            }

            let embedding_json = serde_json::to_string(&embedding).unwrap_or_default();
            if let Err(e) = sqlx::query(
                "INSERT OR REPLACE INTO memory_vec (chunk_id, embedding) VALUES (?, ?)"
            )
            .bind(&chunk_id)
            .bind(&embedding_json)
            .execute(pool)
            .await
            {
                tracing::warn!("Indexer: failed to insert embedding: {}", e);
            }
        }

        if let Some(mtime) = mtime {
            Self::update_index_state(path, mtime, chunk_count, pool).await;
        }

        tracing::info!("Indexed {} with {} chunks", path.display(), chunk_count);
    }

    async fn is_up_to_date(path: &Path, mtime: i64, pool: &Pool<Sqlite>) -> bool {
        let result: Option<(i64, i32)> = sqlx::query_as(
            "SELECT mtime_unix, chunk_count FROM memory_index_state WHERE file_path = ?"
        )
        .bind(path.to_string_lossy().to_string())
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        if let Some((stored_mtime, _)) = result {
            return stored_mtime == mtime;
        }

        false
    }

    async fn delete_chunks_for_file(path: &Path, pool: &Pool<Sqlite>) {
        let _ = sqlx::query("DELETE FROM memory_chunks WHERE file_path = ?")
            .bind(path.to_string_lossy().to_string())
            .execute(pool)
            .await;
    }

    async fn update_index_state(path: &Path, mtime: i64, chunk_count: usize, pool: &Pool<Sqlite>) {
        let _ = sqlx::query(
            "INSERT OR REPLACE INTO memory_index_state (file_path, mtime_unix, chunk_count, indexed_at) VALUES (?, ?, ?, ?)"
        )
        .bind(path.to_string_lossy().to_string())
        .bind(mtime)
        .bind(chunk_count as i32)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await;
    }

    fn classify_path(path: &Path) -> String {
        let path_str = path.to_string_lossy().to_lowercase();
        
        if path_str.contains("journal") {
            "journal".to_string()
        } else if path_str.contains("knowledge") {
            "knowledge".to_string()
        } else if path_str.contains("entities") || path_str.contains("entity") {
            "entity".to_string()
        } else if path_str.contains("reflection") {
            "reflection".to_string()
        } else {
            "journal".to_string()
        }
    }

    pub async fn get_index_stats(&self) -> Result<IndexStats, MemoryError> {
        let total_chunks: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM memory_chunks")
            .fetch_one(&self.pool)
            .await
            .map_err(MemoryError::Database)?;

        let indexed_files: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT file_path) FROM memory_index_state")
            .fetch_one(&self.pool)
            .await
            .map_err(MemoryError::Database)?;

        Ok(IndexStats {
            total_chunks: total_chunks.0 as usize,
            indexed_files: indexed_files.0 as usize,
            queue_depth: 0,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexStats {
    pub total_chunks: usize,
    pub indexed_files: usize,
    pub queue_depth: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedder::EmbeddingProvider;
    use std::sync::Arc;

    fn create_test_embedder() -> Arc<Embedder> {
        let provider = EmbeddingProvider::Local {
            url: "http://localhost:8081".to_string(),
            model: "nomic-embed-text".to_string(),
        };
        Arc::new(Embedder::new(provider, 768))
    }

    #[tokio::test]
    async fn test_indexer_config_default() {
        let config = IndexerConfig::default();
        assert_eq!(config.batch_size, 16);
        assert_eq!(config.embed_rate_limit_ms, 50);
        assert_eq!(config.embedding_dim, 768);
    }

    #[tokio::test]
    async fn test_indexer_config_clone() {
        let config = IndexerConfig::default();
        let cloned = config.clone();
        assert_eq!(config.batch_size, cloned.batch_size);
        assert_eq!(config.embedding_dim, cloned.embedding_dim);
    }

    #[tokio::test]
    async fn test_index_stats_default() {
        let stats = IndexStats {
            total_chunks: 0,
            indexed_files: 0,
            queue_depth: 0,
        };
        assert_eq!(stats.total_chunks, 0);
        assert_eq!(stats.indexed_files, 0);
        assert_eq!(stats.queue_depth, 0);
    }

    #[tokio::test]
    async fn test_index_stats_values() {
        let stats = IndexStats {
            total_chunks: 100,
            indexed_files: 5,
            queue_depth: 2,
        };
        assert_eq!(stats.total_chunks, 100);
        assert_eq!(stats.indexed_files, 5);
        assert_eq!(stats.queue_depth, 2);
    }
}
