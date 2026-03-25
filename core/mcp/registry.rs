use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

#[derive(Clone, Debug)]
pub struct ToolSpec {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Debug)]
pub struct ToolEntry {
    pub spec: ToolSpec,
    pub enabled: bool,
}

lazy_static! {
    pub static ref REGISTRY: Mutex<HashMap<String, ToolEntry>> = Mutex::new(HashMap::new());
}

pub fn register_tool(name: &str, version: &str, description: &str) {
    let mut reg = REGISTRY.lock().unwrap();
    reg.insert(
        name.to_string(),
        ToolEntry {
            spec: ToolSpec {
                name,
                version,
                description,
            },
            enabled: true,
        },
    );
}

pub fn list_tools() -> Vec<ToolEntry> {
    REGISTRY.lock().unwrap().values().cloned().collect()
}

pub fn bootstrap_two_tools() {
    register_tool("computer-use", "0.1.0", "MVP Computer Use hand/tool");
    register_tool("browser", "0.1.0", "Browser automation tool for Hands");
}
