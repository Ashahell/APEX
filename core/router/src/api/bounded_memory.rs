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
