use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct RouterMetrics {
    tasks_total: Arc<RwLock<HashMap<String, u64>>>,
    tasks_by_tier: Arc<RwLock<HashMap<String, u64>>>,
    tasks_by_status: Arc<RwLock<HashMap<String, u64>>>,
    total_cost: Arc<RwLock<f64>>,
}

impl RouterMetrics {
    pub fn new() -> Self {
        Self {
            tasks_total: Arc::new(RwLock::new(HashMap::new())),
            tasks_by_tier: Arc::new(RwLock::new(HashMap::new())),
            tasks_by_status: Arc::new(RwLock::new(HashMap::new())),
            total_cost: Arc::new(RwLock::new(0.0)),
        }
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

        MetricsSnapshot {
            tasks_total,
            tasks_by_tier,
            tasks_by_status,
            total_cost,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub tasks_total: HashMap<String, u64>,
    pub tasks_by_tier: HashMap<String, u64>,
    pub tasks_by_status: HashMap<String, u64>,
    pub total_cost: f64,
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
