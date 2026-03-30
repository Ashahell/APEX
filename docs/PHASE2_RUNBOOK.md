# Phase 2 Runbook: Telemetry and Observability Parity

## Overview
- Phase 2 expands telemetry and observability with per-endpoint latency histograms, error rate tracking, monitoring UI, and SLO documentation.
- This runbook provides operational steps for incident response, debugging, and escalation.

## Phase 2 Features (New in v1.8.0+)

| Feature | Description |
|---------|-------------|
| Per-Endpoint Latency | Histogram tracking with p50, p95, p99 percentiles |
| Per-Endpoint Error Rates | Request/error counts with error type breakdown |
| Telemetry Middleware | Tower middleware layer for automatic tracking |
| Monitoring Dashboard UI | New "Telemetry" tab with latency/error tables |
| Endpoint Normalization | ULIDs/UUIDs/numbers → `:id` for aggregation |

---

## Incident Response Procedures

### 1. High Latency on Specific Endpoint

**Symptoms:**
- Monitoring Dashboard shows p95 > 500ms on specific endpoint
- Color-coded yellow/red indicators on latency table
- Users reporting slow responses

**Immediate Steps:**
```bash
# Check per-endpoint latency breakdown
curl -s http://localhost:3000/api/v1/metrics | jq '.telemetry.endpoint_latencies'

# Check specific endpoint stats
curl -s http://localhost:3000/api/v1/metrics | jq '.telemetry.endpoint_latencies["/api/v1/tasks"]'

# Check streaming endpoint latency
curl -s http://localhost:3000/api/v1/metrics | jq '.telemetry.endpoint_latencies["/api/v1/stream/hands/:id"]'
```

**Debug Commands:**
```bash
# Check router logs for slow requests
# Look for requests taking > 500ms

# Check if specific endpoints have high latency
curl -s http://localhost:3000/api/v1/metrics | jq '.telemetry.endpoint_latencies | to_entries | sort_by(.value.p95_ms) | reverse'

# Check streaming connection health
curl -s http://localhost:3000/api/v1/stream/stats | jq '.performance'
```

**Common Causes:**
- LLM inference latency (check llama-server response times)
- Database query performance (check SQLite query times)
- Skill execution timeouts (check skill pool stats)

**Rollback:** Disable telemetry middleware layer if performance impact detected.

---

### 2. Elevated Error Rates

**Symptoms:**
- Monitoring Dashboard shows error rate > 1% on endpoint
- Red/yellow indicators on error rate table
- Error type badges showing specific failure modes

**Immediate Steps:**
```bash
# Check per-endpoint error rates
curl -s http://localhost:3000/api/v1/metrics | jq '.telemetry.endpoint_errors'

# Check specific endpoint errors
curl -s http://localhost:3000/api/v1/metrics | jq '.telemetry.endpoint_errors["/api/v1/tasks"]'

# Check error type breakdown
curl -s http://localhost:3000/api/v1/metrics | jq '.telemetry.endpoint_errors["/api/v1/tasks"].error_types'
```

**Debug Commands:**
```bash
# Check for 4xx errors (client errors)
curl -s http://localhost:3000/api/v1/metrics | jq '[.telemetry.endpoint_errors | to_entries[] | select(.value.error_types["4xx"] > 0)]'

# Check for 5xx errors (server errors)
curl -s http://localhost:3000/api/v1/metrics | jq '[.telemetry.endpoint_errors | to_entries[] | select(.value.error_types["5xx"] > 0)]'

# Check streaming error counts
curl -s http://localhost:3000/api/v1/stream/stats | jq '.errors'
```

**Common Causes:**
- 4xx: Invalid requests, auth failures, missing resources
- 5xx: Internal server errors, database failures, LLM timeouts

**Rollback:** Investigate error types, fix root cause, no need to disable telemetry.

---

### 3. Telemetry Data Missing or Stale

**Symptoms:**
- Monitoring Dashboard shows empty telemetry tables
- No endpoint data in `/api/v1/metrics` response
- Last updated timestamp not changing

**Immediate Steps:**
```bash
# Check if telemetry is included in metrics response
curl -s http://localhost:3000/api/v1/metrics | jq '.telemetry'

# Check if middleware is active (should see latency data after requests)
curl -s http://localhost:3000/api/v1/tasks
curl -s http://localhost:3000/api/v1/metrics | jq '.telemetry.endpoint_latencies'
```

**Debug Commands:**
```bash
# Check router startup logs for telemetry layer
# Look for "TelemetryLayer" in startup logs

# Verify middleware is wired in create_router()
# Check: api/mod.rs has TelemetryLayer import and .layer(TelemetryLayer::new(...))
```

**Common Causes:**
- Middleware not wired in router (check `api/mod.rs`)
- Telemetry module not compiled (check `lib.rs` has `pub mod telemetry_middleware`)

**Rollback:** Re-add telemetry middleware layer to router.

---

## Debug Commands Quick Reference

| Command | Purpose |
|---------|---------|
| `curl -s http://localhost:3000/api/v1/metrics \| jq '.telemetry'` | Full telemetry data |
| `curl -s http://localhost:3000/api/v1/metrics \| jq '.telemetry.endpoint_latencies'` | Per-endpoint latency |
| `curl -s http://localhost:3000/api/v1/metrics \| jq '.telemetry.endpoint_errors'` | Per-endpoint errors |
| `curl -s http://localhost:3000/api/v1/stream/stats` | Streaming stats |
| `cargo test --test telemetry_integration` | Run telemetry tests |

---

## Test Commands

```bash
# Run telemetry integration tests
cd core && cargo test --test telemetry_integration

# Run specific telemetry tests
cd core && cargo test telemetry_latency
cd core && cargo test telemetry_error
cd core && cargo test telemetry_surface
cd core && cargo test router_metrics_includes_telemetry
```

---

## Escalation Paths

| Issue | First Contact | Escalation |
|-------|--------------|------------|
| High latency | @backend-team | @sre-team |
| Error spikes | @backend-team | @engineering-ops |
| Telemetry missing | @backend-team | @infra-team |
| UI dashboard issues | @frontend-team | @engineering-ops |

---

## Rollback Procedure

If Phase 2 changes cause critical issues:

1. **Disable telemetry middleware:**
   ```rust
   // In api/mod.rs, remove or comment out:
   // .layer(TelemetryLayer::new(state.metrics.clone()))
   ```

2. **Restart services:**
   ```bash
   cargo run --release --bin apex-router
   ```

3. **Verify recovery:**
   ```bash
   curl -s http://localhost:3000/api/v1/metrics
   # Should still return metrics without telemetry section
   ```

---

## Verification Checklist

After any incident, verify:

- [ ] `/api/v1/metrics` returns 200 OK
- [ ] Telemetry section includes endpoint_latencies and endpoint_errors
- [ ] Monitoring Dashboard renders Telemetry tab
- [ ] Latency percentiles (p50, p95, p99) are populated
- [ ] Error rates are calculated correctly
- [ ] All 9 telemetry integration tests pass

---

## Contacts

- On-call: @engineering-ops
- Backend Telemetry: @backend-team
- UI Dashboard: @frontend-team
- Infrastructure: @infra-team

---

## Last Updated

- Phase 2: Telemetry and Observability Parity
- Version: 1.0
- Date: 2026-03-31
