//! Monitoring module for Background Process Monitoring
//! 
//! Inspired by Hermes Agent v0.9.0 background process monitoring.
//! Provides watch patterns, hook system, and notification dispatch.

pub mod models;
pub mod watcher;
pub mod hooks;
pub mod notifier;

// Re-export types for easier access
pub use models::*;
pub use watcher::{WatcherRegistry, SharedWatcherRegistry, WatchPatternUpdate};
pub use hooks::{HookEmitter, SharedHookEmitter};
pub use notifier::{NotificationDispatcher, SharedNotificationDispatcher, NotificationChannel, NotificationStats, DispatchResult};
