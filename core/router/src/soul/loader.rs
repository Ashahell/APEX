use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::SoulIdentity;

#[derive(Clone)]
pub struct SoulLoader {
    config: super::SoulConfig,
    cached_identity: Arc<RwLock<Option<SoulIdentity>>>,
}

impl SoulLoader {
    pub fn new(config: super::SoulConfig) -> Self {
        Self {
            config,
            cached_identity: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn load_identity(&self) -> Result<SoulIdentity, SoulError> {
        let soul_path = self.config.soul_dir.join("SOUL.md");
        
        if !soul_path.exists() {
            tracing::info!("SOUL.md not found, creating default");
            let identity = SoulIdentity::default();
            self.save_identity(&identity).await?;
            return Ok(identity);
        }

        let content = tokio::fs::read_to_string(&soul_path)
            .await
            .map_err(|e| SoulError::IoError(e.to_string()))?;

        self.parse_soul_md(&content)
    }

    pub async fn load_identity_uncached(&self) -> Result<SoulIdentity, SoulError> {
        let soul_path = self.config.soul_dir.join("SOUL.md");
        
        if !soul_path.exists() {
            return Ok(SoulIdentity::default());
        }

        let content = tokio::fs::read_to_string(&soul_path)
            .await
            .map_err(|e| SoulError::IoError(e.to_string()))?;

        self.parse_soul_md(&content)
    }

    pub async fn save_identity(&self, identity: &SoulIdentity) -> Result<(), SoulError> {
        tokio::fs::create_dir_all(&self.config.soul_dir)
            .await
            .map_err(|e| SoulError::IoError(e.to_string()))?;

        if self.config.backup_enabled {
            self.create_backup().await?;
        }

        let content = self.render_soul_md(identity);
        let soul_path = self.config.soul_dir.join("SOUL.md");
        
        tokio::fs::write(&soul_path, content)
            .await
            .map_err(|e| SoulError::IoError(e.to_string()))?;

        let mut cached = self.cached_identity.write().await;
        *cached = Some(identity.clone());

        Ok(())
    }

    async fn create_backup(&self) -> Result<(), SoulError> {
        let soul_path = self.config.soul_dir.join("SOUL.md");
        if !soul_path.exists() {
            return Ok(());
        }

        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%SZ");
        let backup_name = format!("SOUL.md.backup.{}", timestamp);
        let backup_path = self.config.soul_dir.join(backup_name);

        tokio::fs::copy(&soul_path, &backup_path)
            .await
            .map_err(|e| SoulError::IoError(e.to_string()))?;

        tracing::info!(backup = %backup_path.display(), "Created SOUL.md backup");
        Ok(())
    }

    pub fn parse_soul_md(&self, content: &str) -> Result<SoulIdentity, SoulError> {
        let mut identity = SoulIdentity::default();
        let mut in_values = false;
        let mut in_capabilities = false;
        let mut in_goals = false;
        let mut _in_constitution = false;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("## Identity") {
                continue;
            }
            if line.starts_with("- **Name**") {
                if let Some(name) = Self::extract_value(line) {
                    identity.name = name;
                }
            }
            if line.starts_with("- **Version**") {
                if let Some(version) = Self::extract_value(line) {
                    identity.version = version;
                }
            }
            if line.starts_with("- **Created**") {
                if let Some(created) = Self::extract_value(line) {
                    identity.created = created;
                }
            }
            if line.starts_with("- **Wake Count**") {
                if let Some(count) = Self::extract_value(line) {
                    identity.wake_count = count.parse().unwrap_or(0);
                }
            }
            if line.starts_with("## Purpose") {
                // Next lines until next ## are purpose
                continue;
            }
            if line == "## Values" {
                in_values = true;
                continue;
            }
            if line == "## Capabilities" {
                in_values = false;
                in_capabilities = true;
                continue;
            }
            if line == "## Current Goals" {
                in_capabilities = false;
                in_goals = true;
                continue;
            }
            if line.starts_with("# CONSTITUTION") || line.starts_with("## CONSTITUTION") {
                in_goals = false;
                _in_constitution = true;
                continue;
            }
            if in_values && line.starts_with("- **") {
                if let Some(value) = self.parse_value(line) {
                    identity.values.push(value);
                }
            }
            if in_capabilities && line.starts_with("- ") {
                if let Some(cap) = self.parse_capability(line) {
                    identity.capabilities.push(cap);
                }
            }
            if in_goals && line.starts_with("- [") {
                if let Some(goal) = self.parse_goal(line) {
                    identity.current_goals.push(goal);
                }
            }
        }

        Ok(identity)
    }

    fn extract_value(line: &str) -> Option<String> {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() > 1 {
            Some(parts[1].trim().to_string())
        } else {
            None
        }
    }

    fn parse_value(&self, line: &str) -> Option<super::Value> {
        let line = line.trim_start_matches("- **");
        let parts: Vec<&str> = line.splitn(2, "**:").collect();
        if parts.len() == 2 {
            Some(super::Value {
                name: parts[0].to_string(),
                description: parts[1].trim().to_string(),
                priority: 1,
            })
        } else {
            None
        }
    }

    fn parse_capability(&self, line: &str) -> Option<super::Capability> {
        let line = line.trim_start_matches("- ");
        Some(super::Capability {
            name: line.split(':').next()?.to_string(),
            description: line.split(':').nth(1)?.trim().to_string(),
            tier: "T1".to_string(),
        })
    }

    fn parse_goal(&self, line: &str) -> Option<super::Goal> {
        let line = line.trim_start_matches("- [");
        let status = line.chars().next()?.to_string();
        let rest = line[3..].splitn(2, ']').nth(1)?.trim().to_string();
        
        Some(super::Goal {
            description: rest.to_string(),
            status,
            priority: 1,
            deadline: None,
        })
    }

    pub fn render_soul_md(&self, identity: &SoulIdentity) -> String {
        let mut content = String::new();
        
        content.push_str("# SOUL.md\n");
        content.push_str("# This file defines who I am. I read it every time I wake.\n\n");
        
        content.push_str("## Identity\n");
        content.push_str(&format!("- **Name**: {}\n", identity.name));
        content.push_str(&format!("- **Version**: {}\n", identity.version));
        content.push_str(&format!("- **Created**: {}\n", identity.created));
        content.push_str(&format!("- **Wake Count**: {}\n\n", identity.wake_count));
        
        content.push_str("## Purpose\n");
        content.push_str(&identity.purpose);
        content.push_str("\n\n");
        
        content.push_str("## Values\n");
        for value in &identity.values {
            content.push_str(&format!("- **{}**: {}\n", value.name, value.description));
        }
        content.push('\n');
        
        content.push_str("## Capabilities\n");
        for cap in &identity.capabilities {
            content.push_str(&format!("- {}: {}\n", cap.name, cap.description));
        }
        content.push('\n');
        
        content.push_str("## Current Goals\n");
        for goal in &identity.current_goals {
            content.push_str(&format!("- [{}] {} (Priority: {})\n", 
                goal.status, goal.description, goal.priority));
        }
        content.push('\n');
        
        content.push_str("---\n");
        content.push_str("# CONSTITUTION\n");
        content.push_str("# These values are protected. Modification requires T3 authorization.\n\n");
        content.push_str(&format!("CONSTITUTION_VERSION: {}\n", identity.constitution.version));
        content.push_str(&format!("IMMUTABLE_VALUES: {}\n", 
            identity.constitution.immutable_values.join(", ")));
        
        content
    }

    pub async fn load_fragments(&self) -> Result<HashMap<String, String>, SoulError> {
        let mut fragments = HashMap::new();
        
        if !self.config.fragments_dir.exists() {
            return Ok(fragments);
        }

        let mut entries = tokio::fs::read_dir(&self.config.fragments_dir)
            .await
            .map_err(|e| SoulError::IoError(e.to_string()))?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| SoulError::IoError(e.to_string()))? {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                let name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(|e| SoulError::IoError(e.to_string()))?;
                
                fragments.insert(name, content);
            }
        }

        Ok(fragments)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SoulError {
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Constitution violation: {0}")]
    ConstitutionViolation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

impl SoulLoader {
    pub async fn increment_wake_count(&self) -> Result<(), SoulError> {
        let mut identity = self.load_identity_uncached().await?;
        identity.wake_count += 1;
        self.save_identity(&identity).await
    }

    pub async fn backup_identity(&self) -> Result<(), SoulError> {
        self.create_backup().await
    }

    pub async fn update_identity(&self, content: &str) -> Result<(), SoulError> {
        let identity = self.parse_soul_md(content)?;
        self.save_identity(&identity).await
    }
}
