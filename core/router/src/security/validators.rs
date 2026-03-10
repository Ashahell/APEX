//! Security Validators: MCP server and Cron/Scheduled task validation
//!
//! This module provides:
//! - MCP server configuration validation
//! - Scheduled task (cron) validation
//! - Connection security validation

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Result of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

// =============================================================================
// MCP Server Validators
// =============================================================================

/// Validate MCP server configuration
pub fn validate_mcp_server_config(
    name: &str,
    command: &str,
    args: &[String],
    env: &Option<std::collections::HashMap<String, String>>,
) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Validate name
    if name.trim().is_empty() {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Server name cannot be empty".to_string(),
            severity: ValidationSeverity::Error,
        });
    }

    if name.len() > 100 {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Server name too long (max 100 characters)".to_string(),
            severity: ValidationSeverity::Error,
        });
    }

    // Validate command
    if command.trim().is_empty() {
        errors.push(ValidationError {
            field: "command".to_string(),
            message: "Command cannot be empty".to_string(),
            severity: ValidationSeverity::Error,
        });
    }

    // Check for dangerous commands
    let dangerous_commands = ["sudo", "su", "chmod", "chown"];
    for dc in dangerous_commands {
        if command.contains(dc) {
            warnings.push(format!(
                "Command contains potentially dangerous binary: {}",
                dc
            ));
        }
    }

    // Validate args
    if args.len() > 50 {
        errors.push(ValidationError {
            field: "args".to_string(),
            message: "Too many arguments (max 50)".to_string(),
            severity: ValidationSeverity::Error,
        });
    }

    // Check for dangerous args
    let dangerous_args = ["-x", "--execute", "eval", "exec"];
    for arg in args {
        for da in dangerous_args {
            if arg.contains(da) {
                warnings.push(format!(
                    "Argument contains potentially dangerous pattern: {}",
                    da
                ));
            }
        }
    }

    // Validate env
    if let Some(env_vars) = env {
        if env_vars.len() > 20 {
            warnings.push(
                "Many environment variables - ensure only necessary ones are set".to_string(),
            );
        }

        // Check for sensitive env vars
        let sensitive = ["API_KEY", "SECRET", "PASSWORD", "TOKEN", "CREDENTIAL"];
        for (key, _) in env_vars {
            for s in sensitive {
                if key.to_uppercase().contains(s) {
                    warnings.push(format!(
                        "Environment variable '{}' appears to contain sensitive data",
                        key
                    ));
                }
            }
        }
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Validate MCP tool name
pub fn validate_mcp_tool_name(name: &str) -> ValidationResult {
    let mut errors = Vec::new();

    if name.trim().is_empty() {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Tool name cannot be empty".to_string(),
            severity: ValidationSeverity::Error,
        });
    }

    // Check for valid characters (alphanumeric, underscore, hyphen)
    let valid_chars = name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-');
    if !valid_chars {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Tool name can only contain alphanumeric characters, underscores, and hyphens"
                .to_string(),
            severity: ValidationSeverity::Error,
        });
    }

    if name.len() > 100 {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Tool name too long (max 100 characters)".to_string(),
            severity: ValidationSeverity::Error,
        });
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings: vec![],
    }
}

// =============================================================================
// Cron/Schedule Validators
// =============================================================================

/// Validate cron expression
pub fn validate_cron_expression(expression: &str) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let parts: Vec<&str> = expression.trim().split_whitespace().collect();

    // Standard cron has 5 parts: minute, hour, day of month, month, day of week
    // Extended cron (with seconds) has 6 parts
    if parts.len() != 5 && parts.len() != 6 {
        errors.push(ValidationError {
            field: "expression".to_string(),
            message: format!(
                "Invalid cron expression: expected 5 or 6 parts, got {}",
                parts.len()
            ),
            severity: ValidationSeverity::Error,
        });
        return ValidationResult {
            is_valid: false,
            errors,
            warnings,
        };
    }

    // Validate each part
    if let Some(minute) = parts.get(0) {
        validate_cron_field(minute, 0, 59, "minute", &mut errors);
    }

    if let Some(hour) = parts.get(1) {
        validate_cron_field(hour, 0, 23, "hour", &mut errors);
    }

    if let Some(dom) = parts.get(2) {
        validate_cron_field(dom, 1, 31, "day of month", &mut errors);
    }

    if let Some(month) = parts.get(3) {
        validate_cron_field(month, 1, 12, "month", &mut errors);
    }

    if let Some(dow) = parts.get(4) {
        validate_cron_field(dow, 0, 7, "day of week", &mut errors);
    }

    // Warnings for potentially dangerous schedules
    if parts.len() >= 2 {
        // Very frequent execution warnings
        if parts[0] == "*" && parts[1] == "*" {
            warnings.push("Every minute - may cause high load".to_string());
        }
        if parts[0] != "*" && parts[1] == "*" {
            warnings.push("Every hour - relatively frequent".to_string());
        }
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
    }
}

fn validate_cron_field(
    field: &str,
    min: u32,
    max: u32,
    name: &str,
    errors: &mut Vec<ValidationError>,
) {
    // Handle special characters
    if field == "*" {
        return; // Wildcard is valid
    }

    // Handle ranges (e.g., 1-5)
    if field.contains('-') && !field.contains(',') && !field.contains('/') {
        let range_parts: Vec<&str> = field.split('-').collect();
        if range_parts.len() == 2 {
            if let (Ok(start), Ok(end)) =
                (range_parts[0].parse::<u32>(), range_parts[1].parse::<u32>())
            {
                if start < min || end > max || start > end {
                    errors.push(ValidationError {
                        field: name.to_string(),
                        message: format!(
                            "Invalid range: {}-{} (valid: {}-{})",
                            start, end, min, max
                        ),
                        severity: ValidationSeverity::Error,
                    });
                }
                return;
            }
        }
    }

    // Handle step values (e.g., */5)
    if field.contains('/') {
        let step_parts: Vec<&str> = field.split('/').collect();
        if step_parts.len() == 2 {
            if let Ok(step) = step_parts[1].parse::<u32>() {
                if step == 0 {
                    errors.push(ValidationError {
                        field: name.to_string(),
                        message: "Step value cannot be 0".to_string(),
                        severity: ValidationSeverity::Error,
                    });
                }
                return;
            }
        }
    }

    // Handle lists (e.g., 1,2,3)
    if field.contains(',') {
        for item in field.split(',') {
            if let Ok(num) = item.parse::<u32>() {
                if num < min || num > max {
                    errors.push(ValidationError {
                        field: name.to_string(),
                        message: format!("Value {} out of range ({}-{})", num, min, max),
                        severity: ValidationSeverity::Error,
                    });
                }
            }
        }
        return;
    }

    // Single value
    if let Ok(num) = field.parse::<u32>() {
        if num < min || num > max {
            errors.push(ValidationError {
                field: name.to_string(),
                message: format!("Value {} out of range ({}-{})", num, min, max),
                severity: ValidationSeverity::Error,
            });
        }
    } else {
        errors.push(ValidationError {
            field: name.to_string(),
            message: format!("Invalid value: {}", field),
            severity: ValidationSeverity::Error,
        });
    }
}

/// Validate scheduled task configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskConfig {
    pub name: String,
    pub cron_expression: String,
    pub skill_name: Option<String>,
    pub task_type: TaskType,
    pub max_duration_secs: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    SkillExecution,
    DataSync,
    Cleanup,
    Backup,
    Custom,
}

pub fn validate_scheduled_task(config: &ScheduledTaskConfig) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Validate name
    if config.name.trim().is_empty() {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Task name cannot be empty".to_string(),
            severity: ValidationSeverity::Error,
        });
    }

    // Validate cron
    let cron_result = validate_cron_expression(&config.cron_expression);
    if !cron_result.is_valid {
        for err in cron_result.errors {
            errors.push(ValidationError {
                field: "cron_expression".to_string(),
                message: err.message,
                severity: err.severity,
            });
        }
    }
    warnings.extend(cron_result.warnings);

    // Validate max duration
    if config.max_duration_secs == 0 {
        errors.push(ValidationError {
            field: "max_duration_secs".to_string(),
            message: "Max duration must be greater than 0".to_string(),
            severity: ValidationSeverity::Error,
        });
    }

    if config.max_duration_secs > 3600 {
        warnings.push("Max duration over 1 hour - ensure task can complete in time".to_string());
    }

    // Validate skill name if skill execution
    if config.task_type == TaskType::SkillExecution {
        if config.skill_name.is_none() || config.skill_name.as_ref().unwrap().trim().is_empty() {
            errors.push(ValidationError {
                field: "skill_name".to_string(),
                message: "Skill name required for SkillExecution tasks".to_string(),
                severity: ValidationSeverity::Error,
            });
        }
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Validate connection timeout
pub fn validate_timeout(timeout_ms: u64) -> ValidationResult {
    let mut errors = Vec::new();

    if timeout_ms == 0 {
        errors.push(ValidationError {
            field: "timeout".to_string(),
            message: "Timeout must be greater than 0".to_string(),
            severity: ValidationSeverity::Error,
        });
    }

    if timeout_ms > 300_000 {
        errors.push(ValidationError {
            field: "timeout".to_string(),
            message: "Timeout exceeds 5 minutes".to_string(),
            severity: ValidationSeverity::Warning,
        });
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cron_minute() {
        let result = validate_cron_expression("* * * * *");
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_cron_invalid() {
        let result = validate_cron_expression("* * *");
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_cron_out_of_range() {
        let result = validate_cron_expression("70 * * * *");
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_mcp_server() {
        let result = validate_mcp_server_config("test", "node", &[], &None);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_mcp_server_empty_name() {
        let result = validate_mcp_server_config("", "node", &[], &None);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_scheduled_task() {
        let config = ScheduledTaskConfig {
            name: "test_task".to_string(),
            cron_expression: "0 * * * *".to_string(),
            skill_name: Some("test.skill".to_string()),
            task_type: TaskType::SkillExecution,
            max_duration_secs: 300,
            enabled: true,
        };
        let result = validate_scheduled_task(&config);
        assert!(result.is_valid);
    }
}
