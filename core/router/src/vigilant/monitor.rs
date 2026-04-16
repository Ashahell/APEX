//! Threshold monitor for tracking agent behavior metrics

use crate::vigilant::models::AlertType;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Threshold configuration for behavior monitoring
#[derive(Debug, Clone)]
pub struct ThresholdConfig {
    /// Max steps without output before warning
    pub no_progress_steps: u32,
    /// Max iterations of same action before loop detection
    pub loop_detection_threshold: u32,
    /// Window in seconds for threshold counting
    pub window_secs: u32,
    /// Timeout warning threshold in seconds
    pub timeout_warning_secs: u32,
    /// High memory usage threshold (percentage)
    pub high_memory_pct: u8,
    /// Max errors before spike alert
    pub error_spike_count: u32,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            no_progress_steps: 10,
            loop_detection_threshold: 100,
            window_secs: 60,
            timeout_warning_secs: 60,
            high_memory_pct: 90,
            error_spike_count: 5,
        }
    }
}

/// Metrics tracked per task
#[derive(Debug, Clone, Default)]
pub struct TaskMetrics {
    /// Current step count
    pub step_count: u32,
    /// Last output hash (to detect no progress)
    pub last_output_hash: Option<u64>,
    /// Action history for loop detection
    pub action_history: Vec<String>,
    /// Error count in current window
    pub error_count: u32,
    /// Last activity timestamp
    pub last_activity: i64,
}

/// Threshold monitor for tracking task behavior
#[derive(Debug, Default)]
pub struct ThresholdMonitor {
    /// Metrics by task ID
    metrics: HashMap<String, TaskMetrics>,
    /// Configuration
    config: ThresholdConfig,
}

impl ThresholdMonitor {
    /// Create a new threshold monitor
    pub fn new(config: ThresholdConfig) -> Self {
        Self {
            metrics: HashMap::new(),
            config,
        }
    }

    /// Get or create metrics for a task
    fn get_metrics(&mut self, task_id: &str) -> &mut TaskMetrics {
        if !self.metrics.contains_key(task_id) {
            self.metrics.insert(
                task_id.to_string(),
                TaskMetrics {
                    last_activity: chrono::Utc::now().timestamp(),
                    ..Default::default()
                },
            );
        }
        self.metrics.get_mut(task_id).unwrap()
    }

    /// Record a step for a task
    pub fn record_step(&mut self, task_id: &str, action: &str, output: &str) -> Vec<AlertType> {
        let mut alerts = Vec::new();
        
        // Store config values before borrowing metrics
        let no_progress_steps = self.config.no_progress_steps;
        let loop_threshold = self.config.loop_detection_threshold;
        
        let metrics = self.get_metrics(task_id);
        
        metrics.step_count += 1;
        metrics.last_activity = chrono::Utc::now().timestamp();

        // Check for no progress
        let output_hash = Some(hash_output(output));
        if metrics.last_output_hash == output_hash && metrics.step_count > no_progress_steps {
            alerts.push(AlertType::NoProgress {
                task_id: task_id.to_string(),
                steps: metrics.step_count,
            });
        }
        metrics.last_output_hash = output_hash;

        // Check for infinite loop
        metrics.action_history.push(action.to_string());
        if metrics.action_history.len() > loop_threshold as usize {
            // Count repetitions
            if let Some(last) = metrics.action_history.last() {
                let repetitions = metrics
                    .action_history
                    .iter()
                    .rev()
                    .take_while(|a| *a == last)
                    .count() as u32;
                
                if repetitions >= loop_threshold {
                    alerts.push(AlertType::InfiniteLoop {
                        task_id: task_id.to_string(),
                        iterations: repetitions,
                    });
                    // Reset to prevent repeated alerts
                    metrics.action_history.clear();
                }
            }
        }

        alerts
    }

    /// Record an error for a task
    pub fn record_error(&mut self, task_id: &str) -> Vec<AlertType> {
        let mut alerts = Vec::new();
        
        // Store config value before borrowing metrics
        let error_spike_count = self.config.error_spike_count;
        
        let metrics = self.get_metrics(task_id);
        
        metrics.error_count += 1;
        metrics.last_activity = chrono::Utc::now().timestamp();

        // Check for error spike
        if metrics.error_count >= error_spike_count {
            alerts.push(AlertType::ErrorSpike {
                task_id: task_id.to_string(),
                error_count: metrics.error_count,
            });
        }

        alerts
    }

    /// Check timeout for a task
    pub fn check_timeout(&self, task_id: &str, remaining_secs: u32) -> Option<AlertType> {
        if remaining_secs <= self.config.timeout_warning_secs {
            Some(AlertType::TimeoutWarning {
                task_id: task_id.to_string(),
                remaining_secs,
            })
        } else {
            None
        }
    }

    /// Check for high memory usage
    pub fn check_memory(&self, usage_pct: u8) -> Option<AlertType> {
        if usage_pct >= self.config.high_memory_pct {
            Some(AlertType::HighMemoryUsage { percentage: usage_pct })
        } else {
            None
        }
    }

    /// Handle an execution pattern detection
    pub fn handle_pattern(&self, pattern: &str, task_id: &str) -> AlertType {
        AlertType::PatternDetected {
            pattern: pattern.to_string(),
            task_id: task_id.to_string(),
        }
    }

    /// Clean up stale metrics
    pub fn cleanup(&mut self, max_age_secs: i64) {
        let now = chrono::Utc::now().timestamp();
        self.metrics.retain(|_, m| now - m.last_activity < max_age_secs);
    }

    /// Get metrics for a task
    pub fn get_metrics_for(&self, task_id: &str) -> Option<TaskMetrics> {
        self.metrics.get(task_id).cloned()
    }
}

/// Simple hash function for output comparison
fn hash_output(output: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    output.hash(&mut hasher);
    hasher.finish()
}

/// Thread-safe wrapper
#[derive(Debug, Clone, Default)]
pub struct SharedThresholdMonitor(pub Arc<RwLock<ThresholdMonitor>>);

impl SharedThresholdMonitor {
    pub fn new(config: ThresholdConfig) -> Self {
        Self(Arc::new(RwLock::new(ThresholdMonitor::new(config))))
    }

    pub async fn record_step(&self, task_id: &str, action: &str, output: &str) -> Vec<AlertType> {
        let mut monitor = self.0.write().await;
        monitor.record_step(task_id, action, output)
    }

    pub async fn record_error(&self, task_id: &str) -> Vec<AlertType> {
        let mut monitor = self.0.write().await;
        monitor.record_error(task_id)
    }

    pub async fn check_timeout(&self, task_id: &str, remaining_secs: u32) -> Option<AlertType> {
        let monitor = self.0.read().await;
        monitor.check_timeout(task_id, remaining_secs)
    }

    pub async fn check_memory(&self, usage_pct: u8) -> Option<AlertType> {
        let monitor = self.0.read().await;
        monitor.check_memory(usage_pct)
    }

    pub async fn handle_pattern(&self, pattern: &str, task_id: &str) -> AlertType {
        let monitor = self.0.read().await;
        monitor.handle_pattern(pattern, task_id)
    }

    pub async fn cleanup(&self, max_age_secs: i64) {
        let mut monitor = self.0.write().await;
        monitor.cleanup(max_age_secs);
    }

    pub async fn get_metrics_for(&self, task_id: &str) -> Option<TaskMetrics> {
        let monitor = self.0.read().await;
        monitor.get_metrics_for(task_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_progress_detection() {
        let mut monitor = ThresholdMonitor::new(ThresholdConfig {
            no_progress_steps: 3,
            ..Default::default()
        });

        // First step
        let alerts = monitor.record_step("task-1", "action1", "output1");
        assert!(alerts.is_empty());

        // Repeat same output
        let alerts = monitor.record_step("task-1", "action2", "output1");
        assert!(alerts.is_empty());

        // Third repeat triggers after 4 steps with same output
        let alerts = monitor.record_step("task-1", "action3", "output1");
        assert!(alerts.is_empty());

        // Fourth repeat triggers the alert
        let alerts = monitor.record_step("task-1", "action4", "output1");
        assert!(!alerts.is_empty());
        assert!(matches!(
            &alerts[0],
            AlertType::NoProgress { task_id, steps: 4 } if task_id == "task-1"
        ));
    }

    #[test]
    fn test_loop_detection() {
        let mut monitor = ThresholdMonitor::new(ThresholdConfig {
            loop_detection_threshold: 5,
            ..Default::default()
        });

        // Repeat same action
        for i in 0..6 {
            let alerts = monitor.record_step("task-1", "repeat_action", &format!("output{}", i));
            if i == 5 {
                assert!(!alerts.is_empty());
                assert!(matches!(
                    &alerts[0],
                    AlertType::InfiniteLoop { task_id, iterations: 6 } if task_id == "task-1"
                ));
            }
        }
    }
}
