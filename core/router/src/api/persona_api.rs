//! Persona API - REST endpoints for persona management
//!
//! Feature 2: Persona Assembly

use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::persona::{
    ModelConfig, Persona, PersonaManager, PromptPiece, PromptPieceType, VoiceConfig,
};
use crate::unified_config::persona_constants::*;

/// Create persona request
#[derive(Debug, Deserialize)]
pub struct CreatePersonaRequest {
    name: String,
    description: Option<String>,
}

/// Update persona request
#[derive(Debug, Deserialize)]
pub struct UpdatePersonaRequest {
    name: Option<String>,
    description: Option<String>,
    prompt_pieces: Option<Vec<PromptPiece>>,
    tools: Option<Vec<String>>,
    voice_config: Option<VoiceConfig>,
    model_config: Option<ModelConfig>,
}

/// Persona response
#[derive(Debug, Serialize)]
pub struct PersonaResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub prompt_pieces: Vec<PromptPiece>,
    pub tools: Vec<String>,
    pub voice_config: VoiceConfig,
    pub model_config: ModelConfig,
    pub is_active: bool,
    pub assembled_prompt: String,
}

impl From<Persona> for PersonaResponse {
    fn from(p: Persona) -> Self {
        let assembled = p.assemble_prompt();
        Self {
            id: p.id,
            name: p.name,
            description: p.description,
            prompt_pieces: p.prompt_pieces,
            tools: p.tools,
            voice_config: p.voice_config,
            model_config: p.model_config,
            is_active: p.is_active,
            assembled_prompt: assembled,
        }
    }
}

/// List personas
pub async fn list_personas(State(_state): State<AppState>) -> Json<Vec<PersonaResponse>> {
    // For now, return default persona (DB storage later)
    let default = PersonaManager::create_default();
    Json(vec![default.into()])
}

/// Get persona by ID
pub async fn get_persona(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PersonaResponse>, String> {
    // For now, only default exists
    if id == "default" {
        let default = PersonaManager::create_default();
        Ok(Json(default.into()))
    } else {
        Err("Persona not found".to_string())
    }
}

/// Create persona
pub async fn create_persona(
    State(_state): State<AppState>,
    Json(payload): Json<CreatePersonaRequest>,
) -> Result<Json<PersonaResponse>, String> {
    // Validate name
    if !PersonaManager::is_valid_name(&payload.name) {
        return Err("Invalid persona name".to_string());
    }

    let mut persona = Persona::new(payload.name);
    persona.description = payload.description;

    persona.validate().map_err(|e| e.to_string())?;

    Ok(Json(persona.into()))
}

/// Update persona
pub async fn update_persona(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePersonaRequest>,
) -> Result<Json<PersonaResponse>, String> {
    if id != "default" {
        return Err("Only default persona can be updated".to_string());
    }

    let mut persona = PersonaManager::create_default();

    if let Some(name) = payload.name {
        persona.name = name;
    }
    if let Some(desc) = payload.description {
        persona.description = Some(desc);
    }
    if let Some(pieces) = payload.prompt_pieces {
        persona.prompt_pieces = pieces;
    }
    if let Some(tools) = payload.tools {
        persona.tools = tools;
    }
    if let Some(vc) = payload.voice_config {
        persona.voice_config = vc;
    }
    if let Some(mc) = payload.model_config {
        persona.model_config = mc;
    }

    persona.validate().map_err(|e| e.to_string())?;

    Ok(Json(persona.into()))
}

/// Delete persona
pub async fn delete_persona(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    if id == "default" {
        return Err("Cannot delete default persona".to_string());
    }

    Ok(Json(serde_json::json!({ "deleted": id })))
}

/// Activate persona
pub async fn activate_persona(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PersonaResponse>, String> {
    // In full implementation, would set is_active in DB
    if id == "default" {
        let mut default = PersonaManager::create_default();
        default.is_active = true;
        Ok(Json(default.into()))
    } else {
        Err("Persona not found".to_string())
    }
}

/// Get active persona
pub async fn get_active_persona(State(_state): State<AppState>) -> Json<PersonaResponse> {
    let default = PersonaManager::create_default();
    Json(default.into())
}

/// Get available prompt piece types
pub async fn get_piece_types() -> Json<Vec<String>> {
    Json(
        PersonaManager::get_piece_types()
            .iter()
            .map(|s| s.to_string())
            .collect(),
    )
}

/// Create router
pub fn create_persona_router() -> Router<AppState> {
    Router::new()
        .route("/personas", get(list_personas))
        .route("/personas", post(create_persona))
        .route("/personas/active", get(get_active_persona))
        .route("/personas/:id", get(get_persona))
        .route("/personas/:id", put(update_persona))
        .route("/personas/:id", delete(delete_persona))
        .route("/personas/:id/activate", post(activate_persona))
        .route("/personas/piece-types", get(get_piece_types))
}
