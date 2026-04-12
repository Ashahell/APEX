# Streaming System

## Overview

APEX implements TinySSE-based streaming for real-time events across multiple channels.

## SSE Endpoints

| Endpoint | Events | Purpose |
|----------|--------|---------|
| `/stream/stats` | System metrics | Real-time monitoring |
| `/stream/hands/:task_id` | Hands agent | Computer use events |
| `/stream/mcp/:task_id` | MCP tools | Tool execution |
| `/stream/task/:task_id` | Task status | Task lifecycle |

## Implementation

### TinySSE
- Iterator-based SSE streams
- Deterministic in-memory streams
- Single SSEItem type alias: `Result<Event, axum::Error>`

### Event Types (11 total)
- task_started, task_progress, task_completed, task_failed
- tool_input, tool_output, tool_error
- agent_thought, agent_action
- mcp_request, mcp_response

### Streaming Features
- Connection pooling
- Backpressure handling
- Automatic reconnection (client)
- Polling fallback when WebSocket unavailable

## API
- `/api/v1/events` - SSE stream for all events
- WebSocket: `/ws` - Real-time updates

## Client
- socket.io-client in UI
- Automatic reconnection
- Connection state indicator (Connected/Degraded/Disconnected)