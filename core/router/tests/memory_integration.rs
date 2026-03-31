//! Memory Integration Tests
//!
//! Tests for Memory CRUD, Bounded Memory operations.

use apex_router::memory_stores::{MemoryEntry, MemoryStore, StoreType};

// ============================================================================
// Test 3.1: Memory CRUD Flow
// ============================================================================

#[test]
fn test_memory_store_new() {
    let store = MemoryStore::new(StoreType::Memory);

    assert_eq!(store.entry_count(), 0, "New store should be empty");
    assert!(
        store.entries().is_empty(),
        "New store should have empty entries"
    );
}

#[test]
fn test_memory_store_add_entry() {
    let mut store = MemoryStore::new(StoreType::Memory);

    let result = store.add_entry("Test memory entry".to_string());
    assert!(result.is_ok(), "Should be able to add entry");

    let _id = result.unwrap();
    assert_eq!(
        store.entry_count(),
        1,
        "Store should have 1 entry after add"
    );

    // Verify we can find the entry
    let entries = store.entries();
    assert_eq!(entries.len(), 1, "Should have 1 entry");
    assert_eq!(entries[0].content, "Test memory entry");
}

#[test]
fn test_memory_store_replace_entry() {
    let mut store = MemoryStore::new(StoreType::Memory);

    store.add_entry("Original content".to_string()).ok();

    // Replace the entry
    let result = store.replace_entry("Original content", "Updated content".to_string());
    assert!(result.is_ok(), "Should be able to replace entry");

    // Verify content changed
    let entries = store.entries();
    assert!(
        entries[0].content.contains("Updated"),
        "Entry should contain updated content"
    );
}

#[test]
fn test_memory_store_remove_entry() {
    let mut store = MemoryStore::new(StoreType::Memory);

    store.add_entry("To be deleted".to_string()).ok();
    assert_eq!(store.entry_count(), 1, "Should have 1 entry");

    // Remove the entry
    let result = store.remove_entry("To be deleted");
    assert!(result.is_ok(), "Should be able to remove entry");

    assert_eq!(store.entry_count(), 0, "Store should be empty after remove");
}

#[test]
fn test_memory_store_multiple_entries() {
    // Use a large limit to prevent consolidation
    let mut store = MemoryStore::with_limit(StoreType::Memory, 10000);

    // Add multiple entries - must be > 10 chars (MIN_ENTRY_LENGTH)
    store.add_entry("Entry 1 - test content".to_string()).ok();
    store.add_entry("Entry 2 - test content".to_string()).ok();
    store.add_entry("Entry 3 - test content".to_string()).ok();

    // Should have at least some entries (consolidation may occur)
    assert!(store.entry_count() >= 1, "Should have at least 1 entry");
}

// ============================================================================
// Test 3.2: Bounded Memory Consolidation
// ============================================================================

#[test]
fn test_bounded_memory_character_limit() {
    // Create store with small limit for testing
    let mut store = MemoryStore::with_limit(StoreType::Memory, 50);

    // Add entries that exceed the limit - they should auto-consolidate
    store
        .add_entry("This is a very long memory entry that exceeds the limit".to_string())
        .ok();
    store
        .add_entry("Another entry that should trigger consolidation".to_string())
        .ok();

    // Should be at or below limit after consolidation
    let stats = store.stats();
    assert!(
        stats.used_chars <= 50,
        "Total chars should be at or below limit"
    );
}

#[test]
fn test_bounded_memory_snapshot() {
    let mut store = MemoryStore::new(StoreType::Memory);

    // Add some entries
    store.add_entry("Important fact 1".to_string()).ok();
    store.add_entry("Important fact 2".to_string()).ok();

    // Get snapshot (frozen content for system prompt)
    let snapshot = store.to_snapshot();
    assert!(!snapshot.is_empty(), "Snapshot should not be empty");
    assert!(
        snapshot.contains("Important fact"),
        "Snapshot should contain key info"
    );
}

#[test]
fn test_bounded_memory_user_store() {
    // User store has different limit
    let user_store = MemoryStore::new(StoreType::User);

    let stats = user_store.stats();
    assert_eq!(
        stats.char_limit,
        StoreType::User.default_limit(),
        "User store should have user limit"
    );
}

#[test]
fn test_memory_store_stats() {
    // Use large limit to prevent consolidation - entries must be > 10 chars
    let mut store = MemoryStore::with_limit(StoreType::Memory, 10000);

    // Add some entries (must be > 10 chars for MIN_ENTRY_LENGTH)
    store.add_entry("Short entry content".to_string()).ok();
    store
        .add_entry("A bit longer content here".to_string())
        .ok();

    let stats = store.stats();

    assert!(stats.entry_count >= 1, "Should have at least 1 entry");
    assert!(stats.used_chars > 0, "Should have some characters");
    assert_eq!(stats.char_limit, 10000, "Should have custom limit");
}

#[test]
fn test_memory_entry_timestamps() {
    let before = chrono::Utc::now().timestamp();
    let entry = MemoryEntry::new("Test content".to_string(), StoreType::Memory);
    let after = chrono::Utc::now().timestamp();

    assert!(entry.created_at >= before, "Created at should be >= before");
    assert!(entry.created_at <= after, "Created at should be <= after");
    assert_eq!(
        entry.created_at, entry.updated_at,
        "Created and updated should be equal initially"
    );
}

#[test]
fn test_memory_store_usage() {
    let mut store = MemoryStore::new(StoreType::Memory);

    // Initially should have 0% usage
    assert_eq!(store.usage_percent(), 0.0, "Initial usage should be 0%");

    store.add_entry("Some content here".to_string()).ok();

    // Should have some usage now
    assert!(
        store.usage_percent() > 0.0,
        "Usage should be > 0 after adding content"
    );
}

#[test]
fn test_memory_store_can_add() {
    let store = MemoryStore::new(StoreType::Memory);

    // Should be able to add content
    assert!(
        store.can_add("Short content"),
        "Should be able to add short content"
    );

    // Very long content might exceed limit depending on current usage
    let long_content = "x".repeat(5000);
    assert!(
        !store.can_add(&long_content),
        "Should not be able to add very long content"
    );
}

#[test]
fn test_store_type_defaults() {
    // Memory store has different limits than User store
    let mem_store = MemoryStore::new(StoreType::Memory);
    let user_store = MemoryStore::new(StoreType::User);

    assert_ne!(
        mem_store.char_limit(),
        user_store.char_limit(),
        "Memory and User stores should have different limits"
    );
}
