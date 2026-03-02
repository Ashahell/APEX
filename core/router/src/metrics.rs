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
