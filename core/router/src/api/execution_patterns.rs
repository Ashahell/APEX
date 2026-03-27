use axum::{
    extract::{Path, Query, State},
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use apex_memory::execution_pattern_repo::{
    ExecutionPattern, ExecutionPatternRepository, PatternAlertTemplate,
};

use crate::api::AppState;

// ============ Router ============

pub fn router() -> Router<AppState> {
    Router::new()
        // Patterns
        .route("/api/v1/patterns", get(list_patterns))
        .route("/api/v1/patterns/task/:task_id", get(get_patterns_by_task))
        .route(
            "/api/v1/patterns/type/:pattern_type",
            get(get_patterns_by_type),
        )
        .route(
            "/api/v1/patterns/severity/:severity",
            get(get_patterns_by_severity),
        )
        .route("/api/v1/patterns/stats", get(get_pattern_stats))
        .route(
            "/api/v1/patterns/task/:task_id",
            delete(delete_patterns_by_task),
        )
        // Alert templates
        .route("/api/v1/patterns/templates", get(list_templates))
        .route(
            "/api/v1/patterns/templates/:pattern_type",
            get(get_template),
        )
}

// ============ Query Types ============

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
}

// ============ Response Types ============

#[derive(Debug, Serialize)]
pub struct PatternResponse {
    pub id: String,
    pub task_id: String,
    pub pattern_type: String,
    pub severity: String,
    pub tool_calls: Option<Vec<String>>,
    pub file_ops: Option<Vec<String>>,
    pub error_count: i32,
    pub details: Option<serde_json::Value>,
    pub detected_at: String,
}

impl From<ExecutionPattern> for PatternResponse {
    fn from(p: ExecutionPattern) -> Self {
        let tool_calls: Option<Vec<String>> = p
            .tool_calls
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok());
        let file_ops: Option<Vec<String>> = p
            .file_ops
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok());
        let details: Option<serde_json::Value> = p
            .details
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok());

        Self {
            id: p.id,
            task_id: p.task_id,
            pattern_type: p.pattern_type,
            severity: p.severity,
            tool_calls,
            file_ops,
            error_count: p.error_count,
            details,
            detected_at: p.detected_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PatternStats {
    pub by_severity: Vec<SeverityCount>,
    pub by_type: Vec<TypeCount>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct SeverityCount {
    pub severity: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct TypeCount {
    pub pattern_type: String,
    pub count: i64,
}

// ============ Handlers ============

// List recent patterns
async fn list_patterns(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<PatternResponse>>, String> {
    let repo = ExecutionPatternRepository::new(&state.pool);
    let limit = query.limit.unwrap_or(50);

    let patterns = repo
        .get_recent(limit)
        .await
        .map_err(|e| format!("Failed to list patterns: {}", e))?;

    Ok(Json(patterns.into_iter().map(|p| p.into()).collect()))
}

// Get patterns by task
async fn get_patterns_by_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Vec<PatternResponse>>, String> {
    let repo = ExecutionPatternRepository::new(&state.pool);

    let patterns = repo
        .get_by_task(&task_id)
        .await
        .map_err(|e| format!("Failed to get patterns: {}", e))?;

    Ok(Json(patterns.into_iter().map(|p| p.into()).collect()))
}

// Get patterns by type
async fn get_patterns_by_type(
    State(state): State<AppState>,
    Path(pattern_type): Path<String>,
) -> Result<Json<Vec<PatternResponse>>, String> {
    let repo = ExecutionPatternRepository::new(&state.pool);

    let patterns = repo
        .get_by_type(&pattern_type)
        .await
        .map_err(|e| format!("Failed to get patterns: {}", e))?;

    Ok(Json(patterns.into_iter().map(|p| p.into()).collect()))
}

// Get patterns by severity
async fn get_patterns_by_severity(
    State(state): State<AppState>,
    Path(severity): Path<String>,
) -> Result<Json<Vec<PatternResponse>>, String> {
    let repo = ExecutionPatternRepository::new(&state.pool);

    let patterns = repo
        .get_by_severity(&severity)
        .await
        .map_err(|e| format!("Failed to get patterns: {}", e))?;

    Ok(Json(patterns.into_iter().map(|p| p.into()).collect()))
}

// Get pattern statistics
async fn get_pattern_stats(State(state): State<AppState>) -> Result<Json<PatternStats>, String> {
    let repo = ExecutionPatternRepository::new(&state.pool);

    let by_severity = repo
        .count_by_severity()
        .await
        .map_err(|e| format!("Failed to get stats: {}", e))?;

    let by_type = repo
        .count_by_type()
        .await
        .map_err(|e| format!("Failed to get stats: {}", e))?;

    let total: i64 = by_severity.iter().map(|(_, c)| c).sum();

    Ok(Json(PatternStats {
        by_severity: by_severity
            .into_iter()
            .map(|(s, c)| SeverityCount {
                severity: s,
                count: c,
            })
            .collect(),
        by_type: by_type
            .into_iter()
            .map(|(t, c)| TypeCount {
                pattern_type: t,
                count: c,
            })
            .collect(),
        total,
    }))
}

// Delete patterns by task
async fn delete_patterns_by_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = ExecutionPatternRepository::new(&state.pool);

    repo.delete_by_task(&task_id)
        .await
        .map_err(|e| format!("Failed to delete patterns: {}", e))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ============ Template Handlers ============

// List alert templates
async fn list_templates(
    State(state): State<AppState>,
) -> Result<Json<Vec<PatternAlertTemplate>>, String> {
    let repo = ExecutionPatternRepository::new(&state.pool);

    let templates = repo
        .list_templates()
        .await
        .map_err(|e| format!("Failed to list templates: {}", e))?;

    Ok(Json(templates))
}

// Get template by pattern type
async fn get_template(
    State(state): State<AppState>,
    Path(pattern_type): Path<String>,
) -> Result<Json<PatternAlertTemplate>, String> {
    let repo = ExecutionPatternRepository::new(&state.pool);

    let template = repo
        .get_template(&pattern_type)
        .await
        .map_err(|e| format!("Template not found: {}", e))?;

    Ok(Json(template))
}
