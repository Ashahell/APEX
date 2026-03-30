# APEX Parity Plan (OpenClaw / Agent Zero / Hermes / OpenFang)

Executive Summary
- Objective: Achieve production-like parity with the OpenClaw, Agent Zero, Hermes, and OpenFang platforms across streaming UX, telemetry, MCP/tool ecosystems, memory architecture, security, UI, and governance.
- Approach: Phase-driven, architectural uplift paired with a vigorous testing, instrumentation, and runbooks program. Each phase yields concrete artifacts (code changes, tests, dashboards, docs) and passes explicit acceptance criteria before moving to the next phase.
- Guardrails: No god code; no magic numbers; every outcome is verifiable by tests, dashboards, and runbooks.

1) Baseline and Scope
- Current status (as of the latest review):
  - Security: T0–T3 permission model exists; HMAC-based auth and TOTP required for high-privilege actions.
  - Hermes-inspired memory: Bounded memory, session search, auto-created skills framework.
  - Streaming: MVP streaming surface with signed URLs; UI hooks wired; metrics endpoint exists but dashboards are not yet part of the surface.
  - UI: AgentZero-inspired UI with theming, streaming panels, and a Kanban-like board elsewhere; core parity with AgentZero visuals is close, but feature breadth and polish remain behind.
- Parity axes to cover (high-level): Streaming UX, telemetry, MCP/tool ecosystems, memory architecture, agent orchestration and tool-generation patterns, UI parity, security/governance, deployment/observability.
- Deliverables per axis: explicit tickets, docs, tests, dashboards.

2) Governance, Sponsorship, and Planning
- Establish parity governance team with clear responsibilities and decision rights.
- Cadence and gates: Gate reviews after each parity phase; quarterly parity health checks.

3) Parity Framework and Scoring
- Define a parity scorecard with non-magic metrics: Feature Coverage, Quality, Observability, Ecosystem, Documentation.
- Each phase publishes a scorecard with explicit pass/fail gates.

4) Phase Structure (executable, no magic numbers)

Phase 0 — Baseline Refactor and Stabilization
- Goal: Stabilize the surface that parity work will extend from; remove blockers.
- Deliverables:
  - Phase 0 Parity Baseline Document: mapping to target platforms across axes.
  - Updated architecture diagrams showing intended parity surfaces.
  - Preflight test suite: confirm core unit/integration tests pass and CI is green.
- Key Tasks (executable steps):
  - Inventory parity gaps per axis; create canonical interfaces for streaming, telemetry, memory, and MCP.
  - Create a Phase 0 parity baseline ticket set with concrete tasks.
  - Add Phase 0 runbook and acceptance criteria for gating.
  - Establish baseline dashboards and a scoring framework.

Phase 1 — Streaming UX Parity Expansion
- Objective: Streaming UX parity with dashboards, signed URL parity, and UI polish.
- Deliverables:
  - Rich streaming panels; signed surface integration; AgentZero-styled parity UI.
- Executable Steps:
  - Backend: extend streaming surface with richer event types; expose richer metrics.
  - UI: extend StreamingDashboard; integrate with signed URL surface; ensure accessibility.
  - QA: end-to-end tests for streaming surface and signed URL flow.
  - Documentation: update STREAMING_ROLLOUT.md with concrete acceptance criteria.

Phase 2 — Telemetry and Observability Parity
- Objective: Telemetry to parity with top platforms; dashboards and SLOs.
- Deliverables: Telemetry endpoint surface; dashboards; SLO docs.
- Executable Steps: expand metrics (latency, per-endpoint error rates); add monitoring UI surface; maintenance plan.

Phase 3 — Security Review and CI Verification
- Objective: Strengthen security posture; ensure CI/CD gates reflect parity milestones.
- Deliverables: expanded security tests; incident/runbooks; governance alignment.
- Executable Steps: broaden attack surface tests (injection, replay, config validation); integrate linting/formatting; ensure CI green.

Phase 4 — MCP, Tool Ecosystem, and Governance Parity
- Objective: Close MCP tooling, marketplace, and governance gap; provide stable governance cadence.
- Deliverables: MCP endpoints enriched; sample tools; marketplace scaffolding; governance artifacts.
- Executable Steps: MCP tool registry and discovery; minimal plugin surface; governance docs.

Phase 5 — Memory Architecture Parity (Hermes-style)
- Objective: Hermes-like memory surface in UI and back end; memory search integration.
- Deliverables: memory viewer in UI; semantic search hooks; TTL semantics.
- Executable Steps: implement bounded memory visibility in UI; ensure indexer surface is testable.

Phase 6 — UI Parity and Theming
- Objective: Align UI visuals with AgentZero parity; ensure accessibility and UX polish.
- Deliverables: consolidated theming; parity-ready dashboards; improved memory/streaming panels.
- Executable Steps: unify theming pipeline; implement parity UI panels.

Phase 7 — Ecosystem Growth: Plugins, Marketplace, and CI/CD Parity
- Objective: Expand to a broader plugin marketplace; scaffold plugin governance and testing.
- Deliverables: plugin signing surface; marketplace skeleton; CI templates.
- Executable Steps: hub endpoints; UI marketplace; plugin governance doc.

Phase 8 — OpenFang/OpenClaw Crosswalks and Governance Cadence
- Objective: Produce formal crosswalks and governance cadence to sustain parity adoption.
- Deliverables: Crosswalk docs; governance charter and cadence.
- Executable Steps: fill in crosswalk templates; run governance planning session.

Phase 9 — Migration Plan and Pilot Rollout
- Objective: Safe incremental migration; pilot with parity-enabled features.
- Deliverables: migration plan; pilot runbooks; rollback tests.
- Executable Steps: define feature toggles; plan pilot scope; implement rollback paths.

Phase 10 — Verification, QA, and Runbooks Consolidation
- Objective: Consolidate parity results, finalize runbooks and governance artifacts.
- Deliverables: parity scorecards; runbooks; final handover documents.
- Executable Steps: run final verification; publish parity artifacts; conduct governance review.

10) Artifacts and Artifacts Map
- Crosswalk templates (to fill): see dedicated crosswalk documents in docs/ (OpenClaw, AgentZero, Hermes, OpenFang).
- Parity Scorecards per phase.
- Runbooks: streaming incident/runbook, telemetry rollout, security incident response, governance cadence.
- Migration plans and rollout notes.

11) Execution Plan (Next Steps)
- Create Phase 0 tickets with clear owners and acceptance criteria as a tracker-ready draft.
- Populate crosswalk templates; fill in feature mappings as plans mature.
- Establish a governance board to sign off on crosswalks and gates.

Appendix: Crosswalk Templates (to be filled)
- See crosswalk_openclaw_apex.md, crosswalk_agentzero_apex.md, crosswalk_hermes_apex.md, crosswalk_openfang_apex.md for structural templates.
