# APEX MCP Implementation Plan

> **Date**: 2026-03-08
> **Status**: ✅ Complete

## Overview

MCP (Model Context Protocol) is an open standard for connecting AI agents to external tools and data sources. This plan implements MCP functionality in APEX similar to AgentZero.

## MCP Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        APEX Router                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐    │
│  │   MCP       │  │   MCP       │  │    Skill        │    │
│  │   Server    │  │   Client    │  │    Pool         │    │
│  │  Manager    │◄─┤  (stdio    │  │                 │    │
│  │ (Pooled)   │  │   JSON-RPC) │  │                 │    │
│  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘    │
│         │                 │                   │            │
│         │         ┌──────┴──────┐           │            │
│         │         │   Tool      │           │            │
│         │         │   Registry  │           │            │
│         └────────►│  (SQLite)  ◄───────────┘            │
│                   └─────────────┬──────────────────────┘    │
│                                 │                           │
│                    ┌────────────▼────────────┐              │
│                    │   Execution Pipeline   │              │
│                    │   (MCP tools mixed   │              │
│                    │    with regular skills)│              │
│                    └───────────────────────┘              │
│                                                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Message Bus (MCP Events)                │   │
│  │   server_connected | tool_started | tool_completed   │   │
│  └─────────────────────────────────────────────────────┘   │
```

## Implemented Components

### 1. Database Layer ✅

**Migration 015 - MCP Servers Table**
```sql
CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    args TEXT,
    env TEXT,
    enabled INTEGER DEFAULT 1,
    status TEXT DEFAULT 'disconnected',
    last_error TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE mcp_tools (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    input_schema TEXT,
    FOREIGN KEY (server_id) REFERENCES mcp_servers(id)
);
```

**Registry Tables**
```sql
CREATE TABLE mcp_registries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE mcp_tools_registry (
    id TEXT PRIMARY KEY,
    registry_id TEXT,
    name TEXT NOT NULL,
    description TEXT,
    input_schema TEXT
);
```

### 2. MCP Client ✅

**Features:**
- Async stdio communication using tokio
- JSON-RPC 2.0 protocol
- Bidirectional communication with response tracking
- Connection state management (Connected, Connecting, Disconnected, Reconnecting, Error)
- Auto-reconnect with exponential backoff

**Configuration:**
```rust
pub struct McpClientConfig {
    pub max_retries: u32,              // Default: 3
    pub retry_delay_ms: u64,           // Initial: 500ms
    pub max_retry_delay_ms: u64,      // Max: 5000ms
    pub request_timeout_secs: u64,      // Default: 30s
    pub auto_reconnect: bool,          // Enable auto-reconnect
    pub max_auto_reconnect: u32,       // 0 = unlimited
    pub health_check_interval_secs: u64, // Default: 30s
}
```

### 3. Connection Pooling ✅

**McpServerManager** provides connection pooling:
```rust
pub struct McpServerPoolConfig {
    pub min_connections: usize,        // Default: 1
    pub max_connections: usize,        // Default: 10
    pub connection_timeout_secs: u64,  // Default: 30s
    pub idle_timeout_secs: u64,       // Default: 300s
    pub health_check_interval_secs: u64, // Default: 60s
}
```

**Pool Features:**
- Connection reuse (acquire/release pattern)
- Server config storage for reconnection
- Health check on all pooled connections
- Statistics tracking (total, active, idle connections)

### 4. MCP Resources Protocol ✅

**Client Methods:**
- `list_resources()` - List available resources
- `read_resource(uri)` - Read specific resource content
- `subscribe_resource(uri)` - Subscribe to resource updates
- `unsubscribe_resource(uri)` - Unsubscribe from updates

**Types:**
```rust
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

pub struct McpResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub blob: Option<String>,
}
```

### 5. MCP Prompts Protocol ✅

**Client Methods:**
- `list_prompts()` - List available prompts
- `get_prompt(name, arguments)` - Get prompt with arguments

**Types:**
```rust
pub struct McpPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Option<Vec<McpPromptArgument>>,
}

pub struct McpPromptMessage {
    pub role: String,
    pub content: McpPromptContent,
}
```

### 6. Skill Integration ✅

MCP tools are exposed as skills in the execution pipeline:
- Tools available via `/api/v1/skills` endpoint
- Direct execution via `/api/v1/mcp/tools/execute`
- Tools named as `mcp:{server_id}:{tool_name}`

### 7. Security - Input Sanitization ✅

**Validation Functions:**
- `sanitize_tool_arguments()` - Sanitize JSON arguments
- `sanitize_tool_name()` - Validate tool name format
- `validate_server_command()` - Block dangerous commands

**Security Features:**
- Maximum nesting depth: 10 levels
- Maximum string length: 100KB
- Maximum object keys: 1000
- Maximum array length: 10000
- Blocked patterns: shell injection, path traversal, code execution

### 8. Real-time Events ✅

**McpMessage Types:**
```rust
pub enum McpMessage {
    ServerConnected { server_id, server_name },
    ServerDisconnected { server_id, reason },
    ServerError { server_id, error },
    ToolStarted { server_id, tool_name, task_id },
    ToolCompleted { server_id, tool_name, success, task_id, duration_ms },
    ToolFailed { server_id, tool_name, error, task_id },
    ToolsUpdated { server_id, tool_count },
}
```

### 9. Monitoring & Metrics ✅

**McpMetrics:**
```rust
pub struct McpMetricsSnapshot {
    pub servers_connected: u64,
    pub servers_disconnected: u64,
    pub tools_executed: u64,
    pub tools_failed: u64,
    pub connections_failed: u64,
    pub reconnections: u64,
    pub avg_tool_execution_time_ms: f64,
    pub total_tool_executions: u64,
}
```

## API Endpoints

### Server Management
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/mcp/servers` | List MCP servers |
| POST | `/api/v1/mcp/servers` | Add MCP server |
| GET | `/api/v1/mcp/servers/:id` | Get server details |
| PUT | `/api/v1/mcp/servers/:id` | Update server |
| DELETE | `/api/v1/mcp/servers/:id` | Remove server |
| POST | `/api/v1/mcp/servers/:id/connect` | Connect to server |
| POST | `/api/v1/mcp/servers/:id/disconnect` | Disconnect |

### Tool Execution
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/mcp/servers/:id/tools` | List available tools |
| POST | `/api/v1/mcp/servers/:id/tools/:tool_name` | Execute tool |

### MCP Tools (Global)
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/mcp/tools` | List all MCP tools from all servers |
| POST | `/api/v1/mcp/tools/execute` | Execute MCP tool directly |

### Registry (Dynamic Discovery)
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/mcp/registries` | List registries |
| POST | `/api/v1/mcp/registries` | Create registry |
| POST | `/api/v1/mcp/registries/validate` | Validate registry name |
| GET | `/api/v1/mcp/registries/:id/tools` | List tools in registry |
| POST | `/api/v1/mcp/registries/:id/discover` | Discover tools |

## Test Results

```
running 91 tests (lib)
running 2 tests (e2e)
running 59 tests (integration)
running 1 test (registry_db)
running 5 tests (validation)
Total: 158 tests passing
```

## Usage

### Adding an MCP Server

```bash
curl -X POST http://localhost:3000/api/v1/mcp/servers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "filesystem",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/dir"]
  }'
```

### Connecting to Server

```bash
curl -X POST http://localhost:3000/api/v1/mcp/servers/{id}/connect
```

### Listing Tools

```bash
curl http://localhost:3000/api/v1/mcp/servers/{id}/tools
```

### Executing a Tool

```bash
curl -X POST http://localhost:3000/api/v1/mcp/servers/{id}/tools/echo \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"message": "Hello!"}}'
```

### Executing MCP Tool (Direct)

```bash
curl -X POST http://localhost:3000/api/v1/mcp/tools/execute \
  -H "Content-Type: application/json" \
  -d '{
    "server_id": "server-id",
    "tool_name": "echo",
    "arguments": {"message": "Hello!"}
  }'
```

## Files Created/Modified

### New Files
- `core/memory/migrations/015_mcp_servers.sql`
- `core/router/src/mcp/mod.rs`
- `core/router/src/mcp/types.rs`
- `core/router/src/mcp/client.rs`
- `core/router/src/mcp/server.rs`
- `core/router/src/mcp/registry.rs`
- `core/router/src/mcp/validation.rs`
- `core/router/src/mcp/client_test.rs`
- `core/router/src/mcp/e2e_test.rs`
- `core/router/src/api/mcp.rs`
- `core/router/src/api/skills.rs` (MCP integration)
- `core/router/src/message_bus.rs` (MCP events)
- `core/router/src/metrics.rs` (MCP metrics)
- `ui/src/components/settings/McpManager.tsx`
- `ui/src/components/settings/McpMarketplace.tsx`
- `test-mcp-server/server.js`

### Modified Files
- `core/memory/src/db.rs` - Migration 015
- `core/memory/src/config_repo.rs` - MCP methods
- `core/router/src/lib.rs` - Add mcp module
- `core/router/src/api/mod.rs` - Add mcp routes
- `core/router/src/main.rs` - Add mcp initialization
- `ui/src/components/settings/Settings.tsx` - Add MCP tabs
- `ui/src/themes/types.ts` - Add MCP badge color

## Future Enhancements

All core features implemented! Potential areas for expansion:

- [ ] MCP server marketplace integration
- [ ] Resource subscription via WebSocket push
- [ ] Prompt templates in UI
- [ ] Connection pooling metrics dashboard
- [ ] MCP tool caching layer
- [ ] Multi-server tool federations
