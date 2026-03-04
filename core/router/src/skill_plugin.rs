use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tier: PermissionTier,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub dependencies: Vec<String>,
    pub runtime: PluginRuntime,
    pub capabilities: Vec<String>,
    pub security: SkillSecurity,
    pub example: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionTier {
    T0,
    T1,
    T2,
    T3,
}

impl PermissionTier {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "T0" => Some(PermissionTier::T0),
            "T1" => Some(PermissionTier::T1),
            "T2" => Some(PermissionTier::T2),
            "T3" => Some(PermissionTier::T3),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionTier::T0 => "T0",
            PermissionTier::T1 => "T1",
            PermissionTier::T2 => "T2",
            PermissionTier::T3 => "T3",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PluginRuntime {
    Typescript,
    Python,
    Bash,
}

impl PluginRuntime {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "typescript" => Some(PluginRuntime::Typescript),
            "python" => Some(PluginRuntime::Python),
            "bash" => Some(PluginRuntime::Bash),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSecurity {
    pub sandbox: bool,
    pub network: bool,
    pub timeout_secs: u32,
}

impl Default for SkillSecurity {
    fn default() -> Self {
        Self {
            sandbox: true,
            network: false,
            timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPlugin {
    pub manifest: SkillManifest,
    pub source_path: PathBuf,
    pub status: PluginStatus,
    pub loaded_at: std::time::Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PluginStatus {
    Loading,
    Loaded,
    HotReloading,
    Unloaded,
    Failed(String),
}

pub struct SkillPluginManager {
    plugins: HashMap<String, SkillPlugin>,
    skills_dir: PathBuf,
}

impl SkillPluginManager {
    pub fn new(skills_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            skills_dir,
        }
    }

    pub async fn load_plugin(&mut self, name: &str) -> Result<&SkillPlugin, PluginError> {
        let plugin_path = self.skills_dir.join(name).join("SKILL.md");
        
        if !plugin_path.exists() {
            return Err(PluginError::NotFound(name.to_string()));
        }

        let manifest = Self::parse_skill_md(&plugin_path)?;
        let plugin = SkillPlugin {
            manifest,
            source_path: plugin_path,
            status: PluginStatus::Loaded,
            loaded_at: std::time::Instant::now(),
        };

        self.plugins.insert(name.to_string(), plugin);
        
        Ok(self.plugins.get(name).unwrap())
    }

    pub async fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.status = PluginStatus::Unloaded;
            self.plugins.remove(name);
            Ok(())
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }

    pub fn get_plugin(&self, name: &str) -> Option<&SkillPlugin> {
        self.plugins.get(name)
    }

    pub fn list_plugins(&self) -> Vec<&SkillPlugin> {
        self.plugins.values().collect()
    }

    fn parse_skill_md(path: &Path) -> Result<SkillManifest, PluginError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PluginError::IoError(e.to_string()))?;

        let mut name = String::new();
        let mut version = String::new();
        let mut author = String::new();
        let mut tier = PermissionTier::T1;
        let mut runtime = PluginRuntime::Typescript;
        let mut description = String::new();
        let mut input_schema = serde_json::json!({"type": "object"});
        let mut output_schema = serde_json::json!({"type": "object"});
        let mut dependencies = Vec::new();
        let mut capabilities = Vec::new();
        let mut security = SkillSecurity::default();
        let mut example = None;

        let mut in_code_block = false;
        let mut code_block_type = String::new();
        let mut code_content = String::new();

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("```") && !in_code_block {
                in_code_block = true;
                code_block_type = line.trim_start_matches("```").trim().to_string();
                code_content = String::new();
                continue;
            }

            if line == "```" && in_code_block {
                in_code_block = false;
                match code_block_type.as_str() {
                    "json" => {
                        if code_content.contains("\"properties\"") || code_content.contains("\"type\"") {
                            if code_content.contains("input") || code_content.contains("input_schema") {
                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&code_content) {
                                    input_schema = parsed;
                                }
                            } else {
                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&code_content) {
                                    output_schema = parsed;
                                }
                            }
                        } else {
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&code_content) {
                                if input_schema.get("type").is_none() {
                                    input_schema = parsed.clone();
                                }
                                output_schema = parsed;
                            }
                        }
                    }
                    "yaml" | "example" => {
                        if let Ok(parsed) = serde_yaml::from_str::<serde_json::Value>(&code_content) {
                            example = Some(parsed.as_object()
                                .map(|m| m.iter()
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect())
                                .unwrap_or_default());
                        }
                    }
                    _ => {}
                }
                code_content = String::new();
                continue;
            }

            if in_code_block {
                code_content.push_str(line);
                code_content.push('\n');
                continue;
            }

            if line.starts_with("**Version**") {
                version = line.split(':').nth(1).map(|s| s.trim().to_string()).unwrap_or_default();
            } else if line.starts_with("**Author**") {
                author = line.split(':').nth(1).map(|s| s.trim().to_string()).unwrap_or_default();
            } else if line.starts_with("**Tier**") {
                if let Some(t) = line.split(':').nth(1).map(|s| s.trim()) {
                    tier = PermissionTier::from_str(t.split_whitespace().next().unwrap_or("T1"))
                        .unwrap_or(PermissionTier::T1);
                }
            } else if line.starts_with("**Runtime**") {
                if let Some(r) = line.split(':').nth(1).map(|s| s.trim()) {
                    runtime = PluginRuntime::from_str(r.split_whitespace().next().unwrap_or("typescript"))
                        .unwrap_or(PluginRuntime::Typescript);
                }
            } else if line.starts_with("## Description") {
                // Description follows
            } else if line.starts_with("- ") && !line.contains(':') {
                capabilities.push(line.trim_start_matches("- ").to_string());
            }
        }

        if let Some(file_stem) = path.file_stem() {
            name = file_stem.to_string_lossy().to_string();
        }

        if name.starts_with("skill.") {
            name = name.trim_start_matches("skill.").to_string();
        }

        Ok(SkillManifest {
            name,
            version,
            description,
            author,
            tier,
            input_schema,
            output_schema,
            dependencies,
            runtime,
            capabilities,
            security,
            example,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginApiResponse {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tier: String,
    pub runtime: String,
    pub status: String,
}

impl From<&SkillPlugin> for PluginApiResponse {
    fn from(plugin: &SkillPlugin) -> Self {
        Self {
            name: plugin.manifest.name.clone(),
            version: plugin.manifest.version.clone(),
            description: plugin.manifest.description.clone(),
            tier: plugin.manifest.tier.as_str().to_string(),
            runtime: format!("{:?}", plugin.manifest.runtime),
            status: format!("{:?}", plugin.status),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),
    
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_tier_from_str() {
        assert_eq!(PermissionTier::from_str("T0"), Some(PermissionTier::T0));
        assert_eq!(PermissionTier::from_str("T1"), Some(PermissionTier::T1));
        assert_eq!(PermissionTier::from_str("t2"), Some(PermissionTier::T2));
        assert_eq!(PermissionTier::from_str("invalid"), None);
    }

    #[test]
    fn test_plugin_runtime_from_str() {
        assert_eq!(PluginRuntime::from_str("typescript"), Some(PluginRuntime::Typescript));
        assert_eq!(PluginRuntime::from_str("python"), Some(PluginRuntime::Python));
        assert_eq!(PluginRuntime::from_str("bash"), Some(PluginRuntime::Bash));
        assert_eq!(PluginRuntime::from_str("invalid"), None);
    }
}
