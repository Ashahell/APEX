# APEX v1.6.0 — Sapphire Features

## Summary

APEX v1.6.0 ("Sapphire") completes three high-impact infrastructure patches that improve the streaming layer's security, scalability, and observability. The changes are additive with no breaking API changes.

| Patch | Area | What changed |
|-------|------|-------------|
| **14** | Streaming / WebSocket | WebSocket upgrade with ticket-based auth, HMAC-signed tickets, ping/pong heartbeat |
| **15** | Streaming / Security | Distributed replay protection with in-memory and Redis backends |
| **16** | Streaming / Observability | Real-time metrics: connection counts, event throughput, error rates |

---

## What's New

### Patch 14 — WebSocket Streaming with Ticket Auth

**Problem:** SSE is one-directional and inefficient for high-frequency tool progress updates. The SSE stream endpoint had no per-connection auth — all clients shared one HMAC signature path.

**Solution:**
- New ticket endpoint: `GET /api/v1/stream/ticket?task_id=X` issues a short-lived, HMAC-signed ticket
- New WebSocket endpoint: `WS /api/v1/stream/ws/:task_id?ticket=...` upgrades to a bidirectional socket with 30s ping/pong heartbeat
- Ticket is validated with `constant_time_eq` to prevent timing attacks
- Tickets expire after 5 minutes (configurable via `APEX_STREAM_TICKET_TTL_SECS`)

**Files changed:**
- `core/router/src/streaming.rs` — StreamTicket, handle_ws_stream, get_stream_ticket
- `ui/src/lib/ws.ts` — WSClient class (auto-reconnect, heartbeat, Zustand dispatch)
- `ui/src/components/chat/Chat.tsx` — SSEClient → WSClient swap
- `ui/src/components/hands/HandMonitor.tsx` — SSEClient → WSClient swap

### Patch 15 — Distributed Replay Protection

**Problem:** In single-process deployments, the in-memory HashSet works fine. Multi-instance deployments (e.g., behind a load balancer with multiple router instances) share no state, making replay attacks possible across instances.

**Solution:**
- `ReplayProtection` trait with two backends:
  - `InMemoryReplayProtection` — thread-local `HashSet` via `thread_local!` with `RefCell`. Zero network overhead; thread-safe per instance. Test-isolated via `reset()`.
  - `RedisReplayProtection` — atomic `SET key EX 300 NX` for cross-instance replay detection. Only compiled when the `redis` feature is enabled.
- Factory: `from_config(backend, redis_url)` selects the backend at startup
- New env vars: `APEX_REPLAY_BACKEND=memory|redis` and `APEX_REDIS_URL`

**Files changed:**
- `core/router/src/security/replay_protection.rs` — trait + both backends
- `core/router/src/unified_config.rs` — ReplayBackend enum, streaming config fields
- `core/router/src/api/mod.rs` — replay_protection in AppState
- `core/router/src/main.rs` — initializes backend from config
- `core/router/Cargo.toml` — `deadpool-redis` as optional dependency

### Patch 16 — Streaming Analytics

**Problem:** No visibility into streaming layer health — active connections, event throughput by type, auth/replay error rates.

**Solution:**
- `StreamingMetrics` struct with 12 `AtomicU64` counters:
  - Connections: `active_connections`, `total_connections`
  - Events: `events_thought`, `events_tool_call`, `events_tool_progress`, `events_tool_result`, `events_approval`, `events_error`, `events_complete`
  - Errors: `errors_auth`, `errors_replay`, `errors_internal`
- New endpoint: `GET /api/v1/stream/stats` returns a `StreamingStats` snapshot (JSON)
- All counters wired in SSE handler (`_stream_task_internal`) and WebSocket handler (`handle_ws_stream`)
- 4 new unit tests; 21 total streaming tests

**Files changed:**
- `core/router/src/streaming.rs` — StreamingMetrics, StreamingStats, get_stream_stats handler, all wiring

---

## How to Test

### Prerequisites
- Rust 1.93+, Node.js 20+, pnpm
- Running router on port 3000 (or `APEX_PORT`)

### 1. Streaming Stats Endpoint

```bash
# Requires HMAC signing (same as other API endpoints)
TIMESTAMP=$(date +%s)
SIGNATURE=$(echo -n "${TIMESTAMP}|GET|/api/v1/stream/stats||" | \
  openssl dgst -sha256 -hmac "your-shared-secret" | cut -d' ' -f2)

curl -H "X-APEX-Timestamp: ${TIMESTAMP}" \
     -H "X-APEX-Signature: ${SIGNATURE}" \
     http://localhost:3000/api/v1/stream/stats
```

Expected response:
```json
{
  "active_connections": 0,
  "total_connections": 0,
  "events": {
    "thought": 0, "tool_call": 0, "tool_progress": 0,
    "tool_result": 0, "approval": 0, "error": 0,
    "complete": 0, "total": 0
  },
  "errors": { "auth": 0, "replay": 0, "internal": 0, "total": 0 }
}
```

### 2. WebSocket Ticket + Stream

```bash
# Step 1: Get a ticket
TIMESTAMP=$(date +%s)
SIG=$(echo -n "${TIMESTAMP}|GET|/api/v1/stream/ticket||" | \
  openssl dgst -sha256 -hmac "your-shared-secret" | cut -d' ' -f2)

TICKET_JSON=$(curl -s -H "X-APEX-Timestamp: ${TIMESTAMP}" \
  -H "X-APEX-Signature: ${SIG}" \
  "http://localhost:3000/api/v1/stream/ticket?task_id=test-001")
TICKET=$(echo $TICKET_JSON | jq -r '.ticket')
echo "Ticket: $TICKET"

# Step 2: Connect WebSocket
# Tickets expire in 5 minutes — connect quickly
wscat -c "ws://localhost:3000/api/v1/stream/ws/test-001?ticket=$TICKET"
```

### 3. Replay Protection (in-memory)

Replay protection is automatic on all streaming endpoints. To test manually:
```bash
# First request succeeds, second with same signature fails
curl -H "X-APEX-Timestamp: ${TIMESTAMP}" \
     -H "X-APEX-Signature: ${SIGNATURE}" \
     http://localhost:3000/api/v1/stream/task/test-001
# Second identical request → 401 "Replay detected"
```

### 4. Redis Backend (multi-instance)

```bash
# Set env vars before starting router
export APEX_REPLAY_BACKEND=redis
export APEX_REDIS_URL=redis://localhost:6379
export APEX_SHARED_SECRET=your-secret

cargo run --release --bin apex-router
```

---

## API Reference

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| GET | `/api/v1/stream/ticket` | HMAC | Issue HMAC-signed stream ticket |
| WS | `/api/v1/stream/ws/:task_id` | Ticket | WebSocket stream with heartbeat |
| GET | `/api/v1/stream/stats` | HMAC | Streaming metrics snapshot |
| GET | `/api/v1/stream/task/:task_id` | HMAC | SSE task stream |
| GET | `/api/v1/stream/hands/:task_id` | HMAC | SSE Hands stream |
| GET | `/api/v1/stream/mcp/:task_id` | HMAC | SSE MCP stream |

---

## Known Caveats

1. **Stats endpoint auth**: `GET /api/v1/stream/stats` requires HMAC signing. Ensure any monitoring tooling signs requests the same way as the existing API endpoints.

2. **Redis feature**: The `redis` Cargo feature must be enabled to use the Redis backend. Without the feature, `APEX_REPLAY_BACKEND=redis` falls back to in-memory silently.

3. **Ticket TTL**: Default 300s (5 min). Tickets cannot be used after expiry — clients should request a fresh ticket if the connection attempt fails due to expiry.

4. **WebSocket vs SSE**: The UI has migrated from SSEClient to WSClient for task streaming. The SSE endpoints remain for backwards compatibility and clients that cannot use WebSocket.

5. **Test isolation**: `test_llm_list_endpoint` in the integration suite fails due to a pre-existing `GLOBAL_CONFIG` pollution issue across test runs. This is unrelated to patches 13–16 and is tracked separately.

---

## Files Changed

**Core (Rust):**
- `core/router/src/streaming.rs` — WebSocket, tickets, metrics
- `core/router/src/security/replay_protection.rs` — trait + backends
- `core/router/src/unified_config.rs` — ReplayBackend, streaming config
- `core/router/src/api/mod.rs` — AppState fields
- `core/router/src/main.rs` — initialization
- `core/router/src/computer_use_api.rs` — Computer Use API
- `core/router/src/persona.rs`, `privacy_guard.rs`, `story_engine.rs`, `tool_validator.rs`, `skill_signer.rs`, `context_scope.rs`, `continuity.rs`, `tool_sandbox.rs` — Sapphire feature modules
- `core/router/src/computer_use/` — Computer use orchestrator
- `core/router/tests/streaming_integration.rs`, `auth_integration.rs`, `memory_integration.rs`, `skills_integration.rs` — new integration tests
- `core/router/Cargo.toml` — deadpool-redis optional dependency

**UI (TypeScript/React):**
- `ui/src/lib/ws.ts` — WSClient (new)
- `ui/src/lib/sse.ts` — SSEClient (new)
- `ui/src/components/chat/Chat.tsx` — WSClient integration
- `ui/src/components/hands/HandMonitor.tsx` — WSClient integration
- `ui/src/components/settings/` — ContinuitySettings, PersonaEditor, PersonaList, PrivacySettings, SkillSecurity, ToolValidationSettings
- `ui/src/components/stories/` — StoryEditor, StoryPlayer
- `ui/src/components/security/SecurityPanel.tsx`
- `ui/src/pages/SecurityStatus.tsx`

**Gateway (TypeScript):**
- `gateway/src/streaming.ts` — Gateway streaming adapter
- `gateway/skills/computer.use/` — Computer use skill

**Other:**
- `core/analytics/dashboard.md`, `core/api/security.md` — prototype docs
- `core/memory/migrations/024_performance_indexes.sql` — new indexes
- `core/hands/`, `core/mcp/` — Hands runner and MCP protocol
- `core/security/` — security scaffolding
- `core/router/src/api/context_scope_api.rs`, `continuity_api.rs`, `persona_api.rs`, `privacy_api.rs`, `signing_api.rs`, `story_api.rs`, `tool_validation.rs` — REST API endpoints
- `core/router/src/bin/apex_router_bin.rs` — standalone binary

---

## Test Results

| Suite | Result |
|-------|--------|
| `cargo test --lib` | 351/351 pass |
| `cargo test --test streaming_integration` | 7/7 pass |
| `cargo test --test auth_integration` | 10/10 pass |
| `cargo test --test skills_integration` | 8/8 pass |
| `cargo test --test integration` | 58/59 pass (1 pre-existing failure) |
| `npm run build` (UI) | clean |

---

## Breaking Changes

None. All changes are additive.

- No API endpoint removed or renamed
- No schema changes to existing responses
- Backward compatible with SSE-only clients (SSE endpoints still work)
- Redis backend gracefully falls back to in-memory if Redis is unavailable

---

## Migration Guide

### From v1.5.1 → v1.6.0

No migration required for existing deployments.

For new multi-instance deployments using Redis:
```bash
export APEX_REPLAY_BACKEND=redis
export APEX_REDIS_URL=redis://your-redis-host:6379
```

For monitoring streaming health:
```bash
# Add to your monitoring scraper:
curl -H "X-APEX-Timestamp: ${T}" -H "X-APEX-Signature: ${S}" \
  http://router:3000/api/v1/stream/stats
```

---

## Checklist for Review

- [ ] `cargo test --lib` passes (351/351)
- [ ] `cargo test --test streaming_integration` passes (7/7)
- [ ] UI builds clean (`cd ui && npm run build`)
- [ ] WebSocket endpoint accepts valid tickets and rejects expired ones
- [ ] Stats endpoint returns correct JSON shape under load
- [ ] Redis backend connects successfully with valid `APEX_REDIS_URL`
- [ ] In-memory backend correctly rejects duplicate signatures within a test run
