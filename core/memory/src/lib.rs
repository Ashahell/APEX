#![allow(clippy::all)]
#![allow(dead_code)]

pub mod audit;
pub mod background_indexer;
pub mod channel_repo;
pub mod chunker;
pub mod db;
pub mod decision_journal;
pub mod embedder;
pub mod hybrid_search;
pub mod msg_repo;
pub mod narrative;
pub mod preferences;
pub mod skill_registry;
pub mod task_repo;
pub mod tasks;
pub mod vector_store;
pub mod working_memory;
pub mod workflow_repo;

pub use background_indexer::{BackgroundIndexer, IndexerConfig, IndexStats};
pub use channel_repo::{Channel, ChannelRepository};
pub use chunker::{chunk_text, ChunkerConfig};
pub use decision_journal::{CreateDecisionEntry, DecisionJournalEntry, DecisionJournalRepository};
pub use embedder::{Embedder, EmbeddingProvider, EmbedError};
pub use hybrid_search::{
    apply_temporal_score, frequency_boost, mmr_select, reciprocal_rank_fusion, rrf_score,
    temporal_decay,
};
pub use narrative::{NarrativeConfig, NarrativeEntry, NarrativeMemory, MemoryStats};
pub use skill_registry::{SkillRegistry, SkillRegistryEntry};
pub use vector_store::{SearchResult as VectorSearchResult, VectorEntry, VectorStore};
pub use working_memory::{CausalLink, Entity, WorkingMemory};
pub use workflow_repo::{CreateWorkflow, UpdateWorkflow, Workflow, WorkflowExecution, WorkflowRepository};

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
