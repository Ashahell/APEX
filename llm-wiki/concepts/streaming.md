# Streaming & Real-Time Updates

**Status:** Implemented (v1.6.0)
**Date:** 2026-04-25
**Source:** [core/router/src/api/streaming.rs](core/router/src/api/streaming.rs)

## Overview

APEX supports multiple real-time communication patterns: SSE, WebSocket, and TinySSE (lightweight custom protocol).

## SSE (Server-Sent Events)

**Endpoint:** `GET /api/v1/events`

Standard SSE stream for task updates:
- `task_created` — New task spawned
- `task_progress` — Step completion
- `task_completed` — Task finished
- `task_failed` — Error occurred
- `thought` — Agent reasoning output

## WebSocket

**Endpoint:** `GET /api/v1/ws`

Full-duplex real-time connection:
- Same event types as SSE
- Bidirectional: client can send commands
- Automatic reconnection with exponential backoff
- Connection state: connected / degraded / disconnected

## TinySSE (Lightweight Protocol)

**Purpose:** In-memory streaming for scenarios where persistent connections aren't needed.

```rust
// TinySseStream wrapper in core/router/src/api/streaming.rs
// Provides StreamExt integration for async streams
// Used for: Hands messages, MCP streaming, task events
```

**Key components:**
- `TinySseStream` — Wraps any `Stream<Item=Bytes>`
- `StreamAuthQuery` — Authentication for streaming endpoints
- `to_sse_event()` — Convert stream items to SSE format

## Implementation

### SSE (`core/router/src/api/tasks.rs`)
```rust
async fn sse_events(State(state): State<AppState>) -> impl IntoResponse {
    // Stream events to client
}
```

### WebSocket (`core/router/src/api/ws.rs`)
```rust
async fn ws_handler(ws: WebSocket, State(state): State<AppState>) -> ... {
    // Full-duplex connection handling
}
```

### UI Client (`ui/src/lib/ws.ts`)
- `WSClient` class
- Auto-reconnect with backoff
- Event handlers for each message type

## Event Flow

```
User Input → API → Worker (skill/deep)
                ↓
         message_bus.on("event")
                ↓
         Stream to client (SSE/WebSocket/TinySSE)
```

## Related

- [project-overview.md](project-overview.md)
- [concepts/architecture.md](concepts/architecture.md)