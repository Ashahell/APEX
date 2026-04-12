Plan to Fix Honest Gaps in APEX

Overview
- Target: Animation polish, Event correlation, Consolidation AI, Hands visualization.
- Approach: Short, incremental, low-risk changes with clear test coverage and rollback plans. Each item includes scope, design, tasks, owners, milestones, and acceptance criteria.

1) Animation polish (UI micro-interactions)
- Objective: Improve perceived responsiveness and polish of transitions and micro-interactions across key UI surfaces (AgentZero styling, Story views, Kanban, Settings, etc.).
- Scope: 3-5 components with noticeable motion gaps; avoid heavy refactors.
- Design: Use Framer Motion or CSS transitions; ensure accessibility (reduced motion support).
- Tasks:
  a. Audit major transition points (App load, panel open/close, button hover, list insert/remove).
  b. Implement motion tokens in CSS variables; add a small motion package (framer-motion) if not present.
  c. Apply motion to 3 selected components: StoryList expansion, Story editor modal, and Kanban card moves.
  d. Accessibility: respect user’s reduced motion preference; guard motion.
- Validation: manual QA and a11y checks; optional automated UI tests for motion presence.
- Milestones: v1 motion baseline in 2 weeks; v1.1 polish pass in 1 more week.
- Acceptance Criteria: Smooth transitions, no layout jank, no motion when reduced motion is enabled.

2) Event correlation IDs (Streaming)
- Objective: Enrich streaming events with deterministic correlation IDs to improve traceability across services and UI.
- Scope: Add correlation_id to all streaming events (start/end/session events, hand events, tool events) and propagate to UI via StreamingSurface types.
- Design: Extend streaming_types.rs with a canonical CorrelationId type; generate correlation_id at session start, propagate through events.
- Tasks:
  a. Extend Rust streaming_types.rs with correlation_id field on all event payloads.
  b. Update event emitters in Hands/MCP to populate correlation_id (session or request-scoped).
  c. Update UI TypeScript types to carry correlation_id; render in streaming UI (visible in debug panel or tooltips).
  d. Update tests: unit tests for correlation_id presence and non-empty values; integration tests for event flow.
- Validation: run existing streaming tests; add new tests; manual end-to-end trace.
- Milestones: initial backend change in 1 sprint; UI update in next sprint; end-to-end test coverage in 2 sprints.
- Acceptance Criteria: All streaming events carry correlation_id, UI shows it in logs, tests pass.

3) Consolidation AI (Memory consolidation)
- Objective: Provide AI-assisted memory consolidation as an optional path (default still rule-based).
- Scope: Toggle in config, small AI helper that can propose consolidation of similar memories, with an allowlist/operator controls.
- Design: Add a feature flag APEX_MEMORY_CONSOLIDATION_AI; implement a new consolidation_ai module in Rust that, when enabled, uses a lightweight LM (or local heuristic) to merge related memories and create a compact summary item; ensure privacy controls.
- Tasks:
  a. Add config flag and UI toggle in Settings → Memory settings.
  b. Implement consolidation_ai.rs (or module) that takes memory entries and outputs merged entries or a summary.
  c. Wire into memory TTL/indexing flow; ensure no data leakage; log actions for audit.
  d. Tests: unit tests for consolidation decision logic; privacy tests; integration tests with sample memories.
- Validation: simulate datasets; verify new consolidated output incrementally; ensure backward compatibility when AI is disabled.
- Milestones: AI path ready for pilot in 3 sprints; evaluate improvements and privacy impact.
- Acceptance Criteria: When enabled, consolidation AI runs without errors, produces sensible merges, and audit logs exist.

4) Hands Visualization (HandMonitor improvements)
- Objective: Improve the Hands visualization for streaming hands with richer metrics and clearer state signals.
- Scope: Optimize UI rendering of HandMonitor, add metrics (latency, status, last event), and optional mini-charts.
- Design: Add a small micro-interaction for hand state transitions; add a compact inline dashboard inside HandMonitor or adjacent panel.
- Tasks:
  a. Inspect HandMonitor.tsx and identify hotspots for perf and UX polish.
  b. Add new fields: latency, last_event, status; wire through from streaming data.
  c. Integrate simple inline charts (SVG sparkline) or small progress indicators.
  d. Update tests UI for new fields and ensure accessibility.
- Validation: manual QA; ensure no perf regressions; test with mocked streaming data.
- Milestones: Hands polish baseline in 1 sprint; extended visuals in another sprint.
- Acceptance Criteria: No visual regressions; clearer hand state representation; small performance overhead.

Risks & Mitigations
- Risk: AI-based consolidation could introduce privacy concerns. Mitigation: keep AI off by default; add a strict opt-in; log all actions for audit; scrub PII where possible.
- Risk: Adding correlation IDs increases payloads slightly; Mitigation: keep it compact; default to short IDs; ensure backward-compatibility.
- Risk: Motion-heavy UI may affect accessibility; Mitigation: respect reduced-motion preference; provide toggle to disable.

Validation & Rollout Plan
- Stage 1: Backend-only changes for correlation IDs; add tests.
- Stage 2: UI changes to display correlation IDs if present; add UI tests.
- Stage 3: Enable Consolidation AI in a feature flag with audit logging; run in staging with synthetic data.
- Stage 4: Hands Visualization enhancements; ensure performance budgets.
- Rollback: if any change causes failures, revert the patch and rollback to previous crosswalk parity state; keep a changelog entry.

Deliverables
- docs/PLAN_GAPS_FIX.md (this document)
- Code changes for correlation IDs, polishing animations, AI consolidation (flagged), HandMonitor enhancements
- Tests: unit/integration/UI tests for new features
- Updated crosswalk parity docs if needed

Owner & Timeline
- Owner: You / Sisyphus AI Agent (coordination via plan)
- Target: complete within 4–6 weeks with staggered validation cycles.

Notes
- This plan is designed to be incremental and reversible.
- All changes must go through the CI gates; add tests to CI where possible.
