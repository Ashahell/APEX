use apex_router::mcp::validation::{validate_registry_input, validate_tool_input};
use serde_json::json;

#[test]
fn test_validate_registry_input_valid() {
    assert!(validate_registry_input("DeterministicRegistry").is_ok());
}

#[test]
fn test_validate_registry_input_invalid() {
    let res = validate_registry_input("");
    assert!(res.is_err());
}

#[test]
fn test_validate_tool_input_valid() {
    let schema = json!({"type": "object", "properties": {"name": {"type": "string"}}});
    assert!(validate_tool_input("tool1", &schema).is_ok());
}

#[test]
fn test_validate_tool_input_invalid_name() {
    let schema = json!({"type": "object"});
    let res = validate_tool_input("", &schema);
    assert!(res.is_err());
}

#[test]
fn test_validate_tool_input_invalid_schema() {
    let schema = json!("not-an-object");
    let res = validate_tool_input("tool2", &schema);
    assert!(res.is_err());
}
