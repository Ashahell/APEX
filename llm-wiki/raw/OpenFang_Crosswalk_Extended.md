# OpenFang vs APEX Crosswalk (Extended) - Rebooted

This document extends the crosswalk with explicit mapping for Hands, MCP, Memory, Security, UI, and API surfaces.

- Hands => MVP Hand manifest, runner, and lifecycle; mapping to HAND.toml, HandRunner, and an MVP Hands API surface
- MCP => Tool Registry bootstrap, a minimal protocol, and a bootstrap path on startup
- Memory/Embeddings => Bounded memory and embedding index skeletons; MVP flow integration points
- Security => Merkle audit trail, prompt scanner, capability gates, and VM sandbox scaffolding
- API/Streaming => Hands lifecycle endpoints, streaming endpoints, and a minimal hands API surface
- UI => Hands Monitor component for status and latency
- Adapters => Slack/Discord/TG skeletons for Hands updates

Gaps and next steps:
- Fill the crosswalk with a concrete mapping table (ownership, contracts, interfaces).
- Extend HAND.toml to include a concrete manifest for governance and rollout.
- Attach concrete owners and due dates to items in the crosswalk.
- Runbook references in the Adoption Runbook to align tasks with governance cadence.
