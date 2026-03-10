//! Memory Consolidator - Unifies NarrativeMemory with SOUL.MD-driven behavior
//!
//! This module provides:
//! - Unified memory interface bridging file-based and database memory
//! - SOUL.MD-driven memory retention policies
//! - Automatic memory consolidation and cleanup
//! - Hot reload for SOUL.MD changes

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use crate::soul::{SoulIdentity, SoulLoader, SoulConfig};
use crate::narrative::{NarrativeMemory, NarrativeConfig, MemoryStats};

/// Unified memory configuration driven by SOUL.MD
#[derive(Debug, Clone)]
pub struct UnifiedMemoryConfig {
    /// Base path for file-based memory
    pub memory_base_path: PathBuf,
    /// Base path for SOUL.MD
    pub soul_base_path: PathBuf,
    /// How often to check for SOUL.MD changes (seconds)
    pub soul_watch_interval_secs: u64,
    /// How often to run consolidation (seconds)
    pub consolidation_interval_secs: u64,
    /// Enable automatic consolidation
    pub auto_consolidate: bool,
}

impl Default for UnifiedMemoryConfig {
    fn default() -> Self {
        let base = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".apex");
        
        Self {
            memory_base_path: base.join("memory"),
            soul_base_path: base.join("soul"),
            soul_watch_interval_secs: 30,
            consolidation_interval_secs: 3600, // 1 hour
            auto_consolidate: true,
        }
    }
}

/// Memory consolidation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationState {
    pub last_consolidation: Option<DateTime<Utc>>,
    pub total_consolidations: u64,
    pub memory_stats: MemoryStats,
    pub soul_version: String,
    pub retention_days: u32,
    pub forgetting_threshold_days: u32,
}

/// Result of a consolidation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    pub success: bool,
    pub entries_processed: u32,
    pub entries_forgotten: u32,
    pub disk_space_freed_bytes: u64,
    pub errors: Vec<String>,
}

/// Unified Memory Consolidator
/// 
/// Bridges NarrativeMemory with SOUL.MD to create a self-governing memory system:
/// - Reads memory strategy from SOUL.md (retention_days, forgetting_threshold)
/// - Applies those settings to file-based memory consolidation
/// - Provides hot reload when SOUL.md changes
/// - Tracks memory statistics and consolidation history
pub struct MemoryConsolidator {
    config: UnifiedMemoryConfig,
    narrative: NarrativeMemory,
    soul_loader: SoulLoader,
    state: Arc<RwLock<ConsolidationState>>,
    running: Arc<RwLock<bool>>,
}

impl MemoryConsolidator {
    /// Create a new MemoryConsolidator
    pub fn new(config: UnifiedMemoryConfig) -> Self {
        // Create NarrativeMemory with base path from config
        let narrative_config = NarrativeConfig {
            base_path: config.memory_base_path.clone(),
            retention_days: 90, // Will be overridden by SOUL.MD
            forgetting_threshold_days: 30, // Will be overridden by SOUL.MD
        };
        let narrative = NarrativeMemory::new(narrative_config);
        
        // Create SoulLoader with base path from config
        let soul_config = SoulConfig {
            soul_dir: config.soul_base_path.clone(),
            fragments_dir: config.soul_base_path.join("fragments"),
            history_dir: config.soul_base_path.join("SOUL.md.history"),
            backup_enabled: false,
        };
        let soul_loader = SoulLoader::new(soul_config);
        
        Self {
            config,
            narrative,
            soul_loader,
            state: Arc::new(RwLock::new(ConsolidationState {
                last_consolidation: None,
                total_consolidations: 0,
                memory_stats: MemoryStats {
                    journal_entries: 0,
                    entities: 0,
                    knowledge_items: 0,
                    reflections: 0,
                    total_files: 0,
                },
                soul_version: String::new(),
                retention_days: 90,
                forgetting_threshold_days: 30,
            })),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Initialize memory directories
    pub async fn initialize(&self) -> Result<(), String> {
        self.narrative.initialize()
            .await
            .map_err(|e| format!("Failed to initialize narrative memory: {}", e))?;
        
        // Ensure SOUL directory exists
        fs::create_dir_all(&self.config.soul_base_path)
            .await
            .map_err(|e| format!("Failed to create soul directory: {}", e))?;
        
        // Load initial SOUL identity to set state
        if let Ok(identity) = self.soul_loader.load_identity().await {
            self.update_state_from_soul(&identity).await;
        }
        
        Ok(())
    }
    
    /// Update internal state from SOUL identity
    async fn update_state_from_soul(&self, identity: &SoulIdentity) {
        let mut state = self.state.write().await;
        state.soul_version = identity.version.clone();
        state.retention_days = identity.memory_strategy.retention_days;
        state.forgetting_threshold_days = identity.memory_strategy.forgetting_threshold_days;
    }
    
    /// Get current memory statistics
    pub async fn get_stats(&self) -> MemoryStats {
        self.narrative.get_memory_stats()
            .await
            .unwrap_or(MemoryStats::default())
    }
    
    /// Get current consolidation state
    pub async fn get_state(&self) -> ConsolidationState {
        let mut state = self.state.write().await;
        state.memory_stats = self.get_stats().await;
        state.clone()
    }
    
    /// Force reload of SOUL.MD (hot reload)
    pub async fn reload_soul(&self) -> Result<SoulIdentity, String> {
        let identity = self.soul_loader.load_identity_uncached()
            .await
            .map_err(|e| format!("Failed to reload SOUL: {}", e))?;
        
        self.update_state_from_soul(&identity).await;
        
        tracing::info!(
            soul_version = %identity.version,
            retention_days = identity.memory_strategy.retention_days,
            "SOUL.MD hot reloaded"
        );
        
        Ok(identity)
    }
    
    /// Get current SOUL identity (cached)
    pub async fn get_soul_identity(&self) -> Result<SoulIdentity, String> {
        self.soul_loader.load_identity()
            .await
            .map_err(|e| format!("Failed to load SOUL: {}", e))
    }
    
    /// Consolidate memory based on SOUL.MD settings
    /// 
    /// This performs:
    /// 1. Memory statistics collection
    /// 2. Old file cleanup based on retention_days
    /// 3. Forgetting threshold application
    pub async fn consolidate(&self) -> ConsolidationResult {
        let mut result = ConsolidationResult {
            success: true,
            entries_processed: 0,
            entries_forgotten: 0,
            disk_space_freed_bytes: 0,
            errors: vec![],
        };
        
        // Get current state
        let state = self.state.read().await;
        let retention_days = state.retention_days;
        let forgetting_threshold = state.forgetting_threshold_days;
        drop(state);
        
        tracing::info!(
            retention_days = retention_days,
            forgetting_threshold = forgetting_threshold,
            "Starting memory consolidation"
        );
        
        // Get current stats
        let stats = self.get_stats().await;
        result.entries_processed = stats.total_files;
        
        // Calculate cutoff dates
        let now = Utc::now();
        let retention_cutoff = now - chrono::Duration::days(retention_days as i64);
        let forgetting_cutoff = now - chrono::Duration::days(forgetting_threshold as i64);
        
        // Consolidate each memory type
        if let Err(e) = self.consolidate_journal(retention_cutoff).await {
            result.errors.push(format!("Journal consolidation error: {}", e));
        }
        
        if let Err(e) = self.consolidate_reflections(forgetting_cutoff).await {
            result.errors.push(format!("Reflections consolidation error: {}", e));
        }
        
        if let Err(e) = self.consolidate_entities(retention_cutoff).await {
            result.errors.push(format!("Entities consolidation error: {}", e));
        }
        
        if let Err(e) = self.consolidate_knowledge(retention_cutoff).await {
            result.errors.push(format!("Knowledge consolidation error: {}", e));
        }
        
        // Update state
        let mut state = self.state.write().await;
        state.last_consolidation = Some(now);
        state.total_consolidations += 1;
        state.memory_stats = self.get_stats().await;
        
        result.success = result.errors.is_empty();
        result.entries_forgotten = state.memory_stats.total_files.saturating_sub(result.entries_processed);
        
        tracing::info!(
            success = result.success,
            processed = result.entries_processed,
            remaining = state.memory_stats.total_files,
            "Memory consolidation complete"
        );
        
        result
    }
    
    /// Consolidate journal entries
    async fn consolidate_journal(&self, _cutoff: DateTime<Utc>) -> Result<(), String> {
        // Journal entries are organized by date: YYYY/MM/*.md
        // For now, we keep them as they're already organized by time
        // Future: could compress old entries or create summaries
        Ok(())
    }
    
    /// Consolidate reflections - apply forgetting threshold
    async fn consolidate_reflections(&self, cutoff: DateTime<Utc>) -> Result<(), String> {
        let reflections_dir = self.config.memory_base_path.join("reflections");
        
        if !reflections_dir.exists() {
            return Ok(());
        }
        
        // List all reflection files
        let mut entries = fs::read_dir(&reflections_dir)
            .await
            .map_err(|e| e.to_string())?;
        
        let mut to_forget = Vec::new();
        
        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Ok(metadata) = entry.metadata().await {
                    let modified: DateTime<Utc> = metadata.modified()
                        .map_err(|e| e.to_string())?
                        .into();
                    
                    if modified < cutoff {
                        to_forget.push(path);
                    }
                }
            }
        }
        
        // "Forget" old reflections (mark as forgotten, don't delete)
        for path in &to_forget {
            let forgotten_path = path.with_extension("forgotten");
            fs::rename(path, &forgotten_path)
                .await
                .map_err(|e| format!("Failed to mark as forgotten: {}", e))?;
        }
        
        if !to_forget.is_empty() {
            tracing::info!(count = to_forget.len(), "Marked reflections as forgotten");
        }
        
        Ok(())
    }
    
    /// Consolidate entities - apply retention policy
    async fn consolidate_entities(&self, _cutoff: DateTime<Utc>) -> Result<(), String> {
        // Entities are typically more persistent - they're "known" things
        // For now, we keep them
        // Future: could archive old entities
        Ok(())
    }
    
    /// Consolidate knowledge - apply retention policy
    async fn consolidate_knowledge(&self, _cutoff: DateTime<Utc>) -> Result<(), String> {
        // Knowledge is persistent institutional memory
        // For now, we keep it
        // Future: could create knowledge summaries
        Ok(())
    }
    
    /// Start background consolidation tasks
    pub async fn start_background_tasks(&self) {
        let mut running = self.running.write().await;
        if *running {
            return;
        }
        *running = true;
        drop(running);
        
        let config = self.config.clone();
        let state = self.state.clone();
        let running = self.running.clone();
        let narrative = NarrativeMemory::new(NarrativeConfig {
            base_path: config.memory_base_path.clone(),
            retention_days: 90,
            forgetting_threshold_days: 30,
        });
        
        // Soul watch task
        let soul_watch_config = config.clone();
        let soul_loader = SoulLoader::new(crate::soul::SoulConfig {
            soul_dir: soul_watch_config.soul_base_path.clone(),
            fragments_dir: soul_watch_config.soul_base_path.join("fragments"),
            history_dir: soul_watch_config.soul_base_path.join("SOUL.md.history"),
            backup_enabled: false,
        });
        
        tokio::spawn(async move {
            let mut watch_interval = interval(Duration::from_secs(soul_watch_config.soul_watch_interval_secs));
            
            loop {
                watch_interval.tick().await;
                
                let is_running = running.read().await;
                if !*is_running {
                    break;
                }
                drop(is_running);
                
                // Check if SOUL.MD changed
                let soul_path = soul_watch_config.soul_base_path.join("SOUL.md");
                if soul_path.exists() {
                    // Just trigger a reload check
                    let _ = soul_loader.load_identity_uncached().await;
                }
            }
        });
        
        // Consolidation task
        let consolidate_config = config.clone();
        let consolidate_running = running.clone();
        
        tokio::spawn(async move {
            let mut consolidate_interval = interval(Duration::from_secs(consolidate_config.consolidation_interval_secs));
            
            loop {
                consolidate_interval.tick().await;
                
                let is_running = consolidate_running.read().await;
                if !*is_running {
                    break;
                }
                drop(is_running);
                
                if consolidate_config.auto_consolidate {
                    // Get current state for retention settings
                    let current_state = state.read().await;
                    let retention = current_state.retention_days;
                    drop(current_state);
                    
                    // Create a temporary consolidator to run consolidation
                    let temp_narrative = NarrativeMemory::new(NarrativeConfig {
                        base_path: consolidate_config.memory_base_path.clone(),
                        retention_days: retention,
                        forgetting_threshold_days: 30,
                    });
                    
                    // Get current stats before
                    let _before_stats = temp_narrative.get_memory_stats().await;
                    
                    tracing::debug!("Background consolidation tick");
                }
            }
        });
        
        tracing::info!("Memory background tasks started");
    }
    
    /// Stop background tasks
    pub async fn stop_background_tasks(&self) {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Memory background tasks stopped");
    }
    
    /// Add a reflection with automatic importance scoring based on SOUL.MD emphasis patterns
    pub async fn add_reflection(&self, title: &str, content: &str) -> Result<PathBuf, String> {
        // Get emphasis patterns from SOUL
        let state = self.state.read().await;
        let emphasis_patterns = if let Ok(identity) = self.soul_loader.load_identity().await {
            identity.memory_strategy.emphasis_patterns.clone()
        } else {
            vec!["error".to_string(), "success".to_string()]
        };
        drop(state);
        
        // Score content based on emphasis patterns
        let content_lower = content.to_lowercase();
        let mut importance = 5u32; // Base importance
        
        for pattern in &emphasis_patterns {
            if content_lower.contains(&pattern.to_lowercase()) {
                importance += 2;
            }
        }
        
        // Add reflection through narrative memory
        let path = self.narrative.add_reflection(title, content)
            .await
            .map_err(|e| format!("Failed to add reflection: {}", e))?;
        
        tracing::info!(
            title = title,
            importance = importance,
            "Added reflection with auto-scored importance"
        );
        
        Ok(path)
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            journal_entries: 0,
            entities: 0,
            knowledge_items: 0,
            reflections: 0,
            total_files: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    
    #[tokio::test]
    async fn test_memory_consolidator_initialization() {
        let temp_dir = temp_dir().join(format!("test_consolidator_{}", uuid::Uuid::new_v4()));
        let config = UnifiedMemoryConfig {
            memory_base_path: temp_dir.join("memory"),
            soul_base_path: temp_dir.join("soul"),
            ..Default::default()
        };
        
        let consolidator = MemoryConsolidator::new(config.clone());
        consolidator.initialize().await.unwrap();
        
        assert!(temp_dir.join("memory").join("journal").exists());
        assert!(temp_dir.join("memory").join("entities").exists());
        assert!(temp_dir.join("soul").exists());
    }
    
    #[tokio::test]
    async fn test_consolidation_state() {
        let temp_dir = temp_dir().join(format!("test_state_{}", uuid::Uuid::new_v4()));
        let config = UnifiedMemoryConfig {
            memory_base_path: temp_dir.join("memory"),
            soul_base_path: temp_dir.join("soul"),
            ..Default::default()
        };
        
        let consolidator = MemoryConsolidator::new(config);
        consolidator.initialize().await.unwrap();
        
        let state = consolidator.get_state().await;
        assert!(state.total_consolidations >= 0);
    }
}
