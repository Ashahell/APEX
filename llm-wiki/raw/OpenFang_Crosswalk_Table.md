# OpenFang to APEX Crosswalk (Table)

- OpenFang Primitive | APEX Equivalent | Data Contracts / Interfaces | Ownership / Notes
---|---|---|---
- Hands (compound) | Hand (MVP) | HAND.toml, System Prompt, Guardrails | Adoption Owner
- MCP (tool registry) | Tool Registry | registry.rs, protocol.rs | MVP bootstrap
- Memory (bounded) | Hermes-style memory store | MemoryStore, EmbeddingIndex (placeholder) | MVP integration
- Security (Merkle audit) | Audit Trail | merkle.rs, audit records | Security lead
- Streaming (SSE/WS) | Live trace stream | /api/v1/streams, SSE endpoints | UI/UX team
- Adapters (Slack/Discord) | Channel adapters | adapters/ folder | Integration team
- UI scaffolding | Hands Monitor UI | HandMonitor component | UI lead
- Memory Whisper | Narrative memory surface | memory/journal | Memory lead
- Hands (compound) | Hand (MVP) | HAND.toml, System Prompt, Guardrails | Adoption Owner
- MCP (tool registry) | Tool Registry | registry.rs, protocol.rs | MVP bootstrap
- Memory (bounded) | Hermes-style memory store | MemoryStore, EmbeddingIndex (placeholder) | MVP integration
- Security (Merkle audit) | Audit Trail | merkle.rs, audit records | Security lead
- Streaming (SSE/WS) | Live trace stream | /api/v1/streams, SSE endpoints | UI/UX team
- Adapters (Slack/Discord) | Channel adapters | adapters/ folder | Integration team

Notes
- This crosswalk is a template. Each row should be fleshed out with concrete contracts, types, and ownership as we finalize the adoption plan.
