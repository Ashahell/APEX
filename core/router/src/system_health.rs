use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct SystemMonitor {
    state: Arc<RwLock<SystemState>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemState {
    pub uptime_secs: u64,
    pub requests_total: u64,
    pub requests_by_endpoint: std::collections::HashMap<String, u64>,
    pub errors_total: u64,
    pub last_error: Option<String>,
    pub avg_response_time_ms: f64,
    pub response_times: Vec<f64>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(SystemState::default())),
        }
    }

    pub async fn record_request(&self, endpoint: &str, response_time_ms: f64, is_error: bool) {
        let mut state = self.state.write().await;
        state.requests_total += 1;

        *state
            .requests_by_endpoint
            .entry(endpoint.to_string())
            .or_insert(0) += 1;

        if is_error {
            state.errors_total += 1;
        }

        state.response_times.push(response_time_ms);
        if state.response_times.len() > 1000 {
            state.response_times.drain(0..100);
        }

        let sum: f64 = state.response_times.iter().sum();
        state.avg_response_time_ms = sum / state.response_times.len() as f64;
    }

    pub async fn record_error(&self, error: String) {
        let mut state = self.state.write().await;
        state.errors_total += 1;
        state.last_error = Some(error);
    }

    pub async fn get_system_health(&self) -> SystemHealth {
        let state = self.state.read().await;

        let uptime = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        SystemHealth {
            uptime_secs: uptime,
            requests_total: state.requests_total,
            errors_total: state.errors_total,
            error_rate: if state.requests_total > 0 {
                (state.errors_total as f64 / state.requests_total as f64) * 100.0
            } else {
                0.0
            },
            avg_response_time_ms: state.avg_response_time_ms,
            requests_by_endpoint: state.requests_by_endpoint.clone(),
            last_error: state.last_error.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub uptime_secs: u64,
    pub requests_total: u64,
    pub errors_total: u64,
    pub error_rate: f64,
    pub avg_response_time_ms: f64,
    pub requests_by_endpoint: std::collections::HashMap<String, u64>,
    pub last_error: Option<String>,
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_monitor_new() {
        let monitor = SystemMonitor::new();
        let health = monitor.get_system_health().await;

        assert_eq!(health.requests_total, 0);
        assert_eq!(health.errors_total, 0);
    }

    #[tokio::test]
    async fn test_record_request() {
        let monitor = SystemMonitor::new();

        monitor.record_request("/api/v1/tasks", 15.5, false).await;
        monitor.record_request("/api/v1/tasks", 22.0, false).await;
        monitor.record_request("/api/v1/tasks", 10.0, true).await;

        let health = monitor.get_system_health().await;

        assert_eq!(health.requests_total, 3);
        assert_eq!(health.errors_total, 1);
        assert!(health.avg_response_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_error_rate() {
        let monitor = SystemMonitor::new();

        for _ in 0..10 {
            monitor.record_request("/api/test", 10.0, false).await;
        }
        monitor.record_request("/api/test", 10.0, true).await;

        let health = monitor.get_system_health().await;

        assert_eq!(health.requests_total, 11);
        assert_eq!(health.errors_total, 1);
        assert!((health.error_rate - 9.09).abs() < 0.1);
    }
}
