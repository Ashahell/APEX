#![allow(clippy::all)]
#![allow(unused_imports)]
#![allow(dead_code)]

pub mod agent_loop;
pub mod api;
pub mod auth;
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
pub mod tool_validator;  // Feature 1: Tool Maker Validation
pub mod tool_sandbox;   // Feature 1: Tool Sandbox
pub mod persona;       // Feature 2: Persona Assembly
pub mod privacy_guard; // Feature 6: Privacy Toggle
pub mod context_scope; // Feature 3: Context Scope Isolation
pub mod continuity;    // Feature 4: Continuity Scheduler
pub mod skill_signer;  // Feature 5: Plugin Signing
pub mod story_engine;  // Feature 7: Story Engine
pub mod computer_use;  // Computer Use implementation (WIP)
pub mod computer_use_hands_api; // Hands MVP API for computer-use
pub mod computer_use_embedding_integration; // demo embedding wiring helper
pub mod computer_use_api; // Minimal Axum API for Computer Use MVP
pub mod streaming;  // Patch 11: SSE streaming endpoints for Hands and MCP
pub mod streaming_types; // Phase 1.3: Streaming types boundary

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
