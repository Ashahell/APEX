//! Bounded Memory API (Hermes-style)
//!
//! Provides REST API for Hermes-compatible bounded memory:
//! - GET /api/v1/memory/bounded/stats     - Get memory statistics
//! - GET /api/v1/memory/bounded/snapshot  - Get frozen snapshot for system prompt
//! - POST /api/v1/memory/bounded/memory  - Add memory entry
//! - POST /api/v1/memory/bounded/user     - Add user profile entry
//! - PUT /api/v1/memory/bounded/:type    - Replace entry
//! - DELETE /api/v1/memory/bounded/:type - Remove entry

use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::api_error::ApiError;
use super::AppState;
use crate::memory_stores::{MemoryError, MemoryStats, MemoryStore, StoreType};
use crate::unified_config::memory_constants::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Application state for bounded memory (stored in AppState)
#[derive(Clone)]
pub struct BoundedMemoryState {
    /// Agent's memory store (MEMORY.md)
    pub memory_store: Arc<Mutex<MemoryStore>>,
    /// User profile store (USER.md)
    pub user_store: Arc<Mutex<MemoryStore>>,
    /// Base directory for memory files
    pub base_dir: String,
}

impl BoundedMemoryState {
    /// Create a new bounded memory state
    pub fn new(base_dir: String) -> Self {
        Self {
            memory_store: Arc::new(Mutex::new(MemoryStore::new(StoreType::Memory))),
            user_store: Arc::new(Mutex::new(MemoryStore::new(StoreType::User))),
            base_dir,
        }
    }

    /// Get the memory directory path
    pub fn memory_dir(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.base_dir).join(MEMORY_DIR)
    }

    /// Get the memory file path
    pub fn memory_file(&self) -> std::path::PathBuf {
        self.memory_dir().join(MEMORY_FILE)
    }

    /// Get the user file path
    pub fn user_file(&self) -> std::path::PathBuf {
        self.memory_dir().join(USER_FILE)
    }

    /// Load stores from disk
    pub async fn load(&self) -> Result<(), String> {
        let memory_path = self.memory_file();
        let user_path = self.user_file();

        // Load memory store
        let memory_store = MemoryStore::load_from_file(&memory_path)
            .await
            .map_err(|e| e.to_string())?;
        *self.memory_store.lock().await = memory_store;

        // Load user store
        let user_store = MemoryStore::load_from_file(&user_path)
            .await
            .map_err(|e| e.to_string())?;
        *self.user_store.lock().await = user_store;

        tracing::info!(
            memory_entries = self.memory_store.lock().await.entry_count(),
            user_entries = self.user_store.lock().await.entry_count(),
            "Loaded bounded memory stores"
        );

        Ok(())
    }

    /// Save stores to disk
    pub async fn save(&self) -> Result<(), String> {
        let memory_path = self.memory_file();
        let user_path = self.user_file();

        // Save memory store
        self.memory_store
            .lock()
            .await
            .save_to_file(&memory_path)
            .await
            .map_err(|e| e.to_string())?;

        // Save user store
        self.user_store
            .lock()
            .await
            .save_to_file(&user_path)
            .await
            .map_err(|e| e.to_string())?;

        tracing::debug!("Saved bounded memory stores to disk");

        Ok(())
    }
}

// ============================================================================
// Router Setup
// ============================================================================

/// Create the bounded memory router
pub fn router() -> Router<AppState> {
    Router::new()
        // Stats and snapshots
        .route("/api/v1/memory/bounded/stats", get(get_stats))
        .route("/api/v1/memory/bounded/snapshot", get(get_snapshot))
        // Memory store operations
        .route("/api/v1/memory/bounded/memory", get(get_memory_entries))
        .route("/api/v1/memory/bounded/memory", post(add_memory_entry))
        .route(
            "/api/v1/memory/bounded/memory/:old_text",
            put(replace_memory_entry),
        )
        .route("/api/v1/memory/bounded/memory", delete(remove_memory_entry))
        // User store operations
        .route("/api/v1/memory/bounded/user", get(get_user_entries))
        .route("/api/v1/memory/bounded/user", post(add_user_entry))
        .route(
            "/api/v1/memory/bounded/user/:old_text",
            put(replace_user_entry),
        )
        .route("/api/v1/memory/bounded/user", delete(remove_user_entry))
        // Consolidation AI
        .route("/api/v1/memory/bounded/consolidate", post(analyze_consolidation))
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddEntryRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct RemoveEntryRequest {
    pub old_text: String,
}

#[derive(Debug, Serialize)]
pub struct AddEntryResponse {
    pub success: bool,
    pub id: String,
    pub used_chars: usize,
    pub char_limit: usize,
    pub usage_percent: f32,
}

#[derive(Debug, Serialize)]
pub struct BoundedMemoryStats {
    pub memory: MemoryStats,
    pub user: MemoryStats,
    pub combined_usage_percent: f32,
}

#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    pub memory_snapshot: String,
    pub user_snapshot: String,
    pub combined: String,
}

#[derive(Debug, Serialize)]
pub struct EntryListResponse {
    pub store_type: String,
    pub entries: Vec<EntryResponse>,
    pub used_chars: usize,
    pub char_limit: usize,
    pub usage_percent: f32,
}

#[derive(Debug, Serialize)]
pub struct EntryResponse {
    pub id: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}

// ============================================================================
// Consolidation AI Types
// ============================================================================

/// Request to analyze memory for consolidation opportunities
#[derive(Debug, Deserialize)]
pub struct ConsolidationAnalyzeRequest {
    /// Maximum number of suggestions to return (default: 5)
    pub max_suggestions: Option<usize>,
    /// Store type to analyze (default: "memory")
    pub store_type: Option<String>,
}

/// A single consolidation suggestion
#[derive(Debug, Serialize)]
pub struct ConsolidationSuggestion {
    /// Unique ID for this suggestion
    pub id: String,
    /// Type of consolidation: "merge", "remove", "update"
    pub suggestion_type: String,
    /// Entry IDs involved in this suggestion
    pub entry_ids: Vec<String>,
    /// Original content snippets
    pub original_contents: Vec<String>,
    /// Suggested merged/updated content (for merge/update suggestions)
    pub suggested_content: Option<String>,
    /// Explanation from the AI
    pub reasoning: String,
    /// Estimated chars saved (for merge suggestions)
    pub chars_saved: Option<usize>,
}

/// Response with consolidation suggestions
#[derive(Debug, Serialize)]
pub struct ConsolidationAnalyzeResponse {
    pub suggestions: Vec<ConsolidationSuggestion>,
    pub total_entries_analyzed: usize,
    pub store_type: String,
    pub analysis_version: String,
}

// ============================================================================
// Handlers: Stats & Snapshot
// ============================================================================

/// GET /api/v1/memory/bounded/stats
async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<BoundedMemoryStats>, (axum::http::StatusCode, String)> {
    let bounded = &state.bounded_memory;

    let memory_stats = bounded.memory_store.lock().await.stats();
    let user_stats = bounded.user_store.lock().await.stats();

    let combined_chars = memory_stats.used_chars + user_stats.used_chars;
    let combined_limit = memory_stats.char_limit + user_stats.char_limit;
    let combined_percent = if combined_limit > 0 {
        combined_chars as f32 / combined_limit as f32
    } else {
        0.0
    };

    Ok(Json(BoundedMemoryStats {
        memory: memory_stats,
        user: user_stats,
        combined_usage_percent: combined_percent,
    }))
}

/// GET /api/v1/memory/bounded/snapshot
async fn get_snapshot(
    State(state): State<AppState>,
) -> Result<Json<SnapshotResponse>, (axum::http::StatusCode, String)> {
    let bounded = &state.bounded_memory;

    let memory_snapshot = bounded.memory_store.lock().await.to_snapshot();
    let user_snapshot = bounded.user_store.lock().await.to_snapshot();

    Ok(Json(SnapshotResponse {
        combined: format!("{}\n\n{}", memory_snapshot, user_snapshot),
        memory_snapshot,
        user_snapshot,
    }))
}

// ============================================================================
// Handlers: Memory Store
// ============================================================================

/// GET /api/v1/memory/bounded/memory
async fn get_memory_entries(
    State(state): State<AppState>,
) -> Result<Json<EntryListResponse>, (axum::http::StatusCode, String)> {
    let store = state.bounded_memory.memory_store.lock().await;

    let entries: Vec<EntryResponse> = store
        .entries()
        .iter()
        .map(|e| EntryResponse {
            id: e.id.clone(),
            content: e.content.clone(),
            created_at: e.created_at,
            updated_at: e.updated_at,
        })
        .collect();

    Ok(Json(EntryListResponse {
        store_type: "memory".to_string(),
        entries,
        used_chars: store.used_chars,
        char_limit: store.char_limit(),
        usage_percent: store.usage_percent(),
    }))
}

/// POST /api/v1/memory/bounded/memory
async fn add_memory_entry(
    State(state): State<AppState>,
    Json(payload): Json<AddEntryRequest>,
) -> Result<Json<AddEntryResponse>, (axum::http::StatusCode, String)> {
    let bounded = &state.bounded_memory;
    let mut store = bounded.memory_store.lock().await;

    let id = store
        .add_entry(payload.content)
        .map_err(|e| map_memory_error(&e))?;

    // Save to disk
    let used_chars = store.used_chars;
    let char_limit = store.char_limit();
    let usage_percent = store.usage_percent();
    drop(store);

    bounded.save().await.map_err(ApiError::internal)?;

    Ok(Json(AddEntryResponse {
        success: true,
        id,
        used_chars,
        char_limit,
        usage_percent,
    }))
}

/// PUT /api/v1/memory/bounded/memory/:old_text
async fn replace_memory_entry(
    State(state): State<AppState>,
    Path(old_text): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let new_content = payload["new_content"].as_str().ok_or_else(|| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            "new_content required".to_string(),
        )
    })?;

    let bounded = &state.bounded_memory;
    let mut store = bounded.memory_store.lock().await;

    store
        .replace_entry(&old_text, new_content.to_string())
        .map_err(|e| map_memory_error(&e))?;

    // Save to disk
    drop(store);
    bounded.save().await.map_err(ApiError::internal)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Entry replaced"
    })))
}

/// DELETE /api/v1/memory/bounded/memory
async fn remove_memory_entry(
    State(state): State<AppState>,
    Json(payload): Json<RemoveEntryRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let bounded = &state.bounded_memory;
    let mut store = bounded.memory_store.lock().await;

    store
        .remove_entry(&payload.old_text)
        .map_err(|e| map_memory_error(&e))?;

    // Save to disk
    drop(store);
    bounded.save().await.map_err(ApiError::internal)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Entry removed"
    })))
}

// ============================================================================
// Handlers: User Store
// ============================================================================

/// GET /api/v1/memory/bounded/user
async fn get_user_entries(
    State(state): State<AppState>,
) -> Result<Json<EntryListResponse>, (axum::http::StatusCode, String)> {
    let store = state.bounded_memory.user_store.lock().await;

    let entries: Vec<EntryResponse> = store
        .entries()
        .iter()
        .map(|e| EntryResponse {
            id: e.id.clone(),
            content: e.content.clone(),
            created_at: e.created_at,
            updated_at: e.updated_at,
        })
        .collect();

    Ok(Json(EntryListResponse {
        store_type: "user".to_string(),
        entries,
        used_chars: store.used_chars,
        char_limit: store.char_limit(),
        usage_percent: store.usage_percent(),
    }))
}

/// POST /api/v1/memory/bounded/user
async fn add_user_entry(
    State(state): State<AppState>,
    Json(payload): Json<AddEntryRequest>,
) -> Result<Json<AddEntryResponse>, (axum::http::StatusCode, String)> {
    let bounded = &state.bounded_memory;
    let mut store = bounded.user_store.lock().await;

    let id = store
        .add_entry(payload.content)
        .map_err(|e| map_memory_error(&e))?;

    // Save to disk
    let used_chars = store.used_chars;
    let char_limit = store.char_limit();
    let usage_percent = store.usage_percent();
    drop(store);

    bounded.save().await.map_err(ApiError::internal)?;

    Ok(Json(AddEntryResponse {
        success: true,
        id,
        used_chars,
        char_limit,
        usage_percent,
    }))
}

/// PUT /api/v1/memory/bounded/user/:old_text
async fn replace_user_entry(
    State(state): State<AppState>,
    Path(old_text): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let new_content = payload["new_content"].as_str().ok_or_else(|| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            "new_content required".to_string(),
        )
    })?;

    let bounded = &state.bounded_memory;
    let mut store = bounded.user_store.lock().await;

    store
        .replace_entry(&old_text, new_content.to_string())
        .map_err(|e| map_memory_error(&e))?;

    // Save to disk
    drop(store);
    bounded.save().await.map_err(ApiError::internal)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Entry replaced"
    })))
}

/// DELETE /api/v1/memory/bounded/user
async fn remove_user_entry(
    State(state): State<AppState>,
    Json(payload): Json<RemoveEntryRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let bounded = &state.bounded_memory;
    let mut store = bounded.user_store.lock().await;

    store
        .remove_entry(&payload.old_text)
        .map_err(|e| map_memory_error(&e))?;

    // Save to disk
    drop(store);
    bounded.save().await.map_err(ApiError::internal)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Entry removed"
    })))
}

// ============================================================================
// Error Mapping
// ============================================================================

fn map_memory_error(error: &MemoryError) -> (axum::http::StatusCode, String) {
    match error {
        MemoryError::EntryTooShort { min_length, actual_length } => (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Entry too short: minimum {} chars, got {} chars", min_length, actual_length),
        ),
        MemoryError::EntryTooLong { max_length, actual_length } => (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Entry too long: maximum {} chars, got {} chars", max_length, actual_length),
        ),
        MemoryError::DuplicateEntry => (
            axum::http::StatusCode::CONFLICT,
            "Duplicate entry".to_string(),
        ),
        MemoryError::CapacityExceeded { current, limit, needed } => (
            axum::http::StatusCode::BAD_REQUEST,
            format!(
                "Memory capacity exceeded: {}/{} chars, need {} more chars. Remove or consolidate entries first.",
                current, limit, needed
            ),
        ),
        MemoryError::EntryNotFound => (
            axum::http::StatusCode::NOT_FOUND,
            "Entry not found".to_string(),
        ),
        MemoryError::MultipleMatches { count, suggestion } => (
            axum::http::StatusCode::BAD_REQUEST,
            format!("{} entries matched. {}", count, suggestion),
        ),
        MemoryError::IoError(msg) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("IO error: {}", msg),
        ),
        MemoryError::ParseError(msg) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Parse error: {}", msg),
        ),
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn create_test_state() -> BoundedMemoryState {
        let temp_dir =
            std::env::temp_dir().join(format!("bounded_memory_test_{}", ulid::Ulid::new()));
        std::fs::create_dir_all(&temp_dir).ok();
        BoundedMemoryState::new(temp_dir.to_string_lossy().to_string())
    }

    #[tokio::test]
    async fn test_bounded_memory_state_creation() {
        let state = create_test_state();

        // Verify stores are created
        assert_eq!(state.memory_store.lock().await.entry_count(), 0);
        assert_eq!(state.user_store.lock().await.entry_count(), 0);

        // Verify paths are set
        assert!(state.memory_dir().exists() || !state.memory_dir().to_string_lossy().is_empty());
    }

    #[tokio::test]
    async fn test_add_and_list_memory_entries() {
        let state = create_test_state();

        // Add memory entry
        {
            let mut store = state.memory_store.lock().await;
            let id = store.add_entry("Test memory entry for the agent".to_string());
            assert!(id.is_ok());
            assert_eq!(store.entry_count(), 1);
        }

        // Verify entry exists
        assert_eq!(state.memory_store.lock().await.entry_count(), 1);
        assert_eq!(state.user_store.lock().await.entry_count(), 0);
    }

    #[tokio::test]
    async fn test_add_and_list_user_entries() {
        let state = create_test_state();

        // Add user entry
        {
            let mut store = state.user_store.lock().await;
            let id = store.add_entry("User preference: prefers markdown".to_string());
            assert!(id.is_ok());
            assert_eq!(store.entry_count(), 1);
        }

        // Verify entry exists
        assert_eq!(state.user_store.lock().await.entry_count(), 1);
        assert_eq!(state.memory_store.lock().await.entry_count(), 0);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let state = create_test_state();

        // Add entries
        state
            .memory_store
            .lock()
            .await
            .add_entry("Memory entry one".to_string())
            .unwrap();
        state
            .memory_store
            .lock()
            .await
            .add_entry("Memory entry two".to_string())
            .unwrap();
        state
            .user_store
            .lock()
            .await
            .add_entry("User entry".to_string())
            .unwrap();

        // Get stats
        let memory_stats = state.memory_store.lock().await.stats();
        let user_stats = state.user_store.lock().await.stats();

        assert_eq!(memory_stats.entry_count, 2);
        assert_eq!(user_stats.entry_count, 1);
    }

    #[tokio::test]
    async fn test_snapshot_generation() {
        let state = create_test_state();

        // Add entries
        state
            .memory_store
            .lock()
            .await
            .add_entry("Important fact about Rust".to_string())
            .unwrap();
        state
            .user_store
            .lock()
            .await
            .add_entry("User works in finance".to_string())
            .unwrap();

        // Generate snapshots
        let memory_snapshot = state.memory_store.lock().await.to_snapshot();
        let user_snapshot = state.user_store.lock().await.to_snapshot();

        assert!(memory_snapshot.contains("Important fact about Rust"));
        assert!(user_snapshot.contains("User works in finance"));
        assert!(memory_snapshot.contains("MEMORY"));
        assert!(user_snapshot.contains("USER PROFILE"));
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir =
            std::env::temp_dir().join(format!("bounded_memory_test_{}", ulid::Ulid::new()));
        std::fs::create_dir_all(&temp_dir).ok();

        // Create state and add entries
        let state = BoundedMemoryState::new(temp_dir.to_string_lossy().to_string());
        state
            .memory_store
            .lock()
            .await
            .add_entry("Persisted memory".to_string())
            .unwrap();
        state
            .user_store
            .lock()
            .await
            .add_entry("Persisted user pref".to_string())
            .unwrap();

        // Save to disk
        state.save().await.unwrap();

        // Create new state and load
        let state2 = BoundedMemoryState::new(temp_dir.to_string_lossy().to_string());
        state2.load().await.unwrap();

        // Verify entries loaded
        assert_eq!(state2.memory_store.lock().await.entry_count(), 1);
        assert_eq!(state2.user_store.lock().await.entry_count(), 1);
        assert_eq!(
            state2.memory_store.lock().await.entries()[0].content,
            "Persisted memory"
        );
    }

    #[tokio::test]
    async fn test_remove_entry() {
        let state = create_test_state();

        // Add entry
        state
            .memory_store
            .lock()
            .await
            .add_entry("To be removed".to_string())
            .unwrap();
        assert_eq!(state.memory_store.lock().await.entry_count(), 1);

        // Remove entry
        state
            .memory_store
            .lock()
            .await
            .remove_entry("To be removed")
            .unwrap();
        assert_eq!(state.memory_store.lock().await.entry_count(), 0);
    }

    #[tokio::test]
    async fn test_replace_entry() {
        let state = create_test_state();

        // Add entry
        state
            .memory_store
            .lock()
            .await
            .add_entry("Old content".to_string())
            .unwrap();

        // Replace entry
        state
            .memory_store
            .lock()
            .await
            .replace_entry("Old", "New content".to_string())
            .unwrap();

        assert_eq!(
            state.memory_store.lock().await.entries()[0].content,
            "New content"
        );
    }

    #[tokio::test]
    async fn test_capacity_limits() {
        // Create store with very small limit
        let mut store = MemoryStore::with_limit(StoreType::Memory, 50);

        // Add entry that fits
        let result1 = store.add_entry("First entry".to_string());
        assert!(result1.is_ok());

        // Add entry that causes capacity to be exceeded
        let result2 = store.add_entry(
            "This very long entry that will exceed the capacity limit of this small store"
                .to_string(),
        );
        assert!(matches!(result2, Err(MemoryError::CapacityExceeded { .. })));
    }

    #[tokio::test]
    async fn test_combined_usage_percent() {
        let state = create_test_state();

        // Add entries to both stores
        state
            .memory_store
            .lock()
            .await
            .add_entry("Memory entry content".to_string())
            .unwrap();
        state
            .user_store
            .lock()
            .await
            .add_entry("User entry content".to_string())
            .unwrap();

        let memory_chars = state.memory_store.lock().await.used_chars;
        let user_chars = state.user_store.lock().await.used_chars;
        let total_limit = state.memory_store.lock().await.char_limit()
            + state.user_store.lock().await.char_limit();
        let combined_chars = memory_chars + user_chars;

        // Combined usage should be calculated correctly
        let expected_percent = (combined_chars as f32 / total_limit as f32) * 100.0;
        assert!(expected_percent > 0.0);
        assert!(expected_percent < 100.0);
    }
}

// ============================================================================
// Handler: Consolidation AI
// ============================================================================

/// POST /api/v1/memory/bounded/consolidate
async fn analyze_consolidation(
    State(state): State<AppState>,
    Json(payload): Json<ConsolidationAnalyzeRequest>,
) -> Result<Json<ConsolidationAnalyzeResponse>, (axum::http::StatusCode, String)> {
    let store_type = payload.store_type.unwrap_or_else(|| "memory".to_string());
    let max_suggestions = payload.max_suggestions.unwrap_or(5);

    // Get entries from the appropriate store
    let entries = if store_type == "user" {
        state.bounded_memory.user_store.lock().await.entries().to_vec()
    } else {
        state.bounded_memory.memory_store.lock().await.entries().to_vec()
    };

    let store_type_str = store_type.clone();
    let total_entries = entries.len();

    // Need at least 2 entries to find consolidation opportunities
    if total_entries < 2 {
        return Ok(Json(ConsolidationAnalyzeResponse {
            suggestions: vec![],
            total_entries_analyzed: total_entries,
            store_type: store_type_str,
            analysis_version: "1.0".to_string(),
        }));
    }

    // Build prompt for LLM to analyze and find consolidation opportunities
    let entries_text: Vec<String> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| format!("{}. [{}] {}", i + 1, e.id, e.content))
        .collect();

    let entries_for_analysis = entries_text.join("\n\n");

    let analysis_prompt = format!(
        r#"You are a memory consolidation AI. Analyze these {} memory entries and find opportunities to consolidate or merge them.

Entries:
{}

Find opportunities to:
1. MERGE: Entries that are redundant, overlapping, or could be combined into one concise entry
2. REMOVE: Entries that are outdated, no longer relevant, or superseded
3. UPDATE: Entries that could be improved with additional context

For each suggestion, respond in ONLY the following JSON format (no other text):
{{"id": "suggestion-1", "type": "merge|remove|update", "entry_ids": ["id1", "id2"], "original_contents": ["content1", "content2"], "suggested_content": "merged content or null", "reasoning": "why this makes sense", "chars_saved": number}}

If no consolidation opportunities exist, respond with an empty JSON array: []

Return a JSON array of suggestions, maximum {} items."#,
        store_type, entries_for_analysis, max_suggestions
    );

    // Try to use LLM for analysis - fall back to rule-based if unavailable
    let suggestions = match crate::llama::LlamaClient::from_env().chat(
        "You are a helpful memory consolidation assistant.",
        &analysis_prompt,
    ).await {
        Ok(response) => {
            // Parse LLM response for JSON suggestions
            parse_llm_suggestions(&response, &entries)
        }
        Err(e) => {
            tracing::warn!(error = %e, "LLM unavailable, using rule-based fallback");
            // Fallback: simple rule-based consolidation detection
            find_rule_based_suggestions(&entries)
        }
    };

    Ok(Json(ConsolidationAnalyzeResponse {
        suggestions,
        total_entries_analyzed: total_entries,
        store_type: store_type_str,
        analysis_version: "1.0".to_string(),
    }))
}

/// Parse JSON suggestions from LLM response
fn parse_llm_suggestions(response: &str, _entries: &[crate::memory_stores::MemoryEntry]) -> Vec<ConsolidationSuggestion> {
    // Try to extract JSON from response
    let json_start = response.find('[').or_else(|| response.find('{'));
    let json_end = response.rfind(']').or_else(|| response.rfind('}'));

    if let (Some(start), Some(end)) = (json_start, json_end) {
        let json_str = &response[start..=end.min(response.len() - 1)];
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
            // Handle both array and single object responses
            let suggestions_array = if parsed.is_array() {
                parsed.as_array().unwrap().clone()
            } else if parsed.is_object() {
                vec![parsed]
            } else {
                return vec![];
            };

            return suggestions_array
                .iter()
                .filter_map(|v| {
                    let obj = v.as_object()?;
                    let id = obj.get("id")?.as_str()?.to_string();
                    let suggestion_type = obj.get("type")?.as_str()?.to_string();
                    let entry_ids: Vec<String> = obj.get("entry_ids")?
                        .as_array()?
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    let original_contents: Vec<String> = obj.get("original_contents")?
                        .as_array()?
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    let suggested_content = obj.get("suggested_content")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let reasoning = obj.get("reasoning")?.as_str()?.to_string();
                    let chars_saved = obj.get("chars_saved")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as usize);

                    Some(ConsolidationSuggestion {
                        id,
                        suggestion_type,
                        entry_ids,
                        original_contents,
                        suggested_content,
                        reasoning,
                        chars_saved,
                    })
                })
                .collect();
        }
    }

    vec![]
}

/// Find suggestions using rule-based analysis (fallback when LLM unavailable)
fn find_rule_based_suggestions(entries: &[crate::memory_stores::MemoryEntry]) -> Vec<ConsolidationSuggestion> {
    let mut suggestions = Vec::new();
    let n = entries.len();

    // Simple duplicate detection
    for i in 0..n {
        for j in (i + 1)..n {
            let content_i = &entries[i].content;
            let content_j = &entries[j].content;

            // Check for near-duplicates (70% similarity threshold)
            let similarity = calculate_similarity(content_i, content_j);

            if similarity > 0.7 {
                let merged = format!("{} {}", content_i.trim_end_matches('.'), content_j.trim_start_matches('.'));
                let chars_saved = content_i.len() + content_j.len() - merged.len();

                suggestions.push(ConsolidationSuggestion {
                    id: format!("suggestion-{}", suggestions.len() + 1),
                    suggestion_type: "merge".to_string(),
                    entry_ids: vec![entries[i].id.clone(), entries[j].id.clone()],
                    original_contents: vec![content_i.clone(), content_j.clone()],
                    suggested_content: Some(merged),
                    reasoning: "High similarity detected between entries - could be merged".to_string(),
                    chars_saved: Some(chars_saved),
                });
            }

            if suggestions.len() >= 3 {
                return suggestions;
            }
        }
    }

    suggestions
}

/// Calculate simple text similarity (Jaccard-based)
fn calculate_similarity(a: &str, b: &str) -> f64 {
    let words_a: std::collections::HashSet<_> = a.split_whitespace().collect();
    let words_b: std::collections::HashSet<_> = b.split_whitespace().collect();

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f64 / union as f64
}
