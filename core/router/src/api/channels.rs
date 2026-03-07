use axum::{
    extract::{Path, State},
    response::IntoResponse,
    response::Json as AxumJson,
    routing::{get, post, put, delete},
    Json, Router,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use apex_memory::ChannelRepository;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/channels", get(list_channels).post(create_channel))
        .route("/api/v1/channels/:id", get(get_channel).put(update_channel).delete(delete_channel))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    pub name: String,
    pub description: Option<String>,
}

async fn list_channels(State(state): State<AppState>) -> Result<AxumJson<Vec<ChannelResponse>>, (StatusCode, String)> {
    let repo = ChannelRepository::new(&state.pool);
    match repo.find_all().await {
        Ok(channels) => Ok(AxumJson(channels.into_iter().map(|c| ChannelResponse {
            id: c.id,
            name: c.name,
            description: c.description,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }).collect())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list channels: {}", e))),
    }
}

async fn get_channel(Path(id): Path<String>, State(state): State<AppState>) -> Result<AxumJson<Option<ChannelResponse>>, (StatusCode, String)> {
    let repo = ChannelRepository::new(&state.pool);
    match repo.find_by_id(&id).await {
        Ok(c) => Ok(AxumJson(c.map(|c| ChannelResponse {
            id: c.id,
            name: c.name,
            description: c.description,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get channel: {}", e))),
    }
}

async fn create_channel(
    State(state): State<AppState>,
    Json(payload): Json<CreateChannelRequest>,
) -> Result<(StatusCode, AxumJson<ChannelResponse>), (StatusCode, String)> {
    let id = ulid::Ulid::new().to_string();
    let create = apex_memory::CreateChannel {
        name: payload.name,
        description: payload.description,
    };
    let repo = ChannelRepository::new(&state.pool);
    
    match repo.create(&id, create).await {
        Ok(_) => {
            if let Ok(Some(c)) = repo.find_by_id(&id).await {
                Ok((StatusCode::CREATED, AxumJson(ChannelResponse {
                    id: c.id,
                    name: c.name,
                    description: c.description,
                    created_at: c.created_at,
                    updated_at: c.updated_at,
                })))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to find created channel".to_string()))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create channel: {}", e))),
    }
}

async fn update_channel(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateChannelRequest>,
) -> Result<AxumJson<Option<ChannelResponse>>, (StatusCode, String)> {
    let update = apex_memory::CreateChannel {
        name: payload.name,
        description: payload.description,
    };
    let repo = ChannelRepository::new(&state.pool);
    
    match repo.update(&id, update).await {
        Ok(_) => {
            if let Ok(Some(c)) = repo.find_by_id(&id).await {
                Ok(AxumJson(Some(ChannelResponse {
                    id: c.id,
                    name: c.name,
                    description: c.description,
                    created_at: c.created_at,
                    updated_at: c.updated_at,
                })))
            } else {
                Ok(AxumJson(None))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update channel: {}", e))),
    }
}

async fn delete_channel(Path(id): Path<String>, State(state): State<AppState>) -> Result<AxumJson<bool>, (StatusCode, String)> {
    let repo = ChannelRepository::new(&state.pool);
    match repo.delete(&id).await {
        Ok(_) => Ok(AxumJson(true)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete channel: {}", e))),
    }
}
