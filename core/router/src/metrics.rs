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
}

impl RouterMetrics {
    pub fn new() -> Self {
        Self {
            tasks_total: Arc::new(RwLock::new(HashMap::new())),
            tasks_by_tier: Arc::new(RwLock::new(HashMap::new())),
            tasks_by_status: Arc::new(RwLock::new(HashMap::new())),
            total_cost: Arc::new(RwLock::new(0.0)),
            mcp: Arc::new(McpMetrics::new()),
        }
    }

    pub fn mcp(&self) -> &McpMetrics {
        &self.mcp
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

        MetricsSnapshot {
            tasks_total,
            tasks_by_tier,
            tasks_by_status,
            total_cost,
            mcp: Some(mcp_metrics),
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub tasks_total: HashMap<String, u64>,
    pub tasks_by_tier: HashMap<String, u64>,
    pub tasks_by_status: HashMap<String, u64>,
    pub total_cost: f64,
    pub mcp: Option<McpMetricsSnapshot>,
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
