use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    response::Json,
    routing::{delete, get, post, put},
    Json as AxumJson, Router,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use apex_memory::dashboard_repo::{
    ChatBookmark, CommandPaletteHistory, DashboardChatHistory, DashboardExport, DashboardLayout,
    DashboardRepository, PinnedMessage, SessionMetadata,
};

use crate::api::AppState;

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct DashboardLayoutRequest {
    pub layout_config: Option<String>,
    pub theme: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardLayoutResponse {
    pub id: String,
    pub layout_config: serde_json::Value,
    pub theme: String,
}

impl From<DashboardLayout> for DashboardLayoutResponse {
    fn from(layout: DashboardLayout) -> Self {
        let config: serde_json::Value =
            serde_json::from_str(&layout.layout_config).unwrap_or(serde_json::json!({}));
        Self {
            id: layout.id,
            layout_config: config,
            theme: layout.theme,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PinMessageRequest {
    pub message_id: String,
    pub channel: String,
    pub thread_id: Option<String>,
    pub task_id: Option<String>,
    pub pin_note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PinMessageResponse {
    pub id: String,
    pub message_id: String,
    pub channel: String,
    pub thread_id: Option<String>,
    pub task_id: Option<String>,
    pub pinned_by: String,
    pub pin_note: Option<String>,
    pub pinned_at: String,
}

impl From<PinnedMessage> for PinMessageResponse {
    fn from(msg: PinnedMessage) -> Self {
        Self {
            id: msg.id,
            message_id: msg.message_id,
            channel: msg.channel,
            thread_id: msg.thread_id,
            task_id: msg.task_id,
            pinned_by: msg.pinned_by,
            pin_note: msg.pin_note,
            pinned_at: msg.pinned_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddBookmarkRequest {
    pub message_id: String,
    pub channel: String,
    pub thread_id: Option<String>,
    pub bookmark_note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkListRequest {
    pub channel: Option<String>,
    pub thread_id: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookmarkResponse {
    pub id: String,
    pub message_id: String,
    pub channel: String,
    pub thread_id: Option<String>,
    pub bookmark_note: Option<String>,
    pub created_at: String,
}

impl From<ChatBookmark> for BookmarkResponse {
    fn from(bookmark: ChatBookmark) -> Self {
        Self {
            id: bookmark.id,
            message_id: bookmark.message_id,
            channel: bookmark.channel,
            thread_id: bookmark.thread_id,
            bookmark_note: bookmark.bookmark_note,
            created_at: bookmark.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateSessionRequest {
    pub model: Option<String>,
    pub thinking_level: Option<String>,
    pub verbose_level: Option<String>,
    pub fast_mode: Option<bool>,
    pub send_policy: Option<String>,
    pub activation_mode: Option<String>,
    pub group_policy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMetadataResponse {
    pub session_id: String,
    pub model: Option<String>,
    pub thinking_level: String,
    pub verbose_level: String,
    pub fast_mode: bool,
    pub send_policy: String,
    pub activation_mode: String,
    pub group_policy: Option<String>,
    pub updated_at: String,
}

impl From<SessionMetadata> for SessionMetadataResponse {
    fn from(meta: SessionMetadata) -> Self {
        Self {
            session_id: meta.session_id,
            model: meta.model,
            thinking_level: meta.thinking_level,
            verbose_level: meta.verbose_level,
            fast_mode: meta.fast_mode != 0,
            send_policy: meta.send_policy,
            activation_mode: meta.activation_mode,
            group_policy: meta.group_policy,
            updated_at: meta.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchMessagesRequest {
    pub q: String,
    pub channel: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageResponse {
    pub id: String,
    pub message_id: String,
    pub channel: String,
    pub thread_id: Option<String>,
    pub author: String,
    pub content: String,
    pub role: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

impl From<DashboardChatHistory> for ChatMessageResponse {
    fn from(msg: DashboardChatHistory) -> Self {
        let metadata: Option<serde_json::Value> = msg
            .metadata
            .as_ref()
            .and_then(|m| serde_json::from_str(m).ok());
        Self {
            id: msg.id,
            message_id: msg.message_id,
            channel: msg.channel,
            thread_id: msg.thread_id,
            author: msg.author,
            content: msg.content,
            role: msg.role,
            metadata,
            created_at: msg.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultResponse {
    pub messages: Vec<ChatMessageResponse>,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct RecordCommandRequest {
    pub command: String,
    pub command_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandHistoryResponse {
    pub command: String,
    pub command_type: String,
    pub frequency: i32,
    pub last_used: String,
}

impl From<CommandPaletteHistory> for CommandHistoryResponse {
    fn from(cmd: CommandPaletteHistory) -> Self {
        Self {
            command: cmd.command,
            command_type: cmd.command_type,
            frequency: cmd.frequency,
            last_used: cmd.last_used,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub session_id: String,
    pub export_format: String,
    pub export_range: String,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResponse {
    pub id: String,
    pub session_id: String,
    pub export_format: String,
    pub export_range: String,
    pub status: String,
    pub created_at: String,
}

impl From<DashboardExport> for ExportResponse {
    fn from(exp: DashboardExport) -> Self {
        Self {
            id: exp.id,
            session_id: exp.session_id,
            export_format: exp.export_format,
            export_range: exp.export_range,
            status: exp.status,
            created_at: exp.created_at,
        }
    }
}

// ============ Router ============

pub fn router() -> Router<AppState> {
    Router::new()
        // Dashboard layout
        .route(
            "/api/v1/dashboard/layout",
            get(get_layout).put(update_layout),
        )
        // Pinned messages
        .route("/api/v1/dashboard/pins", get(list_pins).post(pin_message))
        .route("/api/v1/dashboard/pins/:id", delete(unpin_message))
        // Bookmarks - simplified to POST only for now
        .route("/api/v1/dashboard/bookmarks", post(add_bookmark))
        .route("/api/v1/dashboard/bookmarks/:id", delete(delete_bookmark))
        // Chat search
        .route("/api/v1/dashboard/search", get(search_messages))
        // Session management
        .route("/api/v1/dashboard/sessions", get(list_sessions))
        .route(
            "/api/v1/dashboard/sessions/:session_id",
            get(get_session).put(update_session),
        )
        // Command palette
        .route(
            "/api/v1/dashboard/commands",
            get(list_commands).post(record_command),
        )
        // Export
        .route("/api/v1/dashboard/export", post(create_export))
        .route("/api/v1/dashboard/export/:id", get(get_export))
}

// ============ Handlers ============

// Dashboard Layout

async fn get_layout(
    State(state): State<AppState>,
) -> Result<Json<Option<DashboardLayoutResponse>>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let layout = repo
        .get_layout("default")
        .await
        .map_err(|e| format!("Failed to get layout: {}", e))?;

    Ok(Json(layout.map(|l| l.into())))
}

async fn update_layout(
    State(state): State<AppState>,
    Json(req): Json<DashboardLayoutRequest>,
) -> Result<Json<DashboardLayoutResponse>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let id = Ulid::new().to_string();
    let config = req.layout_config.unwrap_or_else(|| "{}".to_string());
    let theme = req.theme.unwrap_or_else(|| "agentzero".to_string());

    let layout = repo
        .upsert_layout(&id, "default", &config, &theme)
        .await
        .map_err(|e| format!("Failed to update layout: {}", e))?;

    Ok(Json(layout.into()))
}

// Pinned Messages

async fn list_pins(
    State(state): State<AppState>,
    Query(PinListRequest { channel, limit }): Query<PinListRequest>,
) -> Result<Json<Vec<PinMessageResponse>>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let limit = limit.unwrap_or(50);
    let pins = repo
        .list_pinned(channel.as_deref(), limit as i64)
        .await
        .map_err(|e| format!("Failed to list pins: {}", e))?;

    Ok(Json(pins.into_iter().map(|p| p.into()).collect()))
}

#[derive(Debug, Deserialize)]
pub struct PinListRequest {
    pub channel: Option<String>,
    pub limit: Option<i32>,
}

async fn pin_message(
    State(state): State<AppState>,
    Json(req): Json<PinMessageRequest>,
) -> Result<Json<PinMessageResponse>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let id = Ulid::new().to_string();
    let pinned = repo
        .pin_message(
            &id,
            &req.message_id,
            &req.channel,
            req.thread_id.as_deref(),
            req.task_id.as_deref(),
            "user",
            req.pin_note.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to pin message: {}", e))?;

    Ok(Json(pinned.into()))
}

async fn unpin_message(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, String> {
    let repo = DashboardRepository::new(&state.pool);

    repo.unpin_message(&id)
        .await
        .map_err(|e| format!("Failed to unpin message: {}", e))?;

    Ok((StatusCode::NO_CONTENT, ""))
}

// Bookmarks

async fn list_bookmarks(
    State(state): State<AppState>,
    Json(req): Json<BookmarkListRequest>,
) -> Result<Json<Vec<BookmarkResponse>>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let limit = req.limit.unwrap_or(50);
    let bookmarks = repo
        .list_bookmarks(
            req.channel.as_deref(),
            req.thread_id.as_deref(),
            limit as i64,
        )
        .await
        .map_err(|e| format!("Failed to list bookmarks: {}", e))?;

    Ok(Json(bookmarks.into_iter().map(|b| b.into()).collect()))
}

async fn add_bookmark(
    State(state): State<AppState>,
    Json(req): Json<AddBookmarkRequest>,
) -> Result<Json<BookmarkResponse>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let id = Ulid::new().to_string();
    let bookmark = repo
        .add_bookmark(
            &id,
            &req.message_id,
            &req.channel,
            req.thread_id.as_deref(),
            req.bookmark_note.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to add bookmark: {}", e))?;

    Ok(Json(bookmark.into()))
}

async fn delete_bookmark(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, String> {
    let repo = DashboardRepository::new(&state.pool);

    repo.delete_bookmark(&id)
        .await
        .map_err(|e| format!("Failed to delete bookmark: {}", e))?;

    Ok((StatusCode::NO_CONTENT, ""))
}

// Chat Search

async fn search_messages(
    State(state): State<AppState>,
    Query(SearchMessagesRequest {
        q,
        channel,
        limit,
        offset,
    }): Query<SearchMessagesRequest>,
) -> Result<Json<SearchResultResponse>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let limit = limit.unwrap_or(20) as i64;
    let offset = offset.unwrap_or(0) as i64;

    let messages = repo
        .search_chat(&q, channel.as_deref(), limit, offset)
        .await
        .map_err(|e| format!("Failed to search messages: {}", e))?;

    let total = messages.len() as i64;

    Ok(Json(SearchResultResponse {
        messages: messages.into_iter().map(|m| m.into()).collect(),
        total,
    }))
}

// Session Management

async fn list_sessions(
    State(state): State<AppState>,
    Query(SessionListRequest { limit }): Query<SessionListRequest>,
) -> Result<Json<Vec<SessionMetadataResponse>>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let limit = limit.unwrap_or(50) as i64;
    let sessions = repo
        .list_sessions_metadata(limit)
        .await
        .map_err(|e| format!("Failed to list sessions: {}", e))?;

    Ok(Json(sessions.into_iter().map(|s| s.into()).collect()))
}

#[derive(Debug, Deserialize)]
pub struct SessionListRequest {
    pub limit: Option<i32>,
}

async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionMetadataResponse>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let meta = repo
        .find_session_metadata(&session_id)
        .await
        .map_err(|e| format!("Failed to get session: {}", e))?;

    Ok(Json(meta.into()))
}

async fn update_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<UpdateSessionRequest>,
) -> Result<Json<SessionMetadataResponse>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let id = Ulid::new().to_string();
    let meta = repo
        .upsert_session_metadata(
            &id,
            &session_id,
            req.model.as_deref(),
            req.thinking_level.as_deref(),
            req.verbose_level.as_deref(),
            req.fast_mode.map(|f| if f { 1 } else { 0 }),
            req.send_policy.as_deref(),
            req.activation_mode.as_deref(),
            req.group_policy.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to update session: {}", e))?;

    Ok(Json(meta.into()))
}

// Command Palette

async fn list_commands(
    State(state): State<AppState>,
    Query(CommandListRequest {
        command_type,
        limit,
    }): Query<CommandListRequest>,
) -> Result<Json<Vec<CommandHistoryResponse>>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let limit = limit.unwrap_or(20) as i64;
    let commands = repo
        .list_commands(command_type.as_deref(), limit)
        .await
        .map_err(|e| format!("Failed to list commands: {}", e))?;

    Ok(Json(commands.into_iter().map(|c| c.into()).collect()))
}

#[derive(Debug, Deserialize)]
pub struct CommandListRequest {
    pub command_type: Option<String>,
    pub limit: Option<i32>,
}

async fn record_command(
    State(state): State<AppState>,
    Json(req): Json<RecordCommandRequest>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let id = Ulid::new().to_string();
    repo.record_command(&id, &req.command, &req.command_type)
        .await
        .map_err(|e| format!("Failed to record command: {}", e))?;

    Ok(Json(serde_json::json!({ "recorded": true })))
}

// Export

async fn create_export(
    State(state): State<AppState>,
    Json(req): Json<ExportRequest>,
) -> Result<Json<ExportResponse>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let id = Ulid::new().to_string();
    let export = repo
        .create_export(
            &id,
            &req.session_id,
            &req.export_format,
            &req.export_range,
            req.date_from.as_deref(),
            req.date_to.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to create export: {}", e))?;

    Ok(Json(export.into()))
}

async fn get_export(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ExportResponse>, String> {
    let repo = DashboardRepository::new(&state.pool);

    let export = repo
        .find_export(&id)
        .await
        .map_err(|e| format!("Failed to get export: {}", e))?;

    Ok(Json(export.into()))
}
