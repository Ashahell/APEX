//! Phase 5 Memory Integration Tests
//!
//! Tests for TTL semantics, consolidation, and indexer surface.

use apex_router::api::memory_ttl_api::{ConsolidationCandidate, MemoryTTLConfig};

// ============================================================================
// TTL Configuration Tests
// ============================================================================

#[test]
fn ttl_config_default_values() {
    let config = MemoryTTLConfig::default();
    assert_eq!(config.memory_ttl_hours, 720); // 30 days
    assert_eq!(config.user_ttl_hours, 2160); // 90 days
    assert!(config.auto_cleanup_enabled);
    assert_eq!(config.cleanup_interval_hours, 24);
}

#[test]
fn ttl_config_custom_values() {
    let config = MemoryTTLConfig {
        memory_ttl_hours: 168, // 7 days
        user_ttl_hours: 720,   // 30 days
        auto_cleanup_enabled: false,
        cleanup_interval_hours: 12,
    };
    assert_eq!(config.memory_ttl_hours, 168);
    assert_eq!(config.user_ttl_hours, 720);
    assert!(!config.auto_cleanup_enabled);
    assert_eq!(config.cleanup_interval_hours, 12);
}

#[test]
fn ttl_config_serialization() {
    let config = MemoryTTLConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("memory_ttl_hours"));
    assert!(json.contains("user_ttl_hours"));
    assert!(json.contains("auto_cleanup_enabled"));
    assert!(json.contains("cleanup_interval_hours"));
}

#[test]
fn ttl_config_deserialization() {
    let json = r#"{
        "memory_ttl_hours": 168,
        "user_ttl_hours": 720,
        "auto_cleanup_enabled": false,
        "cleanup_interval_hours": 12
    }"#;
    let config: MemoryTTLConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.memory_ttl_hours, 168);
    assert_eq!(config.user_ttl_hours, 720);
    assert!(!config.auto_cleanup_enabled);
}

// ============================================================================
// Consolidation Tests
// ============================================================================

#[test]
fn consolidation_candidate_creation() {
    let candidate = ConsolidationCandidate {
        entries: vec!["entry1".to_string(), "entry2".to_string()],
        suggested_summary: "Combined summary".to_string(),
        char_savings: 100,
        confidence: 0.85,
    };
    assert_eq!(candidate.entries.len(), 2);
    assert_eq!(candidate.char_savings, 100);
    assert!((candidate.confidence - 0.85).abs() < 0.01);
}

#[test]
fn consolidation_candidate_serialization() {
    let candidate = ConsolidationCandidate {
        entries: vec!["entry1".to_string()],
        suggested_summary: "Summary".to_string(),
        char_savings: 50,
        confidence: 0.9,
    };
    let json = serde_json::to_string(&candidate).unwrap();
    assert!(json.contains("entries"));
    assert!(json.contains("suggested_summary"));
    assert!(json.contains("char_savings"));
    assert!(json.contains("confidence"));
}

#[test]
fn consolidation_candidate_deserialization() {
    let json = r#"{
        "entries": ["entry1", "entry2"],
        "suggested_summary": "Combined",
        "char_savings": 75,
        "confidence": 0.75
    }"#;
    let candidate: ConsolidationCandidate = serde_json::from_str(json).unwrap();
    assert_eq!(candidate.entries.len(), 2);
    assert_eq!(candidate.char_savings, 75);
    assert!((candidate.confidence - 0.75).abs() < 0.01);
}

#[test]
fn consolidation_empty_entries() {
    let candidate = ConsolidationCandidate {
        entries: vec![],
        suggested_summary: "Empty".to_string(),
        char_savings: 0,
        confidence: 0.0,
    };
    assert_eq!(candidate.entries.len(), 0);
    assert_eq!(candidate.char_savings, 0);
}

#[test]
fn consolidation_high_confidence() {
    let candidate = ConsolidationCandidate {
        entries: vec!["similar1".to_string(), "similar2".to_string()],
        suggested_summary: "Merged".to_string(),
        char_savings: 200,
        confidence: 0.95,
    };
    assert!(candidate.confidence > 0.9);
    assert!(candidate.char_savings > 100);
}
