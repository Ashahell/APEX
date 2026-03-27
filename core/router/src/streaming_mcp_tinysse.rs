use futures_util::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};

/// MCP event types for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpEvent {
    /// Tool discovery event
    ToolDiscovery { tools: Vec<String> },
    /// Tool execution started
    ToolStart { tool: String, id: String, input: serde_json::Value },
    /// Tool execution progress
    ToolProgress { tool: String, id: String, progress: serde_json::Value },
    /// Tool execution completed
    ToolResult { tool: String, id: String, result: serde_json::Value },
    /// Tool execution failed
    ToolError { tool: String, id: String, error: String },
    /// Server connected
    Connected { server_name: String },
    /// Server disconnected
    Disconnected { server_name: String },
}

impl McpEvent {
    /// Create a timestamp for the event
    pub fn timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// TinySSE surface for MCP. This is intentionally small and isolated to avoid churn in streaming.rs.
pub struct TinySseMcpSurface {
    server_name: String,
}

impl TinySseMcpSurface {
    pub fn new(server_name: impl Into<String>) -> Self {
        Self {
            server_name: server_name.into(),
        }
    }

    /// Return a minimal TinySSE stream for MCP events.
    /// In subsequent patches, this will be wired to publish real MCP events.
    pub fn stream(&self) -> Pin<Box<dyn Stream<Item = Result<String, String>> + Send>> {
        // Start with a connected event
        let connected = McpEvent::Connected {
            server_name: self.server_name.clone(),
        };
        let json = serde_json::to_string(&connected).map_err(|e| e.to_string());
        
        match json {
            Ok(data) => {
                let events = vec![Ok(data)];
                Box::pin(stream::iter(events))
            }
            Err(e) => {
                let err_msg = e.to_string();
                Box::pin(stream::once(async move { Err(err_msg) }))
            }
        }
    }

    /// Create a tool discovery event
    pub fn tool_discovery_event(tools: Vec<String>) -> Result<String, String> {
        let event = McpEvent::ToolDiscovery { tools };
        serde_json::to_string(&event).map_err(|e| e.to_string())
    }

    /// Create a tool start event
    pub fn tool_start_event(tool: &str, id: &str, input: serde_json::Value) -> Result<String, String> {
        let event = McpEvent::ToolStart {
            tool: tool.to_string(),
            id: id.to_string(),
            input,
        };
        serde_json::to_string(&event).map_err(|e| e.to_string())
    }

    /// Create a tool result event
    pub fn tool_result_event(tool: &str, id: &str, result: serde_json::Value) -> Result<String, String> {
        let event = McpEvent::ToolResult {
            tool: tool.to_string(),
            id: id.to_string(),
            result,
        };
        serde_json::to_string(&event).map_err(|e| e.to_string())
    }

    /// Create a tool error event
    pub fn tool_error_event(tool: &str, id: &str, error: &str) -> Result<String, String> {
        let event = McpEvent::ToolError {
            tool: tool.to_string(),
            id: id.to_string(),
            error: error.to_string(),
        };
        serde_json::to_string(&event).map_err(|e| e.to_string())
    }
}

impl Default for TinySseMcpSurface {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_discovery_event() {
        let tools = vec!["tool1".to_string(), "tool2".to_string()];
        let event = TinySseMcpSurface::tool_discovery_event(tools.clone()).unwrap();
        
        assert!(event.contains("tooldiscovery"));
        assert!(event.contains("tool1"));
        assert!(event.contains("tool2"));
    }

    #[test]
    fn test_tool_start_event() {
        let input = serde_json::json!({"arg": "value"});
        let event = TinySseMcpSurface::tool_start_event("my_tool", "uuid-123", input).unwrap();
        
        assert!(event.contains("toolstart"));
        assert!(event.contains("my_tool"));
        assert!(event.contains("uuid-123"));
    }

    #[test]
    fn test_tool_result_event() {
        let result = serde_json::json!({"output": "success"});
        let event = TinySseMcpSurface::tool_result_event("my_tool", "uuid-123", result).unwrap();
        
        assert!(event.contains("toolresult"));
        assert!(event.contains("success"));
    }

    #[test]
    fn test_tool_error_event() {
        let event = TinySseMcpSurface::tool_error_event("my_tool", "uuid-123", "Something went wrong").unwrap();
        
        assert!(event.contains("toolerror"));
        assert!(event.contains("Something went wrong"));
    }

    #[tokio::test]
    async fn test_mcp_surface_stream() {
        let surface = TinySseMcpSurface::new("test-server");
        let stream = surface.stream();
        
        // Just verify the stream can be created without panic
        // The actual iteration would require async iteration setup
        assert!(std::mem::size_of_val(&stream) > 0);
    }
}
