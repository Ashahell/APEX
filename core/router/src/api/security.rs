//! Security API endpoints
//!
//! Provides endpoints for:
//! - Anomaly detection status and history
//! - Injection pattern management
//! - Security configuration
//! - Security health checks

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::security::{Anomaly, AnomalyDetector, AnomalySeverity, AnomalyType, InjectionClassifier};
use super::AppState;

/// Create security router
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/security/anomalies", get(get_anomalies))
        .route("/api/v1/security/anomalies/count", get(get_anomaly_count))
        .route("/api/v1/security/anomalies/:severity", get(get_anomalies_by_severity))
        .route("/api/v1/security/stats", get(get_security_stats))
        .route("/api/v1/security/injection/analyze", post(analyze_input))
        .route("/api/v1/security/injection/patterns", get(get_injection_patterns))
        .route("/api/v1/security/health", get(security_health))
}

/// Response for anomaly list
#[derive(Debug, Serialize, Deserialize)]
pub struct AnomaliesResponse {
    pub anomalies: Vec<AnomalyResponse>,
    pub total: usize,
}

/// Simplified anomaly response
#[derive(Debug, Serialize, Deserialize)]
pub struct AnomalyResponse {
    pub id: String,
    pub anomaly_type: String,
    pub severity: String,
    pub skill_name: Option<String>,
    pub task_id: Option<String>,
    pub description: String,
    pub detected_at: String,
}

impl From<Anomaly> for AnomalyResponse {
    fn from(a: Anomaly) -> Self {
        Self {
            id: a.id,
            anomaly_type: a.anomaly_type.as_str().to_string(),
            severity: a.severity.as_str().to_string(),
            skill_name: a.skill_name,
            task_id: a.task_id,
            description: a.description,
            detected_at: a.detected_at,
        }
    }
}

/// Get all anomalies
async fn get_anomalies(State(state): State<AppState>) -> Json<AnomaliesResponse> {
    let detector = state.anomaly_detector.as_ref();
    
    if let Some(detector) = detector {
        let anomalies = detector.get_anomalies().await;
        let total = anomalies.len();
        let anomalies: Vec<AnomalyResponse> = anomalies.into_iter().map(AnomalyResponse::from).collect();
        
        Json(AnomaliesResponse { anomalies, total })
    } else {
        Json(AnomaliesResponse { anomalies: vec![], total: 0 })
    }
}

/// Get anomaly count
async fn get_anomaly_count(State(state): State<AppState>) -> Json<serde_json::Value> {
    let detector = state.anomaly_detector.as_ref();
    
    if let Some(detector) = detector {
        let anomalies = detector.get_anomalies().await;
        let critical = anomalies.iter().filter(|a| a.severity == AnomalySeverity::Critical).count();
        let high = anomalies.iter().filter(|a| a.severity == AnomalySeverity::High).count();
        let medium = anomalies.iter().filter(|a| a.severity == AnomalySeverity::Medium).count();
        let low = anomalies.iter().filter(|a| a.severity == AnomalySeverity::Low).count();
        
        Json(serde_json::json!({
            "total": anomalies.len(),
            "critical": critical,
            "high": high,
            "medium": medium,
            "low": low
        }))
    } else {
        Json(serde_json::json!({
            "total": 0,
            "critical": 0,
            "high": 0,
            "medium": 0,
            "low": 0
        }))
    }
}

/// Get anomalies by severity
async fn get_anomalies_by_severity(
    Path(severity): Path<String>,
    State(state): State<AppState>,
) -> Json<AnomaliesResponse> {
    let severity = match severity.to_lowercase().as_str() {
        "critical" => AnomalySeverity::Critical,
        "high" => AnomalySeverity::High,
        "medium" => AnomalySeverity::Medium,
        "low" => AnomalySeverity::Low,
        _ => return Json(AnomaliesResponse { anomalies: vec![], total: 0 }),
    };
    
    let detector = state.anomaly_detector.as_ref();
    
    if let Some(detector) = detector {
        let anomalies = detector.get_anomalies_by_severity(severity).await;
        let total = anomalies.len();
        let anomalies: Vec<AnomalyResponse> = anomalies.into_iter().map(AnomalyResponse::from).collect();
        
        Json(AnomaliesResponse { anomalies, total })
    } else {
        Json(AnomaliesResponse { anomalies: vec![], total: 0 })
    }
}

/// Security statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityStatsResponse {
    pub anomaly_detector_healthy: bool,
    pub skills_tracked: usize,
    pub injection_classifier_available: bool,
    pub recent_executions_analyzed: usize,
}

/// Get security statistics
async fn get_security_stats(State(state): State<AppState>) -> Json<SecurityStatsResponse> {
    let detector = state.anomaly_detector.as_ref();
    
    let (healthy, skills_tracked, recent_executions) = if let Some(detector) = detector {
        let health = detector.health_status().await;
        (health.status == "healthy", health.skills_tracked, health.recent_executions)
    } else {
        (false, 0, 0)
    };
    
    Json(SecurityStatsResponse {
        anomaly_detector_healthy: healthy,
        skills_tracked,
        injection_classifier_available: true,
        recent_executions_analyzed: recent_executions,
    })
}

/// Request for injection analysis
#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub skill_name: Option<String>,
    pub input: String,
}

/// Response for injection analysis
#[derive(Debug, Serialize)]
pub struct AnalyzeInputResponse {
    pub is_safe: bool,
    pub threat_level: String,
    pub injection_type: Option<String>,
    pub matched_pattern: Option<String>,
    pub message: String,
    pub should_block: bool,
}

/// Analyze input for injection attempts
async fn analyze_input(
    State(_state): State<AppState>,
    Json(payload): Json<AnalyzeRequest>,
) -> Json<AnalyzeInputResponse> {
    let result = if let Some(skill_name) = payload.skill_name {
        InjectionClassifier::analyze_skill_input(&skill_name, &payload.input)
    } else {
        InjectionClassifier::analyze(&payload.input)
    };
    
    Json(AnalyzeInputResponse {
        is_safe: result.is_safe,
        threat_level: result.threat_level.as_str().to_string(),
        injection_type: result.injection_type.map(|t| t.as_str().to_string()),
        matched_pattern: result.matched_pattern,
        message: result.message,
        should_block: result.should_block,
    })
}

/// Injection pattern info
#[derive(Debug, Serialize)]
pub struct InjectionPatternInfo {
    pub description: String,
    pub pattern_type: String,
    pub severity: String,
}

/// Get registered injection patterns
async fn get_injection_patterns() -> Json<Vec<InjectionPatternInfo>> {
    let patterns = InjectionClassifier::get_patterns();
    
    let infos: Vec<InjectionPatternInfo> = patterns
        .into_iter()
        .map(|(desc, ptype, sev)| InjectionPatternInfo {
            description: desc.to_string(),
            pattern_type: ptype.to_string(),
            severity: sev.to_string(),
        })
        .collect();
    
    Json(infos)
}

/// Security health check response
#[derive(Debug, Serialize)]
pub struct SecurityHealthResponse {
    pub status: String,
    pub components: SecurityComponents,
}

#[derive(Debug, Serialize)]
pub struct SecurityComponents {
    pub anomaly_detector: String,
    pub injection_classifier: String,
    pub content_hash: String,
}

/// Security health check
async fn security_health(State(state): State<AppState>) -> Json<SecurityHealthResponse> {
    let mut components = SecurityComponents {
        anomaly_detector: "unknown".to_string(),
        injection_classifier: "healthy".to_string(),
        content_hash: "healthy".to_string(),
    };
    
    let mut all_healthy = true;
    
    // Check anomaly detector
    if let Some(detector) = state.anomaly_detector.as_ref() {
        let health = detector.health_status().await;
        if health.status == "healthy" {
            components.anomaly_detector = "healthy".to_string();
        } else {
            components.anomaly_detector = "degraded".to_string();
            all_healthy = false;
        }
    } else {
        components.anomaly_detector = "not_initialized".to_string();
        all_healthy = false;
    }
    
    let status = if all_healthy { "healthy" } else { "degraded" };
    
    Json(SecurityHealthResponse {
        status: status.to_string(),
        components,
    })
}

// Import Path for axum
use axum::extract::Path;
