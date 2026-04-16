//! API endpoints for Vigilant Mode - Alert Monitoring
//! 
//! This module integrates the vigilant module with the API router.

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::api::api_error::ApiError;
use crate::api::AppState;
use crate::vigilant::{AlertRule, AlertRuleUpdate, AlertType, DetectedPattern, EmailConfig};

/// Create the vigilant router (uses AppState for proper router merging)
pub fn create_vigilant_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/vigilant/rules", get(list_rules))
        .route("/api/v1/vigilant/rules", post(create_rule))
        .route("/api/v1/vigilant/rules/:id", get(get_rule))
        .route("/api/v1/vigilant/rules/:id", post(update_rule))
        .route("/api/v1/vigilant/rules/:id", axum::routing::delete(delete_rule))
        .route("/api/v1/vigilant/rules/:id/reset-cooldown", post(reset_cooldown))
        .route("/api/v1/vigilant/alerts", get(list_alerts))
        .route("/api/v1/vigilant/alerts/active", get(get_active_alerts))
        .route("/api/v1/vigilant/alerts/:id/acknowledge", post(acknowledge_alert))
        .route("/api/v1/vigilant/alerts/:id/dismiss", post(dismiss_alert))
        .route("/api/v1/vigilant/alerts/:id/resolve", post(resolve_alert))
        .route("/api/v1/vigilant/alerts/history", get(get_alert_history))
        .route("/api/v1/vigilant/stats", get(get_stats))
        .route("/api/v1/vigilant/analytics", get(get_analytics))
        .route("/api/v1/vigilant/escalation/pending", get(get_pending_escalation))
        .route("/api/v1/vigilant/escalation/process", post(process_escalations))
        .route("/api/v1/vigilant/trigger", post(trigger_alert))
        .route("/api/v1/vigilant/metrics/:task_id", get(get_task_metrics))
        .route("/api/v1/vigilant/patterns/create-rule", post(create_rule_from_pattern))
        .route("/api/v1/vigilant/email/config", get(get_email_config))
        .route("/api/v1/vigilant/email/config", post(set_email_config))
        .route("/api/v1/vigilant/email/config", delete(delete_email_config))
        .route("/api/v1/vigilant/email/test", post(test_email_config))
}

/// List all alert rules
async fn list_rules(
    State(state): State<AppState>,
) -> Result<Json<Vec<AlertRule>>, (axum::http::StatusCode, String)> {
    let rules = state.vigilant_state.rule_engine.list().await;
    Ok(Json(rules))
}

/// Create a new alert rule
async fn create_rule(
    State(state): State<AppState>,
    Json(payload): Json<CreateRuleRequest>,
) -> Result<Json<AlertRule>, (axum::http::StatusCode, String)> {
    let rule = AlertRule::new(
        ulid::Ulid::new().to_string(),
        payload.name,
        payload.alert_type,
        payload.severity,
        payload.cooldown_secs,
        payload.actions,
    )
    .map_err(|e| ApiError::bad_request(e.to_string()))?;

    state
        .vigilant_state
        .rule_engine
        .add(rule.clone())
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(rule))
}

/// Get a specific alert rule
async fn get_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AlertRule>, (axum::http::StatusCode, String)> {
    state
        .vigilant_state
        .rule_engine
        .get(&id)
        .await
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Rule {} not found", id)))
}

/// Update an alert rule
async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(updates): Json<AlertRuleUpdate>,
) -> Result<Json<AlertRule>, (axum::http::StatusCode, String)> {
    state
        .vigilant_state
        .rule_engine
        .update(&id, updates)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    state
        .vigilant_state
        .rule_engine
        .get(&id)
        .await
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Rule {} not found", id)))
}

/// Delete an alert rule
async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // Don't allow deleting built-in rules
    if id.starts_with("builtin-") {
        return Err(ApiError::forbidden("Cannot delete built-in rules".to_string()));
    }

    state.vigilant_state.rule_engine.remove(&id).await;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// Reset cooldown for a rule
async fn reset_cooldown(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    state.vigilant_state.rule_engine.reset_cooldown(&id).await;
    Ok(Json(serde_json::json!({ "reset": true })))
}

/// List all alerts
async fn list_alerts(
    State(state): State<AppState>,
    Query(params): Query<AlertsQuery>,
) -> Result<Json<Vec<crate::vigilant::Alert>>, (axum::http::StatusCode, String)> {
    let mut alerts = state.vigilant_state.dispatcher.get_all().await;
    
    if let Some(status) = &params.status {
        alerts.retain(|a| format!("{:?}", a.status).to_lowercase() == status.to_lowercase());
    }

    Ok(Json(alerts))
}

/// Get active alerts only
async fn get_active_alerts(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::vigilant::Alert>>, (axum::http::StatusCode, String)> {
    let alerts = state.vigilant_state.dispatcher.get_active().await;
    Ok(Json(alerts))
}

/// Acknowledge an alert
async fn acknowledge_alert(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AcknowledgeRequest>,
) -> Result<Json<crate::vigilant::Alert>, (axum::http::StatusCode, String)> {
    state
        .vigilant_state
        .dispatcher
        .acknowledge(&id, payload.by)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    state
        .vigilant_state
        .dispatcher
        .get_all()
        .await
        .into_iter()
        .find(|a| a.id == id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Alert {} not found", id)))
}

/// Dismiss an alert
async fn dismiss_alert(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    state
        .vigilant_state
        .dispatcher
        .dismiss(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "dismissed": true })))
}

/// Resolve an alert
async fn resolve_alert(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    state
        .vigilant_state
        .dispatcher
        .resolve(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "resolved": true })))
}

/// Get alert history
async fn get_alert_history(
    State(_state): State<AppState>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (axum::http::StatusCode, String)> {
    let limit = params.limit.unwrap_or(100) as usize;
    Ok(Json(Vec::with_capacity(limit)))
}

/// Get vigilant statistics
async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let rules = state.vigilant_state.rule_engine.list().await;
    let alerts = state.vigilant_state.dispatcher.get_all().await;
    let dispatcher_stats = state.vigilant_state.dispatcher.stats().await;
    let active_alerts = state.vigilant_state.dispatcher.get_active().await;

    let mut by_severity: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for alert in &alerts {
        let key = format!("{:?}", alert.severity);
        *by_severity.entry(key).or_insert(0) += 1;
    }

    Ok(Json(serde_json::json!({
        "rules": {
            "total": rules.len(),
            "enabled": rules.iter().filter(|r| r.enabled).count(),
        },
        "alerts": {
            "total": alerts.len(),
            "active": active_alerts.len(),
            "by_severity": by_severity,
        },
        "dispatcher": {
            "triggered": dispatcher_stats.alerts_triggered,
            "actions_executed": dispatcher_stats.actions_executed,
        }
    })))
}

/// Trigger an alert manually (for testing)
async fn trigger_alert(
    State(state): State<AppState>,
    Json(payload): Json<TriggerAlertRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let alert_type = payload.alert_type;

    let alerts = state.vigilant_state.rule_engine.check(&alert_type).await;

    let mut triggered_count = 0;
    for alert in alerts {
        if state.vigilant_state.dispatcher.dispatch(alert).await.is_ok() {
            triggered_count += 1;
        }
    }

    Ok(Json(serde_json::json!({
        "alerts_triggered": triggered_count,
    })))
}

/// Get metrics for a specific task
async fn get_task_metrics(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let metrics = state.vigilant_state.threshold_monitor.get_metrics_for(&task_id).await;

    match metrics {
        Some(m) => Ok(Json(serde_json::json!({
            "task_id": task_id,
            "step_count": m.step_count,
            "error_count": m.error_count,
            "last_activity": m.last_activity,
            "action_history_length": m.action_history.len(),
        }))),
        None => Ok(Json(serde_json::json!({
            "task_id": task_id,
            "found": false,
        }))),
    }
}

/// Get email configuration
async fn get_email_config(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let config = state.vigilant_state.dispatcher.get_email_config().await;
    Ok(Json(serde_json::json!({
        "email": config,
    })))
}

/// Set email configuration
async fn set_email_config(
    State(state): State<AppState>,
    Json(payload): Json<SetEmailConfigRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let config = EmailConfig {
        smtp_host: payload.smtp_host,
        smtp_port: payload.smtp_port,
        username: payload.username,
        password: payload.password,
        from_address: payload.from_address,
        use_tls: payload.use_tls,
    };

    state.vigilant_state.dispatcher.set_email_config(config).await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Email configuration saved",
    })))
}

/// Delete email configuration
async fn delete_email_config(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // Set empty config to disable
    state.vigilant_state.dispatcher.set_email_config(EmailConfig::default()).await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Email configuration deleted",
    })))
}

/// Test email configuration
async fn test_email_config(
    State(state): State<AppState>,
    Json(payload): Json<TestEmailRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // In production, this would actually send a test email
    // For now, just validate the config is set
    let config = state.vigilant_state.dispatcher.get_email_config().await;

    if config.map(|c| c.configured).unwrap_or(false) {
        tracing::info!("Email test requested for: {}", payload.test_email);
        Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Test email would be sent to {}", payload.test_email),
        })))
    } else {
        Err(ApiError::bad_request("Email configuration not set".to_string()))
    }
}

/// Get alert analytics
async fn get_analytics(
    State(state): State<AppState>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let hours = params.hours.unwrap_or(24);
    let analytics = state.vigilant_state.dispatcher.get_analytics(hours).await;

    Ok(Json(serde_json::json!({
        "analytics": analytics,
        "period_hours": hours,
    })))
}

/// Get alerts pending escalation
async fn get_pending_escalation(
    State(state): State<AppState>,
    Query(params): Query<EscalationQuery>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let wait_secs = params.wait_secs.unwrap_or(300);
    let alerts = state.vigilant_state.dispatcher.get_pending_escalation(wait_secs).await;

    Ok(Json(serde_json::json!({
        "pending_count": alerts.len(),
        "alerts": alerts,
    })))
}

/// Process pending escalations
async fn process_escalations(
    State(state): State<AppState>,
    Json(payload): Json<ProcessEscalationRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let alerts = state.vigilant_state.dispatcher.get_pending_escalation(payload.wait_secs).await;
    let mut escalated_count = 0;

    for mut alert in alerts {
        if alert.escalation_level < payload.max_level {
            alert.escalate(alert.escalation_level + 1);

            // Execute escalation actions
            for action in &payload.escalation_actions {
                if let Err(e) = state.vigilant_state.dispatcher.execute_action_internal(action, &alert).await {
                    tracing::warn!("Failed to execute escalation action: {}", e);
                }
            }

            escalated_count += 1;
        }
    }

    Ok(Json(serde_json::json!({
        "escalated_count": escalated_count,
        "message": format!("Escalated {} alerts", escalated_count),
    })))
}

/// Create a router for pattern suggestions (uses AppState for DB access)
pub fn create_pattern_suggestions_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/vigilant/patterns/suggestions", get(get_pattern_suggestions))
}

/// Get alert rule suggestions from detected patterns
pub async fn get_pattern_suggestions(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    use apex_memory::execution_pattern_repo::{ExecutionPatternRepository, ExecutionPattern};

    let repo = ExecutionPatternRepository::new(&state.pool);

    // Get recent patterns
    let patterns = repo.get_recent(100).await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    // Group by pattern type
    let mut pattern_groups: std::collections::HashMap<String, DetectedPattern> = std::collections::HashMap::new();

    for pattern in &patterns {
        let entry = pattern_groups.entry(pattern.pattern_type.clone()).or_insert_with(|| {
            DetectedPattern {
                pattern_type: pattern.pattern_type.clone(),
                severity: pattern.severity.clone(),
                occurrences: 0,
                last_occurrence: pattern.detected_at.clone(),
                affected_tasks: Vec::new(),
            }
        });
        entry.occurrences += 1;
        if pattern.detected_at > entry.last_occurrence {
            entry.last_occurrence = pattern.detected_at.clone();
        }
        if !entry.affected_tasks.contains(&pattern.task_id) {
            entry.affected_tasks.push(pattern.task_id.clone());
        }
    }

    // Convert to suggestions
    let suggestions: Vec<_> = pattern_groups
        .values()
        .filter(|p| p.occurrences >= 2) // Only suggest if seen 2+ times
        .map(|p| p.to_rule_suggestion())
        .collect();

    Ok(Json(serde_json::json!({
        "suggestions": suggestions,
        "total_patterns": patterns.len(),
    })))
}

/// Create alert rule from detected pattern
async fn create_rule_from_pattern(
    State(state): State<AppState>,
    Json(payload): Json<CreateRuleFromPatternRequest>,
) -> Result<Json<AlertRule>, (axum::http::StatusCode, String)> {
    let alert_type = match payload.pattern_type.as_str() {
        "tool_call_loop" => AlertType::InfiniteLoop {
            task_id: String::new(),
            iterations: payload.threshold.unwrap_or(10),
        },
        "no_progress" => AlertType::NoProgress {
            task_id: String::new(),
            steps: payload.threshold.unwrap_or(10),
        },
        "error_cascade" => AlertType::ErrorSpike {
            task_id: String::new(),
            error_count: payload.threshold.unwrap_or(5),
        },
        "file_creation_burst" => AlertType::ResourceExhaustion {
            task_id: String::new(),
            resource: "file_descriptors".to_string(),
        },
        "timeout_warning" => AlertType::TimeoutWarning {
            task_id: String::new(),
            remaining_secs: payload.threshold.unwrap_or(60),
        },
        _ => AlertType::PatternDetected {
            pattern: payload.pattern_type.clone(),
            task_id: String::new(),
        },
    };

    let rule = AlertRule::new(
        format!("auto-{}-{}", payload.pattern_type, ulid::Ulid::new()),
        payload.name.unwrap_or_else(|| format!("Auto: {} Detection", payload.pattern_type)),
        alert_type,
        payload.severity,
        payload.cooldown_secs.unwrap_or(300),
        payload.actions,
    )
    .map_err(|e| ApiError::bad_request(e.to_string()))?;

    state
        .vigilant_state
        .rule_engine
        .add(rule.clone())
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(rule))
}

// Request/Response types

#[derive(Debug, Deserialize)]
pub struct CreateRuleRequest {
    pub name: String,
    pub alert_type: AlertType,
    pub severity: crate::vigilant::AlertSeverity,
    #[serde(default)]
    pub cooldown_secs: u32,
    pub actions: Vec<crate::vigilant::AlertAction>,
}

#[derive(Debug, Deserialize)]
pub struct AlertsQuery {
    pub status: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<u32>,
    pub rule_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AcknowledgeRequest {
    pub by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TriggerAlertRequest {
    pub alert_type: AlertType,
}

#[derive(Debug, Deserialize)]
pub struct SetEmailConfigRequest {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub use_tls: bool,
}

#[derive(Debug, Deserialize)]
pub struct TestEmailRequest {
    pub test_email: String,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub hours: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct EscalationQuery {
    pub wait_secs: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ProcessEscalationRequest {
    pub wait_secs: u32,
    pub max_level: u8,
    pub escalation_actions: Vec<crate::vigilant::AlertAction>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRuleFromPatternRequest {
    pub pattern_type: String,
    pub name: Option<String>,
    pub severity: crate::vigilant::AlertSeverity,
    #[serde(default = "default_cooldown")]
    pub cooldown_secs: Option<u32>,
    pub actions: Vec<crate::vigilant::AlertAction>,
    pub threshold: Option<u32>,
}

fn default_cooldown() -> Option<u32> {
    Some(300)
}
