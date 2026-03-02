pub mod audit;
pub mod db;
pub mod msg_repo;
pub mod preferences;
pub mod skill_registry;
pub mod task_repo;
pub mod tasks;
pub mod vector_store;

pub use skill_registry::{SkillRegistry, SkillRegistryEntry};

pub use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Audit error: {0}")]
    Audit(String),
}

pub type MemoryResult<T> = Result<T, MemoryError>;
