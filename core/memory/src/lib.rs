#![allow(clippy::all)]
#![allow(dead_code)]

pub mod audit;
pub mod background_indexer;
pub mod channel_repo;
pub mod channel_settings_repo;
pub mod chunker;
pub mod config_repo;
pub mod consolidated;
pub mod dashboard_repo;
pub mod db;
pub mod decision_journal;
pub mod embedder;
pub mod execution_pattern_repo;
pub mod hybrid_search;
pub mod msg_repo;
pub mod multimodal_repo;
pub mod narrative;
pub mod pdf_repo;
pub mod preferences;
pub mod provider_repo;
pub mod secrets_repo;
pub mod session_control_repo;
pub mod skill_registry;
pub mod slack_block_repo;
pub mod task_repo;
pub mod tasks;
pub mod ttl_cleanup;
pub mod workflow_repo;
pub mod working_memory;

pub use channel_settings_repo::{
    get_channel_display_name, get_channel_icon, ChannelSettings, ChannelSettingsRepository,
    ChannelTemplate, ChannelWebhook, SUPPORTED_CHANNEL_TYPES,
};
pub use execution_pattern_repo::{
    ExecutionPattern, ExecutionPatternRepository, PatternAlertTemplate,
};
pub use secrets_repo::{
    get_category_info, get_predefined_secret_ids, is_predefined_secret, SecretAccessLog, SecretRef,
    SecretRotationLog, SecretsRepository,
};
pub use slack_block_repo::{SlackBlockRepository, SlackBlockTemplate};

#[cfg(feature = "vector-search")]
pub mod sqlite_vec;

pub use audit::{AuditEntry, AuditRepository, CreateAuditEntry};
pub use channel_repo::{ChannelRepository, CreateChannel};
pub use config_repo::{ConfigEntry, ConfigRepository, McpServer, McpTool};
pub use consolidated::{
    ConsolidationResult, MemoryConsolidator, SoulMemoryConfig, UnifiedMemoryStats,
};
pub use dashboard_repo::{
    ChatBookmark, CommandPaletteHistory, DashboardChatHistory, DashboardExport, DashboardLayout,
    DashboardRepository, PinnedMessage, SessionMetadata,
};
pub use decision_journal::{CreateDecisionEntry, DecisionJournalEntry, DecisionJournalRepository};
pub use embedder::{EmbedError, Embedder, EmbeddingProvider};
pub use hybrid_search::{
    apply_temporal_score, frequency_boost, mmr_select, reciprocal_rank_fusion, rrf_score,
    temporal_decay,
};
pub use narrative::{MemoryStats, NarrativeConfig, NarrativeEntry, NarrativeMemory};
pub use preferences::PreferencesRepository;
pub use session_control_repo::{
    SessionAttachment, SessionCheckpoint, SessionControlRepository, SessionResumeHistory,
    SessionState, SessionYieldLog,
};
pub use skill_registry::{SkillRegistry, SkillRegistryEntry};
pub use task_repo::TaskRepository;
pub use tasks::{CreateTask, Task, TaskPriority, TaskStatus, TaskTier};
pub use ttl_cleanup::{CleanupReport, TtlCleanup};
pub use workflow_repo::{
    CreateWorkflow, UpdateWorkflow, Workflow, WorkflowExecution, WorkflowRepository,
};
pub use working_memory::{CausalLink, Entity, WorkingMemory};

pub use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("Decision entry not found: {0}")]
    DecisionNotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Audit error: {0}")]
    Audit(String),

    #[error("Narrative error: {0}")]
    Narrative(String),

    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Embedding error: {0}")]
    Embedding(#[from] EmbedError),

    #[error("Search error: {0}")]
    Search(String),
}

pub type MemoryResult<T> = Result<T, MemoryError>;
