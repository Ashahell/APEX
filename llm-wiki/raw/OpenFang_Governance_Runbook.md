# OpenFang Adoption Runbook (Pilot)

Purpose
- Provide the step-by-step operational guide for a pilot of OpenFang adoption in APEX.

Pilot Scope
- Phase 1 MVP: Hands MVP (Computer Use) + MCP scaffold + memory scaffolds + MVP API surface.

Preconditions
- Governance cadence and guardrails are approved.
- MVP endpoints are live on the local Axum server.
- Crosswalks are drafted.

Runbook Steps
- Step 1: Validate Hands manifest and runner skeleton exist in core/hands/computer-use/HAND.toml and core/hands/hand_runner.rs
- Step 2: Start the MVP server and verify MVP endpoints respond as expected
- Step 3: Create a sample Hand via /hands/start and verify lifecycle via /hands/status/:name
- Step 4: Exercise /hands/stream/:name and /hands/logs/:name (placeholders now; plan real streaming in next batch)
- Step 5: Trigger /api/v1/computer-use/execute and ensure an MVP task_id is returned and stored in memory
- Step 6: Review governance artifacts and capture learnings for the next cadence

Rollout Criteria
- All MVP endpoints respond deterministically and memory state is updated correctly
- The Hands runner lifecycle is exercised at least once
- Safety guardrails are demonstrably invoked in the MVP flows (manually checked)

Runbook Validation
- Execute test sequence and log results; collect learnings for next iterations
