use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    response::Json as AxumJson,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::AppState;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/soul", get(get_soul).put(update_soul))
        .route("/api/v1/soul/fragments", get(get_soul_fragments))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SoulIdentityResponse {
    pub name: String,
    pub version: String,
    pub created: String,
    pub wake_count: u64,
    pub purpose: String,
    pub values: Vec<ValueResponse>,
    pub capabilities: Vec<CapabilityResponse>,
    pub autonomy_config: AutonomyConfigResponse,
    pub memory_strategy: MemoryStrategyResponse,
    pub relationships: Vec<RelationshipResponse>,
    pub affiliations: Vec<AffiliationResponse>,
    pub current_goals: Vec<GoalResponse>,
    pub reflections: Vec<ReflectionResponse>,
    pub constitution: ConstitutionResponse,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValueResponse {
    pub name: String,
    pub description: String,
    pub priority: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CapabilityResponse {
    pub name: String,
    pub description: String,
    pub tier: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutonomyConfigResponse {
    pub heartbeat_interval_minutes: u64,
    pub max_actions_per_wake: u32,
    pub require_approval_for: Vec<String>,
    pub social_context_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryStrategyResponse {
    pub retention_days: u32,
    pub forgetting_threshold_days: u32,
    pub emphasis_patterns: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelationshipResponse {
    pub agent_id: String,
    pub relationship_type: String,
    pub trust_level: f32,
    pub last_contact: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AffiliationResponse {
    pub name: String,
    pub role: String,
    pub since: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoalResponse {
    pub description: String,
    pub status: String,
    pub priority: u32,
    pub deadline: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReflectionResponse {
    pub timestamp: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConstitutionResponse {
    pub preamble: String,
    pub principles: Vec<PrincipleResponse>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrincipleResponse {
    pub name: String,
    pub description: String,
    pub rule: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SoulUpdateRequest {
    pub name: Option<String>,
    pub purpose: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SoulFragmentsResponse {
    pub fragments: Vec<SoulFragmentResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SoulFragmentResponse {
    pub fragment_type: String,
    pub title: String,
    pub content: String,
}

async fn get_soul(
    State(state): State<AppState>,
) -> Result<AxumJson<Option<SoulIdentityResponse>>, (StatusCode, String)> {
    match state.soul_loader.load_identity().await {
        Ok(identity) => Ok(AxumJson(Some(convert_identity(identity)))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load soul: {}", e),
        )),
    }
}

async fn update_soul(
    State(state): State<AppState>,
    Json(payload): Json<SoulUpdateRequest>,
) -> Result<AxumJson<SoulIdentityResponse>, (StatusCode, String)> {
    if let Ok(mut identity) = state.soul_loader.load_identity().await {
        if let Some(name) = payload.name {
            identity.name = name;
        }
        if let Some(purpose) = payload.purpose {
            identity.purpose = purpose;
        }

        let _ = state.soul_loader.save_identity(&identity).await;

        return Ok(AxumJson(convert_identity(identity)));
    }

    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to update soul".to_string(),
    ))
}

async fn get_soul_fragments(
    State(state): State<AppState>,
) -> Result<AxumJson<SoulFragmentsResponse>, (StatusCode, String)> {
    if let Ok(identity) = state.soul_loader.load_identity().await {
        let mut fragments = vec![];

        fragments.push(SoulFragmentResponse {
            fragment_type: "purpose".to_string(),
            title: "Purpose".to_string(),
            content: identity.purpose.clone(),
        });

        for value in &identity.values {
            fragments.push(SoulFragmentResponse {
                fragment_type: "value".to_string(),
                title: value.name.clone(),
                content: value.description.clone(),
            });
        }

        for goal in &identity.current_goals {
            fragments.push(SoulFragmentResponse {
                fragment_type: "goal".to_string(),
                title: goal.description.clone(),
                content: goal.description.clone(),
            });
        }

        Ok(AxumJson(SoulFragmentsResponse { fragments }))
    } else {
        Ok(AxumJson(SoulFragmentsResponse { fragments: vec![] }))
    }
}

fn convert_identity(identity: crate::soul::SoulIdentity) -> SoulIdentityResponse {
    SoulIdentityResponse {
        name: identity.name,
        version: identity.version,
        created: identity.created,
        wake_count: identity.wake_count,
        purpose: identity.purpose,
        values: identity
            .values
            .into_iter()
            .map(|v| ValueResponse {
                name: v.name,
                description: v.description,
                priority: v.priority,
            })
            .collect(),
        capabilities: identity
            .capabilities
            .into_iter()
            .map(|c| CapabilityResponse {
                name: c.name,
                description: c.description,
                tier: c.tier,
            })
            .collect(),
        autonomy_config: AutonomyConfigResponse {
            heartbeat_interval_minutes: identity.autonomy_config.heartbeat_interval_minutes,
            max_actions_per_wake: identity.autonomy_config.max_actions_per_wake,
            require_approval_for: identity.autonomy_config.require_approval_for,
            social_context_enabled: identity.autonomy_config.social_context_enabled,
        },
        memory_strategy: MemoryStrategyResponse {
            retention_days: identity.memory_strategy.retention_days,
            forgetting_threshold_days: identity.memory_strategy.forgetting_threshold_days,
            emphasis_patterns: identity.memory_strategy.emphasis_patterns,
        },
        relationships: identity
            .relationships
            .into_iter()
            .map(|r| RelationshipResponse {
                agent_id: r.agent_id,
                relationship_type: r.relationship_type,
                trust_level: r.trust_level,
                last_contact: r.last_contact,
            })
            .collect(),
        affiliations: identity
            .affiliations
            .into_iter()
            .map(|a| AffiliationResponse {
                name: a.name,
                role: a.role,
                since: Some(a.joined),
            })
            .collect(),
        current_goals: identity
            .current_goals
            .into_iter()
            .map(|g| GoalResponse {
                description: g.description,
                status: g.status,
                priority: g.priority,
                deadline: g.deadline,
            })
            .collect(),
        reflections: identity
            .reflections
            .into_iter()
            .map(|r| ReflectionResponse {
                timestamp: r.timestamp,
                content: r.content,
            })
            .collect(),
        constitution: ConstitutionResponse {
            preamble: identity.constitution.version,
            principles: vec![],
            constraints: identity.constitution.immutable_values,
        },
    }
}
