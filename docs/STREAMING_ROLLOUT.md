Streaming UI Parity Rollout Plan (PR C)

Overview
- This document describes the end-to-end rollout plan to achieve parity for APEX streaming UI with OpenClaw, Agent Zero, Hermes, and OpenFang. It defines rollout phases, success criteria, governance, and runbooks.

Goals
- Deliver a production-like, wired UI path for streaming endpoints including validation, telemetry, and runbooks.
- Ensure UI parity in UX, data contracts, telemetry, and security controls.
- Establish robust rollout and rollback processes with clear metrics and alerts.

Parity Targets (UI-focused)
- OpenClaw: UI for Skill Marketplace, streaming dashboards, and end-to-end tool execution visuals.
- Agent Zero: Real-time UI consumption of Hands, MCP, Task, and Stats with polished UX and theming.
- Hermes: UI integration with bounded memory/session visibility and narrative/metrics integration.
- OpenFang: Telemetry ready; dashboards for streaming latency, throughput, and reliability; SLO-oriented metrics.

Rollout Phases
- Phase 1 (Weeks 0-2): Parity foundations
  - Finalize PR A (backend) to support UI-friendly SSE auth (query-based) and ensure test coverage.
  - PR B (UI skeleton) merged and builds in CI; basic UI skeleton visible in UI.
  - Create a parity runbook in this document and start a small pilot with internal teams.

- Phase 2 (Weeks 3-6): UI integration depth
  - Implement StreamingDashboard.tsx wiring for Hands, MCP, Task panels.
  - Add EventSource hooks and a small UI test to verify end-to-end event flow.
  - Implement UI telemetry hooks (basic Prometheus metric hooks if applicable).

- Phase 3 (Weeks 7-10): Telemetry & SLOs
  - Add Prometheus scraping for streaming endpoints and streaming UI checks.
  - Introduce SLO/SLI baselines for streaming latency and error rates.
  - Prepare formal UI parity acceptance criteria with runbooks.

- Phase 4 (Weeks 11-12): Runbooks, governance, final parity review
  - Publish final runbooks and governance docs.
  - Do a final parity review against all four platforms and close remaining gaps.

DoR (Definition of Ready) for PR C
- PRs include engineering tasks, tests, and documentation updates.
- All changes are backed by tests and CI checks.
- Owners identified and acceptance criteria defined.
- No open critical blockers.

Definition of Done (DoD) for PR C
- UI wiring path is implemented and builds in CI.
- Backend parity (PR A) is stable and merged.
- Parity runbooks published (docs/STREAMING_ROLLOUT.md done).
- Telemetry scaffolding is in place (even if minimal).
- Documentation updated to reflect parity goals.

Runbooks & Governance
- Rollout plan with feature flags, monitoring, and rollback steps.
- Incident response for streaming UI issues (latency spikes, dropped streams).
- Data governance and retention for streaming traces.

Risks & Mitigations
- UI auth changes: Ensure token-based or proxy-based signed URL paths are secure and don't leak secrets.
- UI latency: Test with realistic streaming payloads and implement backpressure handling in UI hooks.
- Telemetry overload: Rate-limit or sample streaming telemetry to avoid heavy load.

Ownership
- Backend parity (PR A) owner: [Name/Team]
- UI skeleton parity (PR B) owner: [Name/Team]
- Parity & Runbooks (PR C) owner: [Name/Team]

Notes
- The runbooks will evolve as UI wiring progresses and feedback is gathered from internal users.

---

## Service Level Objectives (SLOs)

### Streaming Endpoints SLOs

| Metric | Target | Description |
|--------|--------|-------------|
| **Availability** | 99.9% | `/stream/*` endpoints respond within 1s |
| **Latency p95** | < 500ms | Time from event generation to SSE delivery |
| **Error Rate** | < 0.1% | Failed auth, invalid payloads, server errors |
| **Connection Duration** | < 5 min avg | Average SSE connection lifecycle |

### UI Streaming SLOs

| Metric | Target | Description |
|--------|--------|-------------|
| **Reconnection Success** | > 99% | Auto-reconnect after transient failures |
| **Event Delivery** | > 99.9% | Events received by UI without drops |
| **First Byte Time** | < 2s | Time to first SSE event on page load |

### Alerting Rules

| Alert | Condition | Severity |
|-------|-----------|----------|
| High Error Rate | `errors_total / events_total > 0.01` for 5m | Critical |
| High Latency | `latency_p95 > 1s` for 5m | Warning |
| Connection Failures | `connection_failures > 10` in 5m | Warning |
| Active Connections Drop | `active_connections < expected * 0.5` | Critical |

### Monitoring Dashboard

The `/api/v1/metrics` endpoint now exposes streaming metrics:

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
