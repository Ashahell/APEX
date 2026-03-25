// Extended MCP protocol extension (minimal, for MVP)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpMessage {
    Discover { name: String },
    DiscoverResp { name: String, available: bool },
    Invoke { tool: String, payload: Vec<u8> },
    Response { tool: String, payload: Vec<u8> },
}

pub trait McpClient {
    fn send(&self, msg: McpMessage) -> Result<(), String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpProtocol {
    // Placeholder for protocol state
}

impl McpProtocol {
    pub fn new() -> Self {
        McpProtocol {}
    }
    pub fn discover_tools(&self) -> Vec<String> {
        vec!["computer-use".to_string(), "browser".to_string()]
    }
}
