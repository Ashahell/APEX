use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    response::Json as AxumJson,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/heartbeat/config",
            get(get_heartbeat_config).post(update_heartbeat_config),
        )
        .route("/api/v1/heartbeat/stats", get(get_heartbeat_stats))
        .route("/api/v1/heartbeat/trigger", post(trigger_heartbeat))
        .route("/api/v1/heartbeat/toggle", post(toggle_heartbeat))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatConfigResponse {
    pub enabled: bool,
    pub interval_minutes: u64,
    pub jitter_percent: u32,
    pub cooldown_seconds: u64,
    pub max_actions_per_wake: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatStatsResponse {
    pub is_running: bool,
    pub wake_count: u64,
    pub last_wake: Option<String>,
    pub next_scheduled_wake: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatToggleResponse {
    pub enabled: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatConfigUpdate {
    pub interval_minutes: Option<u64>,
    pub jitter_percent: Option<u32>,
    pub cooldown_seconds: Option<u64>,
    pub max_actions_per_wake: Option<u32>,
}

async fn get_heartbeat_config(
    State(state): State<AppState>,
) -> Result<AxumJson<HeartbeatConfigResponse>, (StatusCode, String)> {
    let config = state.config.heartbeat.clone();
    Ok(AxumJson(HeartbeatConfigResponse {
        enabled: config.enabled,
        interval_minutes: config.interval_minutes,
        jitter_percent: config.jitter_percent,
        cooldown_seconds: config.cooldown_seconds,
        max_actions_per_wake: config.max_actions_per_wake,
    }))
}

async fn update_heartbeat_config(
    State(mut state): State<AppState>,
    Json(payload): Json<HeartbeatConfigUpdate>,
) -> Result<AxumJson<HeartbeatConfigResponse>, (StatusCode, String)> {
    let mut config = state.config.heartbeat.clone();

    if let Some(interval) = payload.interval_minutes {
        config.interval_minutes = interval;
    }
    if let Some(jitter) = payload.jitter_percent {
        config.jitter_percent = jitter;
    }
    if let Some(cooldown) = payload.cooldown_seconds {
        config.cooldown_seconds = cooldown;
    }
    if let Some(max_actions) = payload.max_actions_per_wake {
        config.max_actions_per_wake = max_actions;
    }

    state.config.heartbeat = config.clone();

    Ok(AxumJson(HeartbeatConfigResponse {
        enabled: config.enabled,
        interval_minutes: config.interval_minutes,
        jitter_percent: config.jitter_percent,
        cooldown_seconds: config.cooldown_seconds,
        max_actions_per_wake: config.max_actions_per_wake,
    }))
}

async fn get_heartbeat_stats(
    State(state): State<AppState>,
) -> Result<AxumJson<HeartbeatStatsResponse>, (StatusCode, String)> {
    let is_running = state.heartbeat_scheduler.is_running().await;
    let wake_count = state.heartbeat_scheduler.get_wake_count().await;
    let last_wake = state.heartbeat_scheduler.get_last_wake().await;

    Ok(AxumJson(HeartbeatStatsResponse {
        is_running,
        wake_count,
        last_wake,
        next_scheduled_wake: None,
    }))
}

async fn trigger_heartbeat(
    State(state): State<AppState>,
) -> Result<AxumJson<HeartbeatStatsResponse>, (StatusCode, String)> {
    state.heartbeat_scheduler.force_wake().await;

    let is_running = state.heartbeat_scheduler.is_running().await;
    let wake_count = state.heartbeat_scheduler.get_wake_count().await;
    let last_wake = state.heartbeat_scheduler.get_last_wake().await;

    Ok(AxumJson(HeartbeatStatsResponse {
        is_running,
        wake_count,
        last_wake,
        next_scheduled_wake: None,
    }))
}

async fn toggle_heartbeat(
    State(state): State<AppState>,
) -> Result<AxumJson<HeartbeatToggleResponse>, (StatusCode, String)> {
    let currently_running = state.heartbeat_scheduler.is_running().await;

    if currently_running {
        state.heartbeat_scheduler.stop().await;
        Ok(AxumJson(HeartbeatToggleResponse {
            enabled: false,
            message: "Heartbeat daemon stopped".to_string(),
        }))
    } else {
        state.heartbeat_scheduler.start().await;
        Ok(AxumJson(HeartbeatToggleResponse {
            enabled: true,
            message: "Heartbeat daemon started".to_string(),
        }))
    }
}
