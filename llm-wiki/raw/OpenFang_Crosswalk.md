# OpenFang to APEX Crosswalk

This document maps OpenFang primitives to APEX equivalents to inform the adoption plan.

- Expanded to reflect Hands, MCP, Memory, Security, Streaming, and UI endpoints.

- Hands -> Hand-based autonomy packaging (HAND.toml, System Prompt, SKILL.md)
- MCP -> Tool registry and routing protocol
- Memory/Embeddings -> Hermes-style memory and vector embeddings
- Security -> Merkle audit trails, prompt-safety, gates, sandboxing
- Streaming -> SSE/WS endpoints for hands/tasks
- Adapters -> Channel adapters mapping to Hands updates
- Life-cycle governance -> Hand signing and marketplace concepts

Key crosswalk gaps to close in Phase 1
- Define minimal Hand manifest format and a runner surface in APEX
- Define a tiny registry protocol for 2 tools (computer-use, browser)
- Introduce bounded memory skeleton and embedding placeholder
- Introduce merchant-like governance for Hands (signatures, versions)
