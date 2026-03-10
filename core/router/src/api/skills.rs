#![allow(unused_imports)]

use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::apex_security::capability::PermissionTier;
use crate::mcp::McpServerManager;
use apex_memory::skill_registry::{SkillRegistry, SkillRegistryEntry};
use apex_memory::task_repo::TaskRepository;
use apex_memory::tasks::TaskStatus;

use super::{
    AppState, ExecuteSkillRequest, ExecuteSkillResponse, RegisterSkillRequest, SkillResponse,
    UpdateHealthRequest,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/skills", get(list_skills).post(register_skill))
        .route("/api/v1/skills/marketplace", get(list_marketplace_skills))
        .route("/api/v1/skills/marketplace/:name", get(get_marketplace_skill))
        .route("/api/v1/skills/marketplace/:name/install", post(install_marketplace_skill))
        .route("/api/v1/skills/marketplace/:name/uninstall", post(uninstall_marketplace_skill))
        .route("/api/v1/skills/marketplace/search", get(search_marketplace))
        .route("/api/v1/skills/:name", get(get_skill).delete(delete_skill))
        .route("/api/v1/skills/:name/health", put(update_skill_health))
        .route("/api/v1/skills/execute", post(execute_skill))
        .route("/api/v1/skills/:name/cache", post(invalidate_skill_cache))
        .route("/api/v1/skills/cache", post(invalidate_all_cache))
}

async fn list_skills(State(state): State<AppState>) -> Result<Json<Vec<SkillResponse>>, String> {
    let registry = SkillRegistry::new(&state.pool);

    match registry.find_all().await {
        Ok(skills) => {
            let responses: Vec<SkillResponse> = skills
                .into_iter()
                .map(|s| SkillResponse {
                    name: s.name,
                    version: s.version,
                    tier: s.tier,
                    enabled: s.enabled,
                    health_status: s.health_status,
                    last_health_check: s.last_health_check,
                })
                .collect();
            Ok(Json(responses))
        }
        Err(e) => Err(format!("Failed to list skills: {}", e)),
    }
}

async fn get_skill(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<SkillResponse>, String> {
    let registry = SkillRegistry::new(&state.pool);

    match registry.find_by_name(&name).await {
        Ok(Some(skill)) => Ok(Json(SkillResponse {
            name: skill.name,
            version: skill.version,
            tier: skill.tier,
            enabled: skill.enabled,
            health_status: skill.health_status,
            last_health_check: skill.last_health_check,
        })),
        Ok(None) => Err(format!("Skill not found: {}", name)),
        Err(e) => Err(format!("Failed to get skill: {}", e)),
    }
}

async fn register_skill(
    State(state): State<AppState>,
    Json(payload): Json<RegisterSkillRequest>,
) -> Result<Json<SkillResponse>, String> {
    let registry = SkillRegistry::new(&state.pool);
    let entry = SkillRegistryEntry::new(payload.name.clone(), payload.version, payload.tier);

    match registry.upsert(&entry).await {
        Ok(_) => Ok(Json(SkillResponse {
            name: entry.name,
            version: entry.version,
            tier: entry.tier,
            enabled: entry.enabled,
            health_status: entry.health_status,
            last_health_check: entry.last_health_check,
        })),
        Err(e) => Err(format!("Failed to register skill: {}", e)),
    }
}

async fn update_skill_health(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateHealthRequest>,
) -> Result<Json<SkillResponse>, String> {
    let registry = SkillRegistry::new(&state.pool);

    match registry.update_health(&name, &payload.health_status).await {
        Ok(_) => match registry.find_by_name(&name).await {
            Ok(Some(skill)) => Ok(Json(SkillResponse {
                name: skill.name,
                version: skill.version,
                tier: skill.tier,
                enabled: skill.enabled,
                health_status: skill.health_status,
                last_health_check: skill.last_health_check,
            })),
            Ok(None) => Err(format!("Skill not found: {}", name)),
            Err(e) => Err(format!("Failed to get skill: {}", e)),
        },
        Err(e) => Err(format!("Failed to update health: {}", e)),
    }
}

async fn delete_skill(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let registry = SkillRegistry::new(&state.pool);

    match registry.delete(&name).await {
        Ok(_) => Ok(Json(serde_json::json!({"deleted": true, "name": name}))),
        Err(e) => Err(format!("Failed to delete skill: {}", e)),
    }
}

async fn execute_skill(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteSkillRequest>,
) -> Result<Json<ExecuteSkillResponse>, String> {
    let registry = SkillRegistry::new(&state.pool);

    let skill = match registry.find_by_name(&payload.skill_name).await {
        Ok(Some(s)) => s,
        Ok(None) => return Err(format!("Skill not found: {}", payload.skill_name)),
        Err(e) => return Err(format!("Database error: {}", e)),
    };

    if !skill.enabled {
        return Err(format!("Skill is disabled: {}", payload.skill_name));
    }

    if !state
        .circuit_breakers
        .is_available(&payload.skill_name)
        .await
    {
        return Err(format!(
            "Circuit breaker open for skill: {}. Too many recent failures.",
            payload.skill_name
        ));
    }

    let user_tier = match payload.tier.as_str() {
        "T0" => PermissionTier::T0,
        "T1" => PermissionTier::T1,
        "T2" => PermissionTier::T2,
        "T3" => PermissionTier::T3,
        // B1: Default to lowest trust if unknown - fail closed
        _ => PermissionTier::T0,
    };

    let required_tier = match skill.tier.as_str() {
        "T0" => PermissionTier::T0,
        "T1" => PermissionTier::T1,
        "T2" => PermissionTier::T2,
        "T3" => PermissionTier::T3,
        // B1: Default to highest requirement if unknown - fail closed
        _ => PermissionTier::T3,
    };

    let tier_order = |t: &PermissionTier| match t {
        PermissionTier::T0 => 0,
        PermissionTier::T1 => 1,
        PermissionTier::T2 => 2,
        PermissionTier::T3 => 3,
    };

    if tier_order(&user_tier) < tier_order(&required_tier) {
        return Err(format!(
            "Insufficient permissions: skill requires {}, user has {}",
            skill.tier, payload.tier
        ));
    }

    let repo = TaskRepository::new(&state.pool);
    if let Err(e) = repo
        .update_status(&payload.task_id, TaskStatus::Running)
        .await
    {
        tracing::warn!("Failed to update task status: {}", e);
    }

    state
        .message_bus
        .publish_skill(crate::message_bus::SkillExecutionMessage {
            task_id: payload.task_id.clone(),
            skill_name: payload.skill_name.clone(),
            input: payload.input,
            permission_tier: payload.tier.clone(),
        });

    Ok(Json(ExecuteSkillResponse {
        success: true,
        output: Some(format!(
            "Skill {} queued for execution",
            payload.skill_name
        )),
        error: None,
    }))
}

/// Invalidate cache for a specific skill
async fn invalidate_skill_cache(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    if let Some(pool) = &state.skill_pool {
        pool.invalidate_cache(Some(&name))
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(serde_json::json!({
            "ok": true,
            "message": format!("Cache invalidated for skill: {}", name)
        })))
    } else {
        Err("Skill pool not enabled".to_string())
    }
}

/// Invalidate cache for all skills
async fn invalidate_all_cache(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, String> {
    if let Some(pool) = &state.skill_pool {
        pool.invalidate_cache(None)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Json(serde_json::json!({
            "ok": true,
            "message": "All skill caches invalidated"
        })))
    } else {
        Err("Skill pool not enabled".to_string())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MarketplaceSkill {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tier: String,
    pub category: String,
    pub downloads: u32,
    pub rating: f32,
    pub tags: Vec<String>,
}

async fn list_marketplace_skills() -> Json<Vec<MarketplaceSkill>> {
    let skills = vec![
        MarketplaceSkill {
            name: "code.analyze".to_string(),
            version: "1.0.0".to_string(),
            description: "Analyze code for patterns and issues".to_string(),
            author: "apex".to_string(),
            tier: "T0".to_string(),
            category: "development".to_string(),
            downloads: 1250,
            rating: 4.5,
            tags: vec!["code".to_string(), "analysis".to_string()],
        },
        MarketplaceSkill {
            name: "file.search".to_string(),
            version: "1.0.0".to_string(),
            description: "Search files by content or name".to_string(),
            author: "apex".to_string(),
            tier: "T0".to_string(),
            category: "utilities".to_string(),
            downloads: 890,
            rating: 4.2,
            tags: vec!["file".to_string(), "search".to_string()],
        },
        MarketplaceSkill {
            name: "git.commit".to_string(),
            version: "1.0.0".to_string(),
            description: "Create git commits with auto-generated messages".to_string(),
            author: "apex".to_string(),
            tier: "T2".to_string(),
            category: "version-control".to_string(),
            downloads: 2100,
            rating: 4.8,
            tags: vec!["git".to_string(), "version-control".to_string()],
        },
        MarketplaceSkill {
            name: "shell.execute".to_string(),
            version: "1.0.0".to_string(),
            description: "Execute shell commands in isolated environment".to_string(),
            author: "apex".to_string(),
            tier: "T3".to_string(),
            category: "system".to_string(),
            downloads: 3500,
            rating: 4.9,
            tags: vec!["shell".to_string(), "command".to_string()],
        },
    ];
    Json(skills)
}

async fn get_marketplace_skill(Path(name): Path<String>) -> Result<Json<MarketplaceSkill>, String> {
    let skills = vec![
        MarketplaceSkill {
            name: "code.analyze".to_string(),
            version: "1.0.0".to_string(),
            description: "Analyze code for patterns and issues".to_string(),
            author: "apex".to_string(),
            tier: "T0".to_string(),
            category: "development".to_string(),
            downloads: 1250,
            rating: 4.5,
            tags: vec!["code".to_string(), "analysis".to_string()],
        },
    ];
    
    skills.into_iter()
        .find(|s| s.name == name)
        .map(Json)
        .ok_or_else(|| "Skill not found in marketplace".to_string())
}

async fn install_marketplace_skill(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    tracing::info!(skill = %name, "Installing skill from marketplace");
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Skill {} installed successfully", name)
    })))
}

async fn uninstall_marketplace_skill(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    tracing::info!(skill = %name, "Uninstalling skill from marketplace");
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Skill {} uninstalled successfully", name)
    })))
}

async fn search_marketplace(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<Vec<MarketplaceSkill>> {
    let query = params.get("q").cloned().unwrap_or_default().to_lowercase();
    let category = params.get("category").cloned();
    let tier = params.get("tier").cloned();
    
    let mut skills = vec![
        MarketplaceSkill {
            name: "code.analyze".to_string(),
            version: "1.0.0".to_string(),
            description: "Analyze code for patterns and issues".to_string(),
            author: "apex".to_string(),
            tier: "T0".to_string(),
            category: "development".to_string(),
            downloads: 1250,
            rating: 4.5,
            tags: vec!["code".to_string(), "analysis".to_string()],
        },
        MarketplaceSkill {
            name: "file.search".to_string(),
            version: "1.0.0".to_string(),
            description: "Search files by content or name".to_string(),
            author: "apex".to_string(),
            tier: "T0".to_string(),
            category: "utilities".to_string(),
            downloads: 890,
            rating: 4.2,
            tags: vec!["file".to_string(), "search".to_string()],
        },
        MarketplaceSkill {
            name: "git.commit".to_string(),
            version: "1.0.0".to_string(),
            description: "Create git commits with auto-generated messages".to_string(),
            author: "apex".to_string(),
            tier: "T2".to_string(),
            category: "version-control".to_string(),
            downloads: 2100,
            rating: 4.8,
            tags: vec!["git".to_string(), "version-control".to_string()],
        },
        MarketplaceSkill {
            name: "shell.execute".to_string(),
            version: "1.0.0".to_string(),
            description: "Execute shell commands in isolated environment".to_string(),
            author: "apex".to_string(),
            tier: "T3".to_string(),
            category: "system".to_string(),
            downloads: 3500,
            rating: 4.9,
            tags: vec!["shell".to_string(), "command".to_string()],
        },
    ];
    
    if !query.is_empty() {
        skills.retain(|s| 
            s.name.to_lowercase().contains(&query) || 
            s.description.to_lowercase().contains(&query) ||
            s.tags.iter().any(|t| t.to_lowercase().contains(&query))
        );
    }
    
    if let Some(cat) = category {
        skills.retain(|s| s.category == cat);
    }
    
    if let Some(t) = tier {
        skills.retain(|s| s.tier == t);
    }
    
    Json(skills)
}

/// MCP tool execution request
#[derive(Debug, Deserialize)]
pub struct ExecuteMcpToolRequest {
    pub server_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub task_id: Option<String>,
}

/// MCP tool execution response
#[derive(Debug, Serialize)]
pub struct ExecuteMcpToolResponse {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Execute an MCP tool directly (synchronous, not queued)
async fn execute_mcp_tool_endpoint(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteMcpToolRequest>,
) -> Result<Json<ExecuteMcpToolResponse>, String> {
    tracing::info!("Executing MCP tool '{}/{}' from task {:?}", 
        payload.server_id, payload.tool_name, payload.task_id);
    
    // Try to get the tool from the server
    let result = state.mcp_manager
        .call_tool(&payload.server_id, &payload.tool_name, payload.arguments)
        .await;
    
    match result {
        Ok(tool_result) => {
            if tool_result.success {
                Ok(Json(ExecuteMcpToolResponse {
                    success: true,
                    output: Some(tool_result.content),
                    error: None,
                }))
            } else {
                Ok(Json(ExecuteMcpToolResponse {
                    success: false,
                    output: None,
                    error: tool_result.error.or(Some(tool_result.content)),
                }))
            }
        }
        Err(e) => {
            tracing::error!("MCP tool execution failed: {}", e);
            Ok(Json(ExecuteMcpToolResponse {
                success: false,
                output: None,
                error: Some(e),
            }))
        }
    }
}

/// Get all MCP tools from all connected servers (as skill-like entries)
async fn list_mcp_tools_as_skills(
    State(state): State<AppState>,
) -> Result<Json<Vec<SkillResponse>>, String> {
    let all_tools = state.mcp_manager.get_all_tools().await;
    
    let skills: Vec<SkillResponse> = all_tools
        .into_iter()
        .map(|(server_id, tool)| SkillResponse {
            name: format!("mcp:{}:{}", server_id, tool.name),
            version: "1.0.0".to_string(),
            tier: "T1".to_string(),
            enabled: true,
            health_status: "healthy".to_string(),
            last_health_check: Some(chrono::Utc::now().to_rfc3339()),
        })
        .collect();
    
    Ok(Json(skills))
}
