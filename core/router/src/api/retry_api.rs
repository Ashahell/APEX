//! Retry Policy API Endpoints
//! 
//! v1.10.0: Task Retry Policies API

use std::sync::Arc;
use tokio::sync::RwLock;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use chrono::Utc;

use crate::api::api_error::ApiError;
use crate::api::AppState;
use crate::retry::{RetryPolicy, RetryAttempt, RetryStatus, 
    CreateRetryPolicyRequest, UpdateRetryPolicyRequest, 
    ApplyRetryPolicyRequest, ApplyRetryPolicyResponse};

/// Shared retry policy state
pub struct RetryState {
    pub policies: RwLock<Vec<RetryPolicy>>,
    pub attempts: RwLock<Vec<RetryAttempt>>,
}

impl RetryState {
    pub fn new() -> Self {
        Self {
            policies: RwLock::new(Vec::new()),
            attempts: RwLock::new(Vec::new()),
        }
    }

    pub async fn list_policies(&self) -> Vec<RetryPolicy> {
        self.policies.read().await.clone()
    }

    pub async fn get_policy(&self, id: &str) -> Option<RetryPolicy> {
        self.policies.read().await.iter()
            .find(|p| p.id == id)
            .cloned()
    }

    pub async fn create_policy(&self, mut policy: RetryPolicy) -> RetryPolicy {
        policy.id = ulid::Ulid::new().to_string();
        policy.created_at = Utc::now();
        policy.updated_at = Utc::now();
        self.policies.write().await.push(policy.clone());
        policy
    }

    pub async fn update_policy(&self, id: &str, update: UpdateRetryPolicyRequest) -> Option<RetryPolicy> {
        let mut policies = self.policies.write().await;
        if let Some(policy) = policies.iter_mut().find(|p| p.id == id) {
            if let Some(name) = update.name {
                policy.name = name;
            }
            if let Some(desc) = update.description {
                policy.description = Some(desc);
            }
            if let Some(max) = update.max_attempts {
                policy.max_attempts = max;
            }
            if let Some(delay) = update.initial_delay_secs {
                policy.initial_delay_secs = delay;
            }
            if let Some(mul) = update.backoff_multiplier {
                policy.backoff_multiplier = mul;
            }
            if let Some(max_delay) = update.max_delay_secs {
                policy.max_delay_secs = max_delay;
            }
            if let Some(jitter) = update.jitter {
                policy.jitter = jitter;
            }
            if let Some(statuses) = update.retry_on_statuses {
                policy.retry_on_statuses = statuses;
            }
            if let Some(enabled) = update.enabled {
                policy.enabled = enabled;
            }
            policy.updated_at = Utc::now();
            return Some(policy.clone());
        }
        None
    }

    pub async fn delete_policy(&self, id: &str) -> bool {
        let mut policies = self.policies.write().await;
        let pos = policies.iter().position(|p| p.id == id);
        if let Some(pos) = pos {
            policies.remove(pos);
            true
        } else {
            false
        }
    }

    pub async fn apply_policy(&self, policy_id: &str, task_id: &str) -> Option<(RetryAttempt, u64)> {
        let policy = self.get_policy(policy_id).await?;
        if !policy.enabled {
            return None;
        }

        let attempt_number = self.attempts.read().await
            .iter()
            .filter(|a| a.task_id == task_id && a.policy_id == policy_id)
            .count() as u32 + 1;

        if attempt_number > policy.max_attempts {
            return None;
        }

        let delay = policy.calculate_delay(attempt_number);
        
        let mut attempt = RetryAttempt::new(
            task_id.to_string(),
            policy_id.to_string(),
            attempt_number,
        );
        attempt.delay_used_secs = delay;
        attempt.mark_running();

        self.attempts.write().await.push(attempt.clone());
        Some((attempt, delay))
    }

    pub async fn get_attempts(&self, task_id: Option<&str>) -> Vec<RetryAttempt> {
        let attempts = self.attempts.read().await;
        match task_id {
            Some(id) => attempts.iter().filter(|a| a.task_id == id).cloned().collect(),
            None => attempts.clone(),
        }
    }

    pub async fn update_attempt(&self, attempt_id: &str, status: RetryStatus, error: Option<String>) {
        let mut attempts = self.attempts.write().await;
        if let Some(attempt) = attempts.iter_mut().find(|a| a.id == attempt_id) {
            match status {
                RetryStatus::Success => attempt.mark_success(),
                RetryStatus::Failed => attempt.mark_failed(error.unwrap_or_default()),
                RetryStatus::Exhausted => attempt.mark_exhausted(),
                RetryStatus::Cancelled => attempt.mark_cancelled(),
                _ => {}
            }
        }
    }
}

impl Default for RetryState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the retry policy router
pub fn create_retry_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/retry-policies", get(list_policies))
        .route("/api/v1/retry-policies/:id", get(get_policy))
        .route("/api/v1/retry-policies", post(create_policy))
        .route("/api/v1/retry-policies/:id", put(update_policy))
        .route("/api/v1/retry-policies/:id", delete(delete_policy))
        .route("/api/v1/retry-policies/:id/apply", post(apply_policy))
        .route("/api/v1/retry-attempts", get(list_attempts))
}

/// GET /api/v1/retry-policies - List all retry policies
async fn list_policies(
    State(state): State<AppState>,
) -> Result<Json<Vec<RetryPolicy>>, (axum::http::StatusCode, String)> {
    let policies = state.retry_state.list_policies().await;
    Ok(Json(policies))
}

/// GET /api/v1/retry-policies/:id - Get a specific retry policy
async fn get_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RetryPolicy>, (axum::http::StatusCode, String)> {
    state.retry_state.get_policy(&id).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Policy {} not found", id)))
}

/// POST /api/v1/retry-policies - Create a new retry policy
async fn create_policy(
    State(state): State<AppState>,
    Json(req): Json<CreateRetryPolicyRequest>,
) -> Result<Json<RetryPolicy>, (axum::http::StatusCode, String)> {
    let mut policy = RetryPolicy::new(req.name);
    policy.description = req.description;
    
    if let Some(max) = req.max_attempts {
        policy.max_attempts = max;
    }
    if let Some(delay) = req.initial_delay_secs {
        policy.initial_delay_secs = delay;
    }
    if let Some(mul) = req.backoff_multiplier {
        policy.backoff_multiplier = mul;
    }
    if let Some(max_delay) = req.max_delay_secs {
        policy.max_delay_secs = max_delay;
    }
    if let Some(jitter) = req.jitter {
        policy.jitter = jitter;
    }
    if let Some(statuses) = req.retry_on_statuses {
        policy.retry_on_statuses = statuses;
    }

    let created = state.retry_state.create_policy(policy).await;
    Ok(Json(created))
}

/// PUT /api/v1/retry-policies/:id - Update a retry policy
async fn update_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateRetryPolicyRequest>,
) -> Result<Json<RetryPolicy>, (axum::http::StatusCode, String)> {
    state.retry_state.update_policy(&id, req).await.map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Policy {} not found", id)))
}

/// DELETE /api/v1/retry-policies/:id - Delete a retry policy
async fn delete_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    if state.retry_state.delete_policy(&id).await {
        Ok(Json(serde_json::json!({ "success": true })))
    } else {
        Err(ApiError::not_found(format!("Policy {} not found", id)))
    }
}

/// POST /api/v1/retry-policies/:id/apply - Apply policy to a task
async fn apply_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ApplyRetryPolicyRequest>,
) -> Result<Json<ApplyRetryPolicyResponse>, (axum::http::StatusCode, String)> {
    match state.retry_state.apply_policy(&id, &req.task_id).await {
        Some((attempt, delay)) => {
            let next_retry = Utc::now() + chrono::Duration::seconds(delay as i64);
            Ok(Json(ApplyRetryPolicyResponse {
                success: true,
                attempt_id: Some(attempt.id),
                next_retry_at: Some(next_retry),
                message: format!("Retry scheduled for attempt {}", attempt.attempt_number),
            }))
        }
        None => Ok(Json(ApplyRetryPolicyResponse {
            success: false,
            attempt_id: None,
            next_retry_at: None,
            message: "Failed to schedule retry".to_string(),
        })),
    }
}

/// GET /api/v1/retry-attempts - List retry attempts
#[derive(Debug, serde::Deserialize)]
pub struct AttemptsQuery {
    pub task_id: Option<String>,
}

async fn list_attempts(
    State(state): State<AppState>,
    Query(query): Query<AttemptsQuery>,
) -> Result<Json<Vec<RetryAttempt>>, (axum::http::StatusCode, String)> {
    let attempts = state.retry_state.get_attempts(query.task_id.as_deref()).await;
    Ok(Json(attempts))
}
