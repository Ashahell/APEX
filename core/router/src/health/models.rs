//! Health Check Data Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health status for a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
    Checking,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
            HealthStatus::Checking => write!(f, "checking"),
        }
    }
}

/// Individual component health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub component: String,
    pub status: HealthStatus,
    pub last_check: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
    pub last_failure: Option<DateTime<Utc>>,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub consecutive_failures: u32,
    pub check_interval_secs: u64,
    pub metadata: HashMap<String, String>,
}

impl ComponentHealth {
    pub fn new(component: String, check_interval_secs: u64) -> Self {
        Self {
            component,
            status: HealthStatus::Unknown,
            last_check: None,
            last_success: None,
            last_failure: None,
            response_time_ms: None,
            error_message: None,
            consecutive_failures: 0,
            check_interval_secs,
            metadata: HashMap::new(),
        }
    }

    pub fn mark_healthy(&mut self, response_time_ms: u64) {
        self.status = HealthStatus::Healthy;
        self.last_check = Some(Utc::now());
        self.last_success = Some(Utc::now());
        self.response_time_ms = Some(response_time_ms);
        self.error_message = None;
        self.consecutive_failures = 0;
    }

    pub fn mark_unhealthy(&mut self, error: String, response_time_ms: u64) {
        self.status = HealthStatus::Unhealthy;
        self.last_check = Some(Utc::now());
        self.last_failure = Some(Utc::now());
        self.response_time_ms = Some(response_time_ms);
        self.error_message = Some(error);
        self.consecutive_failures += 1;
    }

    pub fn mark_degraded(&mut self, error: String, response_time_ms: u64) {
        self.status = HealthStatus::Degraded;
        self.last_check = Some(Utc::now());
        self.response_time_ms = Some(response_time_ms);
        self.error_message = Some(error);
    }

    pub fn mark_checking(&mut self) {
        self.status = HealthStatus::Checking;
    }
}

/// Overall system health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub overall_status: HealthStatus,
    pub healthy_count: u32,
    pub degraded_count: u32,
    pub unhealthy_count: u32,
    pub unknown_count: u32,
    pub components: Vec<ComponentHealth>,
    pub last_updated: DateTime<Utc>,
}

impl Default for HealthSummary {
    fn default() -> Self {
        Self {
            overall_status: HealthStatus::Unknown,
            healthy_count: 0,
            degraded_count: 0,
            unhealthy_count: 0,
            unknown_count: 0,
            components: Vec::new(),
            last_updated: Utc::now(),
        }
    }
}

/// Health check history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthHistoryEntry {
    pub id: String,
    pub component: String,
    pub status: HealthStatus,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Request to run a manual health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunHealthCheckRequest {
    pub force: Option<bool>,
}

/// Response for health check run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunHealthCheckResponse {
    pub component: String,
    pub status: HealthStatus,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Configuration for a health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub timeout_secs: u64,
    pub retry_count: u32,
    pub degraded_threshold_ms: u64,
    pub unhealthy_threshold_ms: u64,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 60,
            timeout_secs: 5,
            retry_count: 3,
            degraded_threshold_ms: 1000,
            unhealthy_threshold_ms: 5000,
        }
    }
}

/// System component identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemComponent {
    Llm,
    Database,
    SkillPool,
    MemoryIndexer,
    VmPool,
    MessageBus,
    WebSocket,
    SkillWorker,
}

impl SystemComponent {
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemComponent::Llm => "llm",
            SystemComponent::Database => "database",
            SystemComponent::SkillPool => "skill_pool",
            SystemComponent::MemoryIndexer => "memory_indexer",
            SystemComponent::VmPool => "vm_pool",
            SystemComponent::MessageBus => "message_bus",
            SystemComponent::WebSocket => "websocket",
            SystemComponent::SkillWorker => "skill_worker",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SystemComponent::Llm => "LLM Server",
            SystemComponent::Database => "Database",
            SystemComponent::SkillPool => "Skill Pool",
            SystemComponent::MemoryIndexer => "Memory Indexer",
            SystemComponent::VmPool => "VM Pool",
            SystemComponent::MessageBus => "Message Bus",
            SystemComponent::WebSocket => "WebSocket",
            SystemComponent::SkillWorker => "Skill Worker",
        }
    }

    pub fn all() -> Vec<SystemComponent> {
        vec![
            SystemComponent::Llm,
            SystemComponent::Database,
            SystemComponent::SkillPool,
            SystemComponent::MemoryIndexer,
            SystemComponent::VmPool,
            SystemComponent::MessageBus,
            SystemComponent::WebSocket,
            SystemComponent::SkillWorker,
        ]
    }
}
