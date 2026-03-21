#![allow(clippy::all)]
#![allow(unused_imports)]
#![allow(dead_code)]

pub mod agent_loop;
pub mod api;
pub mod circuit_breaker;
pub mod classifier;
pub mod deep_task_worker;
pub mod dynamic_tools;
pub mod execution_stream;
pub mod governance;
pub mod heartbeat;
pub mod hub_client;
pub mod llama;
pub mod memory_stores;
pub mod message_bus;
pub mod metrics;
pub mod moltbook;
pub mod mcp;
pub mod narrative;
pub mod notification;
pub mod rate_limiter;
pub mod response_cache;
pub mod security;
pub mod secret_store;
pub mod skill_hot_reload;
pub mod skill_manager;
pub mod session_search;
pub mod skill_pool;
pub mod skill_pool_ipc;
pub mod skill_worker;
pub mod soul;
pub mod subagent;
pub mod enhanced_rate_limiter;
pub mod system_component;
pub mod component_registry;
pub mod system_health;
pub mod totp;
pub mod unified_config;
pub mod user_profile;
pub mod vm_pool;
pub mod webhook;
pub mod websocket;

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
