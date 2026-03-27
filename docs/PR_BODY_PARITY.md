PR Body: Parity Rollout (PR C) for Streaming UI

Title
- feat(streaming): Parity rollout, UI wiring, and runbooks

Summary
- This PR closes the parity gap for the APEX streaming UI by delivering:
  - Final parity rollout documentation (STREAMING_ROLLOUT.md)
  - Runbooks for runtime operations (STREAMING_RUNBOOKS.md)
  - Telemetry rollout plan (TELEMETRY_ROLLOUT.md)
- It builds on PR A (backend: UI-friendly SSE auth) and PR B (UI skeleton) to deliver a wired UI parity path.

What is included
- docs/STREAMING_ROLLOUT.md: rollout plan and DoD
- docs/STREAMING_RUNBOOKS.md: incident response and rollback
- docs/TELEMETRY_ROLLOUT.md: telemetry rollout plan
- docs/PARITY_RUNBOOKS.md: parity governance and operational runbooks (already present)
- docs/UI_WIREDUP_PARITY.md: updated parity references and UI wiring guidance
- docs/UI_WIREDUP_PARITY.md (updated): reference to the final parity plan
- docs/UI_WIREDUP_PARITY.md: UI contract and skeleton status notes

Testing & Validation
- Full CI must pass: unit tests, integration tests for streaming, UI skeleton tests (where applicable).
- End-to-end parity checks in staging must confirm that the UI can bind to streams securely and render events end-to-end.

Rollout Plan & Gating
- Phase-based rollout with feature flags to enable/disable parity UI wiring incrementally.
- Rollback plan to revert UI wiring without disrupting backend parity.
- Monitoring: basic dashboards for streaming latency, event counts, error rates.

Definition of Done (DoD)
- All parity docs added and referenced by PRs.
- PR A and PR B green CI; PR C merged with DoD satisfied.
- End-to-end parity validated in staging.

Owners
- Backend parity: @backend-team
- UI parity: @frontend-team
- Telemetry: @infra-team
