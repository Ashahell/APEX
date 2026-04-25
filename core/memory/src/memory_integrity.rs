//! Memory Integrity Service
//!
//! Verifies the integrity of memory_chunks and vector store.
//! Uses SHA-256 hash sidecar for content verification.
//!
//! Inspired by Agent Zero v1.8 FAISS integrity check (CVE-2026-4308 mitigation).

use sha2::{Digest, Sha256};
use sqlx::{Pool, Sqlite};
use std::time::Instant;

use crate::MemoryError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct HashStoreEntry {
    pub chunk_id: String,
    pub content_hash: String,
    pub word_count: i64,
    pub chunk_index: i64,
    pub created_at: String,
    pub verified_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntegrityMeta {
    pub last_check: Option<String>,
    pub total_chunks: i64,
    pub valid_chunks: i64,
    pub corrupted_chunks: i64,
    pub repaired_chunks: i64,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntegrityReport {
    pub status: String,
    pub total_chunks: i64,
    pub valid_chunks: i64,
    pub corrupted_chunks: i64,
    pub corrupt_chunk_ids: Vec<String>,
    pub last_check: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepairReport {
    pub total_repaired: i64,
    pub failed: i64,
    pub repaired_chunk_ids: Vec<String>,
    pub failed_chunk_ids: Vec<String>,
}

pub struct MemoryIntegrity {
    pool: Pool<Sqlite>,
}

impl MemoryIntegrity {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub fn compute_content_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn get_meta(&self) -> Result<IntegrityMeta, MemoryError> {
        let row = sqlx::query_as::<_, (Option<String>, i64, i64, i64, i64, String)>(
            "SELECT last_check, total_chunks, valid_chunks, corrupted_chunks, repaired_chunks, status FROM integrity_meta WHERE id = 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(match row {
            Some((last_check, total, valid, corrupted, repaired, status)) => IntegrityMeta {
                last_check,
                total_chunks: total,
                valid_chunks: valid,
                corrupted_chunks: corrupted,
                repaired_chunks: repaired,
                status,
            },
            None => IntegrityMeta {
                last_check: None,
                total_chunks: 0,
                valid_chunks: 0,
                corrupted_chunks: 0,
                repaired_chunks: 0,
                status: "unknown".to_string(),
            },
        })
    }

    pub async fn compute_all_hashes(&self) -> Result<i64, MemoryError> {
        let chunks = sqlx::query_as::<_, (String, String, i64, i64)>(
            "SELECT mc.id, mc.content, mc.word_count, mc.chunk_index FROM memory_chunks mc WHERE mc.id NOT IN (SELECT chunk_id FROM hash_store)",
        )
        .fetch_all(&self.pool)
        .await?;

        let count = chunks.len() as i64;
        for (chunk_id, content, word_count, chunk_index) in chunks {
            let hash = Self::compute_content_hash(&content);
            sqlx::query(
                "INSERT OR IGNORE INTO hash_store (chunk_id, content_hash, word_count, chunk_index, status) VALUES (?, ?, ?, ?, 'pending')",
            )
            .bind(&chunk_id)
            .bind(&hash)
            .bind(word_count)
            .bind(chunk_index)
            .execute(&self.pool)
            .await?;
        }
        Ok(count)
    }

    pub async fn update_hash(&self, chunk_id: &str, content: &str) -> Result<(), MemoryError> {
        let hash = Self::compute_content_hash(content);
        sqlx::query(
            "INSERT OR REPLACE INTO hash_store (chunk_id, content_hash, status, verified_at) VALUES (?, ?, 'pending', NULL)",
        )
        .bind(chunk_id)
        .bind(&hash)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn verify_integrity(&self) -> Result<IntegrityReport, MemoryError> {
        let start = Instant::now();

        sqlx::query("UPDATE integrity_meta SET status = 'checking' WHERE id = 1")
            .execute(&self.pool)
            .await?;

        let chunks = sqlx::query_as::<_, (String, String, String, i64, i64)>(
            "SELECT mc.id, mc.content, hs.content_hash, mc.word_count, mc.chunk_index FROM memory_chunks mc JOIN hash_store hs ON mc.id = hs.chunk_id WHERE hs.status != 'corrupted'",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut valid = 0i64;
        let mut corrupted = 0i64;
        let mut corrupt_ids = Vec::new();

        for (chunk_id, content, stored_hash, _word_count, _chunk_index) in chunks {
            let computed_hash = Self::compute_content_hash(&content);
            if computed_hash == stored_hash {
                valid += 1;
                sqlx::query("UPDATE hash_store SET status = 'valid', verified_at = datetime('now') WHERE chunk_id = ?")
                    .bind(&chunk_id)
                    .execute(&self.pool)
                    .await?;
            } else {
                corrupted += 1;
                corrupt_ids.push(chunk_id.clone());
                sqlx::query("UPDATE hash_store SET status = 'corrupted', verified_at = datetime('now') WHERE chunk_id = ?")
                    .bind(&chunk_id)
                    .execute(&self.pool)
                    .await?;
            }
        }

        let total = valid + corrupted;
        let status = if corrupted > 0 {
            "corrupted"
        } else if total == 0 {
            "empty"
        } else {
            "valid"
        };

        sqlx::query(
            "UPDATE integrity_meta SET last_check = datetime('now'), total_chunks = ?, valid_chunks = ?, corrupted_chunks = ?, status = ? WHERE id = 1",
        )
        .bind(total)
        .bind(valid)
        .bind(corrupted)
        .bind(status)
        .execute(&self.pool)
        .await?;

        let meta = self.get_meta().await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(IntegrityReport {
            status: meta.status,
            total_chunks: total,
            valid_chunks: valid,
            corrupted_chunks: corrupted,
            corrupt_chunk_ids: corrupt_ids,
            last_check: meta.last_check,
            duration_ms,
        })
    }

    pub async fn repair_chunks(&self, chunk_ids: &[String]) -> Result<RepairReport, MemoryError> {
        if chunk_ids.is_empty() {
            return Ok(RepairReport {
                total_repaired: 0,
                failed: 0,
                repaired_chunk_ids: vec![],
                failed_chunk_ids: vec![],
            });
        }

        let ids_json = serde_json::to_string(chunk_ids).unwrap_or("[]".to_string());
        let chunks = sqlx::query_as::<_, (String, String, i64, i64)>(
            "SELECT mc.id, mc.content, mc.word_count, mc.chunk_index FROM memory_chunks mc WHERE mc.id IN (SELECT value FROM json_each(?1))",
        )
        .bind(&ids_json)
        .fetch_all(&self.pool)
        .await?;

        let mut repaired = 0i64;
        let mut failed = 0i64;
        let mut repaired_ids = Vec::new();
        let mut failed_ids = Vec::new();

        for (chunk_id, content, _word_count, _chunk_index) in chunks {
            let new_hash = Self::compute_content_hash(&content);
            let result = sqlx::query(
                "UPDATE hash_store SET content_hash = ?, status = 'repaired', verified_at = datetime('now') WHERE chunk_id = ?",
            )
            .bind(&new_hash)
            .bind(&chunk_id)
            .execute(&self.pool)
            .await;

            match result {
                Ok(_) => {
                    repaired += 1;
                    repaired_ids.push(chunk_id.clone());
                }
                Err(_) => {
                    failed += 1;
                    failed_ids.push(chunk_id);
                }
            }
        }

        sqlx::query("UPDATE integrity_meta SET repaired_chunks = repaired_chunks + ? WHERE id = 1")
            .bind(repaired)
            .execute(&self.pool)
            .await?;

        Ok(RepairReport {
            total_repaired: repaired,
            failed,
            repaired_chunk_ids: repaired_ids,
            failed_chunk_ids: failed_ids,
        })
    }

    pub async fn repair_all_corrupted(&self) -> Result<RepairReport, MemoryError> {
        let corrupted = sqlx::query_as::<_, (String,)>(
            "SELECT chunk_id FROM hash_store WHERE status = 'corrupted'",
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|(id,)| id)
        .collect::<Vec<_>>();

        self.repair_chunks(&corrupted).await
    }

    pub async fn get_pending_chunks(&self, limit: i64) -> Result<Vec<HashStoreEntry>, MemoryError> {
        let entries = sqlx::query_as::<_, HashStoreEntry>(
            "SELECT chunk_id, content_hash, word_count, chunk_index, created_at, verified_at, status FROM hash_store WHERE status = 'pending' LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }

    pub async fn get_corrupted_chunks(&self) -> Result<Vec<HashStoreEntry>, MemoryError> {
        let entries = sqlx::query_as::<_, HashStoreEntry>(
            "SELECT chunk_id, content_hash, word_count, chunk_index, created_at, verified_at, status FROM hash_store WHERE status = 'corrupted'",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }

    pub async fn hash_count(&self) -> Result<i64, MemoryError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM hash_store")
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0)
    }

    pub async fn chunk_count(&self) -> Result<i64, MemoryError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM memory_chunks")
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0)
    }

    pub async fn vector_count(&self) -> Result<i64, MemoryError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM memory_vec0")
            .fetch_optional(&self.pool)
            .await?
            .unwrap_or((0i64,));
        Ok(count.0)
    }

    pub async fn get_sync_status(&self) -> Result<serde_json::Value, MemoryError> {
        let hash_count = self.hash_count().await?;
        let chunk_count = self.chunk_count().await?;
        let vector_count = self.vector_count().await?;

        let text_synced = hash_count >= chunk_count;
        let vector_synced = vector_count >= chunk_count;

        Ok(serde_json::json!({
            "hash_count": hash_count,
            "chunk_count": chunk_count,
            "vector_count": vector_count,
            "text_synced": text_synced,
            "vector_synced": vector_synced,
            "pending_hashes": chunk_count - hash_count,
            "pending_vectors": chunk_count - vector_count,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_content_hash() {
        let hash1 = MemoryIntegrity::compute_content_hash("Hello, world!");
        let hash2 = MemoryIntegrity::compute_content_hash("Hello, world!");
        let hash3 = MemoryIntegrity::compute_content_hash("Hello, world!!");

        assert_eq!(hash1.len(), 64);
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_compute_content_hash_empty() {
        let hash = MemoryIntegrity::compute_content_hash("");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_compute_content_hash_deterministic() {
        let content = "The quick brown fox jumps over the lazy dog";
        let hash1 = MemoryIntegrity::compute_content_hash(content);
        let hash2 = MemoryIntegrity::compute_content_hash(content);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_content_hash_unicode() {
        let hash1 = MemoryIntegrity::compute_content_hash("こんにちは世界");
        let hash2 = MemoryIntegrity::compute_content_hash("こんにちは世界");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, MemoryIntegrity::compute_content_hash("hello"));
    }

    #[test]
    fn test_compute_content_hash_large() {
        let content = "x".repeat(1_000_000);
        let hash = MemoryIntegrity::compute_content_hash(&content);
        assert_eq!(hash.len(), 64);
    }
}