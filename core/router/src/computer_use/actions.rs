use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyModifier {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputerAction {
    Click { x: i32, y: i32, button: MouseButton },
    DoubleClick { x: i32, y: i32, button: MouseButton },
    RightClick { x: i32, y: i32 },
    Hover { x: i32, y: i32 },
    Drag { from_x: i32, from_y: i32, to_x: i32, to_y: i32 },
    Type { text: String },
    KeyPress { key: String, modifiers: Vec<KeyModifier> },
    HotKey { keys: Vec<String> },
    Scroll { x: i32, y: i32, delta_x: i32, delta_y: i32 },
    Wait { duration_ms: u64 },
    Screenshot,
    Bash { command: String },
    ReadFile { path: String },
    WriteFile { path: String, content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub ok: bool,
    pub cost: f64,
    pub is_completion: bool,
    pub state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub action: ComputerAction,
    pub result: ActionResult,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionExecutor {
    // In a real implementation this would hold a VM/runner context
}

impl ActionExecutor {
    pub async fn execute(&self, _action: ComputerAction) -> Result<ActionResult, ActionError> {
        // Minimal stub: pretend action succeeded with zero cost
        Ok(ActionResult { ok: true, cost: 0.0, is_completion: false, state: None })
    }
}
