# OpenFang Security — Streaming (Patch 11)

> **Status**: MVP (Patch 11) | **Security Level**: T2 | **Owner**: Security WG

## Overview

Patch 11 introduces Server-Sent Events (SSE) streaming for Hands and MCP task events, enabling real-time progress updates to the UI. This document describes the streaming contract, security posture, and the path to production hardening.

---

## Streaming Architecture

### Endpoint Surface

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| `GET` | `/api/v1/stream/hands/:task_id` | SSE stream for a Hands task | HMAC |
| `GET` | `/api/v1/stream/mcp/:task_id` | SSE stream for an MCP task | HMAC |
| `GET` | `/api/v1/stream/task/:task_id` | Generic SSE stream (alias) | HMAC |

### SSE Event Types

Each SSE event has an `event` type and a JSON `data` payload:

| Event Type | When Emitted | Key Fields |
|------------|--------------|------------|
| `connected` | Stream opened | `task_id`, `timestamp` |
| `thought` | Agent thinking | `step`, `content` |
| `tool_call` | Tool invocation | `step`, `tool`, `input` |
| `tool_progress` | Tool running | `step`, `tool`, `output` |
| `tool_result` | Tool completed | `step`, `tool`, `success`, `output` |
| `approval_needed` | T2/T3 confirmation | `step`, `tier`, `action`, `consequences` |
| `error` | Execution error | `step`, `message` |
| `complete` | Task done | `output`, `steps`, `tools_used` |
| `stream_closed` | Server closes stream | `task_id` |
| `: keepalive` (comment) | 30s interval | — |

### Example SSE Stream

```
event: connected
data: {"type":"connected","task_id":"01AR...","timestamp":"2026-03-25T10:00:00Z"}

event: thought
data: {"type":"thought","step":0,"content":"Starting task execution..."}

event: tool_call
data: {"type":"tool_call","step":1,"tool":"shell.execute","input":{"command":"ls -la"}}

event: tool_result
data: {"type":"tool_result","step":1,"tool":"shell.execute","success":true,"output":"..."}

event: complete
data: {"type":"complete","output":"Task finished","steps":3,"tools_used":["shell.execute"]}
```

---

## Security Posture

### Authentication (MVP)

Streaming endpoints use **connection-time HMAC authentication**:
- HMAC-SHA256 of `timestamp + "GET" + path + ""`
- Timestamp must be within **5 minutes** of server time (replay window)
- Signature passed as query parameters on the initial connection (MVP limitation — native EventSource cannot set headers)

**Query parameters added by the client:**
```
GET /api/v1/stream/hands/task-123
  ?__timestamp=1742985600
  &__nonce=uuid-v4
  &__signature=<hmac-hex>
```

**Limitations (Patch 11 MVP):**
- Signature is visible in URLs and server logs
- Replay window is 5 minutes (same as REST)
- No per-stream revocation mechanism

**Production upgrade path (Patch 14+):**
- Use `@microsoft/fetch-event-source` to set headers instead of query params
- Or upgrade to WebSocket with an auth handshake message after connection

### Replay Protection

- In-memory HashSet tracks observed HMAC signatures/nonces
- Duplicate signatures are rejected with `401 Unauthorized`
- **MVP limitation**: In-memory store is single-process; restart clears state
- **Production**: Replace with Redis or similar with TTL-based expiry

### Config Gate

Streaming is **disabled by default** (`APEX_STREAMING_ENABLED=0`). Enable with:
```bash
export APEX_STREAMING_ENABLED=1
```

### HMAC Verification Path

```
Client                       Router                         Security Layer
  |                              |                                  |
  |-- GET /stream/hands/:id ---->|                                  |
  |  ?__timestamp=...            |                                  |
  |  ?__nonce=...               |                                  |
  |  ?__signature=...            |  extract params                  |
  |                              |--------------------------------->|  verify HMAC
  |                              |                                  |  record nonce
  |                              |<---------------------------------|
  |                              |  OK / 401 Unauthorized           |
  |                              |                                  |
  |<-- SSE stream begins --------|                                  |
  |  event: connected            |                                  |
  |  event: thought             |  ExecutionStreamManager          |
  |  event: tool_result         |  (broadcast::channel)           |
  ...                            |                                  |
```

---

## Config

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `APEX_STREAMING_ENABLED` | `0` (off) | Enable SSE streaming endpoints |
| `APEX_STREAMING_MAX_SESSION_SECS` | `3600` | Max stream lifetime before auto-termination |

### AppConfig Struct

```rust
pub struct StreamingConfig {
    pub enabled: bool,           // APEX_STREAMING_ENABLED
    pub max_session_secs: u64,   // APEX_STREAMING_MAX_SESSION_SECS
}
```

---

## Files Added/Modified by Patch 11

| File | Change |
|------|--------|
| `core/router/src/streaming.rs` | **Add** — SSE endpoint handlers and event conversion |
| `core/router/src/lib.rs` | **Update** — `pub mod streaming` |
| `core/router/src/api/mod.rs` | **Update** — wire streaming router into main router |
| `core/router/src/unified_config.rs` | **Update** — `StreamingConfig` struct + `AppConfig.streaming` field |
| `core/router/src/security/mod.rs` | **Update** — `pub mod replay_protection` |
| `core/router/src/security/replay_protection.rs` | **Add** — in-memory replay detection |
| `gateway/src/streaming.ts` | **Add** — TypeScript SSE client with HMAC URL signing |
| `docs/OpenFang_Security_Stream.md` | **Add** — this document |

---

## Tests Added

### Patch 11 — Unit Tests (streaming.rs, replay_protection.rs)
| Test | Location | Coverage |
|------|---------|----------|
| `test_execution_event_to_sse_*` (10 tests) | `streaming.rs` | All ExecutionEvent → SSE conversions |
| `test_streaming_error_to_sse_*` (5 tests) | `streaming.rs` | StreamingError → SSE error events |
| `test_create_streaming_router_does_not_panic` | `streaming.rs` | Router construction smoke test |
| `test_fresh_signature_not_replay` | `replay_protection.rs` | Fresh signature allowed |
| `test_duplicate_signature_is_replay` | `replay_protection.rs` | Duplicate rejected |
| `test_reset_clears_signatures` | `replay_protection.rs` | Reset functionality |
| `test_record_and_check_is_atomic` | `replay_protection.rs` | Atomic check-and-record |

### Patch 12 — Security Tests
| Test | Location | Coverage |
|------|---------|----------|
| `valid_stream_signature_passes` | `hmac_tests.rs` | Stream GET with valid HMAC |
| `stream_signature_rejects_different_method` | `hmac_tests.rs` | GET vs POST mismatch |
| `stream_signature_rejects_different_path` | `hmac_tests.rs` | Task-A vs Task-B path mismatch |
| `stream_signature_rejects_future_timestamp` | `hmac_tests.rs` | >5-min future rejected |
| `stream_signature_rejects_past_timestamp` | `hmac_tests.rs` | >5-min past rejected |
| `stream_signature_at_boundary_of_window` | `hmac_tests.rs` | 4m59s within window |
| `different_secrets_produce_different_signatures` | `hmac_tests.rs` | Secret isolation |
| `stream_signature_is_deterministic` | `hmac_tests.rs` | Deterministic signing |
| `streaming_router_builds_without_panic` | `streaming_integration.rs` | Router construction |
| `streaming_disabled_returns_503` | `streaming_integration.rs` | Config gate |
| `nonexistent_stream_returns_not_found` | `streaming_integration.rs` | StreamNotFound error |
| `replay_protection_rejects_duplicate_signature` | `streaming_integration.rs` | Replay detection |
| `replay_protection_allows_distinct_signatures` | `streaming_integration.rs` | No false positives |
| `streaming_error_to_sse_all_variants` | `streaming_integration.rs` | All error types → SSE |
| `streaming_config_respects_env_vars` | `streaming_integration.rs` | Config parsing |

---

## Next Steps (Patch 13+)

| Patch | Goal |
|-------|------|
| **Patch 13** | UI wiring — render SSE events in Hands UI; auto-reconnect; backpressure |
| **Patch 14** | WebSocket upgrade path — richer interaction, binary frames, auth handshake |
| **Patch 15** | Distributed replay protection — Redis backend with TTL |
| **Patch 16** | Streaming analytics — connection counts, event throughput, error rates |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Replay attacks on streams | Medium (MVP) | High | In-memory detector; Redis in Patch 15 |
| Signature exposure in URLs | High | Medium | Move to header auth in Patch 14 |
| Long-lived stream exhaustion | Low | Medium | `max_session_secs` config gate |
| SSE connection hijacking | Low | High | TLS required; Origin validation in Patch 14 |
| HMAC timing attacks | Low | Medium | `hmac::HmacVerifier` uses constant-time comparison |

---

## Rollback Plan

If issues are found after rollout:
1. Set `APEX_STREAMING_ENABLED=0` (immediate disable via env var)
2. Streaming routes remain mounted but return `503 Service Unavailable`
3. UI gracefully falls back to polling (existing behavior)
4. No data loss: ExecutionStreamManager retains events until consumed
