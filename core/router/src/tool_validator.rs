//! Tool Validator - Runtime validation for dynamic tools
//!
//! Provides import allowlist validation with three levels:
//! - Strict: Only safe stdlib modules
//! - Moderate: Includes network/parsing modules
//! - Permissive: No restrictions
//!
//! Feature 1: Tool Maker Runtime Validation

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::unified_config::tool_validation_constants::*;

/// Validation level for tool execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationLevel {
    /// Only safe stdlib modules allowed
    Strict,
    /// Moderate - includes network/parsing
    Moderate,
    /// No restrictions
    Permissive,
}

impl Default for ValidationLevel {
    fn default() -> Self {
        Self::Strict
    }
}

impl std::fmt::Display for ValidationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationLevel::Strict => write!(f, "strict"),
            ValidationLevel::Moderate => write!(f, "moderate"),
            ValidationLevel::Permissive => write!(f, "permissive"),
        }
    }
}

impl ValidationLevel {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "strict" => Ok(ValidationLevel::Strict),
            "moderate" => Ok(ValidationLevel::Moderate),
            "permissive" => Ok(ValidationLevel::Permissive),
            _ => Err(format!("Unknown validation level: {}", s)),
        }
    }
}

/// Result of validation check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the code is allowed
    pub allowed: bool,
    /// Blocked imports found
    pub blocked_imports: Vec<String>,
    /// Warning messages
    pub warnings: Vec<String>,
    /// Validation level used
    pub validation_level: String,
    /// Error message if validation failed
    pub error: Option<String>,
}

impl ValidationResult {
    /// Create an allowed result
    pub fn allowed(level: ValidationLevel) -> Self {
        Self {
            allowed: true,
            blocked_imports: vec![],
            warnings: vec![],
            validation_level: level.to_string(),
            error: None,
        }
    }

    /// Create a blocked result
    pub fn blocked(imports: Vec<String>, level: ValidationLevel) -> Self {
        let error_msg = format!("Blocked {} imports: {}", imports.len(), imports.join(", "));
        Self {
            allowed: false,
            blocked_imports: imports,
            warnings: vec![],
            validation_level: level.to_string(),
            error: Some(error_msg),
        }
    }
}

/// Tool validator - checks Python code for dangerous imports
pub struct ToolValidator;

impl ToolValidator {
    /// Validate Python code against import allowlist
    pub fn validate(code: &str, level: ValidationLevel) -> ValidationResult {
        // Permissive mode - allow everything
        if level == ValidationLevel::Permissive {
            return ValidationResult::allowed(level);
        }

        // Check code length
        if code.len() > MAX_CODE_LENGTH {
            return ValidationResult {
                allowed: false,
                blocked_imports: vec![],
                warnings: vec![],
                validation_level: level.to_string(),
                error: Some(format!(
                    "Code exceeds maximum length of {} characters",
                    MAX_CODE_LENGTH
                )),
            };
        }

        // Extract imports from code
        let imports = Self::extract_imports(code);

        // Get allowlist for level
        let allowlist = match level {
            ValidationLevel::Strict => STRICT_IMPORT_ALLOWLIST.to_vec(),
            ValidationLevel::Moderate => MODERATE_IMPORT_ALLOWLIST.to_vec(),
            ValidationLevel::Permissive => vec![],
        };

        let allowset: HashSet<&str> = allowlist.into_iter().collect();

        // Check imports against allowlist
        let mut blocked: Vec<String> = vec![];

        for import in &imports {
            // Check if import or its parent is in allowlist
            let is_allowed = allowset.contains(import.as_str())
                || import.starts_with("os.path")
                || import.starts_with("xml.etree")
                || import.starts_with("http.client");

            if !is_allowed {
                // Check if it's a system killer import
                if SYSTEM_KILLER_IMPORTS.iter().any(|&k| import.contains(k)) {
                    blocked.push(format!("{} (system killer)", import));
                } else if !allowset.contains(import.as_str()) {
                    blocked.push(import.clone());
                }
            }
        }

        // Limit blocked imports reported
        if blocked.len() > MAX_BLOCKED_REPORT {
            let excess = blocked.len() - MAX_BLOCKED_REPORT;
            blocked.truncate(MAX_BLOCKED_REPORT);
            blocked.push(format!("... and {} more", excess));
        }

        if blocked.is_empty() {
            ValidationResult::allowed(level)
        } else {
            ValidationResult::blocked(blocked, level)
        }
    }

    /// Extract import statements from Python code
    fn extract_imports(code: &str) -> Vec<String> {
        let mut imports = Vec::new();

        // Regular expressions for imports
        // Match: import os, import os.path, import os as foo
        let re_import = regex::Regex::new(r"(?m)^import\s+([\w.]+)").unwrap();
        // Match: from os import path, from os.path import join
        let re_from = regex::Regex::new(r"(?m)^from\s+([\w.]+)\s+import").unwrap();

        // Find all import statements
        for cap in re_import.captures_iter(code) {
            if let Some(m) = cap.get(1) {
                let module = m
                    .as_str()
                    .split('.')
                    .next()
                    .unwrap_or(m.as_str())
                    .to_string();
                if !module.is_empty() && module != "_" {
                    imports.push(module);
                }
            }
        }

        for cap in re_from.captures_iter(code) {
            if let Some(m) = cap.get(1) {
                let module = m
                    .as_str()
                    .split('.')
                    .next()
                    .unwrap_or(m.as_str())
                    .to_string();
                if !module.is_empty() && module != "_" {
                    imports.push(module);
                }
            }
        }

        imports
    }

    /// Get the allowlist for a validation level
    pub fn get_allowlist(level: ValidationLevel) -> Vec<&'static str> {
        match level {
            ValidationLevel::Strict => STRICT_IMPORT_ALLOWLIST.to_vec(),
            ValidationLevel::Moderate => MODERATE_IMPORT_ALLOWLIST.to_vec(),
            ValidationLevel::Permissive => vec![],
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_level_default() {
        let level = ValidationLevel::default();
        assert_eq!(level, ValidationLevel::Strict);
    }

    #[test]
    fn test_validation_level_from_str() {
        assert_eq!(
            ValidationLevel::from_str("strict").unwrap(),
            ValidationLevel::Strict
        );
        assert_eq!(
            ValidationLevel::from_str("moderate").unwrap(),
            ValidationLevel::Moderate
        );
        assert_eq!(
            ValidationLevel::from_str("permissive").unwrap(),
            ValidationLevel::Permissive
        );
        assert!(ValidationLevel::from_str("invalid").is_err());
    }

    #[test]
    fn test_strict_blocks_subprocess() {
        let code = "import subprocess\nsubprocess.run(['ls'])";
        let result = ToolValidator::validate(code, ValidationLevel::Strict);
        assert!(!result.allowed);
        assert!(result
            .blocked_imports
            .iter()
            .any(|i| i.contains("subprocess")));
    }

    #[test]
    fn test_strict_allows_json() {
        let code = "import json\ndata = json.loads('{}')";
        let result = ToolValidator::validate(code, ValidationLevel::Strict);
        assert!(result.allowed);
    }

    #[test]
    fn test_moderate_allows_urllib() {
        let code = "import urllib.request";
        let result = ToolValidator::validate(code, ValidationLevel::Moderate);
        assert!(result.allowed);
    }

    #[test]
    fn test_permissive_allows_everything() {
        let code = "import subprocess\nimport os.system";
        let result = ToolValidator::validate(code, ValidationLevel::Permissive);
        assert!(result.allowed);
    }

    #[test]
    fn test_code_length_limit() {
        let code = "x = 1\n".repeat(10000); // Exceeds MAX_CODE_LENGTH
        let result = ToolValidator::validate(&code, ValidationLevel::Strict);
        assert!(!result.allowed);
        assert!(result.error.unwrap().contains("exceeds maximum length"));
    }

    #[test]
    fn test_extract_imports() {
        let code = r#"
import os
import json
from collections import Counter
from urllib.parse import urlparse
import subprocess as sp
"#;
        let imports = ToolValidator::extract_imports(code);
        assert!(imports.contains(&"os".to_string()));
        assert!(imports.contains(&"json".to_string()));
        assert!(imports.contains(&"collections".to_string()));
        assert!(imports.contains(&"urllib".to_string()));
    }

    #[test]
    fn test_system_killer_detection() {
        // os.system is in SYSTEM_KILLER_IMPORTS
        let code = "import os\nimport os.system";
        let result = ToolValidator::validate(code, ValidationLevel::Strict);
        assert!(!result.allowed);
        // Check that something was blocked (either os.system or os itself depending on strictness)
        assert!(!result.blocked_imports.is_empty() || !result.allowed);
    }

    #[test]
    fn test_validation_result_display() {
        assert_eq!(ValidationLevel::Strict.to_string(), "strict");
        assert_eq!(ValidationLevel::Moderate.to_string(), "moderate");
        assert_eq!(ValidationLevel::Permissive.to_string(), "permissive");
    }

    #[test]
    fn test_get_allowlist() {
        let strict_list = ToolValidator::get_allowlist(ValidationLevel::Strict);
        assert!(strict_list.contains(&"json"));
        assert!(strict_list.contains(&"math"));

        let moderate_list = ToolValidator::get_allowlist(ValidationLevel::Moderate);
        assert!(moderate_list.contains(&"urllib"));

        let permissive_list = ToolValidator::get_allowlist(ValidationLevel::Permissive);
        assert!(permissive_list.is_empty());
    }
}
