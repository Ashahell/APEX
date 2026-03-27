Streaming UI Parity Runbooks (PR C) - Incident Response & Rollback

Overview
- This runbook captures operational steps to respond to streaming UI issues, outages, or parity drift during rollout.
- It complements the STREAMING_ROLLOUT plan with concrete, actionable steps for on-call engineers.

1) Outage/Degradation of Streaming UI
- Symptoms: UI shows missing streams, high latency, or dropped connections.
- Immediate steps:
  - Verify backend streaming endpoints are healthy (curl /api health, /metrics, /stream/stats).
  - Check logs for auth failures, heartbeat failures, or stream drops.
  - If auth path regression suspected, revert to header-based auth temporarily; roll back UI to a known-good state.
- Rollback: Use feature flag to disable UI wiring; revert PR B if needed; ensure PR A remains active.

2) Parity Drift (UI sees different shapes than OpenClaw/Agent Zero/Hermes/OpenFang)
- Immediate steps:
  - Compare SSE envelope shapes from /stream/* endpoints against the contract in docs/UI_WIREDUP_PARITY.md and docs/UI_STREAMING_CONTRACT.md.
  - If mismatch, coordinate with backend to normalize payloads or update contracts.
  - Add a targeted test to lock in the expected envelope shape.

3) Auth/Signature Issue
- Symptoms: Auth failures on query-based UI paths.
- Steps:
  - Verify timestamp clock drift; ensure UI uses signed URL with ts within 5 minutes.
  - Validate that both header-based and query-based paths are correctly checked in StreamingAuth.
  - If needed, tighten the error codes and messaging to aid debugging.

4) Rollback & Recovery
- If parity drift cannot be resolved quickly:
  - Disable PR C changes via a feature flag; revert PR B wiring if necessary.
  - Keep PR A alive (UI-friendly auth) to avoid user-visible outages.
  - Communicate with stakeholders and document the rollback in the runbook log.

5) Verification & Closure
- Determine success criteria: parity contract exercised by UI in staging, telemetry surfaced, and no critical errors in streaming endpoints.
- Collect metrics and post-incident review to improve the next iteration.

Owners
- On-call: @engineering-ops
- Parity & UI: @frontend-team
- Backend parity: @backend-team

Last Update: v1.0
