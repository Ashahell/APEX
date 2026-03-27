//! Session Search API (Hermes-style Session Search)
//!
//! Endpoints for searching conversations and sessions.

use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::api::api_error::ApiError;
use crate::session_search::{SearchQuery, SearchResult, SearchStats};

/// GET /api/v1/search/sessions - Search sessions
async fn search_sessions(
    State(state): State<crate::api::AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, (axum::http::StatusCode, String)> {
    let query = SearchQuery {
        q: params.q,
        limit: params.limit,
        offset: params.offset,
        include_context: params.include_context,
    };

    let results = state
        .session_search
        .search(&query)
        .await
        .map_err(|e| ApiError::internal(format!("Search failed: {}", e)))?;

    let total = state
        .session_search
        .get_stats()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get stats: {}", e)))?
        .total_results;

    Ok(Json(SearchResponse { results, total }))
}

/// GET /api/v1/search/sessions/stats - Get search statistics
async fn get_search_stats(
    State(state): State<crate::api::AppState>,
) -> Result<Json<SearchStats>, (axum::http::StatusCode, String)> {
    state
        .session_search
        .get_stats()
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(format!("Failed to get stats: {}", e)))
}

/// POST /api/v1/search/reindex - Rebuild search index
async fn reindex_sessions(
    State(state): State<crate::api::AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let count = state
        .session_search
        .rebuild_index()
        .await
        .map_err(|e| ApiError::internal(format!("Reindex failed: {}", e)))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "indexed_count": count
    })))
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub include_context: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
}

use serde::Serialize;

/// Create session search router
pub fn router() -> Router<crate::api::AppState> {
    Router::new()
        .route("/api/v1/search/sessions", get(search_sessions))
        .route("/api/v1/search/sessions/stats", get(get_search_stats))
        .route("/api/v1/search/reindex", post(reindex_sessions))
}
