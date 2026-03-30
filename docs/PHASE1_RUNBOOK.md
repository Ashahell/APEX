# Phase 1 Runbook: Streaming UX Parity Expansion

## Overview
- Phase 1 expands the streaming UX with richer event types, metrics, signed URL surface, and UI panels.
- This runbook provides operational steps for incident response, debugging, and escalation.

## Phase 1 Features (New in v1.7.0+)

| Feature | Description |
|---------|-------------|
| Rich Event Types | session_start, session_end, checkpoint, user_intervention |
| Performance Metrics | connection_duration_total_ms, events_per_second_sum, avg_connection_duration_ms |
| Signed URL Surface | Query param auth for SSE connections |
| UI StreamingDashboard | Hands/MCP/Task/Stats panels |

---

## Incident Response Procedures

### 1. High Latency in Streaming

**Symptoms:**
- UI shows slow event streaming (>500ms p95)
- Client buffers filling up
- /api/v1/stream/stats shows high events_per_second_sum

**Immediate Steps:**
```bash
# Check streaming stats endpoint
curl -s http://localhost:3000/api/v1/stream/stats | jq .

# Check active connections
curl -s http://localhost:3000/api/v1/stream/stats | jq '.active_connections'

# Check events per second rate
curl -s http://localhost:3000/api/v1/stream/stats | jq '.events'
```

**Debug Commands:**
```bash
# Check router logs for streaming errors
tail -f logs/apex-router.log | grep -i stream

# Check metrics endpoint for latency histogram
curl -s http://localhost:3000/api/v1/metrics | grep -i stream

# Monitor connection duration metrics
curl -s http://localhost:3000/api/v1/stream/stats | jq '.performance'
```

**Rollback:** Disable streaming via feature flag in AppConfig, revert to non-streaming UI.

---

### 2. Connection Drops

**Symptoms:**
- UI shows "Disconnected" state
- active_connections drops unexpectedly
- Client auto-reconnects

**Immediate Steps:**
```bash
# Check if connections are being tracked
curl -s http://localhost:3000/api/v1/stream/stats

# Check total_connections vs active_connections
curl -s http://localhost:3000/api/v1/stream/stats | jq '{total: .total_connections, active: .active_connections}'

# Check for error counts
curl -s http://localhost:3000/api/v1/stream/stats | jq '.errors'
```

**Debug Commands:**
```bash
# Check for auth errors
curl -s http://localhost:3000/api/v1/stream/stats | jq '.errors.auth'

# Check for replay protection rejections
curl -s http://localhost:3000/api/v1/stream/stats | jq '.errors.replay'

# Check for internal errors
curl -s http://localhost:3000/api/v1/stream/stats | jq '.errors.internal'
```

**Rollback:** Disable streaming in config, restart router.

---

### 3. Authentication Failures (Signed URL)

**Symptoms:**
- 401/403 errors on streaming endpoints
- SSE connections fail immediately
- "Auth required" in logs

**Immediate Steps:**
```bash
# Test signed URL generation (requires router running)
# The signed URL should have __timestamp and __signature query params

# Check if streaming is enabled
curl -s http://localhost:3000/api/v1/config | jq '.streaming.enabled'

# Test with expired timestamp (should fail)
curl -s "http://localhost:3000/api/v1/stream/hands/test-task?__timestamp=1234567890&__signature=bad"

# Check replay protection store
curl -s http://localhost:3000/api/v1/stream/stats | jq '.errors.replay'
```

**Debug Commands:**
```bash
# Verify HMAC signing is working
# Check timestamp is within 5 minutes (300 seconds)
# Server rejects: expired timestamps (>5 min old), bad signatures

# Check logs for auth failures
tail -f logs/apex-router.log | grep -i "auth\|signature\|hmac"
```

**Common Issues:**
- Clock drift between client and server → use NTP sync
- Wrong shared secret → verify APEX_SHARED_SECRET matches
- Timestamp format → use Unix epoch seconds

**Rollback:** Temporarily disable auth (APEX_AUTH_DISABLED=1) for debugging.

---

### 4. Missing Event Types (Phase 1)

**Symptoms:**
- UI missing session_start, session_end, checkpoint, user_intervention events
- Stats show 0 for new event types

**Immediate Steps:**
```bash
# Check if new event types are being tracked
curl -s http://localhost:3000/api/v1/stream/stats | jq '.events.session_start'
curl -s http://localhost:3000/api/v1/stream/stats | jq '.events.session_end'
curl -s http://localhost:3000/api/v1/stream/stats | jq '.events.checkpoint'
curl -s http://localhost:3000/api/v1/stream/stats | jq '.events.user_intervention'
```

**Debug Commands:**
```bash
# Verify streaming_types.rs has the new event types
grep -n "SessionStart\|SessionEnd\|Checkpoint\|UserIntervention" core/router/src/streaming_types.rs

# Check if StreamingMetrics is tracking them
grep -n "events_session_start\|events_session_end\|events_checkpoint\|events_user_intervention" core/router/src/streaming.rs
```

---

## Debug Commands Quick Reference

| Command | Purpose |
|---------|---------|
| `curl -s http://localhost:3000/api/v1/stream/stats` | Full streaming stats |
| `curl -s http://localhost:3000/api/v1/stream/stats \| jq '.events'` | Event counts |
| `curl -s http://localhost:3000/api/v1/stream/stats \| jq '.errors'` | Error counts |
| `curl -s http://localhost:3000/api/v1/stream/stats \| jq '.performance'` | Performance metrics |
| `curl -s http://localhost:3000/api/v1/metrics \| grep stream` | Prometheus metrics |
| `tail -f logs/apex-router.log \| grep stream` | Real-time logs |

---

## Test Commands

```bash
# Run streaming integration tests
cd core && cargo test --test streaming_integration

# Run specific Phase 1 tests
cd core && cargo test stream_metrics_phase1
cd core && cargo test signed_url
cd core && cargo test expired_timestamp

# Test signed URL flow manually
# Generate signed URL with timestamp and signature
# Endpoint: /api/v1/stream/hands/:task_id?__timestamp=<ts>&__signature=<sig>
```

---

## Escalation Paths

| Issue | First Contact | Escalation |
|-------|--------------|------------|
| Streaming backend | @backend-team | @engineering-ops |
| UI Dashboard | @frontend-team | @engineering-ops |
| Auth/Security | @security-team | @governance-board |
| Performance | @backend-team | @sre-team |

---

## Rollback Procedure

If Phase 1 changes cause critical issues:

1. **Disable streaming feature flag:**
   ```bash
   # In config or environment
   APEX_STREAMING_ENABLED=false
   ```

2. **Revert to header-based auth (if needed):**
   - UI falls back to non-streaming mode
   - Keep PR A alive (UI-friendly auth)

3. **Restart services:**
   ```bash
   # Restart router
   cargo run --release --bin apex-router
   ```

4. **Verify recovery:**
   ```bash
   curl -s http://localhost:3000/api/v1/stream/stats
   # Should show active_connections: 0
   ```

---

## Verification Checklist

After any incident, verify:

- [ ] `/api/v1/stream/stats` returns 200 OK
- [ ] All event types track correctly
- [ ] Performance metrics update
- [ ] Signed URL auth works
- [ ] No auth/replay errors in stats
- [ ] UI streaming panels render correctly

---

## Contacts

- On-call: @engineering-ops
- Backend Streaming: @backend-team
- UI Dashboard: @frontend-team
- Security/Auth: @security-team

---

## Last Updated

- Phase 1: Streaming UX Parity Expansion
- Version: 1.0
- Date: 2026-03-29
