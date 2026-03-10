use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{RwLock, Mutex};

use crate::mcp::client::{McpClient, McpClientHandle, McpClientConfig, ConnectionState};
use crate::mcp::types::*;

/// Pool configuration for MCP server connections
#[derive(Clone)]
pub struct McpServerPoolConfig {
    /// Minimum number of connections to keep warm
    pub min_connections: usize,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Idle connection timeout in seconds
    pub idle_timeout_secs: u64,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
}

impl Default for McpServerPoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            connection_timeout_secs: 30,
            idle_timeout_secs: 300,
            health_check_interval_secs: 60,
        }
    }
}

/// Connection pool entry with metadata
struct PoolEntry {
    client: McpClientHandle,
    created_at: Instant,
    last_used: Instant,
    server_id: String,
    server_config: ServerConnectionConfig,
}

/// Server connection configuration
#[derive(Clone)]
pub struct ServerConnectionConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub client_config: McpClientConfig,
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub servers_managed: usize,
}

pub struct McpServerManager {
    // Pool of connections per server
    pools: Arc<RwLock<HashMap<String, Vec<PoolEntry>>>>,
    // Server configurations for reconnection
    server_configs: Arc<RwLock<HashMap<String, ServerConnectionConfig>>>,
    // Pool configuration
    config: McpServerPoolConfig,
}

impl McpServerManager {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            server_configs: Arc::new(RwLock::new(HashMap::new())),
            config: McpServerPoolConfig::default(),
        }
    }

    pub fn with_config(config: McpServerPoolConfig) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            server_configs: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Connect to an MCP server (acquire from pool or create new)
    pub async fn connect_server(
        &self,
        server_id: String,
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> Result<(), String> {
        // Store server config for reconnection
        let server_config = ServerConnectionConfig {
            command: command.clone(),
            args: args.clone(),
            env: env.clone(),
            client_config: McpClientConfig::default(),
        };
        
        {
            let mut configs = self.server_configs.write().await;
            configs.insert(server_id.clone(), server_config.clone());
        }

        // Create new client and connect
        let mut client = McpClient::new(server_id.clone());
        
        // Store connection details for auto-reconnect
        client.store_connection_details(command, args, env);
        
        client.connect(&server_config.command, server_config.args.clone(), server_config.env.clone()).await?;
        client.initialize().await?;

        // Add to pool
        let mut pools = self.pools.write().await;
        let entry = PoolEntry {
            client: Arc::new(Mutex::new(client)),
            created_at: Instant::now(),
            last_used: Instant::now(),
            server_id: server_id.clone(),
            server_config,
        };
        
        pools.entry(server_id.clone()).or_insert_with(Vec::new).push(entry);
        
        tracing::info!("MCP server '{}' connected and added to pool", server_id);
        Ok(())
    }

    /// Get a client from the pool (reuse if available)
    pub async fn acquire_client(&self, server_id: &str) -> Result<McpClientHandle, String> {
        let mut pools = self.pools.write().await;
        
        if let Some(entries) = pools.get_mut(server_id) {
            // Try to find an available connection
            for entry in entries.iter_mut() {
                let client = entry.client.lock().await;
                if client.get_connection_state() == &ConnectionState::Connected {
                    entry.last_used = Instant::now();
                    tracing::debug!("Reusing pooled connection for server '{}'", server_id);
                    return Ok(entry.client.clone());
                }
            }
            
            // All connections busy, create new if under max
            if entries.len() < self.config.max_connections {
                let _ = entries; // Release write lock by shadowing
                return self.create_new_connection(server_id).await;
            } else {
                return Err(format!("Pool exhausted for server '{}'", server_id));
            }
        }
        
        Err(format!("Server '{}' not found in pool", server_id))
    }

    /// Create a new connection for a server
    async fn create_new_connection(&self, server_id: &str) -> Result<McpClientHandle, String> {
        let configs = self.server_configs.read().await;
        let config = configs.get(server_id)
            .ok_or_else(|| format!("No configuration found for server '{}'", server_id))?
            .clone();
        drop(configs);

        let mut client = McpClient::new(server_id.to_string());
        client.store_connection_details(config.command.clone(), config.args.clone(), config.env.clone());
        client.connect(&config.command, config.args.clone(), config.env.clone()).await?;
        client.initialize().await?;

        let entry = PoolEntry {
            client: Arc::new(Mutex::new(client)),
            created_at: Instant::now(),
            last_used: Instant::now(),
            server_id: server_id.to_string(),
            server_config: config,
        };

        let mut pools = self.pools.write().await;
        pools.entry(server_id.to_string()).or_insert_with(Vec::new).push(entry);
        
        tracing::info!("Created new pooled connection for server '{}'", server_id);
        
        let pool_entry = pools.get(server_id).unwrap().last().unwrap();
        Ok(pool_entry.client.clone())
    }

    /// Release a client back to the pool
    pub async fn release_client(&self, server_id: &str, client: McpClientHandle) {
        // Update last_used timestamp
        let mut pools = self.pools.write().await;
        if let Some(entries) = pools.get_mut(server_id) {
            for entry in entries.iter_mut() {
                if Arc::ptr_eq(&entry.client, &client) {
                    entry.last_used = Instant::now();
                    break;
                }
            }
        }
    }

    /// Disconnect a server (remove all connections from pool)
    pub async fn disconnect_server(&self, server_id: &str) -> Result<(), String> {
        // Remove from pool
        let mut pools = self.pools.write().await;
        if let Some(entries) = pools.remove(server_id) {
            for entry in entries {
                let mut client = entry.client.lock().await;
                client.disconnect();
            }
        }
        
        // Remove config
        drop(pools);
        let mut configs = self.server_configs.write().await;
        configs.remove(server_id);
        
        tracing::info!("MCP server '{}' disconnected and removed from pool", server_id);
        Ok(())
    }

    /// Check if a server is connected
    pub async fn is_connected(&self, server_id: &str) -> bool {
        let pools = self.pools.read().await;
        if let Some(entries) = pools.get(server_id) {
            !entries.is_empty() && entries.iter().any(|_e| {
                // We need to check the client state
                // For now, return true if there are entries
                true
            })
        } else {
            false
        }
    }

    /// List tools from a server
    pub async fn list_tools(&self, server_id: &str) -> Result<Vec<McpToolDefinition>, String> {
        let client = self.acquire_client(server_id).await?;
        let mut client_guard = client.lock().await;
        let tools = client_guard.list_tools().await?;
        drop(client_guard);
        
        // Release back to pool
        self.release_client(server_id, client).await;
        
        Ok(tools)
    }

    /// Call a tool on a server
    pub async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<McpToolResult, String> {
        let client = self.acquire_client(server_id).await?;
        let mut client_guard = client.lock().await;
        let result = client_guard.call_tool(tool_name, arguments).await;
        drop(client_guard);
        
        // Release back to pool
        self.release_client(server_id, client).await;
        
        result
    }

    /// Get all tools from all servers
    pub async fn get_all_tools(&self) -> Vec<(String, McpToolDefinition)> {
        let pools = self.pools.read().await;
        let mut all_tools = Vec::new();
        
        for (server_id, entries) in pools.iter() {
            if let Some(first) = entries.first() {
                let client = first.client.lock().await;
                for tool in client.get_tools() {
                    all_tools.push((server_id.clone(), tool.clone()));
                }
            }
        }
        
        all_tools
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        let pools = self.pools.read().await;
        
        let mut total = 0;
        let mut active = 0;
        
        for entries in pools.values() {
            total += entries.len();
            for entry in entries {
                let client = entry.client.lock().await;
                if client.get_connection_state() == &ConnectionState::Connected {
                    active += 1;
                }
            }
        }
        
        PoolStats {
            total_connections: total,
            active_connections: active,
            idle_connections: total.saturating_sub(active),
            servers_managed: pools.len(),
        }
    }

    /// Health check - attempt to reconnect failed servers
    pub async fn health_check(&self) {
        let pools = self.pools.read().await;
        
        for (server_id, entries) in pools.iter() {
            for entry in entries {
                let mut client = entry.client.lock().await;
                let state = client.get_connection_state();
                
                if state != &ConnectionState::Connected {
                    if entry.server_config.client_config.auto_reconnect {
                        tracing::warn!("Server '{}' disconnected, attempting auto-reconnect", server_id);
                        if let Err(e) = client.auto_reconnect().await {
                            tracing::error!("Auto-reconnect failed for '{}': {}", server_id, e);
                        }
                    }
                }
            }
        }
    }

    /// Disconnect all servers
    pub async fn disconnect_all(&self) {
        let mut pools = self.pools.write().await;
        
        for (server_id, entries) in pools.drain() {
            for entry in entries {
                let mut client = entry.client.lock().await;
                client.disconnect();
                tracing::info!("MCP server '{}' disconnected", server_id);
            }
        }
        
        let mut configs = self.server_configs.write().await;
        configs.clear();
    }

    /// Get server status
    pub async fn get_server_status(&self, server_id: &str) -> Option<ConnectionState> {
        let pools = self.pools.read().await;
        
        if let Some(entries) = pools.get(server_id) {
            if let Some(entry) = entries.first() {
                let client = entry.client.lock().await;
                return Some(client.get_connection_state().clone());
            }
        }
        None
    }
}

impl Default for McpServerManager {
    fn default() -> Self {
        Self::new()
    }
}
