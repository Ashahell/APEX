# Hermes to APEX Crosswalk (Template)

Owner: [Name]
Last Updated: [Date]

Purpose
- Map Hermes bounded memory and session search concepts to APEX parity surfaces.
- Provide data contracts, ownership, and notes.

Hermes Primitive | APEX Equivalent | Data Contracts / Interfaces | Ownership / Notes
--- | --- | --- | ---
- Bounded memory store | Hermes bounded memory surface | Data: BoundedMemoryStore { put/get/delete, TTL } | Hermes surface owner
- Session search | Memory search API surface | Data: MemoryQuery, MemoryResult | Hermes search owner
- Auto-created skills | Skills pipeline surface | Data: SkillSpec, YAML Frontmatter | Skills Hub Owner
- Narrative memory / memory viewer | Narrative memory panel | Data: NarrativeEntry, TimeStamp | UI Owner
- Observability surface parity | Observability dashboards | Data: ObservabilityPlan | Ops Owner
- Semantic hooks surface | Hermes semantic hooks surface | Data: SemanticHook, Context | Hermes owner
- Semantic hooks surface | Hermes semantic hooks surface | Data: SemanticHook, Context | Hermes owner
- Memory indexing engine | Hermes Memory Indexer surface | Data: MemoryIndex, EmbeddingIndex | Hermes indexer owner

Notes
- Hermes patterns should be integrated with security and audits.
- Memory indexing engine | Hermes Memory Indexer surface | Data: MemoryIndex, EmbeddingIndex | Hermes indexer owner
- Memory visibility surface | Hermes visibility surface | Data: MemorySnapshot, TTL | Hermes memory owner
