Streaming Telemetry Rollout (PR C)

Overview
- Provides a plan to introduce telemetry for the streaming subsystem to enable parity with OpenFang and OpenClaw-like observability.
- Covers metrics, tracing, dashboards, sampling, and rollout governance.

## Implementation Status: COMPLETE

### What has been implemented:
- **Metrics Endpoint**: `/api/v1/metrics` now exposes streaming metrics
- **StreamingMetrics**: Thread-safe atomic counters in `streaming_types.rs`
- **Real-time Stats**: `/stream/stats` endpoint provides live metrics

### Metrics exposed:
```json
{
  "streaming": {
    "active_connections": 42,
    "total_connections": 1250,
    "events": {
      "thought": 5420,
      "tool_call": 3150,
      "tool_progress": 890,
      "tool_result": 2890,
      "approval_needed": 45,
      "error": 12,
      "complete": 890
    },
    "errors": {
      "auth": 2,
      "replay": 0,
      "internal": 10
    }
  }
}
```

### SLO Targets:
| Metric | Target |
|--------|--------|
| Availability | 99.9% |
| Latency p95 | < 500ms |
| Error Rate | < 0.1% |

Rollout & Backout
- Feature flag gating; rollback path to disable telemetry if risk detected.
- If telemetry introduces performance overhead, reduce sampling rates or disable non-critical metrics.

Owners
- Telemetry lead: @infra-team
- Backend metrics: @backend-team
- Frontend telemetry consumption: @frontend-team
