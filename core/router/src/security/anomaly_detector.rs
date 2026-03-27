//! Anomaly Detection: Monitors skill execution for unusual patterns
//!
//! This module provides:
//! - Statistical anomaly detection for execution patterns
//! - Resource usage monitoring
//! - Behavioral analysis
//! - Alert generation for suspicious activity

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Types of anomalies that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    HighFrequency,
    UnusualDuration,
    ResourceSpike,
    ErrorRateSpike,
    UnusualSkill,
    InputSizeAnomaly,
    SequentialFailures,
    TimePatternAnomaly,
    // Death Spiral patterns (OpenClaw-inspired)
    FileCreationBurst,
    ToolCallLoop,
    NoSideEffects,
    ErrorCascade,
}

impl AnomalyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnomalyType::HighFrequency => "high_frequency",
            AnomalyType::UnusualDuration => "unusual_duration",
            AnomalyType::ResourceSpike => "resource_spike",
            AnomalyType::ErrorRateSpike => "error_rate_spike",
            AnomalyType::UnusualSkill => "unusual_skill",
            AnomalyType::InputSizeAnomaly => "input_size_anomaly",
            AnomalyType::SequentialFailures => "sequential_failures",
            AnomalyType::TimePatternAnomaly => "time_pattern_anomaly",
            // Death Spiral patterns
            AnomalyType::FileCreationBurst => "file_creation_burst",
            AnomalyType::ToolCallLoop => "tool_call_loop",
            AnomalyType::NoSideEffects => "no_side_effects",
            AnomalyType::ErrorCascade => "error_cascade",
        }
    }
}

/// Severity of detected anomaly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AnomalySeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnomalySeverity::Low => "low",
            AnomalySeverity::Medium => "medium",
            AnomalySeverity::High => "high",
            AnomalySeverity::Critical => "critical",
        }
    }
}

/// A detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub id: String,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub skill_name: Option<String>,
    pub task_id: Option<String>,
    pub description: String,
    pub details: HashMap<String, String>,
    pub detected_at: String,
}

/// Statistics for a skill's execution history
#[derive(Debug, Clone)]
pub struct SkillStats {
    pub skill_name: String,
    pub total_executions: u64,
    pub total_errors: u64,
    pub total_duration_ms: u64,
    pub avg_duration_ms: f64,
    pub max_duration_ms: u64,
    pub min_duration_ms: u64,
    pub last_execution: Option<Instant>,
    pub recent_durations: VecDeque<u64>,
    pub recent_errors: u64,
    pub recent_input_sizes: VecDeque<usize>,
}

/// Configuration constants for anomaly detection
mod config_constants {
    /// Maximum executions per minute before triggering high frequency alert
    pub const DEFAULT_MAX_EXECUTIONS_PER_MINUTE: u32 = 60;
    /// Standard deviation multiplier for duration anomalies
    pub const DEFAULT_DURATION_STD_DEV_MULTIPLIER: f64 = 3.0;
    /// Maximum input size in bytes (1MB)
    pub const DEFAULT_MAX_INPUT_SIZE_BYTES: usize = 1_000_000;
    /// Number of sequential failures before alerting
    pub const DEFAULT_SEQUENTIAL_FAILURE_THRESHOLD: u32 = 5;
    /// Window for recent stats (in number of executions)
    pub const DEFAULT_STATS_WINDOW_SIZE: usize = 100;
    /// Minimum executions before anomaly detection starts
    pub const DEFAULT_MIN_EXECUTIONS_FOR_ANALYSIS: u32 = 10;
    /// Max file creates per execution before burst alert
    pub const DEFAULT_FILE_BURST_THRESHOLD: u32 = 10;
    /// Max same tool calls before loop alert
    pub const DEFAULT_TOOL_LOOP_THRESHOLD: u32 = 5;
    /// Max tool calls with no side effects before alert
    pub const DEFAULT_NO_SIDE_EFFECTS_THRESHOLD: u32 = 10;
    /// Max sequential errors before cascade alert
    pub const DEFAULT_ERROR_CASCADE_THRESHOLD: u32 = 3;
}

/// Configuration for anomaly detection
#[derive(Debug, Clone)]
pub struct AnomalyConfig {
    /// Maximum executions per minute before triggering high frequency alert
    pub max_executions_per_minute: u32,
    /// Standard deviation multiplier for duration anomalies
    pub duration_std_dev_multiplier: f64,
    /// Maximum input size in bytes
    pub max_input_size_bytes: usize,
    /// Number of sequential failures before alerting
    pub sequential_failure_threshold: u32,
    /// Window for recent stats (in number of executions)
    pub stats_window_size: usize,
    /// Minimum executions before anomaly detection starts
    pub min_executions_for_analysis: u32,
    // Death Spiral Detection (OpenClaw-inspired)
    /// Max file creates per 5 seconds before burst alert
    pub file_burst_threshold: u32,
    /// Max same tool calls in a row before loop alert
    pub tool_loop_threshold: u32,
    /// Max tool calls with no state change before alert
    pub no_side_effects_threshold: u32,
    /// Max sequential errors before cascade alert
    pub error_cascade_threshold: u32,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        use config_constants::*;
        Self {
            max_executions_per_minute: DEFAULT_MAX_EXECUTIONS_PER_MINUTE,
            duration_std_dev_multiplier: DEFAULT_DURATION_STD_DEV_MULTIPLIER,
            max_input_size_bytes: DEFAULT_MAX_INPUT_SIZE_BYTES,
            sequential_failure_threshold: DEFAULT_SEQUENTIAL_FAILURE_THRESHOLD,
            stats_window_size: DEFAULT_STATS_WINDOW_SIZE,
            min_executions_for_analysis: DEFAULT_MIN_EXECUTIONS_FOR_ANALYSIS,
            // Death Spiral Detection defaults
            file_burst_threshold: DEFAULT_FILE_BURST_THRESHOLD,
            tool_loop_threshold: DEFAULT_TOOL_LOOP_THRESHOLD,
            no_side_effects_threshold: DEFAULT_NO_SIDE_EFFECTS_THRESHOLD,
            error_cascade_threshold: DEFAULT_ERROR_CASCADE_THRESHOLD,
        }
    }
}

/// Anomaly Detector - monitors skill execution patterns
pub struct AnomalyDetector {
    config: AnomalyConfig,
    /// Skill name -> statistics
    skill_stats: Arc<RwLock<HashMap<String, SkillStats>>>,
    /// Recent executions for frequency analysis
    recent_executions: Arc<RwLock<VecDeque<ExecutionRecord>>>,
    /// All detected anomalies
    anomalies: Arc<RwLock<Vec<Anomaly>>>,
}

/// Record of a single execution for frequency analysis
#[derive(Debug, Clone)]
struct ExecutionRecord {
    skill_name: String,
    timestamp: Instant,
    duration_ms: u64,
    success: bool,
    input_size: usize,
    // Death Spiral tracking
    files_created: u32,
    tool_calls: Vec<String>,
    had_side_effects: bool,
}

impl AnomalyDetector {
    /// Create a new anomaly detector with default configuration
    pub fn new() -> Self {
        Self::with_config(AnomalyConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: AnomalyConfig) -> Self {
        Self {
            config,
            skill_stats: Arc::new(RwLock::new(HashMap::new())),
            recent_executions: Arc::new(RwLock::new(VecDeque::new())),
            anomalies: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a skill execution for analysis
    ///
    /// # Arguments
    /// * `skill_name` - Name of the skill executed
    /// * `task_id` - Task ID for tracking
    /// * `duration_ms` - Execution duration in milliseconds
    /// * `success` - Whether execution succeeded
    /// * `input_size` - Size of input in bytes
    /// * `files_created` - Number of files created (for death spiral detection)
    /// * `tool_calls` - List of tools called (for loop detection)
    /// * `had_side_effects` - Whether execution had observable side effects
    pub async fn record_execution(
        &self,
        skill_name: &str,
        task_id: &str,
        duration_ms: u64,
        success: bool,
        input_size: usize,
        files_created: Option<u32>,
        tool_calls: Option<Vec<String>>,
        had_side_effects: Option<bool>,
    ) -> Option<Anomaly> {
        let now = Instant::now();

        // Extract death spiral data for anomaly detection (before moving into block)
        let files_created_val = files_created.unwrap_or(0);
        let tool_calls_val = tool_calls.unwrap_or_default();
        let had_side_effects_val = had_side_effects.unwrap_or(true);

        // Record in recent executions for frequency analysis
        {
            let mut recent = self.recent_executions.write().await;
            recent.push_back(ExecutionRecord {
                skill_name: skill_name.to_string(),
                timestamp: now,
                duration_ms,
                success,
                input_size,
                files_created: files_created_val,
                tool_calls: tool_calls_val.clone(),
                had_side_effects: had_side_effects_val,
            });

            // Keep only last minute of executions
            let cutoff = now - Duration::from_secs(60);
            while recent.front().map_or(true, |r| r.timestamp < cutoff) {
                recent.pop_front();
            }
        }

        // Update skill-specific stats
        let anomaly = {
            let mut stats_map = self.skill_stats.write().await;
            let stats = stats_map
                .entry(skill_name.to_string())
                .or_insert_with(|| SkillStats {
                    skill_name: skill_name.to_string(),
                    total_executions: 0,
                    total_errors: 0,
                    total_duration_ms: 0,
                    avg_duration_ms: 0.0,
                    max_duration_ms: 0,
                    min_duration_ms: u64::MAX,
                    last_execution: None,
                    recent_durations: VecDeque::with_capacity(self.config.stats_window_size),
                    recent_errors: 0,
                    recent_input_sizes: VecDeque::with_capacity(self.config.stats_window_size),
                });

            // Update stats
            stats.total_executions += 1;
            stats.total_duration_ms += duration_ms;
            stats.avg_duration_ms = stats.total_duration_ms as f64 / stats.total_executions as f64;
            stats.max_duration_ms = stats.max_duration_ms.max(duration_ms);
            stats.min_duration_ms = stats.min_duration_ms.min(duration_ms);
            stats.last_execution = Some(now);

            // Maintain window of recent data
            stats.recent_durations.push_back(duration_ms);
            if stats.recent_durations.len() > self.config.stats_window_size {
                stats.recent_durations.pop_front();
            }

            if !success {
                stats.total_errors += 1;
                stats.recent_errors += 1;
            }

            stats.recent_input_sizes.push_back(input_size);
            if stats.recent_input_sizes.len() > self.config.stats_window_size {
                stats.recent_input_sizes.pop_front();
            }

            // Run anomaly detection (last expression in block returns the value)
            self.detect_anomalies(
                stats,
                task_id,
                input_size,
                files_created_val,
                &tool_calls_val,
                had_side_effects_val,
            )
            .await
        };

        // Store anomaly if detected
        if let Some(ref a) = anomaly {
            let mut anomalies = self.anomalies.write().await;
            anomalies.push(a.clone());

            // Keep only last 1000 anomalies
            if anomalies.len() > 1000 {
                anomalies.remove(0);
            }
        }

        anomaly
    }

    /// Detect anomalies for a skill's execution
    async fn detect_anomalies(
        &self,
        stats: &SkillStats,
        task_id: &str,
        input_size: usize,
        files_created: u32,
        tool_calls: &[String],
        had_side_effects: bool,
    ) -> Option<Anomaly> {
        // Skip if not enough data
        if stats.total_executions < self.config.min_executions_for_analysis as u64 {
            return None;
        }

        // Check 1: High frequency execution
        let recent_count = {
            let recent = self.recent_executions.read().await;
            recent
                .iter()
                .filter(|r| r.skill_name == stats.skill_name)
                .count() as u32
        };

        if recent_count > self.config.max_executions_per_minute {
            return Some(self.create_anomaly(
                AnomalyType::HighFrequency,
                AnomalySeverity::High,
                Some(&stats.skill_name),
                Some(task_id),
                format!(
                    "Skill {} executed {} times in last minute (limit: {})",
                    stats.skill_name, recent_count, self.config.max_executions_per_minute
                ),
            ));
        }

        // Check 2: Unusual duration (statistical outlier)
        if stats.recent_durations.len() >= 10 {
            let mean = stats.recent_durations.iter().sum::<u64>() as f64
                / stats.recent_durations.len() as f64;
            let variance = stats
                .recent_durations
                .iter()
                .map(|&d| (d as f64 - mean).powi(2))
                .sum::<f64>()
                / stats.recent_durations.len() as f64;
            let std_dev = variance.sqrt();

            if let Some(&last_duration) = stats.recent_durations.back() {
                if (last_duration as f64)
                    > mean + (std_dev * self.config.duration_std_dev_multiplier)
                {
                    return Some(self.create_anomaly(
                        AnomalyType::UnusualDuration,
                        AnomalySeverity::Medium,
                        Some(&stats.skill_name),
                        Some(task_id),
                        format!(
                            "Skill {} duration {}ms is {}σ above average ({:.0}ms)",
                            stats.skill_name,
                            last_duration,
                            (last_duration as f64 - mean) / std_dev,
                            mean
                        ),
                    ));
                }
            }
        }

        // Check 3: Input size anomaly
        if input_size > self.config.max_input_size_bytes {
            return Some(self.create_anomaly(
                AnomalyType::InputSizeAnomaly,
                AnomalySeverity::Medium,
                Some(&stats.skill_name),
                Some(task_id),
                format!(
                    "Input size {} bytes exceeds limit {} bytes",
                    input_size, self.config.max_input_size_bytes
                ),
            ));
        }

        // Check 4: Sequential failures
        if stats.recent_errors >= self.config.sequential_failure_threshold as u64 {
            let recent_total = stats.recent_durations.len() as u64;
            let error_rate = (stats.recent_errors as f64 / recent_total as f64) * 100.0;

            if error_rate > 50.0 {
                return Some(self.create_anomaly(
                    AnomalyType::SequentialFailures,
                    AnomalySeverity::Critical,
                    Some(&stats.skill_name),
                    Some(task_id),
                    format!(
                        "Skill {} has {} consecutive failures ({:.1}% error rate)",
                        stats.skill_name, stats.recent_errors, error_rate
                    ),
                ));
            }
        }

        // === Death Spiral Detection (OpenClaw-inspired) ===

        // Check 5: File Creation Burst - many files created in short time
        if files_created > self.config.file_burst_threshold {
            return Some(self.create_anomaly(
                AnomalyType::FileCreationBurst,
                AnomalySeverity::Critical,
                Some(&stats.skill_name),
                Some(task_id),
                format!(
                    "Skill {} created {} files in single execution (threshold: {})",
                    stats.skill_name, files_created, self.config.file_burst_threshold
                ),
            ));
        }

        // Check 6: Tool Call Loop - same tool called repeatedly
        if !tool_calls.is_empty() {
            // Find most frequent tool call
            let mut tool_counts: std::collections::HashMap<&String, u32> =
                std::collections::HashMap::new();
            for tool in tool_calls {
                *tool_counts.entry(tool).or_insert(0) += 1;
            }

            if let Some((_, &count)) = tool_counts.iter().max_by_key(|(_, c)| *c) {
                if count >= self.config.tool_loop_threshold {
                    return Some(self.create_anomaly(
                        AnomalyType::ToolCallLoop,
                        AnomalySeverity::High,
                        Some(&stats.skill_name),
                        Some(task_id),
                        format!(
                            "Skill {} called tool {} {} times (possible loop)",
                            stats.skill_name,
                            tool_counts.iter().max_by_key(|(_, c)| *c).unwrap().0,
                            count
                        ),
                    ));
                }
            }
        }

        // Check 7: No Side Effects - execution without observable state changes
        // Only check if we have enough history to compare
        if !had_side_effects && stats.total_executions >= 5 {
            let recent = self.recent_executions.read().await;
            let skill_recent: Vec<_> = recent
                .iter()
                .filter(|r| r.skill_name == stats.skill_name)
                .collect();

            if skill_recent.len() >= 5 {
                let no_effect_count = skill_recent
                    .iter()
                    .rev()
                    .take(self.config.no_side_effects_threshold as usize)
                    .filter(|r| !r.had_side_effects)
                    .count();

                if no_effect_count >= self.config.no_side_effects_threshold as usize {
                    return Some(self.create_anomaly(
                        AnomalyType::NoSideEffects,
                        AnomalySeverity::Medium,
                        Some(&stats.skill_name),
                        Some(task_id),
                        format!(
                            "Skill {} had no side effects in {} consecutive executions",
                            stats.skill_name, no_effect_count
                        ),
                    ));
                }
            }
        }

        // Check 8: Error Cascade - errors increasing over time
        let recent = self.recent_executions.read().await;
        let skill_recent: Vec<_> = recent
            .iter()
            .filter(|r| r.skill_name == stats.skill_name)
            .collect();

        if skill_recent.len() >= 3 {
            let mut error_sequence = Vec::new();
            for record in skill_recent
                .iter()
                .rev()
                .take(self.config.error_cascade_threshold as usize)
            {
                error_sequence.push(!record.success);
            }

            // Check if all recent executions failed
            if error_sequence.iter().all(|&e| e)
                && error_sequence.len() >= self.config.error_cascade_threshold as usize
            {
                return Some(self.create_anomaly(
                    AnomalyType::ErrorCascade,
                    AnomalySeverity::Critical,
                    Some(&stats.skill_name),
                    Some(task_id),
                    format!(
                        "Skill {} failed {} times in a row",
                        stats.skill_name,
                        error_sequence.len()
                    ),
                ));
            }
        }

        None
    }

    /// Create an anomaly record
    fn create_anomaly(
        &self,
        anomaly_type: AnomalyType,
        severity: AnomalySeverity,
        skill_name: Option<&str>,
        task_id: Option<&str>,
        description: String,
    ) -> Anomaly {
        let mut details = HashMap::new();
        if let Some(s) = skill_name {
            details.insert("skill_name".to_string(), s.to_string());
        }
        if let Some(t) = task_id {
            details.insert("task_id".to_string(), t.to_string());
        }

        Anomaly {
            id: ulid::Ulid::new().to_string(),
            anomaly_type,
            severity,
            skill_name: skill_name.map(String::from),
            task_id: task_id.map(String::from),
            description,
            details,
            detected_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Get all detected anomalies
    pub async fn get_anomalies(&self) -> Vec<Anomaly> {
        self.anomalies.read().await.clone()
    }

    /// Get anomalies filtered by severity
    pub async fn get_anomalies_by_severity(&self, severity: AnomalySeverity) -> Vec<Anomaly> {
        self.anomalies
            .read()
            .await
            .iter()
            .filter(|a| a.severity == severity)
            .cloned()
            .collect()
    }

    /// Get statistics for a specific skill
    pub async fn get_skill_stats(&self, skill_name: &str) -> Option<SkillStats> {
        self.skill_stats.read().await.get(skill_name).cloned()
    }

    /// Get all skill statistics
    pub async fn get_all_stats(&self) -> Vec<SkillStats> {
        self.skill_stats.read().await.values().cloned().collect()
    }

    /// Clear all recorded data (for testing)
    pub async fn clear(&self) {
        self.skill_stats.write().await.clear();
        self.recent_executions.write().await.clear();
        self.anomalies.write().await.clear();
    }

    /// Get detector health status
    pub async fn health_status(&self) -> AnomalyDetectorHealth {
        let stats_count = self.skill_stats.read().await.len();
        let anomalies_count = self.anomalies.read().await.len();
        let recent_count = self.recent_executions.read().await.len();

        AnomalyDetectorHealth {
            skills_tracked: stats_count,
            anomalies_detected: anomalies_count,
            recent_executions: recent_count,
            status: if anomalies_count > 100 {
                "degraded".to_string()
            } else {
                "healthy".to_string()
            },
        }
    }
}

/// Health status for the anomaly detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectorHealth {
    pub skills_tracked: usize,
    pub anomalies_detected: usize,
    pub recent_executions: usize,
    pub status: String,
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_high_frequency_detection() {
        let detector = AnomalyDetector::with_config(AnomalyConfig {
            max_executions_per_minute: 5,
            ..Default::default()
        });

        // Execute same skill 10 times rapidly
        for i in 0..10 {
            detector
                .record_execution(
                    "test.skill",
                    &format!("task-{}", i),
                    100,
                    true,
                    100,
                    None,
                    None,
                    None,
                )
                .await;
        }

        let anomalies = detector.get_anomalies().await;
        assert!(!anomalies.is_empty());
        assert!(anomalies
            .iter()
            .any(|a| a.anomaly_type == AnomalyType::HighFrequency));
    }

    #[tokio::test]
    async fn test_duration_anomaly() {
        let detector = AnomalyDetector::new();

        // Record normal executions
        for i in 0..20 {
            detector
                .record_execution(
                    "test.skill",
                    &format!("task-{}", i),
                    100,
                    true,
                    100,
                    None,
                    None,
                    None,
                )
                .await;
        }

        // Record one very long execution
        let anomaly = detector
            .record_execution(
                "test.skill",
                "task-long",
                10_000,
                true,
                100,
                None,
                None,
                None,
            )
            .await;

        assert!(anomaly.is_some());
        assert_eq!(anomaly.unwrap().anomaly_type, AnomalyType::UnusualDuration);
    }

    #[tokio::test]
    async fn test_sequential_failures() {
        let detector = AnomalyDetector::with_config(AnomalyConfig {
            sequential_failure_threshold: 3,
            min_executions_for_analysis: 1,
            ..Default::default()
        });

        // Record failures
        for i in 0..5 {
            detector
                .record_execution(
                    "test.skill",
                    &format!("task-{}", i),
                    100,
                    false,
                    100,
                    None,
                    None,
                    None,
                )
                .await;
        }

        let anomalies = detector.get_anomalies().await;
        assert!(anomalies
            .iter()
            .any(|a| a.anomaly_type == AnomalyType::SequentialFailures));
    }

    #[tokio::test]
    async fn test_input_size_anomaly() {
        let detector = AnomalyDetector::with_config(AnomalyConfig {
            max_input_size_bytes: 100,
            min_executions_for_analysis: 1,
            ..Default::default()
        });

        let anomaly = detector
            .record_execution(
                "test.skill",
                "task-1",
                100,
                true,
                10_000_000,
                None,
                None,
                None,
            )
            .await;

        assert!(anomaly.is_some());
        assert_eq!(anomaly.unwrap().anomaly_type, AnomalyType::InputSizeAnomaly);
    }

    #[tokio::test]
    async fn test_file_creation_burst() {
        let detector = AnomalyDetector::with_config(AnomalyConfig {
            file_burst_threshold: 5,
            min_executions_for_analysis: 1,
            ..Default::default()
        });

        // Create a burst of files
        let anomaly = detector
            .record_execution(
                "test.skill",
                "task-1",
                100,
                true,
                100,
                Some(10), // files_created > threshold
                None,
                Some(true),
            )
            .await;

        assert!(anomaly.is_some());
        assert_eq!(
            anomaly.unwrap().anomaly_type,
            AnomalyType::FileCreationBurst
        );
    }

    #[tokio::test]
    async fn test_tool_call_loop() {
        let detector = AnomalyDetector::with_config(AnomalyConfig {
            tool_loop_threshold: 3,
            min_executions_for_analysis: 1,
            ..Default::default()
        });

        // Same tool called 5 times
        let tool_calls = vec![
            "file.read".to_string(),
            "file.read".to_string(),
            "file.read".to_string(),
            "file.read".to_string(),
            "file.read".to_string(),
        ];

        let anomaly = detector
            .record_execution(
                "test.skill",
                "task-1",
                100,
                true,
                100,
                None,
                Some(tool_calls),
                Some(true),
            )
            .await;

        assert!(anomaly.is_some());
        assert_eq!(anomaly.unwrap().anomaly_type, AnomalyType::ToolCallLoop);
    }
}
