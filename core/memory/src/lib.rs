#![allow(clippy::all)]
#![allow(dead_code)]

pub mod audit;
pub mod channel_repo;
pub mod db;
pub mod decision_journal;
pub mod msg_repo;
pub mod narrative;
pub mod preferences;
pub mod skill_registry;
pub mod task_repo;
pub mod tasks;
pub mod vector_store;
pub mod workflow_repo;

pub use channel_repo::{Channel, ChannelRepository};
pub use decision_journal::{DecisionJournalEntry, DecisionJournalRepository, CreateDecisionEntry};
pub use narrative::{NarrativeMemory, NarrativeConfig, NarrativeEntry, MemoryStats};
pub use skill_registry::{SkillRegistry, SkillRegistryEntry};
pub use workflow_repo::{Workflow, WorkflowExecution, WorkflowRepository, CreateWorkflow, UpdateWorkflow};

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
}

pub type MemoryResult<T> = Result<T, MemoryError>;
