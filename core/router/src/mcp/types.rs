use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
    pub enabled: bool,
}

impl McpServerConfig {
    pub fn new(id: String, name: String, command: String) -> Self {
        Self {
            id,
            name,
            command,
            args: Vec::new(),
            env: std::collections::HashMap::new(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpServerStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl Default for McpServerStatus {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl std::fmt::Display for McpServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpServerStatus::Disconnected => write!(f, "disconnected"),
            McpServerStatus::Connecting => write!(f, "connecting"),
            McpServerStatus::Connected => write!(f, "connected"),
            McpServerStatus::Error => write!(f, "error"),
        }
    }
}

impl From<&str> for McpServerStatus {
    fn from(s: &str) -> Self {
        match s {
            "connecting" => McpServerStatus::Connecting,
            "connected" => McpServerStatus::Connected,
            "error" => McpServerStatus::Error,
            _ => McpServerStatus::Disconnected,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub success: bool,
    pub content: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInitializeRequest {
    pub protocol_version: Option<String>,
    pub capabilities: serde_json::Value,
    pub client_info: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInitializeResponse {
    pub protocol_version: String,
    pub capabilities: McpCapabilities,
    pub server_info: McpServerInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpCapabilities {
    pub tools: Option<serde_json::Value>,
    pub resources: Option<serde_json::Value>,
    pub prompts: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum McpJsonRpcMessage {
    #[serde(rename = "initialize")]
    Initialize(McpJsonRpcRequest<McpInitializeRequest>),

    #[serde(rename = "initializeResponse")]
    InitializeResponse(McpJsonRpcResponse<McpInitializeResponse>),

    #[serde(rename = "tools/list")]
    ListTools,

    #[serde(rename = "tools/listResponse")]
    ListToolsResponse(McpJsonRpcResponse<Vec<McpToolDefinition>>),

    #[serde(rename = "tools/call")]
    CallTool(McpJsonRpcRequest<McpToolCall>),

    #[serde(rename = "tools/callResponse")]
    CallToolResponse(McpJsonRpcResponse<McpToolResult>),

    #[serde(rename = "notifications/initialized")]
    Initialized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpJsonRpcRequest<T> {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(flatten)]
    pub params: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpJsonRpcResponse<T> {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(flatten)]
    pub result: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpJsonRpcError {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub error: McpJsonRpcErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpJsonRpcErrorDetail {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

// =============================================================================
// MCP Resources Protocol
// =============================================================================

/// Resource definition from MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub blob: Option<String>,
}

/// Resource list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceList {
    pub resources: Vec<McpResource>,
}

/// Resource read request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceReadRequest {
    pub uri: String,
}

/// Resource subscription request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceSubscribeRequest {
    pub uri: String,
}

// =============================================================================
// MCP Prompts Protocol
// =============================================================================

/// Prompt definition from MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Option<Vec<McpPromptArgument>>,
}

/// Prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: Option<bool>,
}

/// Prompt message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptMessage {
    pub role: String,
    pub content: McpPromptContent,
}

/// Prompt content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

/// Prompt get request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptGetRequest {
    pub name: String,
    pub arguments: Option<serde_json::Value>,
}

/// Prompt list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptList {
    pub prompts: Vec<McpPrompt>,
}

/// Prompt get response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptGetResponse {
    pub messages: Vec<McpPromptMessage>,
}
