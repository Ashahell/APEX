use axum::{
    extract::{Query, State},
    response::IntoResponse,
    response::Json as AxumJson,
    routing::{get, post},
    Json, Router,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::moltbook::MoltbookClient;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/moltbook/status", get(get_moltbook_status))
        .route("/api/v1/moltbook/connect", post(connect_moltbook))
        .route("/api/v1/moltbook/disconnect", post(disconnect_moltbook))
        .route("/api/v1/moltbook/agents", get(list_agents))
        .route("/api/v1/social/profile", get(get_social_profile))
        .route("/api/v1/social/post", post(create_post))
        .route("/api/v1/social/notifications", get(get_notifications))
        .route("/api/v1/social/agents/search", get(search_agents))
        .route("/api/v1/social/agents/directory", get(get_agent_directory))
        .route("/api/v1/social/trust", get(assess_trust))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoltbookStatusResponse {
    pub connected: bool,
    pub agent_id: Option<String>,
    pub server_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoltbookConnectResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
}

impl From<crate::moltbook::AgentProfile> for AgentResponse {
    fn from(profile: crate::moltbook::AgentProfile) -> Self {
        Self {
            id: profile.id,
            name: profile.name,
            description: profile.description,
            status: "online".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocialProfileResponse {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
    pub followers: u32,
    pub following: u32,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostResponse {
    pub id: String,
    pub content: String,
    pub created_at: String,
    pub author_id: String,
}

impl From<crate::moltbook::Post> for PostResponse {
    fn from(post: crate::moltbook::Post) -> Self {
        Self {
            id: post.id,
            content: post.content,
            created_at: post.created_at,
            author_id: post.author_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub id: String,
    pub notification_type: String,
    pub content: String,
    pub from_agent_id: String,
    pub created_at: String,
    pub read: bool,
}

impl From<crate::moltbook::Notification> for NotificationResponse {
    fn from(notification: crate::moltbook::Notification) -> Self {
        Self {
            id: notification.id,
            notification_type: notification.notification_type,
            content: notification.message,
            from_agent_id: notification.from_agent_id,
            created_at: notification.created_at,
            read: notification.read,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TrustQuery {
    pub agent_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrustResponse {
    pub agent_id: String,
    pub trust_level: f32,
    pub assessment: String,
}

async fn get_moltbook_status(State(state): State<AppState>) -> Result<AxumJson<MoltbookStatusResponse>, (StatusCode, String)> {
    match &state.moltbook {
        Some(client) => {
            let connected = client.is_connected().await;
            let agent_id = if client.is_enabled() {
                Some(client.agent_id().to_string())
            } else {
                None
            };
            let server_url = if client.is_enabled() {
                Some(client.server_url().to_string())
            } else {
                None
            };
            Ok(AxumJson(MoltbookStatusResponse {
                connected,
                agent_id,
                server_url,
            }))
        }
        None => Ok(AxumJson(MoltbookStatusResponse {
            connected: false,
            agent_id: None,
            server_url: None,
        })),
    }
}

async fn connect_moltbook(State(state): State<AppState>) -> Result<AxumJson<MoltbookConnectResponse>, (StatusCode, String)> {
    match &state.moltbook {
        Some(client) => {
            if !client.is_enabled() {
                return Ok(AxumJson(MoltbookConnectResponse {
                    success: false,
                    message: "Moltbook is not enabled in configuration".to_string(),
                }));
            }
            
            match client.connect_ref().await {
                Ok(_) => Ok(AxumJson(MoltbookConnectResponse {
                    success: true,
                    message: "Connected to Moltbook".to_string(),
                })),
                Err(e) => Ok(AxumJson(MoltbookConnectResponse {
                    success: false,
                    message: format!("Failed to connect: {}", e),
                })),
            }
        }
        None => Ok(AxumJson(MoltbookConnectResponse {
            success: false,
            message: "Moltbook client not initialized".to_string(),
        })),
    }
}

async fn disconnect_moltbook(State(state): State<AppState>) -> Result<AxumJson<MoltbookConnectResponse>, (StatusCode, String)> {
    match &state.moltbook {
        Some(client) => {
            match client.disconnect().await {
                Ok(_) => Ok(AxumJson(MoltbookConnectResponse {
                    success: true,
                    message: "Disconnected from Moltbook".to_string(),
                })),
                Err(e) => Ok(AxumJson(MoltbookConnectResponse {
                    success: false,
                    message: format!("Failed to disconnect: {}", e),
                })),
            }
        }
        None => Ok(AxumJson(MoltbookConnectResponse {
            success: false,
            message: "Moltbook client not initialized".to_string(),
        })),
    }
}

async fn list_agents(State(state): State<AppState>) -> Result<AxumJson<Vec<AgentResponse>>, (StatusCode, String)> {
    match &state.moltbook {
        Some(client) => {
            if !client.is_connected().await {
                return Ok(AxumJson(vec![]));
            }
            match client.get_agent_directory().await {
                Ok(agents) => Ok(AxumJson(agents.into_iter().map(AgentResponse::from).collect())),
                Err(_) => Ok(AxumJson(vec![])),
            }
        }
        None => Ok(AxumJson(vec![])),
    }
}

async fn get_social_profile(State(state): State<AppState>) -> Result<AxumJson<Option<SocialProfileResponse>>, (StatusCode, String)> {
    match &state.moltbook {
        Some(client) => {
            if !client.is_connected().await {
                return Ok(AxumJson(None));
            }
            match client.get_profile().await {
                Some(profile) => Ok(AxumJson(Some(SocialProfileResponse {
                    id: profile.id,
                    name: profile.name,
                    bio: profile.description,
                    followers: 0,
                    following: 0,
                }))),
                None => Ok(AxumJson(None)),
            }
        }
        None => Ok(AxumJson(None)),
    }
}

async fn create_post(
    State(state): State<AppState>,
    Json(payload): Json<CreatePostRequest>,
) -> Result<AxumJson<PostResponse>, (StatusCode, String)> {
    match &state.moltbook {
        Some(client) => {
            if !client.is_connected().await {
                return Err((StatusCode::BAD_REQUEST, "Not connected to Moltbook".to_string()));
            }
            match client.post_update(&payload.content).await {
                Ok(post) => Ok(AxumJson(PostResponse::from(post))),
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to post: {}", e))),
            }
        }
        None => Err((StatusCode::SERVICE_UNAVAILABLE, "Moltbook not available".to_string())),
    }
}

async fn get_notifications(State(state): State<AppState>) -> Result<AxumJson<Vec<NotificationResponse>>, (StatusCode, String)> {
    match &state.moltbook {
        Some(client) => {
            if !client.is_connected().await {
                return Ok(AxumJson(vec![]));
            }
            match client.check_notifications().await {
                Ok(notifications) => Ok(AxumJson(notifications.into_iter().map(NotificationResponse::from).collect())),
                Err(_) => Ok(AxumJson(vec![])),
            }
        }
        None => Ok(AxumJson(vec![])),
    }
}

async fn search_agents(
    Query(query): Query<SearchQuery>,
    State(state): State<AppState>,
) -> Result<AxumJson<Vec<AgentResponse>>, (StatusCode, String)> {
    match &state.moltbook {
        Some(client) => {
            if !client.is_connected().await {
                return Ok(AxumJson(vec![]));
            }
            let search_term = query.q.as_deref().unwrap_or("");
            match client.search_agents(search_term).await {
                Ok(agents) => Ok(AxumJson(agents.into_iter().map(AgentResponse::from).collect())),
                Err(_) => Ok(AxumJson(vec![])),
            }
        }
        None => Ok(AxumJson(vec![])),
    }
}

async fn get_agent_directory(State(state): State<AppState>) -> Result<AxumJson<Vec<AgentResponse>>, (StatusCode, String)> {
    list_agents(State(state)).await
}

async fn assess_trust(
    Query(query): Query<TrustQuery>,
    State(state): State<AppState>,
) -> Result<AxumJson<TrustResponse>, (StatusCode, String)> {
    match &state.moltbook {
        Some(client) => {
            if !client.is_connected().await {
                return Ok(AxumJson(TrustResponse {
                    agent_id: query.agent_id,
                    trust_level: 0.0,
                    assessment: "Not connected to Moltbook".to_string(),
                }));
            }
            match client.assess_trust(&query.agent_id).await {
                Ok(assessment) => Ok(AxumJson(TrustResponse {
                    agent_id: assessment.agent_id,
                    trust_level: assessment.overall_trust as f32,
                    assessment: format!(
                        "Direct: {:.2}, Web of Trust: {:.2}, Institutional: {:.2}, Behavioral: {:.2}",
                        assessment.direct_trust, assessment.web_of_trust, assessment.institutional_vouch, assessment.behavioral_score
                    ),
                })),
                Err(e) => Ok(AxumJson(TrustResponse {
                    agent_id: query.agent_id,
                    trust_level: 0.0,
                    assessment: format!("Unable to assess: {}", e),
                })),
            }
        }
        None => Ok(AxumJson(TrustResponse {
            agent_id: query.agent_id,
            trust_level: 0.0,
            assessment: "Moltbook not available".to_string(),
        })),
    }
}
