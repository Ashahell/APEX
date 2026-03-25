//! Skills Integration Tests
//!
//! Tests for Skills registration and management.

use apex_router::skill_manager::{SkillManager, SkillCreateRequest, SkillPatchRequest};

// ============================================================================
// Test 6.1: Skill Registration Flow
// ============================================================================

#[tokio::test]
async fn test_skill_manager_new() {
    let temp_dir = std::env::temp_dir().join(format!("apex_skill_test_{}", ulid::Ulid::new()));
    std::fs::create_dir_all(&temp_dir).ok();
    
    let manager = SkillManager::new(temp_dir);
    
    // List skills should work
    let result = manager.list_skills();
    assert!(result.is_ok(), "Should be able to list skills");
}

#[tokio::test]
async fn test_skill_manager_create_skill() {
    let temp_dir = std::env::temp_dir().join(format!("apex_skill_test_{}", ulid::Ulid::new()));
    std::fs::create_dir_all(&temp_dir).ok();
    
    let manager = SkillManager::new(temp_dir.clone());
    
    let request = SkillCreateRequest {
        name: "test-skill".to_string(),
        content: "# Test Skill\n\nDescription here.".to_string(),
        category: Some("testing".to_string()),
        description: Some("A test skill".to_string()),
        source_task_id: None,
    };
    
    let result = manager.create_skill(request).await;
    assert!(result.is_ok(), "Should be able to create skill");
    
    // Verify skill exists
    assert!(manager.skill_exists("test-skill"), "Skill should exist after creation");
}

#[tokio::test]
async fn test_skill_manager_get_content() {
    let temp_dir = std::env::temp_dir().join(format!("apex_skill_test_{}", ulid::Ulid::new()));
    std::fs::create_dir_all(&temp_dir).ok();
    
    let manager = SkillManager::new(temp_dir.clone());
    
    // Create a skill first
    let request = SkillCreateRequest {
        name: "content-test".to_string(),
        content: "# Test Content\n\nSome content here.".to_string(),
        category: None,
        description: Some("Test skill".to_string()),
        source_task_id: None,
    };
    
    manager.create_skill(request).await.ok();
    
    // Get the content
    let content = manager.get_skill_content("content-test").await;
    assert!(content.is_ok(), "Should be able to get skill content");
    assert!(content.unwrap().contains("Test Content"), "Content should match");
}

#[tokio::test]
async fn test_skill_manager_delete_skill() {
    let temp_dir = std::env::temp_dir().join(format!("apex_skill_test_{}", ulid::Ulid::new()));
    std::fs::create_dir_all(&temp_dir).ok();
    
    let manager = SkillManager::new(temp_dir.clone());
    
    // Create a skill
    let request = SkillCreateRequest {
        name: "to-delete".to_string(),
        content: "# To Delete\n\nThis will be deleted.".to_string(),
        category: None,
        description: Some("Will be deleted".to_string()),
        source_task_id: None,
    };
    
    manager.create_skill(request).await.ok();
    assert!(manager.skill_exists("to-delete"), "Skill should exist before delete");
    
    // Delete the skill
    let result = manager.delete_skill("to-delete").await;
    assert!(result.is_ok(), "Should be able to delete skill");
    
    // Verify it's gone
    assert!(!manager.skill_exists("to-delete"), "Skill should not exist after delete");
}

#[tokio::test]
async fn test_skill_manager_patch_skill() {
    let temp_dir = std::env::temp_dir().join(format!("apex_skill_test_{}", ulid::Ulid::new()));
    std::fs::create_dir_all(&temp_dir).ok();
    
    let manager = SkillManager::new(temp_dir.clone());
    
    // Create a skill
    let request = SkillCreateRequest {
        name: "to-patch".to_string(),
        content: "# Original Content\n\nSome text.".to_string(),
        category: None,
        description: Some("Original description".to_string()),
        source_task_id: None,
    };
    
    manager.create_skill(request).await.ok();
    
    // Patch the skill - replace "Original" with "Updated"
    let patch = SkillPatchRequest {
        old_string: "Original".to_string(),
        new_string: "Updated".to_string(),
    };
    
    let result = manager.patch_skill("to-patch", patch).await;
    assert!(result.is_ok(), "Should be able to patch skill");
    
    // Verify content was updated
    let content = manager.get_skill_content("to-patch").await.unwrap();
    assert!(content.contains("Updated"), "Content should be updated");
}

// ============================================================================
// Additional Skills Tests
// ============================================================================

#[tokio::test]
async fn test_skill_manager_list_empty() {
    let temp_dir = std::env::temp_dir().join(format!("apex_skill_test_{}", ulid::Ulid::new()));
    std::fs::create_dir_all(&temp_dir).ok();
    
    let manager = SkillManager::new(temp_dir);
    
    let skills = manager.list_skills().unwrap();
    assert!(skills.is_empty(), "New manager should have no skills");
}

#[tokio::test]
async fn test_skill_manager_auto_created_dir() {
    let temp_dir = std::env::temp_dir().join(format!("apex_skill_test_{}", ulid::Ulid::new()));
    std::fs::create_dir_all(&temp_dir).ok();
    
    let manager = SkillManager::new(temp_dir.clone());
    
    let auto_dir = manager.auto_created_dir();
    assert!(auto_dir.starts_with(&temp_dir), "Auto-created dir should be under base dir");
}

#[tokio::test]
async fn test_skill_manager_find_similar() {
    let temp_dir = std::env::temp_dir().join(format!("apex_skill_test_{}", ulid::Ulid::new()));
    std::fs::create_dir_all(&temp_dir).ok();
    
    let manager = SkillManager::new(temp_dir.clone());
    
    // Create a skill with searchable content
    let request = SkillCreateRequest {
        name: "python-code".to_string(),
        content: "# Python Code Generator\n\nCreates Python functions.".to_string(),
        category: Some("coding".to_string()),
        description: Some("Generates Python code".to_string()),
        source_task_id: None,
    };
    
    manager.create_skill(request).await.ok();
    
    // Find similar
    let results = manager.find_similar("python");
    assert!(results.is_ok(), "Should be able to find similar skills");
    assert!(!results.unwrap().is_empty(), "Should find similar skill");
}
