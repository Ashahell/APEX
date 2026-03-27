//! Security module for APEX skill validation and content integrity
//!
//! This module provides:
//! - ContentHash: SHA-256 based content verification with path normalization
//! - InjectionClassifier: Regex-based detection for prompt/command injection
//! - AnomalyDetector: Statistical anomaly detection for execution patterns
//! - EncryptedNarrative: AES-256-GCM encryption for sensitive narrative data
//! - Validators: MCP server and Cron/Scheduled task validation

pub mod anomaly_detector;
pub mod content_hash;
pub mod injection_classifier;
pub mod replay_protection;
pub mod validators; // Patch 11: In-memory replay detection for HMAC streaming

pub use anomaly_detector::{Anomaly, AnomalyDetector, AnomalySeverity, AnomalyType};
pub use content_hash::ContentHash;
pub use injection_classifier::{
    InjectionClassifier, InjectionDetectionResult, InjectionType, ThreatLevel,
};
pub use validators::{ValidationError, ValidationResult, ValidationSeverity};

// Re-export from apex-security
pub use apex_security::{
    is_sensitive_field, EncryptedNarrativeEntry, NarrativeEncryptionConfig, NarrativeKeyManager,
};
