# Phase 1 Gate: Streaming UX Parity Expansion

## Overview
- Phase 1 expands the streaming UX with richer event types, metrics, signed URL surface, and UI panels.
- This document defines the gates, acceptance criteria, and sign-off workflow for Phase 1.

## Gate Criteria (Pass/Fail)

| Gate | Criterion | Verification | Status |
|------|-----------|--------------|--------|
| **1.1** | Rich event types implemented: session_start, session_end, heartbeat, checkpoint, user_intervention | Code review: streaming_types.rs has new event variants | ✅ PASS |
| **1.2** | Streaming metrics expanded: connection_duration, events_per_second, client_latency, stream_health | `/api/v1/metrics` returns new metrics | ✅ PASS |
| **1.3** | Signed URL surface integrated: query param auth works for SSE | curl test with signed URL succeeds | ✅ PASS |
| **1.4** | UI StreamingDashboard wired: Hands/MCP/Task/Stats panels functional | UI renders panels, events stream in | ✅ PASS |
| **1.5** | E2E streaming tests pass: 10+ tests covering auth, delivery, reconnection | `cargo test streaming_integration` passes | ✅ PASS |
| **1.6** | Phase 1 documentation complete: STREAMING_ROLLOUT.md updated, runbook exists | Docs reviewed and merged | ✅ PASS |
| **1.7** | Gate review signed off | Governance sign-off recorded | ⏳ PENDING |

## SLO Targets (Must Meet)

| Metric | Target | Phase 1 Baseline |
|--------|--------|-------------------|
| Availability | 99.9% | 99.5% |
| Latency p95 | < 500ms | < 800ms |
| Error Rate | < 0.1% | < 0.5% |
| Reconnection Success | > 99% | > 95% |

## Sign-off
- Phase 1 Owner signs off and moves to Phase 2.
- Any gate not met requires corrective plan before Phase 2.

## Mitigations and Escalation
- If any gate cannot be met within the agreed window, escalate to governance board with corrective plan.
- Document blockers in phase1_blockers.md.
