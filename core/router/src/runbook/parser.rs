//! Runbook Parser - Parses YAML/JSON runbook definitions

use super::models::*;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// RunbookParser - parses runbook definitions from YAML or JSON
pub struct RunbookParser;

impl RunbookParser {
    /// Parse runbook from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Runbook> {
        serde_yaml::from_str(yaml).map_err(|e| anyhow!("YAML parse error: {}", e))
    }

    /// Parse runbook from JSON string
    pub fn from_json(json: &str) -> Result<Runbook> {
        serde_json::from_str(json).map_err(|e| anyhow!("JSON parse error: {}", e))
    }

    /// Parse runbook from either YAML or JSON (auto-detect)
    pub fn parse(content: &str) -> Result<Runbook> {
        let trimmed = content.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            Self::from_json(trimmed)
        } else {
            Self::from_yaml(trimmed)
        }
    }

    /// Validate a runbook definition
    pub fn validate(runbook: &Runbook) -> Result<()> {
        if runbook.name.is_empty() {
            return Err(anyhow!("Runbook name cannot be empty"));
        }
        if runbook.steps.is_empty() {
            return Err(anyhow!("Runbook must have at least one step"));
        }

        for step in &runbook.steps {
            Self::validate_step(step)?;
        }

        Ok(())
    }

    fn validate_step(step: &RunbookStep) -> Result<()> {
        match step {
            RunbookStep::Webhook { url, .. } => {
                if url.is_empty() {
                    return Err(anyhow!("Webhook URL cannot be empty"));
                }
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return Err(anyhow!("Webhook URL must start with http:// or https://"));
                }
            }
            RunbookStep::ExecuteCommand { command } => {
                // Safety check - block dangerous commands
                let dangerous = ["rm -rf", "dd if=", ":(){:|:&};"];
                for d in dangerous {
                    if command.contains(d) {
                        return Err(anyhow!("Command contains blocked pattern: {}", d));
                    }
                }
            }
            RunbookStep::Delay { ms } => {
                if *ms > 3600000 {
                    // 1 hour max
                    return Err(anyhow!("Delay cannot exceed 1 hour"));
                }
            }
            RunbookStep::CreateTask { input, .. } => {
                if input.is_empty() {
                    return Err(anyhow!("Task input cannot be empty"));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Convert runbook to YAML string
    pub fn to_yaml(runbook: &Runbook) -> Result<String> {
        serde_yaml::to_string(runbook).map_err(|e| anyhow!("YAML serialize error: {}", e))
    }

    /// Convert runbook to JSON string
    pub fn to_json(runbook: &Runbook) -> Result<String> {
        serde_json::to_string_pretty(runbook).map_err(|e| anyhow!("JSON serialize error: {}", e))
    }
}

/// YAML-specific structures for parsing
#[derive(Debug, Deserialize)]
struct YamlRunbook {
    name: String,
    description: Option<String>,
    trigger_alert_type: Option<String>,
    enabled: Option<bool>,
    steps: Vec<YamlStep>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum YamlStep {
    CancelTask {
        task_id: String,
    },
    CreateTask {
        input: String,
        priority: Option<String>,
        project: Option<String>,
    },
    Notify {
        message: String,
    },
    ExecuteCommand {
        command: String,
    },
    Delay {
        ms: u64,
    },
    Webhook {
        url: String,
        method: Option<String>,
        body: Option<String>,
    },
    RestartTask {
        task_id: String,
    },
}

impl From<YamlRunbook> for Runbook {
    fn from(yaml: YamlRunbook) -> Self {
        let steps: Vec<RunbookStep> = yaml.steps.into_iter().map(|s| s.into()).collect();
        Runbook {
            id: ulid::Ulid::new().to_string(),
            name: yaml.name,
            description: yaml.description,
            trigger_alert_type: yaml.trigger_alert_type,
            steps,
            enabled: yaml.enabled.unwrap_or(true),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl From<YamlStep> for RunbookStep {
    fn from(yaml: YamlStep) -> Self {
        match yaml {
            YamlStep::CancelTask { task_id } => RunbookStep::CancelTask { task_id },
            YamlStep::CreateTask {
                input,
                priority,
                project,
            } => RunbookStep::CreateTask {
                input,
                priority,
                project,
            },
            YamlStep::Notify { message } => RunbookStep::Notify { message },
            YamlStep::ExecuteCommand { command } => RunbookStep::ExecuteCommand { command },
            YamlStep::Delay { ms } => RunbookStep::Delay { ms },
            YamlStep::Webhook { url, method, body } => RunbookStep::Webhook {
                url,
                method,
                body,
                headers: None,
            },
            YamlStep::RestartTask { task_id } => RunbookStep::RestartTask { task_id },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
id: "test-runbook-123"
name: "Test Runbook"
description: "A test runbook"
enabled: true
created_at: "2024-01-01T00:00:00Z"
updated_at: "2024-01-01T00:00:00z"
steps:
  - type: delay
    ms: 100
  - type: notify
    message: "Hello"
"#;

        let runbook = RunbookParser::from_yaml(yaml).unwrap();
        assert_eq!(runbook.name, "Test Runbook");
        assert_eq!(runbook.steps.len(), 2);
    }

    #[test]
    fn test_validate_empty_name() {
        let runbook = Runbook::new("".to_string(), "Test".to_string(), vec![]);
        let result = RunbookParser::validate(&runbook);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_dangerous_command() {
        let runbook = Runbook::new(
            "test".to_string(),
            "Test".to_string(),
            vec![RunbookStep::ExecuteCommand {
                command: "rm -rf /".to_string(),
            }],
        );
        let result = RunbookParser::validate(&runbook);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("blocked pattern"));
    }
}
