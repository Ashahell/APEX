use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    response::Json as AxumJson,
    routing::{get, post, put, delete},
    Json, Router,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use apex_memory::DecisionJournalRepository;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/journal", get(list_journal).post(create_journal_entry))
        .route("/api/v1/journal/:id", get(get_journal_entry).put(update_journal_entry).delete(delete_journal_entry))
        .route("/api/v1/journal/search", get(search_journal))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntryResponse {
    pub id: String,
    pub task_id: Option<String>,
    pub title: String,
    pub context: Option<String>,
    pub decision: String,
    pub rationale: Option<String>,
    pub outcome: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateJournalRequest {
    pub task_id: Option<String>,
    pub title: String,
    pub context: Option<String>,
    pub decision: String,
    pub rationale: Option<String>,
    pub outcome: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJournalRequest {
    pub task_id: Option<String>,
    pub title: String,
    pub context: Option<String>,
    pub decision: String,
    pub rationale: Option<String>,
    pub outcome: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct JournalQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JournalSearchResponse {
    pub entries: Vec<JournalEntryResponse>,
    pub total: i64,
}

fn parse_tags(tags_str: Option<String>) -> Option<Vec<String>> {
    tags_str.map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
}

fn convert_entry(e: apex_memory::DecisionJournalEntry) -> JournalEntryResponse {
    JournalEntryResponse {
        id: e.id,
        task_id: e.task_id,
        title: e.title,
        context: e.context,
        decision: e.decision,
        rationale: e.rationale,
        outcome: e.outcome,
        tags: parse_tags(e.tags),
        created_at: e.created_at,
        updated_at: e.updated_at,
    }
}

async fn list_journal(
    Query(query): Query<JournalQuery>,
    State(state): State<AppState>,
) -> Result<AxumJson<JournalSearchResponse>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    let repo = DecisionJournalRepository::new(&state.pool);
    
    match repo.find_all(limit, offset).await {
        Ok(entries) => {
            let total = entries.len() as i64;
            Ok(AxumJson(JournalSearchResponse {
                entries: entries.into_iter().map(convert_entry).collect(),
                total,
            }))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list journal: {}", e))),
    }
}

async fn get_journal_entry(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<AxumJson<Option<JournalEntryResponse>>, (StatusCode, String)> {
    let repo = DecisionJournalRepository::new(&state.pool);
    match repo.find_by_id(&id).await {
        Ok(e) => Ok(AxumJson(e.map(convert_entry))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get entry: {}", e))),
    }
}

async fn create_journal_entry(
    State(state): State<AppState>,
    Json(payload): Json<CreateJournalRequest>,
) -> Result<(StatusCode, AxumJson<JournalEntryResponse>), (StatusCode, String)> {
    let id = ulid::Ulid::new().to_string();
    let create = apex_memory::CreateDecisionEntry {
        task_id: payload.task_id,
        title: payload.title,
        context: payload.context,
        decision: payload.decision,
        rationale: payload.rationale,
        outcome: payload.outcome,
        tags: payload.tags,
    };
    let repo = DecisionJournalRepository::new(&state.pool);
    
    match repo.create(&id, create).await {
        Ok(_) => {
            if let Ok(Some(e)) = repo.find_by_id(&id).await {
                Ok((StatusCode::CREATED, AxumJson(convert_entry(e))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to find created entry".to_string()))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create entry: {}", e))),
    }
}

async fn update_journal_entry(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateJournalRequest>,
) -> Result<AxumJson<Option<JournalEntryResponse>>, (StatusCode, String)> {
    let update = apex_memory::CreateDecisionEntry {
        task_id: payload.task_id,
        title: payload.title,
        context: payload.context,
        decision: payload.decision,
        rationale: payload.rationale,
        outcome: payload.outcome,
        tags: payload.tags,
    };
    let repo = DecisionJournalRepository::new(&state.pool);
    
    match repo.update(&id, update).await {
        Ok(_) => {
            if let Ok(Some(e)) = repo.find_by_id(&id).await {
                Ok(AxumJson(Some(convert_entry(e))))
            } else {
                Ok(AxumJson(None))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update entry: {}", e))),
    }
}

async fn delete_journal_entry(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<AxumJson<bool>, (StatusCode, String)> {
    let repo = DecisionJournalRepository::new(&state.pool);
    match repo.delete(&id).await {
        Ok(_) => Ok(AxumJson(true)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete entry: {}", e))),
    }
}

async fn search_journal(
    Query(query): Query<JournalQuery>,
    State(state): State<AppState>,
) -> Result<AxumJson<JournalSearchResponse>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(20);
    
    if let Some(q) = query.q {
        let repo = DecisionJournalRepository::new(&state.pool);
        match repo.search(&q, limit).await {
            Ok(entries) => {
                let total = entries.len() as i64;
                Ok(AxumJson(JournalSearchResponse {
                    entries: entries.into_iter().map(convert_entry).collect(),
                    total,
                }))
            }
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Search failed: {}", e))),
        }
    } else {
        list_journal(Query(query), State(state)).await
    }
}
