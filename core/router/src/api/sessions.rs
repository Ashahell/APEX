use axum::{
    extract::Path,
    extract::Query,
    extract::State,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use apex_memory::session_control_repo::{
    SessionAttachment, SessionCheckpoint, SessionControlRepository, SessionResumeHistory,
    SessionState, SessionYieldLog,
};

use crate::api::AppState;

// ============ Router ============

pub fn router() -> Router<AppState> {
    Router::new()
        // Yield
        .route("/api/v1/sessions/:session_id/yield", post(yield_session))
        .route(
            "/api/v1/sessions/:session_id/yields",
            get(get_session_yields),
        )
        // Resume
        .route("/api/v1/sessions/resume", post(resume_session))
        .route(
            "/api/v1/sessions/:session_id/resume-history",
            get(get_resume_history),
        )
        // Attachments
        .route(
            "/api/v1/sessions/:session_id/attachments",
            get(list_attachments),
        )
        .route(
            "/api/v1/sessions/:session_id/attachments",
            post(upload_attachment),
        )
        .route(
            "/api/v1/sessions/:session_id/attachments/:attachment_id",
            get(get_attachment),
        )
        .route(
            "/api/v1/sessions/:session_id/attachments/:attachment_id",
            delete(delete_attachment),
        )
        // State persistence
        .route("/api/v1/sessions/:session_id/state", get(get_session_state))
        .route(
            "/api/v1/sessions/:session_id/state",
            post(save_session_state),
        )
        .route(
            "/api/v1/sessions/:session_id/state",
            delete(delete_session_state),
        )
        // Checkpoints
        .route(
            "/api/v1/sessions/:session_id/checkpoints",
            get(list_checkpoints),
        )
        .route(
            "/api/v1/sessions/:session_id/checkpoints",
            post(create_checkpoint),
        )
        .route(
            "/api/v1/sessions/:session_id/checkpoints/:checkpoint_id",
            get(get_checkpoint),
        )
        .route(
            "/api/v1/sessions/:session_id/checkpoints/:checkpoint_id",
            delete(delete_checkpoint),
        )
        .route(
            "/api/v1/sessions/:session_id/checkpoints/by-name/:name",
            get(get_checkpoint_by_name),
        )
}

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct YieldRequest {
    pub yield_payload: Option<String>,
    pub skip_tool_work: Option<bool>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct YieldResponse {
    pub yield_id: String,
    pub status: String,
    pub child_session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ResumeRequest {
    pub resume_session_id: Option<String>,
    pub original_session_id: Option<String>,
    pub resume_type: String,
    pub context_summary: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResumeResponse {
    pub session_id: String,
    pub resumed_from: String,
    pub context_loaded: bool,
}

#[derive(Debug, Deserialize)]
pub struct SaveStateRequest {
    pub state_data: String,
    pub checkpoint_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StateResponse {
    pub session_id: String,
    pub state_data: serde_json::Value,
    pub checkpoint_id: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCheckpointRequest {
    pub checkpoint_name: String,
    pub checkpoint_data: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UploadAttachmentRequest {
    pub task_id: Option<String>,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub file_path: String,
    pub encoding: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AttachmentResponse {
    pub id: String,
    pub session_id: String,
    pub task_id: Option<String>,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub encoding: String,
    pub uploaded_by: String,
    pub created_at: String,
}

impl From<SessionAttachment> for AttachmentResponse {
    fn from(att: SessionAttachment) -> Self {
        Self {
            id: att.id,
            session_id: att.session_id,
            task_id: att.task_id,
            file_name: att.file_name,
            file_type: att.file_type,
            file_size: att.file_size,
            encoding: att.encoding,
            uploaded_by: att.uploaded_by,
            created_at: att.created_at,
        }
    }
}

// ============ Handlers ============

// Yield

async fn yield_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<YieldRequest>,
) -> Result<Json<YieldResponse>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    // Create child session ID
    let child_session_id = Ulid::new().to_string();
    let yield_id = Ulid::new().to_string();

    repo.log_yield(
        &yield_id,
        &session_id,
        &child_session_id,
        req.reason.as_deref(),
        req.yield_payload.as_deref(),
    )
    .await
    .map_err(|e| format!("Failed to log yield: {}", e))?;

    // NOTE: Inter-worker signaling requires NATS or shared state
    // The yield is logged for the subagent to check on next iteration

    Ok(Json(YieldResponse {
        yield_id,
        status: if req.skip_tool_work.unwrap_or(false) {
            "skipped"
        } else {
            "yielded"
        }
        .to_string(),
        child_session_id,
    }))
}

async fn get_session_yields(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<Vec<SessionYieldLog>>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    let yields = repo
        .get_session_yields(&session_id)
        .await
        .map_err(|e| format!("Failed to get yields: {}", e))?;

    Ok(Json(yields))
}

// Resume

async fn resume_session(
    State(state): State<AppState>,
    Json(req): Json<ResumeRequest>,
) -> Result<Json<ResumeResponse>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    // Determine what to resume from
    let (resumed_from, session_id) = if let Some(id) = req.resume_session_id {
        // Resume from an existing session
        (id.clone(), id)
    } else if let Some(original_id) = req.original_session_id {
        // Resume from an archived/original session
        let new_session_id = Ulid::new().to_string();

        // Log the resume
        let resume_id = Ulid::new().to_string();
        repo.log_resume(
            &resume_id,
            &new_session_id,
            &original_id,
            &req.resume_type,
            req.context_summary.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to log resume: {}", e))?;

        (original_id, new_session_id)
    } else {
        return Err("Must provide either resume_session_id or original_session_id".to_string());
    };

    Ok(Json(ResumeResponse {
        session_id: session_id.to_string(),
        resumed_from: resumed_from,
        context_loaded: true,
    }))
}

async fn get_resume_history(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<Vec<SessionResumeHistory>>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    let history = repo
        .get_session_resume_history(&session_id)
        .await
        .map_err(|e| format!("Failed to get resume history: {}", e))?;

    Ok(Json(history))
}

// Attachments

async fn list_attachments(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<Vec<AttachmentResponse>>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    let attachments = repo
        .get_session_attachments(&session_id)
        .await
        .map_err(|e| format!("Failed to list attachments: {}", e))?;

    Ok(Json(attachments.into_iter().map(|a| a.into()).collect()))
}

async fn upload_attachment(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<UploadAttachmentRequest>,
) -> Result<Json<AttachmentResponse>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    let id = Ulid::new().to_string();
    let attachment = repo
        .add_attachment(
            &id,
            &session_id,
            req.task_id.as_deref(),
            &req.file_name,
            &req.file_type,
            req.file_size,
            &req.file_path,
            req.encoding.as_deref().unwrap_or("binary"),
            "user",
        )
        .await
        .map_err(|e| format!("Failed to upload attachment: {}", e))?;

    Ok(Json(attachment.into()))
}

async fn get_attachment(
    State(state): State<AppState>,
    Path((_session_id, attachment_id)): Path<(String, String)>,
) -> Result<Json<AttachmentResponse>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    let attachment = repo
        .get_attachment(&attachment_id)
        .await
        .map_err(|e| format!("Failed to get attachment: {}", e))?;

    Ok(Json(attachment.into()))
}

async fn delete_attachment(
    State(state): State<AppState>,
    Path((_session_id, attachment_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    repo.delete_attachment(&attachment_id)
        .await
        .map_err(|e| format!("Failed to delete attachment: {}", e))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// Session State

async fn get_session_state(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<Option<StateResponse>>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    match repo.get_session_state(&session_id).await {
        Ok(state) => {
            let state_data: serde_json::Value =
                serde_json::from_str(&state.state_data).unwrap_or(serde_json::json!({}));

            Ok(Json(Some(StateResponse {
                session_id: state.session_id,
                state_data,
                checkpoint_id: state.checkpoint_id,
                updated_at: state.updated_at,
            })))
        }
        Err(sqlx::Error::RowNotFound) => Ok(Json(None)),
        Err(e) => Err(format!("Failed to get session state: {}", e)),
    }
}

async fn save_session_state(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<SaveStateRequest>,
) -> Result<Json<StateResponse>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    let id = format!("state_{}", session_id);
    let state = repo
        .save_session_state(
            &id,
            &session_id,
            &req.state_data,
            req.checkpoint_id.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to save session state: {}", e))?;

    let state_data: serde_json::Value =
        serde_json::from_str(&state.state_data).unwrap_or(serde_json::json!({}));

    Ok(Json(StateResponse {
        session_id: state.session_id,
        state_data,
        checkpoint_id: state.checkpoint_id,
        updated_at: state.updated_at,
    }))
}

async fn delete_session_state(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    repo.delete_session_state(&session_id)
        .await
        .map_err(|e| format!("Failed to delete session state: {}", e))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// Checkpoints

async fn list_checkpoints(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<Vec<SessionCheckpoint>>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    let checkpoints = repo
        .get_session_checkpoints(&session_id)
        .await
        .map_err(|e| format!("Failed to list checkpoints: {}", e))?;

    Ok(Json(checkpoints))
}

async fn create_checkpoint(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<CreateCheckpointRequest>,
) -> Result<Json<SessionCheckpoint>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    let id = Ulid::new().to_string();
    let checkpoint = repo
        .create_checkpoint(
            &id,
            &session_id,
            &req.checkpoint_name,
            &req.checkpoint_data,
            req.description.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to create checkpoint: {}", e))?;

    Ok(Json(checkpoint))
}

async fn get_checkpoint(
    State(state): State<AppState>,
    Path((_session_id, checkpoint_id)): Path<(String, String)>,
) -> Result<Json<SessionCheckpoint>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    let checkpoint = repo
        .get_checkpoint(&checkpoint_id)
        .await
        .map_err(|e| format!("Failed to get checkpoint: {}", e))?;

    Ok(Json(checkpoint))
}

async fn get_checkpoint_by_name(
    State(state): State<AppState>,
    Path((session_id, name)): Path<(String, String)>,
) -> Result<Json<SessionCheckpoint>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    let checkpoint = repo
        .get_checkpoint_by_name(&session_id, &name)
        .await
        .map_err(|e| format!("Failed to get checkpoint: {}", e))?;

    Ok(Json(checkpoint))
}

async fn delete_checkpoint(
    State(state): State<AppState>,
    Path((_session_id, checkpoint_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = SessionControlRepository::new(&state.pool);

    repo.delete_checkpoint(&checkpoint_id)
        .await
        .map_err(|e| format!("Failed to delete checkpoint: {}", e))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
