Parity Runbooks for APEX Streaming UI (v1.x)

1) Streaming UI Rollout Runbook
- Objective: Enable wired UI streaming with end-to-end parity.
- Steps:
  - Ensure PR A (UI-friendly auth) is merged and green in CI.
  - Merge PR B (UI skeleton) into main with feature flag gating.
  - Enable UI streaming in staging with a small set of signed endpoints.
  - Validate: UI can render Stats, Hands, MCP, Task streams; no UI crashes.
  - Monitor streaming latency and event consistency; log any anomalies.
- Rollback: If UI issues exist, revert PR B without affecting backend PR A; disable feature flag.

2) Telemetry Parity Rollout
- Objective: Expose streaming telemetry (meters) for UI visibility and platform parity.
- Steps:
  - Implement minimal Prometheus metrics (active_connections, total_connections, events_total, errors_total).
  - Wire metrics to a stable /metrics endpoint.
- Rollback: If telemetry overload, disable exposure or throttle metrics collection.

3) Runbooks for Incidents
- Streaming Disruption: Steps to diagnose, collect logs, and roll back to a safe state.
- Security Incident: Steps for signing verification failures or token leakage.
- Data Retention & Privacy: Steps to archive streaming logs and purge old traces.

4) Governance & Compliance
- Periodic reviews of parity with reference platforms.
- Ensure that all new UI changes pass security guidelines and runbooks are kept up-to-date.

Glossary
- PR A: Backend parity work (UI-friendly SSE auth)
- PR B: UI skeleton wiring
- PR C: Parity/runbooks and rollout plan
