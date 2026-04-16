//! Vigilant Mode - Alert Monitoring System
//! 
//! Inspired by Hermes Agent v0.9.0 Vigilant Mode.
//! Provides proactive monitoring with configurable alert rules and actions.

pub mod models;
pub mod rules;
pub mod monitor;
pub mod dispatcher;

// Re-export types for easier access
pub use models::*;
pub use rules::{AlertRuleEngine, SharedAlertRuleEngine, AlertRuleUpdate, DEFAULT_ALERT_RULES};
pub use monitor::{ThresholdMonitor, SharedThresholdMonitor, ThresholdConfig, TaskMetrics};
pub use dispatcher::{AlertDispatcher, SharedAlertDispatcher, DispatcherStats};
