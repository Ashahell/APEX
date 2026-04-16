//! Health Check Module - Component Health Monitoring
//! 
//! v1.10.0: Health Check Dashboard
//! 
//! Provides health monitoring for all system components with
//! configurable check intervals and historical tracking.

pub mod models;
pub mod api;

pub use models::*;
pub use api::HealthState;
