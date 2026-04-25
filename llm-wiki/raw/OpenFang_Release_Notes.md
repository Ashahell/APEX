# OpenFang Release Notes — ACME MVP Adoption

Date: 2026-04-28
Status: Draft

Overview
- This release cycle delivers the ACME MVP OpenFang adoption journey for APEX, including governance scaffolding, Hands MVP, MCP scaffolding, memory/embedding scaffolding, security foundations, UI scaffolds, onboarding/migration docs, and rollout planning scaffolds. The patch sequence preserved a safe, testable, incremental approach with clear handoffs and sign-off points.

Highlights
- Governance Cadence and Guardrails: templates and scaffolds ready for finalization and sign-off
- Crosswalk: extended OpenFang→APEX crosswalk with Hands, MCP, Memory, Security, API/Streaming, UI, Adapters
- Hands MVP: production-like HAND.toml, HandRunner skeleton, UI scaffolds, and a MVP Hands API surface (start/status/stream/logs)
- Memory/Embedding: EmbeddingIndex placeholder scaffolds plus memory skeletons
- Security: HMAC scaffolding, Merkle-audit trail scaffolding, prompt-scanner scaffolds, VM sandbox scaffolding, capability gates scaffolds
- UI: SecurityPanel and SecurityStatus page scaffolds; MVP adoption dashboard concepts
- Migration/Onboarding: migration runbook and onboarding runbook skeletons
- Rollout: rollout plan, pilot reports, and final handoffs planned

Known caveats
- Some components are scaffolded and will be filled with production-ready logic in subsequent patches.
- The MVP embedding/memory directions are staged for iterative integration.

How to test locally
- Run the local MVP server: cargo run --bin apex_router_bin (from core/router)
- Exercise the MVP endpoints: /api/v1/computer-use/execute, Hands endpoints, and the embedding/memory wiring hooks
- Review governance and crosswalk docs to confirm alignment with your org’s governance cadence

Next steps
- Complete final governance charter and guardrails (Patch 7–8)
- Finalize the crosswalk (Patch 6)
- Finalize Hands manifest and lifecycle (Patch 3–5)
- Complete security integration patches (Patch 19–22)
- Complete rollout dashboards (Patch 18–19 and beyond)
