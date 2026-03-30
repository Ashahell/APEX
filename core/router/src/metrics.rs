use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct RouterMetrics {
    tasks_total: Arc<RwLock<HashMap<String, u64>>>,
    tasks_by_tier: Arc<RwLock<HashMap<String, u64>>>,
    tasks_by_status: Arc<RwLock<HashMap<String, u64>>>,
    total_cost: Arc<RwLock<f64>>,
    mcp: Arc<McpMetrics>,
    // Phase 2: Telemetry surface
    telemetry: Arc<TelemetrySurface>,
}

impl RouterMetrics {
    pub fn new() -> Self {
        Self {
            tasks_total: Arc::new(RwLock::new(HashMap::new())),
            tasks_by_tier: Arc::new(RwLock::new(HashMap::new())),
            tasks_by_status: Arc::new(RwLock::new(HashMap::new())),
            total_cost: Arc::new(RwLock::new(0.0)),
            mcp: Arc::new(McpMetrics::new()),
            telemetry: Arc::new(TelemetrySurface::new()),
        }
    }

    pub fn mcp(&self) -> &McpMetrics {
        &self.mcp
    }

    pub fn telemetry(&self) -> &TelemetrySurface {
        &self.telemetry
    }

    pub async fn record_task(&self, tier: &str, status: &str) {
        {
            let mut total = self.tasks_total.write().await;
            *total.entry("total".to_string()).or_insert(0) += 1;
        }
        {
            let mut by_tier = self.tasks_by_tier.write().await;
            *by_tier.entry(tier.to_string()).or_insert(0) += 1;
        }
        {
            let mut by_status = self.tasks_by_status.write().await;
            *by_status.entry(status.to_string()).or_insert(0) += 1;
        }
    }

    pub async fn record_cost(&self, cost: f64) {
        let mut total = self.total_cost.write().await;
        *total += cost;
    }

    pub async fn get_metrics(&self) -> MetricsSnapshot {
        let tasks_total = self.tasks_total.read().await.clone();
        let tasks_by_tier = self.tasks_by_tier.read().await.clone();
        let tasks_by_status = self.tasks_by_status.read().await.clone();
        let total_cost = *self.total_cost.read().await;
        let mcp_metrics = self.mcp.get_metrics().await;
        let telemetry_snapshot = self.telemetry.get_snapshot().await;

        MetricsSnapshot {
            tasks_total,
            tasks_by_tier,
            tasks_by_status,
            total_cost,
            mcp: Some(mcp_metrics),
            telemetry: Some(telemetry_snapshot),
        }
    }
}

/// MCP-specific metrics
#[derive(Debug, Clone, Default)]
pub struct McpMetrics {
    servers_connected: Arc<RwLock<u64>>,
    servers_disconnected: Arc<RwLock<u64>>,
    tools_executed: Arc<RwLock<u64>>,
    tools_failed: Arc<RwLock<u64>>,
    connections_failed: Arc<RwLock<u64>>,
    reconnections: Arc<RwLock<u64>>,
    tool_execution_times_ms: Arc<RwLock<Vec<u64>>>,
}

impl McpMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn record_server_connected(&self) {
        let mut count = self.servers_connected.write().await;
        *count += 1;
    }

    pub async fn record_server_disconnected(&self) {
        let mut count = self.servers_disconnected.write().await;
        *count += 1;
    }

    pub async fn record_tool_execution(&self, duration_ms: u64, success: bool) {
        if success {
            let mut count = self.tools_executed.write().await;
            *count += 1;
        } else {
            let mut count = self.tools_failed.write().await;
            *count += 1;
        }

        let mut times = self.tool_execution_times_ms.write().await;
        times.push(duration_ms);

        // Keep only last 1000 entries
        if times.len() > 1000 {
            times.remove(0);
        }
    }

    pub async fn record_connection_failed(&self) {
        let mut count = self.connections_failed.write().await;
        *count += 1;
    }

    pub async fn record_reconnection(&self) {
        let mut count = self.reconnections.write().await;
        *count += 1;
    }

    pub async fn get_metrics(&self) -> McpMetricsSnapshot {
        let servers_connected = *self.servers_connected.read().await;
        let servers_disconnected = *self.servers_disconnected.read().await;
        let tools_executed = *self.tools_executed.read().await;
        let tools_failed = *self.tools_failed.read().await;
        let connections_failed = *self.connections_failed.read().await;
        let reconnections = *self.reconnections.read().await;
        let tool_execution_times = self.tool_execution_times_ms.read().await.clone();

        let avg_execution_time = if tool_execution_times.is_empty() {
            0.0
        } else {
            tool_execution_times.iter().sum::<u64>() as f64 / tool_execution_times.len() as f64
        };

        McpMetricsSnapshot {
            servers_connected,
            servers_disconnected,
            tools_executed,
            tools_failed,
            connections_failed,
            reconnections,
            avg_tool_execution_time_ms: avg_execution_time,
            total_tool_executions: tools_executed + tools_failed,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct McpMetricsSnapshot {
    pub servers_connected: u64,
    pub servers_disconnected: u64,
    pub tools_executed: u64,
    pub tools_failed: u64,
    pub connections_failed: u64,
    pub reconnections: u64,
    pub avg_tool_execution_time_ms: f64,
    pub total_tool_executions: u64,
}

/// Per-endpoint latency tracking
#[derive(Debug, Clone, Default)]
pub struct EndpointLatencyTracker {
    latencies_ms: Arc<RwLock<Vec<u64>>>,
    max_samples: usize,
}

impl EndpointLatencyTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            latencies_ms: Arc::new(RwLock::new(Vec::new())),
            max_samples,
        }
    }

    pub async fn record(&self, latency_ms: u64) {
        let mut latencies = self.latencies_ms.write().await;
        latencies.push(latency_ms);
        if latencies.len() > self.max_samples {
            latencies.remove(0);
        }
    }

    pub async fn get_stats(&self) -> LatencyStats {
        let latencies = self.latencies_ms.read().await;
        if latencies.is_empty() {
            return LatencyStats::default();
        }

        let mut sorted = latencies.clone();
        sorted.sort_unstable();

        let total: u64 = sorted.iter().sum();
        let count = sorted.len() as u64;
        let avg = total / count;
        let min = *sorted.first().unwrap_or(&0);
        let max = *sorted.last().unwrap_or(&0);
        let p50 = sorted[(count as f64 * 0.5).min(sorted.len() as f64 - 1.0) as usize];
        let p95 = sorted[(count as f64 * 0.95).min(sorted.len() as f64 - 1.0) as usize];
        let p99 = sorted[(count as f64 * 0.99).min(sorted.len() as f64 - 1.0) as usize];

        LatencyStats {
            count,
            avg_ms: avg,
            min_ms: min,
            max_ms: max,
            p50_ms: p50,
            p95_ms: p95,
            p99_ms: p99,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct LatencyStats {
    pub count: u64,
    pub avg_ms: u64,
    pub min_ms: u64,
    pub max_ms: u64,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
}

/// Per-endpoint error rate tracking
#[derive(Debug, Clone, Default)]
pub struct EndpointErrorTracker {
    requests: Arc<RwLock<u64>>,
    errors: Arc<RwLock<u64>>,
    error_types: Arc<RwLock<HashMap<String, u64>>>,
}

impl EndpointErrorTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn record_request(&self) {
        let mut reqs = self.requests.write().await;
        *reqs += 1;
    }

    pub async fn record_error(&self, error_type: &str) {
        let mut errs = self.errors.write().await;
        *errs += 1;

        let mut types = self.error_types.write().await;
        *types.entry(error_type.to_string()).or_insert(0) += 1;
    }

    pub async fn get_stats(&self) -> ErrorStats {
        let requests = *self.requests.read().await;
        let errors = *self.errors.read().await;
        let error_types = self.error_types.read().await.clone();
        let error_rate = if requests > 0 {
            (errors as f64 / requests as f64) * 100.0
        } else {
            0.0
        };

        ErrorStats {
            requests,
            errors,
            error_rate_pct: (error_rate * 100.0).round() / 100.0,
            error_types,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct ErrorStats {
    pub requests: u64,
    pub errors: u64,
    pub error_rate_pct: f64,
    pub error_types: HashMap<String, u64>,
}

/// Phase 2: Telemetry surface for all endpoints
#[derive(Debug, Clone, Default)]
pub struct TelemetrySurface {
    pub endpoint_latencies: Arc<RwLock<HashMap<String, EndpointLatencyTracker>>>,
    pub endpoint_errors: Arc<RwLock<HashMap<String, EndpointErrorTracker>>>,
}

impl TelemetrySurface {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn record_latency(&self, endpoint: &str, latency_ms: u64) {
        let mut latencies = self.endpoint_latencies.write().await;
        let tracker = latencies
            .entry(endpoint.to_string())
            .or_insert_with(|| EndpointLatencyTracker::new(1000));
        tracker.record(latency_ms).await;
    }

    pub async fn record_request(&self, endpoint: &str) {
        let mut errors = self.endpoint_errors.write().await;
        let tracker = errors
            .entry(endpoint.to_string())
            .or_insert_with(EndpointErrorTracker::new);
        tracker.record_request().await;
    }

    pub async fn record_error(&self, endpoint: &str, error_type: &str) {
        let mut errors = self.endpoint_errors.write().await;
        let tracker = errors
            .entry(endpoint.to_string())
            .or_insert_with(EndpointErrorTracker::new);
        tracker.record_error(error_type).await;
    }

    pub async fn get_snapshot(&self) -> TelemetrySnapshot {
        let latencies = self.endpoint_latencies.read().await;
        let errors = self.endpoint_errors.read().await;

        let mut latency_map = HashMap::new();
        for (endpoint, tracker) in latencies.iter() {
            latency_map.insert(endpoint.clone(), tracker.get_stats().await);
        }

        let mut error_map = HashMap::new();
        for (endpoint, tracker) in errors.iter() {
            error_map.insert(endpoint.clone(), tracker.get_stats().await);
        }

        TelemetrySnapshot {
            endpoint_latencies: latency_map,
            endpoint_errors: error_map,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct TelemetrySnapshot {
    pub endpoint_latencies: HashMap<String, LatencyStats>,
    pub endpoint_errors: HashMap<String, ErrorStats>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub tasks_total: HashMap<String, u64>,
    pub tasks_by_tier: HashMap<String, u64>,
    pub tasks_by_status: HashMap<String, u64>,
    pub total_cost: f64,
    pub mcp: Option<McpMetricsSnapshot>,
    // Phase 2: Telemetry surface
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<TelemetrySnapshot>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_new() {
        let metrics = RouterMetrics::new();
        let snapshot = metrics.get_metrics().await;

        assert_eq!(snapshot.tasks_total.get("total"), None);
        assert!(snapshot.total_cost == 0.0);
    }

    #[tokio::test]
    async fn test_record_task() {
        let metrics = RouterMetrics::new();
        metrics.record_task("deep", "completed").await;

        let snapshot = metrics.get_metrics().await;

        assert_eq!(snapshot.tasks_total.get("total"), Some(&1u64));
        assert_eq!(snapshot.tasks_by_tier.get("deep"), Some(&1u64));
        assert_eq!(snapshot.tasks_by_status.get("completed"), Some(&1u64));
    }

    #[tokio::test]
    async fn test_record_multiple_tasks() {
        let metrics = RouterMetrics::new();

        metrics.record_task("deep", "completed").await;
        metrics.record_task("instant", "completed").await;
        metrics.record_task("deep", "failed").await;

        let snapshot = metrics.get_metrics().await;

        assert_eq!(snapshot.tasks_total.get("total"), Some(&3u64));
        assert_eq!(snapshot.tasks_by_tier.get("deep"), Some(&2u64));
        assert_eq!(snapshot.tasks_by_tier.get("instant"), Some(&1u64));
        assert_eq!(snapshot.tasks_by_status.get("completed"), Some(&2u64));
        assert_eq!(snapshot.tasks_by_status.get("failed"), Some(&1u64));
    }

    #[tokio::test]
    async fn test_record_cost() {
        let metrics = RouterMetrics::new();

        metrics.record_cost(0.50).await;
        metrics.record_cost(1.25).await;

        let snapshot = metrics.get_metrics().await;

        assert_eq!(snapshot.total_cost, 1.75);
    }

    #[tokio::test]
    async fn test_metrics_combined() {
        let metrics = RouterMetrics::new();

        metrics.record_task("deep", "completed").await;
        metrics.record_cost(0.75).await;

        let snapshot = metrics.get_metrics().await;

        assert_eq!(snapshot.tasks_total.get("total"), Some(&1u64));
        assert_eq!(snapshot.tasks_by_tier.get("deep"), Some(&1u64));
        assert_eq!(snapshot.tasks_by_status.get("completed"), Some(&1u64));
        assert_eq!(snapshot.total_cost, 0.75);
    }
}
