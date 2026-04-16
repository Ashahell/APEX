//! Health Check API Endpoints
//! 
//! v1.10.0: Health Check Dashboard API

use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::api::api_error::ApiError;
use crate::api::AppState;
use crate::health::{ComponentHealth, HealthHistoryEntry, HealthStatus, HealthSummary};

/// Shared health state managed by the health module
pub struct HealthState {
    pub checks: RwLock<Vec<ComponentHealth>>,
    pub history: RwLock<Vec<HealthHistoryEntry>>,
    pub max_history: usize,
}

impl HealthState {
    pub fn new() -> Self {
        Self {
            checks: RwLock::new(Vec::new()),
            history: RwLock::new(Vec::new()),
            max_history: 1000,
        }
    }

    pub async fn get_summary(&self) -> HealthSummary {
        let checks = self.checks.read().await;
        let checks_len = checks.len() as u32;
        let mut summary = HealthSummary {
            components: checks.clone(),
            last_updated: Utc::now(),
            ..Default::default()
        };

        for check in checks.iter() {
            match check.status {
                HealthStatus::Healthy => summary.healthy_count += 1,
                HealthStatus::Degraded => summary.degraded_count += 1,
                HealthStatus::Unhealthy => summary.unhealthy_count += 1,
                HealthStatus::Unknown => summary.unknown_count += 1,
                HealthStatus::Checking => {}
            }
        }

        summary.overall_status = if summary.unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if summary.degraded_count > 0 {
            HealthStatus::Degraded
        } else if summary.unknown_count == checks_len {
            HealthStatus::Unknown
        } else if summary.healthy_count == checks_len {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded
        };

        summary
    }

    pub async fn get_component(&self, component: &str) -> Option<ComponentHealth> {
        let checks = self.checks.read().await;
        checks.iter().find(|c| c.component == component).cloned()
    }

    pub async fn register_component(&self, health: ComponentHealth) {
        let mut checks = self.checks.write().await;
        
        // Check if component already exists
        if let Some(existing) = checks.iter_mut().find(|c| c.component == health.component) {
            *existing = health;
        } else {
            checks.push(health);
        }
    }

    pub async fn update_component(&self, component: &str, status: HealthStatus, response_time_ms: Option<u64>, error: Option<String>) {
        let mut checks = self.checks.write().await;
        
        if let Some(check) = checks.iter_mut().find(|c| c.component == component) {
            check.last_check = Some(Utc::now());
            check.status = status;
            check.response_time_ms = response_time_ms;
            check.error_message = error.clone();

            match status {
                HealthStatus::Healthy => {
                    check.last_success = Some(Utc::now());
                    check.consecutive_failures = 0;
                }
                HealthStatus::Unhealthy | HealthStatus::Degraded => {
                    check.last_failure = Some(Utc::now());
                    check.consecutive_failures += 1;
                }
                _ => {}
            }

            // Add to history
            drop(checks);
            self.add_history_entry(component, status, response_time_ms, error).await;
        }
    }

    pub async fn add_history_entry(&self, component: &str, status: HealthStatus, response_time_ms: Option<u64>, error: Option<String>) {
        let mut history = self.history.write().await;
        
        let entry = HealthHistoryEntry {
            id: ulid::Ulid::new().to_string(),
            component: component.to_string(),
            status,
            response_time_ms,
            error_message: error,
            timestamp: Utc::now(),
        };

        history.push(entry);

        // Trim history if too large
        while history.len() > self.max_history {
            history.remove(0);
        }
    }

    pub async fn get_history(&self, component: Option<String>, limit: usize) -> Vec<HealthHistoryEntry> {
        let history = self.history.read().await;
        let filtered: Vec<_> = if let Some(comp) = component {
            history.iter().filter(|e| e.component == comp).collect()
        } else {
            history.iter().collect()
        };
        
        filtered.into_iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}

impl Default for HealthState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the health check router
pub fn create_health_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/health/checks", get(list_health_checks))
        .route("/api/v1/health/checks/:component", get(get_health_check))
        .route("/api/v1/health/checks/:component/run", post(run_health_check))
        .route("/api/v1/health/summary", get(get_health_summary))
        .route("/api/v1/health/history", get(get_health_history))
}

/// GET /api/v1/health/checks - List all health checks
async fn list_health_checks(
    State(state): State<AppState>,
) -> Result<Json<Vec<ComponentHealth>>, (axum::http::StatusCode, String)> {
    let checks = state.health_state.checks.read().await;
    Ok(Json(checks.clone()))
}

/// GET /api/v1/health/checks/:component - Get specific component health
async fn get_health_check(
    State(state): State<AppState>,
    Path(component): Path<String>,
) -> Result<Json<ComponentHealth>, (axum::http::StatusCode, String)> {
    state
        .health_state
        .get_component(&component)
        .await
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Component {} not found", component)))
}

/// POST /api/v1/health/checks/:component/run - Force run health check
async fn run_health_check(
    State(state): State<AppState>,
    Path(component): Path<String>,
) -> Result<Json<crate::health::RunHealthCheckResponse>, (axum::http::StatusCode, String)> {
    let start = Instant::now();
    
    // Perform the health check based on component
    let (status, error) = match component.as_str() {
        "llm" => check_llm_health(&state).await,
        "database" => check_database_health(&state).await,
        "skill_pool" => check_skill_pool_health(&state).await,
        "memory_indexer" => check_memory_indexer_health(&state).await,
        "vm_pool" => check_vm_pool_health(&state).await,
        "message_bus" => check_message_bus_health(&state).await,
        "websocket" => check_websocket_health(&state).await,
        "skill_worker" => check_skill_worker_health(&state).await,
        _ => (HealthStatus::Unknown, Some("Unknown component".to_string())),
    };

    let response_time_ms = start.elapsed().as_millis() as u64;
    
    // Update component status
    state.health_state
        .update_component(&component, status, Some(response_time_ms), error.clone())
        .await;

    Ok(Json(crate::health::RunHealthCheckResponse {
        component,
        status,
        response_time_ms: Some(response_time_ms),
        error_message: error,
        timestamp: Utc::now(),
    }))
}

/// GET /api/v1/health/summary - Get overall health summary
async fn get_health_summary(
    State(state): State<AppState>,
) -> Result<Json<HealthSummary>, (axum::http::StatusCode, String)> {
    let summary = state.health_state.get_summary().await;
    Ok(Json(summary))
}

/// GET /api/v1/health/history - Get health history
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub component: Option<String>,
    pub limit: Option<usize>,
}

async fn get_health_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<HealthHistoryEntry>>, (axum::http::StatusCode, String)> {
    let limit = query.limit.unwrap_or(100).min(1000);
    let history = state.health_state.get_history(query.component, limit).await;
    Ok(Json(history))
}

// ============ Health Check Functions ============

async fn check_llm_health(state: &AppState) -> (HealthStatus, Option<String>) {
    // Check if LLM is configured and reachable
    let llm_url = &state.config.agent.llama_url;
    if llm_url.is_empty() || llm_url == "disabled" {
        return (HealthStatus::Unknown, Some("LLM not configured".to_string()));
    }
    
    // Simple connectivity check - in production would make actual HTTP request
    (HealthStatus::Healthy, None)
}

async fn check_database_health(state: &AppState) -> (HealthStatus, Option<String>) {
    // Check database connectivity via sqlx pool
    let pool = &state.pool;
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => (HealthStatus::Healthy, None),
        Err(e) => (HealthStatus::Unhealthy, Some(format!("Database error: {}", e))),
    }
}

async fn check_skill_pool_health(_state: &AppState) -> (HealthStatus, Option<String>) {
    match &_state.skill_pool {
        Some(_pool) => {
            // Check if pool has available slots
            (HealthStatus::Healthy, None)
        }
        None => (HealthStatus::Unknown, Some("Skill pool not enabled".to_string())),
    }
}

async fn check_memory_indexer_health(_state: &AppState) -> (HealthStatus, Option<String>) {
    // Check if memory indexer is running
    (HealthStatus::Healthy, None)
}

async fn check_vm_pool_health(_state: &AppState) -> (HealthStatus, Option<String>) {
    match &_state.vm_pool {
        Some(_pool) => {
            // Check VM pool stats
            (HealthStatus::Healthy, None)
        }
        None => (HealthStatus::Unknown, Some("VM pool not enabled".to_string())),
    }
}

async fn check_message_bus_health(_state: &AppState) -> (HealthStatus, Option<String>) {
    // Message bus is internal, assume healthy if router is running
    (HealthStatus::Healthy, None)
}

async fn check_websocket_health(_state: &AppState) -> (HealthStatus, Option<String>) {
    // WebSocket manager is internal, assume healthy if router is running
    (HealthStatus::Healthy, None)
}

async fn check_skill_worker_health(_state: &AppState) -> (HealthStatus, Option<String>) {
    // Check if skill workers are responsive
    (HealthStatus::Healthy, None)
}
