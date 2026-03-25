OpenFang to APEX Crosswalk — Final (Extended)
Date: 2026-04-XX
Status: Draft

Overview
- This document provides a concrete mapping from OpenFang primitives to APEX components for MVP adoption, with explicit data contracts, interfaces, ownership, and responsibilities.

Crosswalk table (OpenFang primitive -> APEX counterpart)
| OpenFang Primitive | APEX Equivalent | Data Contracts / Interfaces | Ownership / Notes |
|---------------------|-----------------|-----------------------------|-------------------|
| Hands (compound) | Hand (APEX MVP) | HAND.toml, System Prompt, Guardrails | Adoption Owner: adoption-team |
| MCP (tool registry) | Tool Registry | registry.rs, protocol.rs | MVP bootstrap path |
| Memory (bounded) | Hermes-style bounded memory | MemoryStore, EmbeddingIndex | MVP integration point |
| Security (Merkle audit) | Audit Trail | merkle.rs, audit records | Security lead |
| Streaming (SSE/WS) | Live trace stream | /api/v1/streams, SSE endpoints | UI team |
| Adapters (Slack/Discord) | Channel adapters | adapters/ folder | Integration team |
| UI (Hands monitor) | Hands Monitor UI | HandMonitor.tsx | UI lead |
| Memory narrative | Narrative memory surface | memory/journal | Memory lead |

Next steps
- Populate concrete owners and due dates for each row.
- Add explicit API contracts for Hands endpoints (start/status/stream/logs) and tie to the manifest.
- Create diagrams for data flow between Hands, MCP, and memory.

This document is a draft and will be refined in subsequent iterations.
