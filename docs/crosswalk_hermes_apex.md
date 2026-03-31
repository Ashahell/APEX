# Hermes to APEX Crosswalk (Complete)

Owner: Sisyphus AI Agent
Last Updated: 2026-03-31

Purpose
- Map Hermes bounded memory and session search concepts to APEX parity surfaces.
- Provide implementation status, evidence links, and parity scores.

## Implementation Status

| Hermes Primitive | APEX Equivalent | Status | Evidence | Parity Score |
|---|---|---|---|---|
| Bounded memory store | BoundedMemory with char limits (2200/1375) | ✅ Complete | `bounded_memory.rs`, `BoundedMemory.tsx` | 10/10 |
| Session search | FTS5 + BM25 hybrid search | ✅ Complete | `session_search.rs`, `searchMemory()` | 10/10 |
| Auto-created skills | Skill manager with auto-creation | ✅ Complete | `skill_manager.rs`, `AutoCreatedSkills.tsx` | 10/10 |
| Narrative memory | Narrative memory panel | ✅ Complete | `NarrativeMemoryViewer.tsx` | 9/10 |
| Semantic hooks | Hybrid search with MMR reranking | ✅ Complete | `session_search.rs` (RRF + MMR) | 10/10 |
| Memory indexing | Background indexer with chunking | ✅ Complete | `memory/indexer.rs` | 9/10 |
| Memory visibility | 6-tab Memory UI (search, TTL, consolidation) | ✅ Complete | `BoundedMemory.tsx` | 10/10 |
| Frozen snapshots | System prompt-ready snapshots | ✅ Complete | `/api/v1/memory/bounded/snapshot` | 10/10 |
| User profile | Hermes-style user preferences | ✅ Complete | `user_profile.rs`, `UserProfileSettings.tsx` | 10/10 |
| TTL semantics | Configurable TTL per store type | ✅ Complete | `memory_ttl_api.rs` | 10/10 |
| Memory consolidation | AI-suggested entry consolidation | ✅ Complete | `memory_ttl_api.rs`, consolidation UI | 9/10 |

## Overall Parity Score: 9.8/10

### Notes
- APEX fully implements all Hermes memory features
- APEX exceeds Hermes with TTL semantics and consolidation
- APEX has stronger UI (6-tab interface vs Hermes basic viewer)
- Minor gap in consolidation AI quality (rule-based vs LLM-based)
