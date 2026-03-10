//! Memory Consolidation - Unified memory system with automatic cleanup
//!
//! This module provides:
//! - Automatic memory consolidation and cleanup
//! - Memory retention policies based on configurable thresholds
//! - Emphasis pattern matching for intelligent memory management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use crate::narrative::{NarrativeMemory, NarrativeConfig};

/// Memory configuration for consolidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulMemoryConfig {
    pub retention_days: u32,
    pub forgetting_threshold_days: u32,
    pub emphasis_patterns: Vec<String>,
    pub auto_consolidate: bool,
    pub consolidate_interval_hours: u32,
}

impl Default for SoulMemoryConfig {
    fn default() -> Self {
        Self {
            retention_days: 90,
            forgetting_threshold_days: 30,
            emphasis_patterns: vec![
                "error".to_string(),
                "correction".to_string(),
                "success".to_string(),
            ],
            auto_consolidate: true,
            consolidate_interval_hours: 24,
        }
    }
}

/// Memory consolidation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    pub journal_entries_kept: u32,
    pub journal_entries_removed: u32,
    pub reflections_kept: u32,
    pub reflections_removed: u32,
    pub entities_kept: u32,
    pub entities_removed: u32,
    pub knowledge_kept: u32,
    pub knowledge_removed: u32,
    pub total_space_freed_bytes: u64,
    pub errors: Vec<String>,
}

/// Unified memory consolidation service
/// 
/// Provides automatic memory consolidation based on configurable retention policies.
/// Works with NarrativeMemory to manage file-based memory cleanup.
pub struct MemoryConsolidator {
    narrative: Arc<NarrativeMemory>,
    config: SoulMemoryConfig,
    base_path: std::path::PathBuf,
}

impl MemoryConsolidator {
    /// Create new consolidator
    pub fn new(narrative_config: NarrativeConfig, memory_config: SoulMemoryConfig) -> Self {
        Self {
            narrative: Arc::new(NarrativeMemory::new(narrative_config.clone())),
            config: memory_config,
            base_path: narrative_config.base_path,
        }
    }

    /// Get the narrative memory
    pub fn narrative(&self) -> Arc<NarrativeMemory> {
        self.narrative.clone()
    }

    /// Get current config
    pub fn config(&self) -> &SoulMemoryConfig {
        &self.config
    }

    /// Update config
    pub fn update_config(&mut self, config: SoulMemoryConfig) {
        self.config = config;
    }

    /// Consolidate memory based on retention settings
    pub async fn consolidate(&self) -> ConsolidationResult {
        let mut result = ConsolidationResult {
            journal_entries_kept: 0,
            journal_entries_removed: 0,
            reflections_kept: 0,
            reflections_removed: 0,
            entities_kept: 0,
            entities_removed: 0,
            knowledge_kept: 0,
            knowledge_removed: 0,
            total_space_freed_bytes: 0,
            errors: vec![],
        };

        let retention_threshold = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);

        // Consolidate journal
        if let Err(e) = self.consolidate_directory(
            &self.base_path.join("journal"),
            &retention_threshold,
            &mut result.journal_entries_kept,
            &mut result.journal_entries_removed,
            &mut result.total_space_freed_bytes,
            &mut result.errors,
        ).await {
            result.errors.push(format!("Journal consolidation error: {}", e));
        }

        // Consolidate reflections
        if let Err(e) = self.consolidate_directory(
            &self.base_path.join("reflections"),
            &retention_threshold,
            &mut result.reflections_kept,
            &mut result.reflections_removed,
            &mut result.total_space_freed_bytes,
            &mut result.errors,
        ).await {
            result.errors.push(format!("Reflections consolidation error: {}", e));
        }

        // Entities and knowledge - just count
        result.entities_kept = self.count_files(&self.base_path.join("entities")).await;
        result.knowledge_kept = self.count_files(&self.base_path.join("knowledge")).await;

        tracing::info!(
            removed = result.journal_entries_removed + result.reflections_removed,
            freed_bytes = result.total_space_freed_bytes,
            "Memory consolidation complete"
        );

        result
    }

    async fn consolidate_directory(
        &self,
        dir: &Path,
        threshold: &DateTime<Utc>,
        kept: &mut u32,
        removed: &mut u32,
        freed_bytes: &mut u64,
        errors: &mut Vec<String>,
    ) -> Result<(), std::io::Error> {
        if !dir.exists() {
            return Ok(());
        }

        let mut to_delete = Vec::new();
        let mut stack = vec![dir.to_path_buf()];

        while let Some(current_dir) = stack.pop() {
            let mut entries = fs::read_dir(&current_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|ext| ext == "md") {
                    if let Ok(metadata) = entry.metadata().await {
                        if let Ok(modified) = metadata.modified() {
                            let modified: DateTime<Utc> = modified.into();

                            if modified < *threshold {
                                to_delete.push((path.clone(), metadata.len()));
                            } else {
                                *kept += 1;
                            }
                        }
                    }
                }
            }
        }

        // Delete old files
        for (path, size) in to_delete {
            match fs::remove_file(&path).await {
                Ok(_) => {
                    *removed += 1;
                    *freed_bytes += size;
                }
                Err(e) => {
                    errors.push(format!("Failed to delete {}: {}", path.display(), e));
                }
            }
        }

        Ok(())
    }

    async fn count_files(&self, dir: &Path) -> u32 {
        let mut count = 0u32;
        
        if !dir.exists() {
            return 0;
        }

        let mut stack = vec![dir.to_path_buf()];

        while let Some(current_dir) = stack.pop() {
            if let Ok(mut entries) = fs::read_dir(&current_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if path.extension().is_some_and(|ext| ext == "md") {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Check if content matches emphasis patterns
    pub fn matches_emphasis(&self, content: &str) -> bool {
        let content_lower = content.to_lowercase();
        self.config.emphasis_patterns.iter().any(|pattern| {
            content_lower.contains(&pattern.to_lowercase())
        })
    }

    /// Score content based on emphasis patterns
    pub fn emphasis_score(&self, content: &str) -> f64 {
        let content_lower = content.to_lowercase();
        let mut score = 0.0;

        for pattern in &self.config.emphasis_patterns {
            if content_lower.contains(&pattern.to_lowercase()) {
                score += 1.0;
            }
        }

        (score / self.config.emphasis_patterns.len() as f64).min(1.0)
    }

    /// Initialize the consolidator
    pub async fn initialize(&self) -> Result<(), String> {
        self.narrative.initialize()
            .await
            .map_err(|e| format!("Failed to initialize narrative: {}", e))
    }
}

/// Memory stats with consolidation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMemoryStats {
    pub journal_entries: u32,
    pub entities: u32,
    pub knowledge_items: u32,
    pub reflections: u32,
    pub total_files: u32,
    pub retention_days: u32,
    pub forgetting_threshold_days: u32,
    pub emphasis_patterns: Vec<String>,
    pub emphasis_matched_entries: u32,
}

impl MemoryConsolidator {
    /// Get unified memory statistics
    pub async fn get_stats(&self) -> Result<UnifiedMemoryStats, std::io::Error> {
        let narrative_stats = self.narrative.get_memory_stats().await?;

        let emphasis_matched = self.count_emphasis_matches().await;

        Ok(UnifiedMemoryStats {
            journal_entries: narrative_stats.journal_entries,
            entities: narrative_stats.entities,
            knowledge_items: narrative_stats.knowledge_items,
            reflections: narrative_stats.reflections,
            total_files: narrative_stats.total_files,
            retention_days: self.config.retention_days,
            forgetting_threshold_days: self.config.forgetting_threshold_days,
            emphasis_patterns: self.config.emphasis_patterns.clone(),
            emphasis_matched_entries: emphasis_matched,
        })
    }

    async fn count_emphasis_matches(&self) -> u32 {
        let mut count = 0u32;
        let dirs = [
            self.base_path.join("journal"),
            self.base_path.join("reflections"),
        ];

        for dir in dirs {
            if !dir.exists() {
                continue;
            }

            let mut stack = vec![dir];
            while let Some(current) = stack.pop() {
                if let Ok(mut entries) = fs::read_dir(&current).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if path.is_dir() {
                            stack.push(path);
                        } else if path.extension().is_some_and(|ext| ext == "md") {
                            if let Ok(content) = fs::read_to_string(&path).await {
                                if self.matches_emphasis(&content) {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        count
    }
}

/// Start background consolidation task
pub async fn start_background_consolidation(
    consolidator: Arc<RwLock<MemoryConsolidator>>,
    interval_hours: u64,
) {
    let mut interval_timer = interval(Duration::from_secs(interval_hours * 3600));

    loop {
        interval_timer.tick().await;

        let consolidator = consolidator.read().await;
        let result = consolidator.consolidate().await;

        if result.errors.is_empty() {
            tracing::debug!(
                removed = result.journal_entries_removed + result.reflections_removed,
                "Background consolidation complete"
            );
        } else {
            tracing::warn!(
                errors = result.errors.len(),
                "Background consolidation had errors"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[tokio::test]
    async fn test_memory_consolidator_creation() {
        let temp_dir = temp_dir().join(format!("test_consolidator_{}", uuid::Uuid::new_v4()));
        let narrative_config = NarrativeConfig {
            base_path: temp_dir.clone(),
            retention_days: 90,
            forgetting_threshold_days: 30,
        };
        let memory_config = SoulMemoryConfig::default();

        let consolidator = MemoryConsolidator::new(narrative_config, memory_config);

        assert_eq!(consolidator.config().retention_days, 90);
        assert_eq!(consolidator.config().emphasis_patterns.len(), 3);
    }

    #[tokio::test]
    async fn test_emphasis_matching() {
        let temp_dir = temp_dir().join(format!("test_emphasis_{}", uuid::Uuid::new_v4()));
        let narrative_config = NarrativeConfig {
            base_path: temp_dir.clone(),
            ..Default::default()
        };
        let memory_config = SoulMemoryConfig::default();

        let consolidator = MemoryConsolidator::new(narrative_config, memory_config);

        // Default emphasis patterns
        assert!(consolidator.matches_emphasis("Found an error in the system"));
        assert!(consolidator.matches_emphasis("This was a success"));
        assert!(!consolidator.matches_emphasis("Just a normal message"));

        assert!(consolidator.emphasis_score("error found") > 0.0);
    }
}
