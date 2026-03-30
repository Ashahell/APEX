# APEX Parity Plan (OpenClaw/Agent Zero/Hermes/OpenFang) – executable plan

This document describes a complete, actionable plan to bring APEX up to parity with its target platforms (OpenClaw, Agent Zero, Hermes, OpenFang). It maps concrete work items, owners, milestones, acceptance criteria, and risk mitigations. The plan is designed to be implementable in the next 12-16 weeks with clearly defined deliverables and gates.

Table of contents:
- Executive summary
- Assumptions & scope
- Target parity map (by domain)
- Phase plan (phases with weeks, tasks, owners, delivery)
- Acceptance criteria by phase
- Testing & QA plan (unit, integration, E2E, performance, security)
- Observability & telemetry plan (metrics, tracing, dashboards)
- Rollout strategy (feature flags, canary, rollback)
- Documentation plan (docs, runbooks, onboarding)
- Risk matrix & mitigation
- Resource & governance plan (team roles, dependencies, cadence)
- Appendix: reference artifacts & links

## Executive summary

APEX has progressed through Phase 2-3 parity efforts (UI wiring, signed URL endpoints, streaming MVP scaffolding, and telemetry scaffolding). The goal of this parity plan is to close the remaining gaps to achieve a production-like, parity-aligned architecture and UX across four target ecosystems (OpenClaw, Agent Zero, Hermes, OpenFang). This plan breaks work into executable phases with concrete deliverables, acceptance criteria, and a realistic 12–16 week cadence.

## Assumptions & scope

- The plan assumes continued alignment with the current repository structure (Rust core router, streaming modules, UI in React/TypeScript, and existing docs).
- The focus is on features and quality that materially affect parity: streaming fidelity, security, telemetry, UI parity, and reliability.
- Non-functional goals include CI stability, test coverage growth, and improved observability.
- This plan intentionally excludes major architectural rewrites unless they clearly unlock parity milestones.

## Target parity map (by domain)

- OpenClaw: streaming dashboards, SSE parity, error handling, telemetry hooks, tool-flow visuals.
- Agent Zero: polished UI, real-time updates, theming, streaming UX polish, user flows.
- Hermes: bounded memory, session search, auto-created skills, telemetry tied to memory/events.
- OpenFang: telemetry, dashboards, SLOs, robust streaming surface, back-end tooling.

## Phase plan (phases with weeks, tasks, owners, delivery)

Phase 0 – Stabilize foundation (Week 0-1)
- Goals: Lock down API surface (streaming_sign), stabilize SSE envelope, ensure tests pass after wiring.
- Deliverables:
  - [ ] Streaming sign router wired end-to-end (core/router) and tested
  - [ ] Lint, unit tests green, cargo check clean
  - [ ] Documented parity scope in STREAMING_ROLLOUT updated (doD alignment)
- Owners: Core Rust team (backend), QA.
- Acceptance criteria: All unit tests pass; streaming endpoints compile; docs reflect current state.

Phase 1 – Phase 2 parity UI wiring (Weeks 2-4)
- Tasks:
  - [ ] Complete UI wiring: StreamingDashboard, Hands, MCP, Task panels render real data from backend
  - [ ] Implement runs for task selector in UI; integrate with /api/v1/tasks
  - [ ] Improve accessibility: ARIA roles, labeling, keyboard navigation checks
- Deliverables:
  - [ ] Streaming UI parity across panels; all panels render with real data
  - [ ] UI tests outline (Playwright/Cypress) added
- Owners: UI/UX team, Frontend.
- Acceptance criteria: UI parity features visible in staging; tests cover basic flows; no critical accessibility blockers.

Phase 2 – Telemetry & Observability (Weeks 5-8)
- Goals: Expose Prometheus-style metrics, tracing hooks, dashboards, and basic SLOs.
- Tasks:
  - [ ] Complete metrics surface at /api/v1/metrics; include required fields (active_connections, total_connections, events, errors, latency)
  - [ ] Add basic OpenTelemetry/graph dashboards JSON for Grafana if applicable
  - [ ] Define and publish SLOs/SLIs for streaming latency and availability
- Deliverables:
  - [ ] Metrics endpoint wired and tested
  - [ ] Grafana/Dashboard JSON artifacts provided or documented
- Owners: Platform/Infra, Backend.
- Acceptance criteria: Metrics surface is stable; dashboards render; SLOs documented with plan to monitor.

Phase 3 – Full parity validation & hardening (Weeks 9-12)
- Goals: Security audit readiness, performance baselining, CI improvements, documentation.
- Tasks:
  - [ ] Conduct security review of streaming endpoints; ensure no obvious surface area leaks
  - [ ] Increase test coverage: add integration tests for streaming surface and end-to-end tests for parity paths
  - [ ] Harden CI: Node 24 in UI pipelines; linting; pre-commit checks; reproducible builds
- Deliverables:
  - [ ] Expanded test suite; updated docs with parity runbooks
- Owners: Security, QA, CI/DevOps.
- Acceptance criteria: No high-severity blockers; CI green; parity docs reflect state.

Phase 4 – Handoff & governance (Weeks 13-16)
- Goals: Final parity review; governance docs; runbooks ready for production-like rollout.
- Tasks:
  - [ ] Final parity review against target platforms; update docs accordingly
  - [ ] Publish runbooks, rollback procedures, and onboarding docs
- Deliverables:
  - [ ] PRs merged; parity completion validated; governance docs finalized
- Owners: Platform teams, Documentation.
- Acceptance criteria: All DoD criteria met, stakeholders sign off, rollout plan in place.

> Note: The schedule assumes steady progress and no major blockers. If blockers arise, we reserve down-scoping or parallelizing tasks to keep milestones intact.

## Acceptance criteria (per phase)
- Phase 0: Foundation stable; no compile/run-time errors; parity doc referenced.
- Phase 1: UI parity panels render data; E2E test skeletons exist; accessibility pass.
- Phase 2: Telemetry exposed; dashboards defined; SLOs defined; metrics surface available.
- Phase 3: Security review and CI hardened; tests green; docs updated.
- Phase 4: Runbooks and governance complete; handoff ready.

## Testing & QA plan
- Unit tests: maintain existing Rust tests; add tests for new streaming_sign, metrics surface, and UI adapters.
- Integration tests: test /api/v1/streams/sign, /api/v1/metrics, and all /stream/* endpoints with mock payloads.
- End-to-end tests (Playwright/Cypress): ensure UI renders streaming panels and reconnects; test signing flow in the UI.
- Performance: baseline latency measurements for first byte, p95 latency for streaming.
- Security: verify HMAC verification path; test token expiry handling; red/green error codes.

## Rollout strategy
- Feature flags: streaming parity features behind flags to enable canary.
- Canary rollout: start with a subset of users; monitor metrics and errors; roll forward or rollback.
- Rollback plan: if parity drift detected, disable UI wiring, revert to a known-good state, and publish runbooks.

## Documentation plan
- Update or add: STREAMING_ROLLOUT.md, TELEMETRY_ROLLOUT.md, UI parity docs (UI_WIREDUP_PARITY.md and UI_STREAMING_CONTRACT.md).
- Include runbooks and onboarding guidance; ensure developer docs reflect current API contracts.

## Governance & Roles
- Core Rust: backend parity and API surface
- UI/UX: parity visuals and interactions
- Platform/Infra: telemetry, dashboards, SLOs
- Security: audits and hardening

## Risks & Mitigation
- Blockers in backend API surface: lock-step parallel work with UI to avoid drift
- Telemetry overload: sampling and rate-limits
- Security regressions: regular audits and CI checks

## Appendix
- References to existing files for parity work:
  - core/router/src/streaming.rs
  - core/router/src/streaming_sign.rs
  - core/router/src/streaming_types.rs
  - core/router/src/api/mod.rs
  - ui/src/components/streaming/StreamingDashboard.tsx
  - docs/STREAMING_ROLLOUT.md
  - docs/TELEMETRY_ROLLOUT.md

End of Plan
