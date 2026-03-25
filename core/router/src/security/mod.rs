//! Security module for APEX skill validation and content integrity
//!
//! This module provides:
//! - ContentHash: SHA-256 based content verification with path normalization
//! - InjectionClassifier: Regex-based detection for prompt/command injection
//! - AnomalyDetector: Statistical anomaly detection for execution patterns
//! - EncryptedNarrative: AES-256-GCM encryption for sensitive narrative data
//! - Validators: MCP server and Cron/Scheduled task validation

pub mod content_hash;
pub mod injection_classifier;
pub mod anomaly_detector;
pub mod validators;
pub mod replay_protection;  // Patch 11: In-memory replay detection for HMAC streaming

pub use content_hash::ContentHash;
pub use injection_classifier::{InjectionClassifier, InjectionDetectionResult, InjectionType, ThreatLevel};
pub use anomaly_detector::{AnomalyDetector, Anomaly, AnomalySeverity, AnomalyType};
pub use validators::{ValidationResult, ValidationError, ValidationSeverity};

// Re-export from apex-security
pub use apex_security::{EncryptedNarrativeEntry, NarrativeKeyManager, NarrativeEncryptionConfig, is_sensitive_field};
