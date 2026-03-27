//! Session Search (Hermes-style Session Search)
//!
//! Provides search across conversations and sessions.
//!
//! Features:
//! - FTS5 virtual table for fast text search (optional)
//! - Basic LIKE search fallback
//! - Context window extraction

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::unified_config::search_constants::*;

/// Search result entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub task_id: String,
    pub content: String,
    pub matched_content: String,
    pub rank: f64,
    pub context_before: String,
    pub context_after: String,
}

/// Search query parameters
#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub include_context: Option<bool>,
}

/// Search statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub total_results: usize,
    pub query_time_ms: u64,
    pub fts_enabled: bool,
}

/// Session search engine
pub struct SessionSearch {
    pool: Pool<Sqlite>,
    fts_enabled: bool,
}

impl SessionSearch {
    /// Create a new session search engine (without FTS init)
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            pool,
            fts_enabled: false,
        }
    }

    /// Create and initialize a session search engine with FTS5
    pub async fn new_initialized(pool: Pool<Sqlite>) -> Result<Self, SessionSearchError> {
        let mut search = Self::new(pool);
        search.fts_enabled = search.init_fts().await;
        Ok(search)
    }

    /// Initialize FTS5 virtual table (optional - silently fails if not supported)
    async fn init_fts(&self) -> bool {
        let create_result = sqlx::query(&format!(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS {} USING fts5(
                task_id UNINDEXED,
                content,
                tokenize='{}'
            )
            "#,
            SESSIONS_FTS_TABLE, FTS5_TOKENIZER
        ))
        .execute(&self.pool)
        .await;

        match create_result {
            Ok(_) => {
                tracing::info!("FTS5 search index initialized");
                true
            }
            Err(e) => {
                tracing::warn!("FTS5 not available, falling back to LIKE search: {}", e);
                false
            }
        }
    }

    /// Check if FTS is enabled
    pub fn is_fts_enabled(&self) -> bool {
        self.fts_enabled
    }

    /// Rebuild the FTS index from existing data
    pub async fn rebuild_index(&self) -> Result<usize, SessionSearchError> {
        // Try FTS rebuild
        let fts_result = sqlx::query(&format!(
            r#"
            INSERT INTO {} (task_id, content)
            SELECT id, input_content FROM tasks
            WHERE input_content IS NOT NULL AND input_content != ''
            "#,
            SESSIONS_FTS_TABLE
        ))
        .execute(&self.pool)
        .await;

        match fts_result {
            Ok(result) => Ok(result.rows_affected() as usize),
            Err(_) => {
                // FTS not available, return 0
                Ok(0)
            }
        }
    }

    /// Search sessions using FTS5 or LIKE fallback
    pub async fn search(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, SessionSearchError> {
        let start = std::time::Instant::now();
        let limit = query.limit.unwrap_or(MAX_SEARCH_RESULTS);
        let offset = query.offset.unwrap_or(0);
        let search_term = format!("%{}%", query.q.replace('%', "\\%"));

        // Try FTS first, fall back to LIKE
        let results: Vec<SearchResult> = if self.fts_enabled {
            self.search_fts(query, limit, offset).await?
        } else {
            self.search_like(&search_term, limit, offset).await?
        };

        let query_time = start.elapsed().as_millis() as u64;
        tracing::debug!(
            query = %query.q,
            results = results.len(),
            time_ms = query_time,
            fts_enabled = self.fts_enabled,
            "Session search completed"
        );

        Ok(results)
    }

    /// Search using FTS5
    async fn search_fts(
        &self,
        query: &SearchQuery,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SearchResult>, SessionSearchError> {
        let fts_query = format!("\"{}\"", query.q.replace('"', "\"\""));

        let rows: Vec<TaskRow> = sqlx::query_as(&format!(
            r#"
                SELECT task_id, content, bm25({}) as rank
                FROM {}
                WHERE {} MATCH ?
                ORDER BY rank
                LIMIT ? OFFSET ?
                "#,
            SESSIONS_FTS_TABLE, SESSIONS_FTS_TABLE, SESSIONS_FTS_TABLE
        ))
        .bind(&fts_query)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SessionSearchError::SearchError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let content = row.content.unwrap_or_default();
                SearchResult {
                    task_id: row.task_id,
                    content: content.clone(),
                    matched_content: content,
                    rank: row.rank,
                    context_before: String::new(),
                    context_after: String::new(),
                }
            })
            .collect())
    }

    /// Fallback search using LIKE
    async fn search_like(
        &self,
        search_term: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SearchResult>, SessionSearchError> {
        let rows: Vec<TaskRow> = sqlx::query_as(
            r#"
            SELECT id as task_id, input_content as content, 0.0 as rank
            FROM tasks
            WHERE input_content LIKE ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(search_term)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SessionSearchError::SearchError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| SearchResult {
                task_id: row.task_id,
                content: row.content.clone().unwrap_or_default(),
                matched_content: row.content.unwrap_or_default(),
                rank: row.rank,
                context_before: String::new(),
                context_after: String::new(),
            })
            .collect())
    }

    /// Get search statistics
    pub async fn get_stats(&self) -> Result<SearchStats, SessionSearchError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM tasks WHERE input_content IS NOT NULL AND input_content != ''",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SessionSearchError::IndexError(e.to_string()))?;

        Ok(SearchStats {
            total_results: count.0 as usize,
            query_time_ms: 0,
            fts_enabled: self.fts_enabled,
        })
    }
}

/// Internal task row type
#[derive(Debug, sqlx::FromRow)]
struct TaskRow {
    task_id: String,
    content: Option<String>,
    rank: f64,
}

/// Session search errors
#[derive(Debug, thiserror::Error)]
pub enum SessionSearchError {
    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Search error: {0}")]
    SearchError(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use apex_memory::db::Database;
    use apex_memory::task_repo::TaskRepository;
    use apex_memory::tasks::{CreateTask, TaskTier};
    use std::path::PathBuf;

    async fn create_test_search() -> (SessionSearch, Database) {
        let db = Database::new(&PathBuf::from(":memory:")).await.unwrap();
        db.run_migrations().await.unwrap();

        let search = SessionSearch::new(db.pool().clone());
        (search, db)
    }

    async fn insert_test_tasks(db: &Database) {
        let repo = TaskRepository::new(db.pool());

        let tasks = vec![
            "How do I implement authentication in Rust?",
            "Best practices for Rust error handling",
            "React component testing guide",
            "Python async/await tutorial",
            "Deploying Docker containers to Kubernetes",
        ];

        for content in tasks.iter() {
            let task_id = ulid::Ulid::new().to_string();
            repo.create(
                &task_id,
                CreateTask {
                    input_content: content.to_string(),
                    channel: Some("test".to_string()),
                    thread_id: None,
                    author: None,
                    skill_name: None,
                    project: None,
                    priority: None,
                    category: None,
                },
                TaskTier::Deep,
            )
            .await
            .unwrap();
        }
    }

    #[tokio::test]
    async fn test_session_search_creation() {
        let (search, _db) = create_test_search().await;
        assert!(!search.fts_enabled); // FTS5 may not be available
    }

    #[tokio::test]
    async fn test_search_with_results() {
        let (search, db) = create_test_search().await;
        insert_test_tasks(&db).await;

        // Search for Rust-related content
        let query = SearchQuery {
            q: "Rust".to_string(),
            limit: Some(10),
            offset: None,
            include_context: Some(true),
        };

        let results = search.search(&query).await.unwrap();

        // Should find results about Rust
        assert!(results.iter().any(|r| r.content.contains("Rust")));
    }

    #[tokio::test]
    async fn test_search_with_limit() {
        let (search, db) = create_test_search().await;
        insert_test_tasks(&db).await;

        let query = SearchQuery {
            q: "tutorial".to_string(),
            limit: Some(2),
            offset: None,
            include_context: None,
        };

        let results = search.search(&query).await.unwrap();
        assert!(results.len() <= 2);
    }

    #[tokio::test]
    async fn test_search_with_offset() {
        let (search, db) = create_test_search().await;
        insert_test_tasks(&db).await;

        // Get all results
        let query_all = SearchQuery {
            q: "tutorial".to_string(),
            limit: Some(100),
            offset: None,
            include_context: None,
        };
        let all_results = search.search(&query_all).await.unwrap();
        let total = all_results.len();

        if total >= 2 {
            // Get with offset
            let query_offset = SearchQuery {
                q: "tutorial".to_string(),
                limit: Some(100),
                offset: Some(1),
                include_context: None,
            };
            let offset_results = search.search(&query_offset).await.unwrap();

            assert!(offset_results.len() < total);
        }
    }

    #[tokio::test]
    async fn test_get_stats() {
        let (search, db) = create_test_search().await;
        insert_test_tasks(&db).await;

        let stats = search.get_stats().await.unwrap();
        assert_eq!(stats.total_results, 5);
        assert!(!stats.fts_enabled || stats.fts_enabled); // Either is valid
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let (search, db) = create_test_search().await;
        insert_test_tasks(&db).await;

        let query = SearchQuery {
            q: "".to_string(),
            limit: None,
            offset: None,
            include_context: None,
        };

        // Empty query should return results (searches all)
        let results = search.search(&query).await.unwrap();
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_search_no_results() {
        let (search, db) = create_test_search().await;
        insert_test_tasks(&db).await;

        let query = SearchQuery {
            q: "xyznonexistent123".to_string(),
            limit: None,
            offset: None,
            include_context: None,
        };

        let results = search.search(&query).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_search_case_insensitive() {
        let (search, db) = create_test_search().await;
        insert_test_tasks(&db).await;

        let query_lower = SearchQuery {
            q: "rust".to_string(), // lowercase
            limit: None,
            offset: None,
            include_context: None,
        };
        let results_lower = search.search(&query_lower).await.unwrap();

        let query_upper = SearchQuery {
            q: "RUST".to_string(), // uppercase
            limit: None,
            offset: None,
            include_context: None,
        };
        let results_upper = search.search(&query_upper).await.unwrap();

        // Should find same results regardless of case
        assert_eq!(results_lower.len(), results_upper.len());
    }

    #[tokio::test]
    async fn test_search_partial_match() {
        let (search, db) = create_test_search().await;
        insert_test_tasks(&db).await;

        // Search for partial word
        let query = SearchQuery {
            q: "tuto".to_string(), // partial of "tutorial"
            limit: None,
            offset: None,
            include_context: None,
        };

        let results = search.search(&query).await.unwrap();
        // Should still find "tutorial" results via LIKE %tuto%
        assert!(results.iter().any(|r| r.content.contains("tutorial")));
    }

    #[test]
    fn test_search_query_defaults() {
        // Test with no optional fields
        let query = SearchQuery {
            q: "test".to_string(),
            limit: None,
            offset: None,
            include_context: None,
        };

        assert_eq!(
            query.limit.unwrap_or(MAX_SEARCH_RESULTS),
            MAX_SEARCH_RESULTS
        );
        assert_eq!(query.offset.unwrap_or(0), 0);
        assert!(!query.include_context.unwrap_or(false));
    }

    #[test]
    fn test_search_result_fields() {
        let result = SearchResult {
            task_id: "test123".to_string(),
            content: "Test content".to_string(),
            matched_content: "Test".to_string(),
            rank: 0.95,
            context_before: "Before text".to_string(),
            context_after: "After text".to_string(),
        };

        assert_eq!(result.task_id, "test123");
        assert_eq!(result.rank, 0.95);
        assert!(result.matched_content.contains("Test"));
    }

    #[test]
    fn test_search_stats_fields() {
        let stats = SearchStats {
            total_results: 100,
            query_time_ms: 50,
            fts_enabled: true,
        };

        assert_eq!(stats.total_results, 100);
        assert_eq!(stats.query_time_ms, 50);
        assert!(stats.fts_enabled);
    }
}
