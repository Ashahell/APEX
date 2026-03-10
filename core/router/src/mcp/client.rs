use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command as AsyncCommand;
use std::process::Stdio;

use crate::mcp::types::*;

/// Response sender type - maps request ID to response channel
type ResponseSender = mpsc::Sender<Result<serde_json::Value, String>>;

/// Configuration for MCP client retry behavior
#[derive(Clone)]
pub struct McpClientConfig {
    /// Maximum number of retry attempts for connection
    pub max_retries: u32,
    /// Initial delay between retries in milliseconds
    pub retry_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_retry_delay_ms: u64,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Enable auto-reconnect on connection loss
    pub auto_reconnect: bool,
    /// Maximum auto-reconnect attempts (0 = unlimited)
    pub max_auto_reconnect: u32,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 500,
            max_retry_delay_ms: 5000,
            request_timeout_secs: 30,
            auto_reconnect: true,
            max_auto_reconnect: 0, // 0 = unlimited
            health_check_interval_secs: 30,
        }
    }
}

/// Connection state for tracking reconnection
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error(String),
}

/// MCP Client implementing JSON-RPC communication over stdio with full bidirectional communication
pub struct McpClient {
    server_id: String,
    child: Option<tokio::process::Child>,
    stdin: Option<mpsc::Sender<String>>,
    
    // Response tracking
    responses: Arc<RwLock<HashMap<u64, ResponseSender>>>,
    next_id: u64,
    
    // Connection state
    connection_state: ConnectionState,
    reconnect_attempts: u32,
    last_error: Option<String>,
    protocol_version: Option<String>,
    server_info: Option<McpServerInfo>,
    tools: Vec<McpToolDefinition>,
    
    // Server connection details for reconnection
    reconnect_command: Option<String>,
    reconnect_args: Option<Vec<String>>,
    reconnect_env: Option<HashMap<String, String>>,
    
    // Retry configuration
    config: McpClientConfig,
}

impl McpClient {
    pub fn new(server_id: String) -> Self {
        Self {
            server_id,
            child: None,
            stdin: None,
            responses: Arc::new(RwLock::new(HashMap::new())),
            next_id: 1,
            connection_state: ConnectionState::Disconnected,
            reconnect_attempts: 0,
            last_error: None,
            protocol_version: None,
            server_info: None,
            tools: Vec::new(),
            reconnect_command: None,
            reconnect_args: None,
            reconnect_env: None,
            config: McpClientConfig::default(),
        }
    }

    /// Create a new client with custom configuration
    pub fn with_config(server_id: String, config: McpClientConfig) -> Self {
        Self {
            server_id,
            child: None,
            stdin: None,
            responses: Arc::new(RwLock::new(HashMap::new())),
            next_id: 1,
            connection_state: ConnectionState::Disconnected,
            reconnect_attempts: 0,
            last_error: None,
            protocol_version: None,
            server_info: None,
            tools: Vec::new(),
            reconnect_command: None,
            reconnect_args: None,
            reconnect_env: None,
            config,
        }
    }

    /// Set retry configuration
    pub fn set_config(&mut self, config: McpClientConfig) {
        self.config = config;
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(attempt: u32, config: &McpClientConfig) -> u64 {
        let delay = config.retry_delay_ms * (2_u64.pow(attempt.min(5)));
        delay.min(config.max_retry_delay_ms)
    }

    /// Connect and spawn the MCP server process with async stdio and retry logic
    pub async fn connect(
        &mut self,
        command: &str,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> Result<(), String> {
        if self.child.is_some() && self.connection_state == ConnectionState::Connected {
            return Err("Already connected".to_string());
        }

        // Store connection details for auto-reconnect
        self.store_connection_details(command.to_string(), args.clone(), env.clone());
        
        let mut last_error = String::new();
        
        // Set connecting state
        self.connection_state = ConnectionState::Connecting;
        
        // Retry loop with exponential backoff
        for attempt in 0..self.config.max_retries {
            // Clean up any previous attempt
            if let Some(mut child) = self.child.take() {
                let _ = child.kill();
            }
            self.stdin = None;

            // Attempt to connect
            match self.try_connect(command, args.clone(), env.clone()).await {
                Ok(()) => {
                    self.connection_state = ConnectionState::Connected;
                    self.reconnect_attempts = 0;
                    self.last_error = None;
                    tracing::info!("MCP client '{}' connected on attempt {}", self.server_id, attempt + 1);
                    return Ok(());
                }
                Err(e) => {
                    last_error = e;
                    self.connection_state = ConnectionState::Error(last_error.clone());
                    self.last_error = Some(last_error.clone());
                    tracing::warn!("MCP client '{}' connection attempt {} failed: {}", self.server_id, attempt + 1, last_error);
                    
                    // Don't retry if this was the last attempt
                    if attempt + 1 >= self.config.max_retries {
                        break;
                    }
                    
                    // Calculate delay with exponential backoff
                    let delay_ms = Self::calculate_retry_delay(attempt, &self.config);
                    tracing::info!("MCP client '{}' retrying in {}ms", self.server_id, delay_ms);
                    time::sleep(time::Duration::from_millis(delay_ms)).await;
                }
            }
        }

        Err(format!("Failed to connect after {} attempts: {}", self.config.max_retries, last_error))
    }

    /// Internal connect attempt (no retry)
    async fn try_connect(
        &mut self,
        command: &str,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> Result<(), String> {
        // Spawn the MCP server process with piped stdin/stdout
        let mut cmd = AsyncCommand::new(command);
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped());

        for (key, value) in env {
            cmd.env(key, value);
        }

        // Spawn the child process
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP server: {}", e))?;

        // Take ownership of stdin and stdout
        let stdin = child.stdin.take()
            .ok_or("Failed to capture stdin")?;
        let stdout = child.stdout.take()
            .ok_or("Failed to capture stdout")?;

        // Create channel for stdin communication
        let (tx, mut rx) = mpsc::channel::<String>(100);
        
        // Clone for stdin writer task
        let server_id = self.server_id.clone();
        
        // Spawn task to write to stdin
        tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(line) = rx.recv().await {
                tracing::debug!("MCP stdin: {}", line);
                if let Err(e) = stdin.write_all(format!("{}\n", line).as_bytes()).await {
                    tracing::error!("Failed to write to MCP stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    tracing::error!("Failed to flush MCP stdin: {}", e);
                    break;
                }
            }
            tracing::info!("MCP server '{}' stdin closed", server_id);
        });

        // Create response map for the reader task
        let responses = self.responses.clone();
        let reader_server_id = self.server_id.clone();
        
        // Spawn task to read from stdout and dispatch responses
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                tracing::debug!("MCP stdout: {}", line);
                
                // Parse the response
                match serde_json::from_str::<serde_json::Value>(&line) {
                    Ok(json) => {
                        // Extract request ID
                        if let Some(id) = json.get("id").and_then(|id| id.as_u64()) {
                            // Find and send to the waiting channel
                            let mut responses_guard = responses.write().await;
                            if let Some(sender) = responses_guard.remove(&id) {
                                let _ = sender.send(Ok(json)).await;
                            }
                        } else if let Some(method) = json.get("method").and_then(|m| m.as_str()) {
                            // Handle notifications (no id)
                            tracing::debug!("MCP notification: {}", method);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse MCP response: {}", e);
                    }
                }
            }
            tracing::info!("MCP server '{}' stdout closed", reader_server_id);
        });

        // Store the child process and communication channels
        self.child = Some(child);
        self.stdin = Some(tx);
        
        tracing::info!("MCP client '{}' connected to server", self.server_id);
        Ok(())
    }

    /// Send a JSON-RPC request and wait for response
    async fn send_request<T: serde::Serialize>(&mut self, method: &str, params: T) -> Result<serde_json::Value, String> {
        let tx = self.stdin.as_ref()
            .ok_or("Not connected")?;
        
        let id = self.next_id;
        self.next_id += 1;

        // Create response channel
        let (response_tx, mut response_rx) = mpsc::channel::<Result<serde_json::Value, String>>(1);
        
        // Register this request ID
        {
            let mut responses = self.responses.write().await;
            responses.insert(id, response_tx);
        }

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        let request_str = request.to_string();
        tracing::debug!("MCP request: {}", request_str);

        tx.send(request_str).await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        // Wait for response with timeout
        let timeout = time::Duration::from_secs(self.config.request_timeout_secs);
        let result = match time::timeout(timeout, response_rx.recv()).await {
            Ok(Some(Ok(result))) => result,
            Ok(Some(Err(e))) => return Err(format!("Failed to receive response: {}", e)),
            Ok(None) => return Err("Channel closed".to_string()),
            Err(_) => return Err("Request timeout".to_string()),
        };

        // Check for error response
        if let Some(error) = result.get("error") {
            return Err(format!("JSON-RPC error: {:?}", error));
        }

        // Extract result
        result.get("result")
            .cloned()
            .ok_or_else(|| "No result in response".to_string())
    }

    /// Initialize the MCP connection with proper handshake
    pub async fn initialize(&mut self) -> Result<(), String> {
        if self.child.is_none() {
            return Err("Not connected".to_string());
        }

        // Send initialize request and get response
        let request = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "apex-router",
                "version": "1.3.0"
            }
        });

        let response = self.send_request("initialize", request).await?;
        
        // Parse server info from response
        if let Some(server_info) = response.get("serverInfo") {
            self.server_info = serde_json::from_value(server_info.clone()).ok();
        }
        
        if let Some(protocol_version) = response.get("protocolVersion").and_then(|v| v.as_str()) {
            self.protocol_version = Some(protocol_version.to_string());
        }

        // Send initialized notification
        let tx = self.stdin.as_ref()
            .ok_or("Not connected")?;
        
        let initialized_notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        
            tx.send(initialized_notification.to_string()).await
            .map_err(|e| format!("Failed to send initialized notification: {}", e))?;

        self.connection_state = ConnectionState::Connected;
        
        tracing::info!("MCP client '{}' initialized", self.server_id);
        Ok(())
    }

    /// List available tools from the MCP server
    pub async fn list_tools(&mut self) -> Result<Vec<McpToolDefinition>, String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not initialized".to_string());
        }

        // Send tools/list request and get response
        let response = self.send_request("tools/list", serde_json::json!({})).await?;
        
        // Parse tools from response
        let tools: Vec<McpToolDefinition> = serde_json::from_value(response.clone())
            .map_err(|e| format!("Failed to parse tools: {}", e))?;
        
        self.tools = tools.clone();
        
        tracing::debug!("MCP client '{}' listed {} tools", self.server_id, tools.len());
        Ok(tools)
    }

    /// Call a tool on the MCP server
    pub async fn call_tool(
        &mut self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<McpToolResult, String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not initialized".to_string());
        }

        // Send tools/call request and get response
        let call_params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });
        
        let response = self.send_request("tools/call", call_params).await?;
        
        // Parse result as McpToolResult
        let result: McpToolResult = serde_json::from_value(response)
            .map_err(|e| format!("Failed to parse tool result: {}", e))?;

        tracing::info!("MCP tool '{}' executed, success: {}", name, result.success);
        Ok(result)
    }

    /// Disconnect from the MCP server
    pub fn disconnect(&mut self) {
        // Note: We can't safely clear the response map here without blocking
        // The channels will be dropped when the client is dropped
        
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
        self.stdin = None;
        self.connection_state = ConnectionState::Disconnected;
        self.tools.clear();
        self.protocol_version = None;
        self.server_info = None;
        self.last_error = None;
        tracing::info!("MCP client '{}' disconnected", self.server_id);
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connection_state == ConnectionState::Connected
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.connection_state == ConnectionState::Connected
    }

    /// Get cached tools
    pub fn get_tools(&self) -> &[McpToolDefinition] {
        &self.tools
    }

    /// Set tools directly (for testing or manual population)
    pub fn set_tools(&mut self, tools: Vec<McpToolDefinition>) {
        self.tools = tools;
    }

    /// Get server info
    pub fn get_server_info(&self) -> Option<&McpServerInfo> {
        self.server_info.as_ref()
    }

    /// Get protocol version
    pub fn get_protocol_version(&self) -> Option<&str> {
        self.protocol_version.as_deref()
    }

    /// Get connection state
    pub fn get_connection_state(&self) -> &ConnectionState {
        &self.connection_state
    }

    /// Get last error
    pub fn get_last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Get reconnect attempts count
    pub fn get_reconnect_attempts(&self) -> u32 {
        self.reconnect_attempts
    }

    /// Store connection details for auto-reconnect
    pub fn store_connection_details(&mut self, command: String, args: Vec<String>, env: HashMap<String, String>) {
        self.reconnect_command = Some(command);
        self.reconnect_args = Some(args);
        self.reconnect_env = Some(env);
    }

    /// Attempt auto-reconnect with stored connection details
    pub async fn auto_reconnect(&mut self) -> Result<(), String> {
        if !self.config.auto_reconnect {
            return Err("Auto-reconnect is disabled".to_string());
        }

        let command = self.reconnect_command.clone()
            .ok_or("No connection details stored for reconnect")?;
        let args = self.reconnect_args.clone()
            .ok_or("No connection details stored for reconnect")?;
        let env = self.reconnect_env.clone()
            .ok_or("No connection details stored for reconnect")?;

        // Check max reconnect attempts
        if self.config.max_auto_reconnect > 0 && self.reconnect_attempts >= self.config.max_auto_reconnect {
            return Err(format!("Max auto-reconnect attempts ({}) reached", self.config.max_auto_reconnect));
        }

        self.connection_state = ConnectionState::Reconnecting;
        self.reconnect_attempts += 1;

        tracing::info!("MCP client '{}' attempting auto-reconnect (attempt {})", 
            self.server_id, self.reconnect_attempts);

        // Attempt to reconnect with retries
        let result = self.connect(&command, args, env).await;
        
        if result.is_ok() {
            // Re-initialize
            if let Err(e) = self.initialize().await {
                let err_str = format!("Re-initialize failed: {}", e);
                self.last_error = Some(err_str.clone());
                self.connection_state = ConnectionState::Error(err_str.clone());
                return Err(err_str);
            }
            
            // Clear tools cache after reconnect
            self.tools.clear();
            
            tracing::info!("MCP client '{}' auto-reconnected successfully", self.server_id);
        } else {
            let error = result.as_ref().unwrap_err();
            let error_str = error.to_string();
            self.last_error = Some(error_str.clone());
            let error_for_state = error_str.clone();
            self.connection_state = ConnectionState::Error(error_for_state);
            return Err(error_str);
        }

        Ok(())
    }

    // =============================================================================
    // MCP Resources Protocol
    // =============================================================================

    /// List available resources from the MCP server
    pub async fn list_resources(&mut self) -> Result<Vec<McpResource>, String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }

        let response = self.send_request("resources/list", serde_json::json!({})).await?;
        
        let resources: Vec<McpResource> = serde_json::from_value(response)
            .map_err(|e| format!("Failed to parse resources: {}", e))?;
        
        tracing::debug!("MCP client '{}' listed {} resources", self.server_id, resources.len());
        Ok(resources)
    }

    /// Read a specific resource
    pub async fn read_resource(&mut self, uri: &str) -> Result<Vec<McpResourceContent>, String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }

        let response = self.send_request("resources/read", serde_json::json!({ "uri": uri })).await?;
        
        let contents: Vec<McpResourceContent> = serde_json::from_value(response)
            .map_err(|e| format!("Failed to parse resource content: {}", e))?;
        
        tracing::debug!("MCP client '{}' read resource '{}'", self.server_id, uri);
        Ok(contents)
    }

    /// Subscribe to resource updates
    pub async fn subscribe_resource(&mut self, uri: &str) -> Result<(), String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }

        self.send_request("resources/subscribe", serde_json::json!({ "uri": uri })).await?;
        
        tracing::debug!("MCP client '{}' subscribed to resource '{}'", self.server_id, uri);
        Ok(())
    }

    /// Unsubscribe from resource updates
    pub async fn unsubscribe_resource(&mut self, uri: &str) -> Result<(), String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }

        self.send_request("resources/unsubscribe", serde_json::json!({ "uri": uri })).await?;
        
        tracing::debug!("MCP client '{}' unsubscribed from resource '{}'", self.server_id, uri);
        Ok(())
    }

    // =============================================================================
    // MCP Prompts Protocol
    // =============================================================================

    /// List available prompts from the MCP server
    pub async fn list_prompts(&mut self) -> Result<Vec<McpPrompt>, String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }

        let response = self.send_request("prompts/list", serde_json::json!({})).await?;
        
        let prompts: Vec<McpPrompt> = serde_json::from_value(response)
            .map_err(|e| format!("Failed to parse prompts: {}", e))?;
        
        tracing::debug!("MCP client '{}' listed {} prompts", self.server_id, prompts.len());
        Ok(prompts)
    }

    /// Get a specific prompt with arguments
    pub async fn get_prompt(
        &mut self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<Vec<McpPromptMessage>, String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }

        let params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });
        
        let response = self.send_request("prompts/get", params).await?;
        
        let prompt_response: McpPromptGetResponse = serde_json::from_value(response)
            .map_err(|e| format!("Failed to parse prompt: {}", e))?;
        
        tracing::debug!("MCP client '{}' got prompt '{}'", self.server_id, name);
        Ok(prompt_response.messages)
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}

/// Handle type for sharing MCP client across tasks
pub type McpClientHandle = Arc<Mutex<McpClient>>;

/// Create a new MCP client handle
pub fn create_client(server_id: String) -> McpClientHandle {
    Arc::new(Mutex::new(McpClient::new(server_id)))
}
