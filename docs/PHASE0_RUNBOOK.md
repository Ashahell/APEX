# Phase 0 Runbook: Parity Baseline Stabilization

Goal
- Establish a stable baseline surface for parity work and create executable artifacts to drive Phase 1 planning.

Preconditions
- Codebase builds and tests pass on the baseline (Phase 0 start state).
- Access to repository and CI pipelines.

Phases and Steps
1) Baseline Document (NDEV-P0-01)
- Create a canonical Parity Baseline document in docs/newdevelopment.md referencing this Phase 0 runbook.
- Capture current status across axes: Streaming, Telemetry, Memory, MCP/Tools, UI, Security, Governance.
- Produce a mapping scaffold to fill crosswalks in later steps.

2) Inventory parity gaps (NDEV-P0-02)
- Inventory gaps by axis with a structured table: Axis, Gap, Impact, Proposed Approach.
- Deliver a short, actionable plan per gap for Phase 1.

3) Define canonical interfaces (NDEV-P0-03)
- Draft interface definitions for: Streaming surface, Telemetry surface, Memory surface, MCP/Tools surface, UI branding surface.
- Provide at least one example payload contract for each interface.

4) Phase 0 gating and acceptance (NDEV-P0-04)
- Identify explicit pass/fail criteria for Phase 0 gates.
- Document sign-off process with owners and review cadence.

5) Phase 0 Runbook (NDEV-P0-05)
- Create a step-by-step, repeatable runbook to reproduce Phase 0 and gate Phase 1.
- Include preflight checks, test commands, acceptance criteria, and a rollback plan.

6) Parity Scorecard skeleton (NDEV-P0-06)
- Create a skeleton parity scorecard to fill per axis after each phase.
- Define pass criteria for Phase 0 in the scorecard.

7) Phase 0 kickoff (NDEV-P0-07)
- Publish a kickoff invite with a phased plan, milestones, and owners.
- Record minutes and decisions.

8) Crosswalk kickoff brief (NDEV-P0-08)
- Prepare a briefing document that explains how to fill crosswalk templates and assigns owners per axis.

Artifacts
- docs/newdevelopment.md references Phase 0 artifacts.
- docs/crosswalk_openclaw_apex.md (template), docs/crosswalk_agentzero_apex.md (template), docs/crosswalk_hermes_apex.md (template), docs/crosswalk_openfang_apex.md (template).
- docs/PHASE0_RUNBOOK.md (this runbook).
- docs/parity-scorecard.md (skeleton).
- docs/phase0_tickets.json (tracker-ready tickets).

Notes
- This runbook is intended to be extended as work progresses and will drive the Phase 1 backlog.

## Phase 0 Execution Details (Concrete Commands)
Prerequisites: Ensure Rust toolchain, Node, Python, and necessary services are installed and accessible.
Steps:
- 1) Build baseline
- Command: cargo check -p apex-router
- Command: cargo test --lib
- 2) Start services (if applicable in dev environment)
- Command: # Start LLM (optional)
- Command: # Start Router
- 3) Validate streaming surface
- Command: curl -s http://localhost:3000/api/v1/streams/sign?path=/stream/stats
- 4) Validate artifacts and crosswalks
  - Command: ls docs/crosswalk_*.md
- 5) Capture baseline run outputs
  - Command: echo 'Phase 0 baseline captured' >> docs/PHASE0_RUNBOOK.md.snapshot

### OS-specific execution blocks

#### Linux/macOS
- 1) Prepare environment
- Command: rustup default stable
- Command: cargo check -p apex-router
- Command: cargo test --lib
- 2) Run parity baseline router (optional in CI)
- Command: cargo run --release --bin apex-router &
- Command: curl -s http://localhost:3000/api/v1/streams/sign?path=/stream/stats
- Observe streaming surface and collect logs to docs/phase0_outputs/linux_run.log

#### Windows
- 1) Prepare environment
- Command: rustup default stable
- Command: cargo check -p apex-router
- Command: cargo test --lib
- 2) Run parity baseline router (in CMD or PowerShell)
- Command: cargo run --release --bin apex-router
- Command: curl -s http://localhost:3000/api/v1/streams/sign?path=/stream/stats
- Observe streaming surface and collect logs to docs/phase0_outputs/windows_run.log

- ### Phase 0 Acceptance Criteria
- Verify Phase 0 gates in docs/phase0_gating.md pass: Baseline artifacts, crosswalk skeletons, tickets populated, runbook executable, scorecard skeleton filled.
- Save a run summary in docs/phase0_outputs with key results and artifacts.

### Phase 0 Verification Artifacts ( recommended outputs )
- Linux run log: docs/phase0_outputs/linux_run.log
- Windows run log: docs/phase0_outputs/windows_run.log
- Runbook execution snapshot: docs/phase0_snapshot.md
- Phase 0 results summary: docs/phase0_outputs/run_summary.md
