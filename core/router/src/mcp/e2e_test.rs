#[cfg(test)]
mod e2e_tests {
    use std::path::PathBuf;
    use std::collections::HashMap;
    
    /// Find the test MCP server script
    fn get_test_server_path() -> PathBuf {
        // Try multiple possible locations
        let candidates = vec![
            PathBuf::from("E:/projects/APEX/test-mcp-server/server.js"),
            PathBuf::from("./test-mcp-server/server.js"),
            PathBuf::from("../test-mcp-server/server.js"),
        ];
        
        for path in &candidates {
            if path.exists() {
                return path.clone();
            }
        }
        
        // Return the first candidate as fallback
        candidates[0].clone()
    }

    #[tokio::test]
    async fn test_mcp_e2e_full_flow() {
        // Skip if test server doesn't exist
        let server_path = get_test_server_path();
        if !server_path.exists() {
            println!("Test server not found at {:?}, skipping E2E test", server_path);
            return;
        }

        let mut client = crate::mcp::McpClient::new("test-e2e".to_string());
        
        // Connect to the test server
        let connect_result = client.connect(
            "node",
            vec![server_path.to_string_lossy().to_string()],
            HashMap::new(),
        ).await;
        
        assert!(connect_result.is_ok(), "Failed to connect: {:?}", connect_result.err());
        assert!(client.is_connected(), "Client should be connected");
        
        // Initialize
        let init_result = client.initialize().await;
        assert!(init_result.is_ok(), "Failed to initialize: {:?}", init_result.err());
        assert!(client.is_initialized(), "Client should be initialized");
        
        // Check server info
        let server_info = client.get_server_info();
        assert!(server_info.is_some(), "Should have server info");
        if let Some(info) = server_info {
            println!("Server info: {} v{}", info.name, info.version);
        }
        
        // List tools
        let tools_result = client.list_tools().await;
        assert!(tools_result.is_ok(), "Failed to list tools: {:?}", tools_result.err());
        let tools = tools_result.unwrap();
        assert!(!tools.is_empty(), "Should have tools");
        
        println!("Found {} tools:", tools.len());
        for tool in &tools {
            println!("  - {}", tool.name);
        }
        
        // Test echo tool
        let echo_result = client.call_tool(
            "echo",
            serde_json::json!({"message": "Hello from test!"})
        ).await;
        assert!(echo_result.is_ok(), "Failed to call echo: {:?}", echo_result.err());
        let echo = echo_result.unwrap();
        assert!(echo.success, "Echo should succeed");
        assert!(echo.content.contains("Hello from test!"), "Should echo message");
        
        println!("Echo result: {}", echo.content);
        
        // Test add tool
        let add_result = client.call_tool(
            "add",
            serde_json::json!({"a": 5, "b": 3})
        ).await;
        assert!(add_result.is_ok(), "Failed to call add: {:?}", add_result.err());
        let add = add_result.unwrap();
        assert!(add.success, "Add should succeed");
        
        println!("Add result: {}", add.content);
        
        // Test get_time tool
        let time_result = client.call_tool(
            "get_time",
            serde_json::json!({})
        ).await;
        assert!(time_result.is_ok(), "Failed to call get_time: {:?}", time_result.err());
        let time = time_result.unwrap();
        assert!(time.success, "get_time should succeed");
        
        println!("Time result: {}", time.content);
        
        // Disconnect
        client.disconnect();
        assert!(!client.is_connected(), "Client should be disconnected");
        
        println!("E2E test completed successfully!");
    }

    #[tokio::test]
    async fn test_mcp_e2e_retry_on_failure() {
        use crate::mcp::client::{McpClient, McpClientConfig};
        
        // Configure with low retries for test
        let config = McpClientConfig {
            max_retries: 2,
            retry_delay_ms: 100,
            max_retry_delay_ms: 500,
            request_timeout_secs: 5,
            auto_reconnect: false,
            max_auto_reconnect: 0,
            health_check_interval_secs: 30,
        };
        
        let mut client = McpClient::with_config("test-retry".to_string(), config);
        
        // Try to connect to non-existent command - should fail after retries
        let result = client.connect(
            "nonexistent-mcp-server-xyz",
            vec![],
            HashMap::new(),
        ).await;
        
        // Should fail with error about spawn failure
        assert!(result.is_err(), "Should fail to connect");
        let err = result.unwrap_err();
        println!("Expected error: {}", err);
        assert!(!client.is_connected(), "Should not be connected");
    }
}
