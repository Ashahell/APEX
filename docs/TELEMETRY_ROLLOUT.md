Streaming Telemetry Rollout (PR C)

Overview
- Provides a plan to introduce telemetry for the streaming subsystem to enable parity with OpenFang and OpenClaw-like observability.
- Covers metrics, tracing, dashboards, sampling, and rollout governance.

What we will expose (initial):
- Metrics endpoint exposure (Prometheus-style):
  - apex_streaming_active_connections (gauge)
  - apex_streaming_total_connections (counter)
  - apex_streaming_events_total (counter, by type)
  - apex_streaming_errors_total (counter, by category)
  - apex_streaming_latency_seconds (summary/histogram)

- Basic tracing hooks (optional for MVP): correlate stream events with a trace_id from the envelope.

Rollout Phases
- Phase 1: Instrumentation foundation
  - Wire existing StreamingMetrics to expose Prometheus-like metrics via a new /metrics endpoint if this already exists or via an export module.
  - Add basic latency histogram for stream processing.
- Phase 2: Dashboards & dashboards wiring
  - Create dashboards in OpenTelemetry-compatible tooling or Grafana-friendly JSON for streaming dashboards.
- Phase 3: Validation & governance
  - Validate telemetry shapes, ensure privacy/compliance considerations for telemetry data.

Rollout & Backout
- Feature flag gating; rollback path to disable telemetry if risk detected.
- If telemetry introduces performance overhead, reduce sampling rates or disable non-critical metrics.

Owners
- Telemetry lead: @infra-team
- Backend metrics: @backend-team
- Frontend telemetry consumption: @frontend-team
