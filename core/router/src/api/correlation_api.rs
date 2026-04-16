//! Correlation API Endpoints
//! 
//! v1.10.0: Alert Correlation API

use std::collections::HashMap;
use tokio::sync::RwLock;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use chrono::Utc;

use crate::api::api_error::ApiError;
use crate::api::AppState;
use crate::correlation::{
    AlertCorrelationRule, AlertGroup, AlertEntry,
    CreateCorrelationRuleRequest, UpdateCorrelationRuleRequest,
    ProcessAlertRequest, ProcessAlertResponse, CorrelationStats,
};

/// Shared correlation state
pub struct CorrelationState {
    pub rules: RwLock<Vec<AlertCorrelationRule>>,
    pub groups: RwLock<Vec<AlertGroup>>,
    pub stats: RwLock<CorrelationStats>,
}

impl CorrelationState {
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
            groups: RwLock::new(Vec::new()),
            stats: RwLock::new(CorrelationStats {
                total_rules: 0,
                enabled_rules: 0,
                active_groups: 0,
                resolved_groups: 0,
                alerts_processed: 0,
                alerts_suppressed: 0,
                alerts_grouped: 0,
            }),
        }
    }

    pub async fn list_rules(&self) -> Vec<AlertCorrelationRule> {
        self.rules.read().await.clone()
    }

    pub async fn get_rule(&self, id: &str) -> Option<AlertCorrelationRule> {
        self.rules.read().await.iter()
            .find(|r| r.id == id)
            .cloned()
    }

    pub async fn create_rule(&self, mut rule: AlertCorrelationRule) -> AlertCorrelationRule {
        rule.id = ulid::Ulid::new().to_string();
        rule.created_at = Utc::now();
        rule.updated_at = Utc::now();
        self.rules.write().await.push(rule.clone());
        self.update_stats().await;
        rule
    }

    pub async fn update_rule(&self, id: &str, update: UpdateCorrelationRuleRequest) -> Option<AlertCorrelationRule> {
        let mut rules = self.rules.write().await;
        let rule_clone = {
            let rule = rules.iter_mut().find(|r| r.id == id)?;
            if let Some(name) = update.name {
                rule.name = name;
            }
            if let Some(desc) = update.description {
                rule.description = Some(desc);
            }
            if let Some(cond) = update.condition {
                rule.condition = cond;
            }
            if let Some(action) = update.action {
                rule.action = action;
            }
            if let Some(enabled) = update.enabled {
                rule.enabled = enabled;
            }
            if let Some(priority) = update.priority {
                rule.priority = priority;
            }
            rule.updated_at = Utc::now();
            rule.clone()
        };
        drop(rules);
        self.update_stats().await;
        Some(rule_clone)
    }

    pub async fn delete_rule(&self, id: &str) -> bool {
        let mut rules = self.rules.write().await;
        let pos = rules.iter().position(|r| r.id == id);
        if let Some(pos) = pos {
            rules.remove(pos);
            drop(rules);
            self.update_stats().await;
            true
        } else {
            false
        }
    }

    pub async fn list_groups(&self, resolved: Option<bool>) -> Vec<AlertGroup> {
        let groups = self.groups.read().await;
        match resolved {
            Some(r) => groups.iter().filter(|g| g.resolved == r).cloned().collect(),
            None => groups.clone(),
        }
    }

    pub async fn get_group(&self, id: &str) -> Option<AlertGroup> {
        self.groups.read().await.iter()
            .find(|g| g.id == id)
            .cloned()
    }

    pub async fn resolve_group(&self, id: &str) -> Option<AlertGroup> {
        let (group_clone, needs_stats_update) = {
            let mut groups = self.groups.write().await;
            match groups.iter_mut().find(|g| g.id == id) {
                Some(group) => {
                    group.resolve();
                    (Some(group.clone()), true)
                }
                None => (None, false),
            }
        };
        if needs_stats_update {
            self.update_stats().await;
        }
        group_clone
    }

    pub async fn process_alert(&self, req: ProcessAlertRequest) -> ProcessAlertResponse {
        let alert = AlertEntry::new(
            req.source.clone(),
            req.message.clone(),
            req.severity.clone(),
        );
        let alert_id = alert.id.clone();

        // Find matching rule
        let rules = self.rules.read().await;
        let mut matched_rule: Option<&AlertCorrelationRule> = None;
        
        for rule in rules.iter() {
            if rule.matches(&req.source, &req.message, &req.severity) {
                matched_rule = Some(rule);
                break;
            }
        }

        if let Some(rule) = matched_rule {
            // Update stats
            {
                let mut stats = self.stats.write().await;
                stats.alerts_processed += 1;
            }

            if rule.action.suppress {
                // Update stats for suppressed
                let mut stats = self.stats.write().await;
                stats.alerts_suppressed += 1;
                return ProcessAlertResponse {
                    alert_id,
                    matched: true,
                    group_id: None,
                    suppressed: true,
                    message: "Alert suppressed by rule".to_string(),
                };
            }

            // Find or create group
            let group_key = format!("{}:{}", rule.id, req.source);
            let mut groups = self.groups.write().await;
            
            // Find existing unresolved group
            let existing = groups.iter_mut()
                .find(|g| g.rule_id == rule.id && g.group_key == req.source && !g.resolved);

            if let Some(group) = existing {
                group.add_alert(alert);
                let group_id = group.id.clone();
                drop(groups);
                
                let mut stats = self.stats.write().await;
                stats.alerts_grouped += 1;

                ProcessAlertResponse {
                    alert_id,
                    matched: true,
                    group_id: Some(group_id),
                    suppressed: false,
                    message: "Alert added to existing group".to_string(),
                }
            } else {
                // Create new group
                let mut new_group = AlertGroup::new(rule.id.clone(), req.source.clone());
                new_group.add_alert(alert);
                let group_id = new_group.id.clone();
                groups.push(new_group);
                drop(groups);

                let mut stats = self.stats.write().await;
                stats.alerts_grouped += 1;
                stats.active_groups += 1;

                ProcessAlertResponse {
                    alert_id,
                    matched: true,
                    group_id: Some(group_id),
                    suppressed: false,
                    message: "New correlation group created".to_string(),
                }
            }
        } else {
            // No matching rule
            let mut stats = self.stats.write().await;
            stats.alerts_processed += 1;

            ProcessAlertResponse {
                alert_id,
                matched: false,
                group_id: None,
                suppressed: false,
                message: "No matching correlation rules".to_string(),
            }
        }
    }

    async fn update_stats(&self) {
        let rules = self.rules.read().await;
        let groups = self.groups.read().await;
        let mut stats = self.stats.write().await;

        stats.total_rules = rules.len() as u32;
        stats.enabled_rules = rules.iter().filter(|r| r.enabled).count() as u32;
        stats.active_groups = groups.iter().filter(|g| !g.resolved).count() as u32;
        stats.resolved_groups = groups.iter().filter(|g| g.resolved).count() as u32;
    }

    pub async fn get_stats(&self) -> CorrelationStats {
        self.stats.read().await.clone()
    }
}

impl Default for CorrelationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the correlation router
pub fn create_correlation_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/alert-correlations", get(list_rules))
        .route("/api/v1/alert-correlations/:id", get(get_rule))
        .route("/api/v1/alert-correlations", post(create_rule))
        .route("/api/v1/alert-correlations/:id", put(update_rule))
        .route("/api/v1/alert-correlations/:id", delete(delete_rule))
        .route("/api/v1/alert-correlations/groups", get(list_groups))
        .route("/api/v1/alert-correlations/groups/:id/resolve", post(resolve_group))
        .route("/api/v1/alert-correlations/process", post(process_alert))
        .route("/api/v1/alert-correlations/stats", get(get_stats))
}

/// GET /api/v1/alert-correlations - List all correlation rules
async fn list_rules(
    State(state): State<AppState>,
) -> Result<Json<Vec<AlertCorrelationRule>>, (axum::http::StatusCode, String)> {
    let rules = state.correlation_state.list_rules().await;
    Ok(Json(rules))
}

/// GET /api/v1/alert-correlations/:id - Get a specific rule
async fn get_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AlertCorrelationRule>, (axum::http::StatusCode, String)> {
    state.correlation_state.get_rule(&id).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Rule {} not found", id)))
}

/// POST /api/v1/alert-correlations - Create a new rule
async fn create_rule(
    State(state): State<AppState>,
    Json(req): Json<CreateCorrelationRuleRequest>,
) -> Result<Json<AlertCorrelationRule>, (axum::http::StatusCode, String)> {
    let mut rule = AlertCorrelationRule::new(req.name, req.condition);
    rule.description = req.description;
    if let Some(action) = req.action {
        rule.action = action;
    }
    if let Some(priority) = req.priority {
        rule.priority = priority;
    }

    let created = state.correlation_state.create_rule(rule).await;
    Ok(Json(created))
}

/// PUT /api/v1/alert-correlations/:id - Update a rule
async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCorrelationRuleRequest>,
) -> Result<Json<AlertCorrelationRule>, (axum::http::StatusCode, String)> {
    state.correlation_state.update_rule(&id, req).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Rule {} not found", id)))
}

/// DELETE /api/v1/alert-correlations/:id - Delete a rule
async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    if state.correlation_state.delete_rule(&id).await {
        Ok(Json(serde_json::json!({ "success": true })))
    } else {
        Err(ApiError::not_found(format!("Rule {} not found", id)))
    }
}

/// GET /api/v1/alert-correlations/groups - List alert groups
#[derive(Debug, serde::Deserialize)]
pub struct GroupsQuery {
    pub resolved: Option<bool>,
}

async fn list_groups(
    State(state): State<AppState>,
    Query(query): Query<GroupsQuery>,
) -> Result<Json<Vec<AlertGroup>>, (axum::http::StatusCode, String)> {
    let groups = state.correlation_state.list_groups(query.resolved).await;
    Ok(Json(groups))
}

/// POST /api/v1/alert-correlations/groups/:id/resolve - Resolve a group
async fn resolve_group(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AlertGroup>, (axum::http::StatusCode, String)> {
    state.correlation_state.resolve_group(&id).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Group {} not found", id)))
}

/// POST /api/v1/alert-correlations/process - Process an incoming alert
async fn process_alert(
    State(state): State<AppState>,
    Json(req): Json<ProcessAlertRequest>,
) -> Result<Json<ProcessAlertResponse>, (axum::http::StatusCode, String)> {
    let response = state.correlation_state.process_alert(req).await;
    Ok(Json(response))
}

/// GET /api/v1/alert-correlations/stats - Get correlation statistics
async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<CorrelationStats>, (axum::http::StatusCode, String)> {
    let stats = state.correlation_state.get_stats().await;
    Ok(Json(stats))
}
