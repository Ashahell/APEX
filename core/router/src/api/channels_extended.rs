use axum::{
    extract::{State, Path, Query},
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use apex_memory::channel_settings_repo::{
    ChannelSettingsRepository, ChannelSettings, ChannelTemplate, ChannelWebhook,
    SUPPORTED_CHANNEL_TYPES, get_channel_display_name, get_channel_icon,
};

use crate::api::AppState;

// ============ Router ============

pub fn router() -> Router<AppState> {
    Router::new()
        // Channel settings
        .route("/api/v1/channels/extended/list", get(list_extended_channels))
        .route("/api/v1/channels/extended/types", get(list_channel_types))
        .route("/api/v1/channels/extended", post(create_channel_settings))
        .route("/api/v1/channels/extended/:id", get(get_channel_settings))
        .route("/api/v1/channels/extended/:id", put(update_channel_settings))
        .route("/api/v1/channels/extended/:id/toggle", put(toggle_channel_enabled))
        .route("/api/v1/channels/extended/:id", delete(delete_channel_settings))
        
        // Templates - use POST with body for delete to avoid route conflicts
        .route("/api/v1/channels/extended/templates", post(create_template))
        .route("/api/v1/channels/extended/templates", delete(delete_template))
        .route("/api/v1/channels/extended/templates/list/:channel_type", get(list_templates))
        
        // Webhooks - use POST with body for delete/toggle to avoid route conflicts  
        .route("/api/v1/channels/extended/webhooks", post(create_webhook))
        .route("/api/v1/channels/extended/webhooks/list/:channel_type", get(list_webhooks))
        .route("/api/v1/channels/extended/webhooks/toggle", put(toggle_webhook))
        .route("/api/v1/channels/extended/webhooks", delete(delete_webhook))
}

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct CreateChannelSettingsRequest {
    pub channel_type: String,
    pub channel_id: String,
    pub settings: serde_json::Value,
    pub credentials: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelSettingsRequest {
    pub settings: Option<serde_json::Value>,
    pub credentials: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ToggleEnabledRequest {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub channel_type: String,
    pub template_name: String,
    pub template_content: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct DeleteByIdRequest {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct ToggleByIdRequest {
    pub id: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub channel_type: String,
    pub channel_id: String,
    pub webhook_url: String,
    pub events: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ChannelSettingsResponse {
    pub id: String,
    pub channel_type: String,
    pub channel_id: String,
    pub settings: serde_json::Value,
    pub is_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    pub display_name: String,
    pub icon: String,
}

impl From<ChannelSettings> for ChannelSettingsResponse {
    fn from(s: ChannelSettings) -> Self {
        let settings: serde_json::Value = serde_json::from_str(&s.settings).unwrap_or(serde_json::json!({}));
        
        Self {
            id: s.id,
            channel_type: s.channel_type.clone(),
            channel_id: s.channel_id,
            settings,
            is_enabled: s.is_enabled == 1,
            created_at: s.created_at,
            updated_at: s.updated_at,
            display_name: get_channel_display_name(&s.channel_type).to_string(),
            icon: get_channel_icon(&s.channel_type).to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ChannelTypeInfo {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub has_credentials: bool,
}

// ============ Handlers ============

// List all extended channels
async fn list_extended_channels(
    State(state): State<AppState>,
) -> Result<Json<Vec<ChannelSettingsResponse>>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    let settings = repo
        .get_enabled()
        .await
        .map_err(|e| format!("Failed to list channels: {}", e))?;
    
    Ok(Json(settings.into_iter().map(|s| s.into()).collect()))
}

// List supported channel types
async fn list_channel_types() -> Json<Vec<ChannelTypeInfo>> {
    let types: Vec<ChannelTypeInfo> = SUPPORTED_CHANNEL_TYPES
        .iter()
        .map(|t| ChannelTypeInfo {
            id: t.to_string(),
            name: get_channel_display_name(t).to_string(),
            icon: get_channel_icon(t).to_string(),
            has_credentials: true,  // Most channels need credentials
        })
        .collect();
    
    Json(types)
}

// Create channel settings
async fn create_channel_settings(
    State(state): State<AppState>,
    Json(req): Json<CreateChannelSettingsRequest>,
) -> Result<Json<ChannelSettingsResponse>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    let id = Ulid::new().to_string();
    let settings_json = serde_json::to_string(&req.settings).unwrap_or("{}".to_string());
    
    // SECURITY NOTE: Credentials should be encrypted before storage
    // See secret_store.rs for encryption utilities
    let credentials_json = req.credentials
        .map(|c| serde_json::to_string(&c).unwrap_or_default());
    
    let settings = repo
        .create_settings(&id, &req.channel_type, &req.channel_id, &settings_json)
        .await
        .map_err(|e| format!("Failed to create channel: {}", e))?;
    
    // Update credentials if provided
    if let Some(creds) = credentials_json {
        let _ = repo.update_settings(&id, &settings_json, Some(&creds)).await;
    }
    
    // Reload to get updated
    let updated = repo
        .get_settings(&id)
        .await
        .map_err(|e| format!("Failed to get channel: {}", e))?;
    
    Ok(Json(updated.into()))
}

// Get channel settings
async fn get_channel_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ChannelSettingsResponse>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    let settings = repo
        .get_settings(&id)
        .await
        .map_err(|e| format!("Failed to get channel: {}", e))?;
    
    Ok(Json(settings.into()))
}

// Update channel settings
async fn update_channel_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateChannelSettingsRequest>,
) -> Result<Json<ChannelSettingsResponse>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    // Get existing
    let existing = repo
        .get_settings(&id)
        .await
        .map_err(|e| format!("Channel not found: {}", e))?;
    
    // Merge settings
    let new_settings = if let Some(new_settings) = req.settings {
        let mut current: serde_json::Value = serde_json::from_str(&existing.settings).unwrap_or(serde_json::json!({}));
        if let serde_json::Value::Object(map) = &mut current {
            if let serde_json::Value::Object(new_map) = new_settings {
                for (k, v) in new_map.iter() {
                    map.insert(k.clone(), v.clone());
                }
            }
        }
        serde_json::to_string(&current).unwrap_or(existing.settings)
    } else {
        existing.settings
    };
    
    // Merge credentials
    let credentials = req.credentials
        .map(|c| serde_json::to_string(&c).unwrap_or_default());
    
    let updated = repo
        .update_settings(&id, &new_settings, credentials.as_deref())
        .await
        .map_err(|e| format!("Failed to update channel: {}", e))?;
    
    Ok(Json(updated.into()))
}

// Toggle channel enabled
async fn toggle_channel_enabled(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ToggleEnabledRequest>,
) -> Result<Json<ChannelSettingsResponse>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    let updated = repo
        .toggle_enabled(&id, req.enabled)
        .await
        .map_err(|e| format!("Failed to toggle channel: {}", e))?;
    
    Ok(Json(updated.into()))
}

// Delete channel settings
async fn delete_channel_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    repo
        .delete_settings(&id)
        .await
        .map_err(|e| format!("Failed to delete channel: {}", e))?;
    
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ============ Template Handlers ============

// List templates for channel type
async fn list_templates(
    State(state): State<AppState>,
    Path(channel_type): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    let templates = repo
        .get_templates(&channel_type)
        .await
        .map_err(|e| format!("Failed to list templates: {}", e))?;
    
    let results: Vec<serde_json::Value> = templates
        .into_iter()
        .map(|t| {
            let content: serde_json::Value = serde_json::from_str(&t.template_content).unwrap_or(serde_json::json!({}));
            serde_json::json!({
                "id": t.id,
                "template_name": t.template_name,
                "template_content": content,
                "is_default": t.is_default == 1,
                "created_at": t.created_at,
            })
        })
        .collect();
    
    Ok(Json(results))
}

// Create template
async fn create_template(
    State(state): State<AppState>,
    Json(req): Json<CreateTemplateRequest>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    let id = Ulid::new().to_string();
    let content = serde_json::to_string(&req.template_content).unwrap_or("{}".to_string());
    
    let template = repo
        .create_template(&id, &req.channel_type, &req.template_name, &content)
        .await
        .map_err(|e| format!("Failed to create template: {}", e))?;
    
    Ok(Json(serde_json::json!({
        "id": template.id,
        "template_name": template.template_name,
        "created_at": template.created_at,
    })))
}

// Delete template
async fn delete_template(
    State(state): State<AppState>,
    Json(req): Json<DeleteByIdRequest>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    repo
        .delete_template(&req.id)
        .await
        .map_err(|e| format!("Failed to delete template: {}", e))?;
    
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ============ Webhook Handlers ============

// List webhooks for channel type
async fn list_webhooks(
    State(state): State<AppState>,
    Path(channel_type): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    let webhooks = repo
        .get_webhooks(&channel_type)
        .await
        .map_err(|e| format!("Failed to list webhooks: {}", e))?;
    
    let results: Vec<serde_json::Value> = webhooks
        .into_iter()
        .map(|w| {
            let events: serde_json::Value = serde_json::from_str(&w.events).unwrap_or(serde_json::json!([]));
            serde_json::json!({
                "id": w.id,
                "channel_id": w.channel_id,
                "webhook_url": w.webhook_url,
                "events": events,
                "is_enabled": w.is_enabled == 1,
                "created_at": w.created_at,
            })
        })
        .collect();
    
    Ok(Json(results))
}

// Create webhook
async fn create_webhook(
    State(state): State<AppState>,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    let id = Ulid::new().to_string();
    let events = serde_json::to_string(&req.events).unwrap_or("[]".to_string());
    
    let webhook = repo
        .create_webhook(&id, &req.channel_type, &req.channel_id, &req.webhook_url, &events)
        .await
        .map_err(|e| format!("Failed to create webhook: {}", e))?;
    
    Ok(Json(serde_json::json!({
        "id": webhook.id,
        "created_at": webhook.created_at,
    })))
}

// Toggle webhook
async fn toggle_webhook(
    State(state): State<AppState>,
    Json(req): Json<ToggleByIdRequest>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    repo
        .toggle_webhook(&req.id, req.enabled)
        .await
        .map_err(|e| format!("Failed to toggle webhook: {}", e))?;
    
    Ok(Json(serde_json::json!({ "success": true })))
}

// Delete webhook
async fn delete_webhook(
    State(state): State<AppState>,
    Json(req): Json<DeleteByIdRequest>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = ChannelSettingsRepository::new(&state.pool);
    
    repo
        .delete_webhook(&req.id)
        .await
        .map_err(|e| format!("Failed to delete webhook: {}", e))?;
    
    Ok(Json(serde_json::json!({ "deleted": true })))
}
