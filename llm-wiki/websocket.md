# WebSocket System

## Overview

APEX implements WebSocket for real-time bidirectional communication with automatic reconnection.

## Components

### WebSocket Server
- Axum WebSocket handler
- Connection management
- Message routing

### WebSocket Client (UI)
- socket.io-client integration
- Auto-reconnection
- Polling fallback

### Connection State
| State | Description |
|-------|-------------|
| Connected | Active WebSocket |
| Degraded | Polling fallback |
| Disconnected | No connection |

## API Endpoints
- `/ws` - WebSocket endpoint
- `/api/v1/ws` - Alternative endpoint
- `/api/v1/events` - SSE alternative

## Features

### Auto-Reconnection
- Exponential backoff
- Max retry attempts
- State restoration

### Fallback
- SSE (Server-Sent Events) when WebSocket unavailable
- Long polling as last resort

### Message Types
- Task updates
- Skill execution events
- Memory changes
- System notifications

## Integration
- Real-time chat updates
- Task progress streaming
- Live monitoring dashboards
- Process group updates