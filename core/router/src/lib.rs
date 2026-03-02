pub mod agent_loop;
pub mod api;
pub mod circuit_breaker;
pub mod classifier;
pub mod deep_task_worker;
pub mod llama;
pub mod message_bus;
pub mod metrics;
pub mod skill_worker;
pub mod vm_pool;

pub use apex_memory;
pub use apex_security;

pub use thiserror::Error;

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("Task error: {0}")]
    Task(String),

    #[error("Classification error: {0}")]
    Classification(String),

    #[error("Capability error: {0}")]
    Capability(#[from] apex_security::SecurityError),

    #[error("Memory error: {0}")]
    Memory(#[from] apex_memory::MemoryError),

    #[error("Queue full")]
    QueueFull,

    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),

    #[error("VM error: {0}")]
    Vm(String),
}

pub type RouterResult<T> = Result<T, RouterError>;
