# APEX Streaming API - UI Contract

This document defines the contract for UI clients to consume APEX streaming endpoints.

## Base URL

```
http://localhost:3000/stream
```

## Authentication

All streaming endpoints require HMAC authentication:

1. **Generate timestamp**: Current Unix timestamp in seconds
2. **Create signature**: HMAC-SHA256 of `timestamp + method + path + body`
3. **Headers**:
   - `X-APEX-Signature`: Hex-encoded signature (64 characters)
   - `X-APEX-Timestamp`: Unix timestamp (within 5 minutes)

### Signature Generation

```typescript
function signRequest(secret: string, method: string, path: string, body: string, timestamp: number): string {
  const message = timestamp.toString() + method + path + body;
  const hmac = crypto.createHmac('sha256', secret);
  hmac.update(message);
  return hmac.digest('hex');
}
```

## Endpoints

### GET /stream/stats

Returns current streaming metrics.

**Response:**
```json
{
  "active_connections": 5,
  "total_connections": 142,
  "events": {
    "thought": 120,
    "tool_call": 85,
    "tool_progress": 234,
    "tool_result": 78,
    "approval_needed": 12,
    "error": 3,
    "complete": 45,
    "total": 577
  },
  "errors": {
    "auth": 2,
    "replay": 0,
    "internal": 1,
    "total": 3
  }
}
```

### GET /stream/hands/:task_id

Stream events from the Hands agent for a specific task.

**Response:** SSE stream

**Event Types:**
- `hands`: Agent thought/action events
- `connected`: Initial connection event
- `error`: Error events

**Example:**
```
event: connected
data: {"task_id":"abc123","connection_id":"uuid-456","message":"connected"}

event: hands
data: {"type":"hands","timestamp":1699123456789,"trace_id":"trace-789","payload":{"thought":"Analyzing the page...","action":"click","target":"button#submit"}}
```

### GET /stream/mcp/:task_id

Stream MCP (Model Context Protocol) tool events.

**Event Types:**
- `mcp`: MCP tool events
- `connected`: Initial connection event
- `error`: Error events

**Example:**
```
event: connected
data: {"type":"connected","timestamp":1699123456789,"trace_id":"trace-789","payload":{"server_name":"mcp-server"}}

event: mcp
data: {"type":"mcp","timestamp":1699123456790,"trace_id":"trace-789","payload":{"tool":"bash","id":"tool-1","input":{"command":"ls -la"}}}
```

### GET /stream/task/:task_id

Stream task execution events.

**Event Types:**
- `task`: Task lifecycle events
- `connected`: Initial connection event
- `heartbeat`: Keep-alive ping
- `error`: Error events

**Example:**
```
event: connected
data: {"type":"connected","timestamp":1699123456789,"trace_id":"trace-789","payload":{"task_id":"abc123","connection_id":"uuid-456"}}

event: heartbeat
data: {"type":"heartbeat","timestamp":1699123456790,"trace_id":null,"payload":{"server_time":1699123456790,"active_connections":5}}

event: task
data: {"type":"task","timestamp":1699123456791,"trace_id":"trace-789","payload":{"status":"running","step":1,"max_steps":10}}
```

## SSE Envelope Format

All events follow this JSON envelope:

```typescript
interface SseEnvelope<T> {
  type: 'connected' | 'disconnected' | 'hands' | 'mcp' | 'task' | 'stats' | 'heartbeat' | 'error';
  timestamp: number;  // Unix timestamp in milliseconds
  trace_id: string | null;
  payload: T;
}
```

## Client Implementation Guide

### Basic TypeScript Client

```typescript
class ApexStreamingClient {
  private baseUrl: string;
  private secret: string;

  constructor(baseUrl: string, secret: string) {
    this.baseUrl = baseUrl;
    this.secret = secret;
  }

  private async fetchWithAuth(url: string): Promise<EventSource> {
    const timestamp = Math.floor(Date.now() / 1000);
    const signature = this.signRequest('GET', '/stream/task/test', '', timestamp);
    
    const eventSource = new EventSource(`${url}?auth=${timestamp}:${signature}`);
    return eventSource;
  }

  private signRequest(method: string, path: string, body: string, timestamp: number): string {
    const message = timestamp.toString() + method + path + body;
    const hmac = crypto.createHmac('sha256', this.secret);
    hmac.update(message);
    return hmac.digest('hex');
  }

  async subscribeToTask(taskId: string, onEvent: (event: any) => void): Promise<void> {
    const url = `${this.baseUrl}/stream/task/${taskId}`;
    const eventSource = await this.fetchWithAuth(url);
    
    eventSource.onmessage = (e) => {
      const data = JSON.parse(e.data);
      onEvent(data);
    };
    
    eventSource.onerror = (e) => {
      console.error('Stream error:', e);
    };
  }
}
```

### Reconnection Strategy

1. **Automatic Reconnect**: EventSource handles reconnection automatically
2. **Backoff**: Start with 1 second delay, double on each failure (max 30 seconds)
3. **State Recovery**: Store last `trace_id` and request missed events on reconnect

### Error Handling

| Status Code | Meaning | Action |
|-------------|---------|--------|
| 401 | Auth failed | Regenerate credentials, retry |
| 403 | Forbidden | Check permissions |
| 404 | Task not found | Task may have completed |
| 503 | Service unavailable | Backoff and retry |

## Payload Schemas

### ConnectionPayload
```json
{
  "task_id": "string",
  "connection_id": "string",
  "message": "string"
}
```

### HeartbeatPayload
```json
{
  "server_time": 1699123456789,
  "active_connections": 5
}
```

### HandsPayload
```json
{
  "thought": "Analyzing the page...",
  "action": "click",
  "target": "button#submit"
}
```

### McpPayload
```json
{
  "tool": "bash",
  "id": "tool-1",
  "input": { "command": "ls -la" },
  "progress": { "percent": 50 },
  "result": { "output": "total 12\ndrwxr-xr-x  2 user user 4096" }
}
```

### TaskPayload
```json
{
  "status": "running",
  "step": 1,
  "max_steps": 10,
  "output": "Processing..."
}
```

## Sample Code

### React Hook

```typescript
import { useEffect, useState, useRef } from 'react';

function useStreamingTask(taskId: string) {
  const [events, setEvents] = useState<any[]>([]);
  const [connected, setConnected] = useState(false);
  const eventSourceRef = useRef<EventSource | null>(null);

  useEffect(() => {
    const connect = async () => {
      const timestamp = Math.floor(Date.now() / 1000);
      const signature = await signRequest(...);
      
      const es = new EventSource(
        `/stream/task/${taskId}?t=${timestamp}&s=${signature}`
      );
      
      es.onopen = () => setConnected(true);
      es.onmessage = (e) => {
        setEvents(prev => [...prev, JSON.parse(e.data)]);
      };
      es.onerror = () => setConnected(false);
      
      eventSourceRef.current = es;
    };
    
    connect();
    
    return () => {
      eventSourceRef.current?.close();
    };
  }, [taskId]);

  return { events, connected };
}
```

## Testing

Use the following curl command to test:

```bash
# Generate auth headers
TIMESTAMP=$(date +%s)
SIGNATURE=$(echo -n "${TIMESTAMP}GET/stream/stats" | openssl dgst -sha256 -hmac "dev-secret" | cut -d' ' -f2)

# Test stats endpoint
curl -H "X-APEX-Timestamp: $TIMESTAMP" \
     -H "X-APEX-Signature: $SIGNATURE" \
     http://localhost:3000/stream/stats
```

## Version History

| Version | Changes |
|---------|---------|
| 1.7.0 | Initial streaming MVP |
| 1.7.1 | Added heartbeat, formal SSE envelope |
