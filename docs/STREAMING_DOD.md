# Streaming MVP - Definition of Done

This document defines the acceptance criteria for all streaming-related work in APEX.

## Security

- [ ] All streaming endpoints require valid HMAC signature (X-APEX-Signature header)
- [ ] Timestamp validation prevents replay attacks (within 5 minute window)
- [ ] Per-endpoint auth checks documented and tested
- [ ] Auth failures return 403 with clear error message
- [ ] TOTP verification for T3-tier operations via streaming

## Observability

- [ ] Prometheus metrics exposed for streaming:
  - `apex_streaming_active_connections` (gauge)
  - `apex_streaming_total_connections` (counter)
  - `apex_streaming_events_total` (counter by type)
  - `apex_streaming_errors_total` (counter by category)
- [ ] Structured logging with correlation IDs (request_id, task_id, client_id)
- [ ] Log levels: INFO for lifecycle events, WARN for degradation, ERROR for failures

## Reliability

- [ ] Heartbeat mechanism sends ping every 30 seconds
- [ ] Client disconnect triggers clean resource cleanup
- [ ] Graceful degradation under high load (return 503 when saturated)
- [ ] Stream timeout after 30 minutes of inactivity

## SSE Semantics

- [ ] Formal event types: `hands`, `mcp`, `task`, `stats`, `heartbeat`, `error`
- [ ] Event envelope includes: `type`, `timestamp`, `payload`, `trace_id`
- [ ] UTF-8 encoding for all text payloads
- [ ] No CORS issues (proper headers for browser consumption)

## MCP Integration

- [ ] Tool discovery events via streaming
- [ ] Tool execution progress via streaming
- [ ] Tool results delivered as structured events

## UI Contract

- [ ] Documented event schemas (JSON Schema)
- [ ] Reconnection strategy documented
- [ ] Sample payloads for each event type
- [ ] Example client code (TypeScript)

## Testing

- [ ] Unit tests for auth middleware
- [ ] Unit tests for SSE envelope serialization
- [ ] Integration tests for all 4 endpoints
- [ ] Concurrency tests (10+ simultaneous clients)
- [ ] Disconnect/reconnect tests
- [ ] Error handling tests

## Performance

- [ ] Latency per event < 10ms (internal processing)
- [ ] Support 100+ concurrent connections
- [ ] Memory bounded per connection (< 1MB)

## Documentation

- [ ] Architecture diagram for streaming flow
- [ ] API contract for each endpoint
- [ ] Runbook for common issues
- [ ] Migration guide (if applicable)

## Code Quality

- [ ] cargo fmt passes
- [ ] cargo clippy passes (no warnings)
- [ ] All tests pass (unit + integration)
- [ ] Documentation builds without warnings

---

## Ticket Template

```markdown
## Title
SP-XXX: [Brief title]

## Description
[What and why]

## Files to Touch
- `core/router/src/streaming.rs`
- `core/router/src/streaming_types.rs`

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2

## Dependencies
- SP-XXX (if any)

## Owner
[Name]

## Testing
- [ ] Unit tests added
- [ ] Integration tests added
- [ ] Manual verification
```

---

## Quick Reference

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/stream/stats` | Current streaming metrics |
| GET | `/stream/hands/:task_id` | Hands agent events |
| GET | `/stream/mcp/:task_id` | MCP tool events |
| GET | `/stream/task/:task_id` | Task execution events |

### Event Types

| Type | Description |
|------|-------------|
| `connected` | Client connected |
| `disconnected` | Client disconnected |
| `hands` | Hands agent event |
| `mcp` | MCP tool event |
| `task` | Task execution event |
| `heartbeat` | Keep-alive ping |
| `error` | Error event |

### Metrics

| Metric | Type | Labels |
|--------|------|--------|
| `apex_streaming_active_connections` | gauge | endpoint |
| `apex_streaming_events_total` | counter | type |
| `apex_streaming_errors_total` | counter | reason |
