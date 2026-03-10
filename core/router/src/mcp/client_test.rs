#[cfg(test)]
mod tests {
    use crate::mcp::client::McpClient;
    use crate::mcp::types::*;
    use std::collections::HashMap;

    #[test]
    fn test_mcp_client_creation() {
        let client = McpClient::new("test-server".to_string());
        assert!(!client.is_connected());
        assert!(client.get_tools().is_empty());
    }

    #[test]
    fn test_mcp_client_set_tools() {
        let mut client = McpClient::new("test-server".to_string());
        
        let tools = vec![
            McpToolDefinition {
                name: "test_tool".to_string(),
                description: Some("A test tool".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": { "type": "string" }
                    }
                }),
            },
        ];
        
        client.set_tools(tools.clone());
        assert_eq!(client.get_tools().len(), 1);
        assert_eq!(client.get_tools()[0].name, "test_tool");
    }

    #[test]
    fn test_mcp_types_serialization() {
        // Test JSON-RPC request serialization
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });
        
        let serialized = request.to_string();
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"method\":\"tools/list\""));
    }

    #[test]
    fn test_mcp_tool_definition() {
        let tool = McpToolDefinition {
            name: "echo".to_string(),
            description: Some("Echo back the input".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            }),
        };
        
        assert_eq!(tool.name, "echo");
        assert!(tool.description.is_some());
        assert!(tool.input_schema.is_object());
    }

    #[test]
    fn test_mcp_tool_result() {
        let result = McpToolResult {
            success: true,
            content: "Hello, world!".to_string(),
            error: None,
        };
        
        assert!(result.success);
        assert_eq!(result.content, "Hello, world!");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_mcp_tool_result_error() {
        let result = McpToolResult {
            success: false,
            content: String::new(),
            error: Some("Tool execution failed".to_string()),
        };
        
        assert!(!result.success);
        assert!(result.error.is_some());
        assert_eq!(result.error.as_ref().unwrap(), "Tool execution failed");
    }

    #[tokio::test]
    async fn test_mcp_client_connect_no_server() {
        let mut client = McpClient::new("test-server".to_string());
        
        // Try to connect to a non-existent command - should fail
        let result = client.connect(
            "nonexistent-command-xyz",
            vec![],
            HashMap::new(),
        ).await;
        
        // Connection should fail with error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mcp_client_initialize_without_connection() {
        let mut client = McpClient::new("test-server".to_string());
        
        // Try to initialize without connecting - should fail
        let result = client.initialize().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not connected"));
    }

    #[tokio::test]
    async fn test_mcp_client_list_tools_without_init() {
        let mut client = McpClient::new("test-server".to_string());
        
        // Try to list tools without initialization - should fail
        let result = client.list_tools().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not initialized"));
    }

    #[tokio::test]
    async fn test_mcp_client_call_tool_without_init() {
        let mut client = McpClient::new("test-server".to_string());
        
        // Try to call tool without initialization - should fail
        let result = client.call_tool(
            "test_tool",
            serde_json::json!({"input": "test"})
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not initialized"));
    }

    #[test]
    fn test_mcp_server_info() {
        let info = McpServerInfo {
            name: "test-mcp-server".to_string(),
            version: "1.0.0".to_string(),
        };
        
        assert_eq!(info.name, "test-mcp-server");
        assert_eq!(info.version, "1.0.0");
    }

    #[test]
    fn test_mcp_capabilities() {
        let capabilities = McpCapabilities {
            tools: Some(serde_json::json!({})),
            resources: Some(serde_json::json!({})),
            prompts: None,
        };
        
        assert!(capabilities.tools.is_some());
        assert!(capabilities.resources.is_some());
        assert!(capabilities.prompts.is_none());
    }
}
