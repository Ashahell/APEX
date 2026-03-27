PR Parity Checklist (PR C) for Streaming UI

- [ ] DoD alignment: All parity DoD items documented and addressed in PR notes.
- [ ] Tests: All unit and integration tests pass in CI; add UI parity tests where applicable.
- [ ] Backend parity (PR A): UI-friendly SSE auth path implemented and tested; header-based path remains intact.
- [ ] UI parity (PR B): UI skeleton builds cleanly; skeleton routes fetch data and render placeholder panels.
- [ ] Telemetry parity (PR C): Telemetry rollout plan defined and references added to STREAMING_ROLLOUT.md and TELEMETRY_ROLLOUT.md.
- [ ] Runbooks: STREAMING_RUNBOOKS.md and STREAMING_ROLLOUT.md reflect rollout plan and incident response.
- [ ] Rollback plan: Phase-based rollout with feature flags; rollback steps documented.
- [ ] Documentation: All parity docs updated and linked from PR body.
- [ ] CI gates: Ensure Node 24 and Rust tests pass; lint/fmt succeed.
- [ ] Review readiness: Assign reviewers from Frontend, Backend, Telemetry, and Platform teams.
- [ ] Security: Validate that no secrets are exposed; UI uses signed URLs or proxy-based signing for streams.
- [ ] Accessibility: Placeholder notes for accessibility in streaming UI panels.
- [ ] Documentation cross-linking: UI wiring doc references updated in PR body.
