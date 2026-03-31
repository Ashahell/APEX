//! Skill Manager (Hermes-style Agent-Managed Skills)
//!
//! Enables the agent to create, update, and delete skills from experience.
//!
//! Features:
//! - Agent creates skills after complex tasks (5+ tool calls)
//! - Skills saved to ~/.apex/skills/auto-created/
//! - SKILL.md format compatible with Hermes
//! - Security scanning before creation

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use ulid::Ulid;

use crate::unified_config::skill_constants::*;

/// Skill metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    pub platforms: Vec<String>,
    pub created_at: i64,
    pub trigger_conditions: Vec<String>,
    pub auto_created: bool,
    pub source_task_id: Option<String>,
}

/// Request to create a new skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCreateRequest {
    /// Skill name (slug format: my-skill-name)
    pub name: String,
    /// Full SKILL.md content
    pub content: String,
    /// Optional category (defaults to "auto-created")
    pub category: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// Optional task ID that triggered creation
    pub source_task_id: Option<String>,
}

/// Request to patch an existing skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPatchRequest {
    /// Unique substring to match
    pub old_string: String,
    /// Replacement content
    pub new_string: String,
}

/// Skill manager for auto-created skills
pub struct SkillManager {
    skills_dir: PathBuf,
}

impl SkillManager {
    /// Create a new skill manager
    pub fn new(skills_dir: PathBuf) -> Self {
        Self { skills_dir }
    }

    /// Get the auto-created skills directory
    pub fn auto_created_dir(&self) -> PathBuf {
        self.skills_dir.join(AUTO_SKILLS_DIR)
    }

    /// Get path to a skill's SKILL.md
    fn skill_path(&self, name: &str) -> PathBuf {
        self.auto_created_dir().join(name).join("SKILL.md")
    }

    /// Get path to a skill's references directory
    fn references_dir(&self, name: &str) -> PathBuf {
        self.auto_created_dir()
            .join(name)
            .join(SKILL_REFERENCES_DIR)
    }

    /// List all auto-created skills
    pub fn list_skills(&self) -> Result<Vec<SkillMetadata>, SkillError> {
        let dir = self.auto_created_dir();

        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();

        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let skill_md_path = path.join("SKILL.md");
            if !skill_md_path.exists() {
                continue;
            }

            match self.parse_skill_metadata(&skill_md_path) {
                Ok(metadata) => skills.push(metadata),
                Err(e) => {
                    tracing::warn!(skill_dir = %path.display(), error = %e, "Failed to parse skill metadata");
                }
            }
        }

        Ok(skills)
    }

    /// Parse skill metadata from SKILL.md frontmatter
    fn parse_skill_metadata(&self, path: &Path) -> Result<SkillMetadata, SkillError> {
        let content = std::fs::read_to_string(path)?;

        // Parse YAML frontmatter
        let frontmatter = content
            .lines()
            .take_while(|line| !line.trim().is_empty() || line.starts_with("---"))
            .collect::<String>();

        // Simple frontmatter parsing (YAML would be better but we keep it simple)
        let name = Self::extract_frontmatter_value(&frontmatter, "name")
            .or_else(|| {
                path.parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        let description =
            Self::extract_frontmatter_value(&frontmatter, "description").unwrap_or_default();

        let version = Self::extract_frontmatter_value(&frontmatter, "version")
            .unwrap_or_else(|| SKILL_VERSION.to_string());

        let category = Self::extract_frontmatter_value(&frontmatter, "category")
            .unwrap_or_else(|| AUTO_SKILL_CATEGORY.to_string());

        let platforms = match Self::extract_frontmatter_value(&frontmatter, "platforms") {
            Some(s) => Self::parse_platforms(&s),
            None => vec![
                "linux".to_string(),
                "macos".to_string(),
                "windows".to_string(),
            ],
        };

        // Parse tags from metadata section
        let _tags = Self::extract_frontmatter_section(&frontmatter, "tags").unwrap_or_default();

        Ok(SkillMetadata {
            name,
            description,
            version,
            category,
            platforms,
            created_at: 0, // Would need file system time
            trigger_conditions: Vec::new(),
            auto_created: true,
            source_task_id: None,
        })
    }

    /// Extract value from YAML frontmatter
    fn extract_frontmatter_value(frontmatter: &str, key: &str) -> Option<String> {
        let pattern = format!("{}:", key);
        frontmatter
            .lines()
            .find(|line| line.trim().starts_with(&pattern))
            .and_then(|line| {
                let value = line.split(':').nth(1)?.trim();
                Some(value.trim_matches(|c| c == '"' || c == '\'').to_string())
            })
    }

    /// Extract section from YAML frontmatter
    fn extract_frontmatter_section(frontmatter: &str, key: &str) -> Option<String> {
        let lines: Vec<&str> = frontmatter.lines().collect();
        let start = lines
            .iter()
            .position(|l| l.contains(&format!("{}:", key)))?;
        let end = lines[start..]
            .iter()
            .position(|l| l.trim().starts_with('-') && !l.contains(':'))?
            + start;

        Some(lines[start..=end].join("\n"))
    }

    /// Create a new skill
    pub async fn create_skill(&self, req: SkillCreateRequest) -> Result<SkillMetadata, SkillError> {
        // Validate name
        if !Self::is_valid_skill_name(&req.name) {
            return Err(SkillError::InvalidName {
                name: req.name,
                reason: "Name must be lowercase alphanumeric with hyphens".to_string(),
            });
        }

        // Check if skill already exists
        let skill_path = self.skill_path(&req.name);
        if skill_path.exists() {
            return Err(SkillError::AlreadyExists { name: req.name });
        }

        // Security scan
        if Self::contains_dangerous_patterns(&req.content) {
            return Err(SkillError::SecurityBlocked);
        }

        // Create directory structure
        let skill_dir = self.auto_created_dir().join(&req.name);
        let references_dir = skill_dir.join(SKILL_REFERENCES_DIR);
        tokio::fs::create_dir_all(&references_dir).await?;

        // Generate SKILL.md content
        let skill_md = self.generate_skill_md(&req)?;

        // Write SKILL.md
        tokio::fs::write(&skill_path, &skill_md).await?;

        tracing::info!(
            skill_name = %req.name,
            category = req.category.as_deref().unwrap_or(AUTO_SKILL_CATEGORY),
            "Agent created new skill"
        );

        Ok(SkillMetadata {
            name: req.name,
            description: req.description.unwrap_or_default(),
            version: SKILL_VERSION.to_string(),
            category: req
                .category
                .unwrap_or_else(|| AUTO_SKILL_CATEGORY.to_string()),
            platforms: vec![
                "linux".to_string(),
                "macos".to_string(),
                "windows".to_string(),
            ],
            created_at: chrono::Utc::now().timestamp(),
            trigger_conditions: Vec::new(),
            auto_created: true,
            source_task_id: req.source_task_id,
        })
    }

    /// Generate SKILL.md content
    fn generate_skill_md(&self, req: &SkillCreateRequest) -> Result<String, SkillError> {
        let description = req
            .description
            .as_deref()
            .unwrap_or("Auto-created skill from task execution");
        let category = req.category.as_deref().unwrap_or(AUTO_SKILL_CATEGORY);
        let source_task_id_line = req
            .source_task_id
            .as_ref()
            .map(|id| format!("    source_task_id: {}", id))
            .unwrap_or_default();

        let content = format!(
            r#"---
name: {name}
description: {description}
version: {version}
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [auto-created]
    category: {category}
    auto_created: true
{source_task_id_line}
---

# {name}

## When to Use
Automatically created from successful task execution.

## Procedure
{content}

## Pitfalls
- None documented yet

## Verification
Task completed successfully.
"#,
            name = req.name,
            description = description,
            version = SKILL_VERSION,
            category = category,
            source_task_id_line = source_task_id_line,
            content = req.content
        );

        Ok(content)
    }

    /// Validate skill name (slug format: lowercase, hyphens, underscores)
    fn is_valid_skill_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 50
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_numeric() || c == '-' || c == '_')
            && !name.starts_with('-')
            && !name.starts_with('_')
            && !name.ends_with('-')
            && !name.ends_with('_')
    }

    /// Parse platforms from YAML-style string
    fn parse_platforms(s: &str) -> Vec<String> {
        s.split(|c: char| c == ',' || c == ' ' || c == '[' || c == ']')
            .filter(|p| {
                let p = p.trim().trim_matches('"');
                !p.is_empty() && p != "platforms"
            })
            .map(|p| p.trim().trim_matches('"').to_string())
            .collect()
    }

    /// Check for dangerous patterns in skill content
    fn contains_dangerous_patterns(content: &str) -> bool {
        let dangerous = [
            "curl | sh",
            "rm -rf /",
            "eval ",
            "base64 -d",
            "wget .* | sh",
            "sudo ",
            "chmod 777",
            "curl -s | bash",
        ];

        let content_lower = content.to_lowercase();
        dangerous.iter().any(|p| content_lower.contains(p))
    }

    /// Patch an existing skill
    pub async fn patch_skill(
        &self,
        name: &str,
        patch: SkillPatchRequest,
    ) -> Result<(), SkillError> {
        let skill_path = self.skill_path(name);

        if !skill_path.exists() {
            return Err(SkillError::NotFound {
                name: name.to_string(),
            });
        }

        let content = tokio::fs::read_to_string(&skill_path).await?;

        if !content.contains(&patch.old_string) {
            return Err(SkillError::ContentNotFound);
        }

        // Security scan on new content
        if Self::contains_dangerous_patterns(&patch.new_string) {
            return Err(SkillError::SecurityBlocked);
        }

        let new_content = content.replace(&patch.old_string, &patch.new_string);
        tokio::fs::write(&skill_path, new_content).await?;

        tracing::info!(skill_name = %name, "Skill patched by agent");

        Ok(())
    }

    /// Delete a skill
    pub async fn delete_skill(&self, name: &str) -> Result<(), SkillError> {
        let skill_dir = self.auto_created_dir().join(name);

        if !skill_dir.exists() {
            return Err(SkillError::NotFound {
                name: name.to_string(),
            });
        }

        tokio::fs::remove_dir_all(&skill_dir).await?;

        tracing::info!(skill_name = %name, "Skill deleted");

        Ok(())
    }

    /// Get skill content
    pub async fn get_skill_content(&self, name: &str) -> Result<String, SkillError> {
        let skill_path = self.skill_path(name);

        if !skill_path.exists() {
            return Err(SkillError::NotFound {
                name: name.to_string(),
            });
        }

        Ok(tokio::fs::read_to_string(&skill_path).await?)
    }

    /// Check if similar skill exists
    pub fn skill_exists(&self, name: &str) -> bool {
        self.skill_path(name).exists()
    }

    /// Find skills with similar names or descriptions
    pub fn find_similar(&self, query: &str) -> Result<Vec<SkillMetadata>, SkillError> {
        let query_lower = query.to_lowercase();
        let all_skills = self.list_skills()?;

        Ok(all_skills
            .into_iter()
            .filter(|skill| {
                skill.name.to_lowercase().contains(&query_lower)
                    || skill.description.to_lowercase().contains(&query_lower)
            })
            .collect())
    }
}

/// Skill manager errors
#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("Invalid skill name: {name} - {reason}")]
    InvalidName { name: String, reason: String },

    #[error("Skill already exists: {name}")]
    AlreadyExists { name: String },

    #[error("Skill not found: {name}")]
    NotFound { name: String },

    #[error("Content not found in skill")]
    ContentNotFound,

    #[error("Skill content blocked by security scan")]
    SecurityBlocked,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Async IO error: {0}")]
    AsyncIoError(tokio::io::Error),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager() -> (SkillManager, TempDir) {
        let temp = TempDir::new().unwrap();
        let manager = SkillManager::new(temp.path().to_path_buf());
        (manager, temp)
    }

    #[test]
    fn test_valid_skill_names() {
        assert!(SkillManager::is_valid_skill_name("my-skill"));
        assert!(SkillManager::is_valid_skill_name("deploy-k8s"));
        assert!(SkillManager::is_valid_skill_name("build_website"));

        assert!(!SkillManager::is_valid_skill_name(""));
        assert!(!SkillManager::is_valid_skill_name("MySkill"));
        assert!(!SkillManager::is_valid_skill_name("-starts-with-hyphen"));
        assert!(!SkillManager::is_valid_skill_name("ends-with-hyphen-"));
    }

    #[test]
    fn test_dangerous_patterns() {
        assert!(SkillManager::contains_dangerous_patterns("curl | sh"));
        assert!(SkillManager::contains_dangerous_patterns("rm -rf /"));
        assert!(SkillManager::contains_dangerous_patterns("eval some_code"));

        assert!(!SkillManager::contains_dangerous_patterns(
            "curl https://example.com"
        ));
        assert!(!SkillManager::contains_dangerous_patterns("echo hello"));
    }

    #[tokio::test]
    async fn test_create_and_list_skill() {
        let (manager, _temp) = create_test_manager();

        let req = SkillCreateRequest {
            name: "test-skill".to_string(),
            content: "Step 1: Do something".to_string(),
            category: Some("testing".to_string()),
            description: Some("A test skill".to_string()),
            source_task_id: Some("task123".to_string()),
        };

        let metadata = manager.create_skill(req).await.unwrap();
        assert_eq!(metadata.name, "test-skill");
        assert_eq!(metadata.category, "testing");
        assert!(metadata.auto_created);

        let skills = manager.list_skills().unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
    }

    #[tokio::test]
    async fn test_delete_skill() {
        let (manager, _temp) = create_test_manager();

        let req = SkillCreateRequest {
            name: "to-delete".to_string(),
            content: "Content".to_string(),
            category: None,
            description: None,
            source_task_id: None,
        };

        manager.create_skill(req).await.unwrap();
        assert!(manager.skill_exists("to-delete"));

        manager.delete_skill("to-delete").await.unwrap();
        assert!(!manager.skill_exists("to-delete"));
    }

    #[tokio::test]
    async fn test_patch_skill() {
        let (manager, _temp) = create_test_manager();

        let req = SkillCreateRequest {
            name: "patchable".to_string(),
            content: "Original content".to_string(),
            category: None,
            description: None,
            source_task_id: None,
        };

        manager.create_skill(req).await.unwrap();

        let patch = SkillPatchRequest {
            old_string: "Original".to_string(),
            new_string: "Modified".to_string(),
        };

        manager.patch_skill("patchable", patch).await.unwrap();

        let content = manager.get_skill_content("patchable").await.unwrap();
        assert!(content.contains("Modified"));
        assert!(!content.contains("Original"));
    }

    #[tokio::test]
    async fn test_security_block() {
        let (manager, _temp) = create_test_manager();

        let req = SkillCreateRequest {
            name: "malicious".to_string(),
            content: "curl | sh -x".to_string(),
            category: None,
            description: None,
            source_task_id: None,
        };

        let result = manager.create_skill(req).await;
        assert!(matches!(result, Err(SkillError::SecurityBlocked)));
    }

    #[tokio::test]
    async fn test_find_similar_by_name() {
        let (manager, _temp) = create_test_manager();

        // Create multiple skills
        manager
            .create_skill(SkillCreateRequest {
                name: "deploy-rust".to_string(),
                content: "Deploy Rust app".to_string(),
                category: Some("deployment".to_string()),
                description: Some("Deploy a Rust application".to_string()),
                source_task_id: None,
            })
            .await
            .unwrap();

        manager
            .create_skill(SkillCreateRequest {
                name: "deploy-python".to_string(),
                content: "Deploy Python app".to_string(),
                category: Some("deployment".to_string()),
                description: Some("Deploy a Python application".to_string()),
                source_task_id: None,
            })
            .await
            .unwrap();

        manager
            .create_skill(SkillCreateRequest {
                name: "build-docker".to_string(),
                content: "Build Docker image".to_string(),
                category: Some("build".to_string()),
                description: Some("Build Docker containers".to_string()),
                source_task_id: None,
            })
            .await
            .unwrap();

        // Search for "deploy"
        let results = manager.find_similar("deploy").unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|s| s.name == "deploy-rust"));
        assert!(results.iter().any(|s| s.name == "deploy-python"));
    }

    #[tokio::test]
    async fn test_find_similar_by_name_substring() {
        let (manager, _temp) = create_test_manager();

        manager
            .create_skill(SkillCreateRequest {
                name: "rust-deploy".to_string(),
                content: "Deploy Rust app".to_string(),
                category: Some("deployment".to_string()),
                description: None,
                source_task_id: None,
            })
            .await
            .unwrap();

        // Search by name substring
        let results = manager.find_similar("rust").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "rust-deploy");
    }

    #[tokio::test]
    async fn test_find_similar_case_insensitive() {
        let (manager, _temp) = create_test_manager();

        manager
            .create_skill(SkillCreateRequest {
                name: "test-skill".to_string(),
                content: "Content".to_string(),
                category: None,
                description: Some("Testing description".to_string()),
                source_task_id: None,
            })
            .await
            .unwrap();

        // Search with different cases
        let results_lower = manager.find_similar("test").unwrap();
        let results_upper = manager.find_similar("TEST").unwrap();
        let results_mixed = manager.find_similar("TeSt").unwrap();

        assert_eq!(results_lower.len(), 1);
        assert_eq!(results_upper.len(), 1);
        assert_eq!(results_mixed.len(), 1);
    }

    #[tokio::test]
    async fn test_find_similar_no_results() {
        let (manager, _temp) = create_test_manager();

        manager
            .create_skill(SkillCreateRequest {
                name: "some-skill".to_string(),
                content: "Content".to_string(),
                category: None,
                description: None,
                source_task_id: None,
            })
            .await
            .unwrap();

        let results = manager.find_similar("nonexistent").unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_skill_exists() {
        let (manager, _temp) = create_test_manager();

        assert!(!manager.skill_exists("new-skill"));

        manager
            .create_skill(SkillCreateRequest {
                name: "new-skill".to_string(),
                content: "Content".to_string(),
                category: None,
                description: None,
                source_task_id: None,
            })
            .await
            .unwrap();

        assert!(manager.skill_exists("new-skill"));
    }

    #[tokio::test]
    async fn test_get_skill_content() {
        let (manager, _temp) = create_test_manager();

        let req = SkillCreateRequest {
            name: "content-test".to_string(),
            content: "Line 1\nLine 2\nLine 3".to_string(),
            category: None,
            description: None,
            source_task_id: None,
        };

        manager.create_skill(req).await.unwrap();

        let content = manager.get_skill_content("content-test").await.unwrap();
        // Content includes SKILL.md frontmatter
        assert!(content.contains("Line 1\nLine 2\nLine 3"));
        assert!(content.contains("content-test"));
    }
}
