use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use ulid::Ulid;

use crate::mcp::types::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// In-memory registries: registry_id -> list of tools
lazy_static! {
    pub static ref REGISTRIES: RwLock<HashMap<String, Vec<McpToolDefinition>>> = {
        let m = HashMap::new();
        RwLock::new(m)
    };
}

lazy_static! {
    pub static ref REGISTRY_NAMES: RwLock<HashMap<String, String>> = {
        let m = HashMap::new();
        RwLock::new(m)
    };
}

pub fn create_registry(name: String) -> String {
    let id = Ulid::new().to_string();
    REGISTRY_NAMES.write().unwrap().insert(id.clone(), name);
    REGISTRIES.write().unwrap().insert(id.clone(), Vec::new());
    id
}

pub fn list_registries() -> Vec<(String, String)> {
    let names = REGISTRY_NAMES.read().unwrap();
    names
        .iter()
        .map(|(id, name)| (id.clone(), name.clone()))
        .collect()
}

pub fn add_tool_to_registry(registry_id: &str, tool: McpToolDefinition) {
    let mut reg = REGISTRIES.write().unwrap();
    reg.entry(registry_id.to_string())
        .or_insert_with(Vec::new)
        .push(tool);
}

pub fn list_tools_in_registry(registry_id: &str) -> Vec<McpToolDefinition> {
    REGISTRIES
        .read()
        .unwrap()
        .get(registry_id)
        .cloned()
        .unwrap_or_else(|| Vec::new())
}

pub fn discover_tools_in_registry(registry_id: &str) -> Vec<McpToolDefinition> {
    // Simulated discovery: add two dummy tools if registry is empty
    let mut existing = list_tools_in_registry(registry_id);
    if existing.is_empty() {
        existing.push(McpToolDefinition {
            name: "dynamic_tool_1".to_string(),
            description: Some("Discovered at runtime".to_string()),
            input_schema: serde_json::json!({"type": "object", "properties": {"input": {"type": "string"}}}),
        });
        existing.push(McpToolDefinition {
            name: "dynamic_tool_2".to_string(),
            description: Some("Another discovered tool".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
        });
        // Persist discovered tools to the registry
        REGISTRIES
            .write()
            .unwrap()
            .insert(registry_id.to_string(), existing.clone());
    }
    existing
}
