OpenFang Migration Runbook (ACME MVP)

Overview
- This runbook defines a repeatable, low-risk migration path from a OpenFang-centric MVP to a cohesive APEX OpenFang adoption in the ACME MVP scenario.

Preconditions
- Governance cadence is active and milestones are defined
- MVP endpoints are running locally
- Hands manifest is finalized
- Crosswalk is populated with ownership and data contracts

Migration Steps (ACME MVP)
- Step 1: Align MVP scope with Hands MVP (Computer Use)
- Step 2: Migrate Hand manifest to APEX HAND.toml (owner, repo, version)
- Step 3: Migrate microservices and modules (MCP bootstrap, memory scaffolds) into APEX structure
- Step 4: Wire new Hands API surface in APEX router (start/status/stream/logs)
- Step 5: Wire EmbeddingIndex scaffolding into the MVP path
- Step 6: Validate security hooks (Merkle audit trail, prompt scanner, gate checks)
- Step 7: Validate the org's pilot with a 1-2 project run
- Step 8: Document learnings and adjust crosswalk and governance docs

Validation & Rollback
- Validation: smoke tests confirm API surface, MVP Hand runner executes, and embeddings wiring functions
- Rollback: revert changes to the prior state in a controlled manner (backup artifacts in docs)
