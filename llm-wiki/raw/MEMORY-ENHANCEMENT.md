# APEX Memory System Enhancement Specification

> **Status**: Draft  
> **Based on**: OpenClaw/Memclawz research (Feb 2026)  
> **Version**: 1.0

---

## Executive Summary

This document outlines enhancements to APEX's memory system based on research into OpenClaw (180K+ stars), Mem0, and Memclawz. The current APEX memory system is primitive—file-based narrative writing with no semantic retrieval. This spec introduces a three-tier memory architecture with hybrid search, entity graphs, and working memory.

---

## Current State

### APEX Memory (Now)

```
memory/
├── journal/           # Task narratives (YYYY/MM/*.md)
├── entities/          # Entity files
├── knowledge/          # Knowledge base
└── reflections/        # Agent reflections
```

**Problems:**
- No vector embeddings
- No semantic search
- No entity relationships
- No working memory
- No temporal decay
- Flat file structure

---

## Target Architecture

### Three-Tier Memory System

```
┌─────────────────────────────────────────────────────────────┐
│                    APEX Memory System                        │
├─────────────────────────────────────────────────────────────┤
│  Working Memory (Hot)    │  Short-Term (Warm)  │ Long-Term │
│  ─────────────────────   │  ─────────────────  │ ─────────  │
│  • Scratchpad            │  • Task context     │ • Knowledge │
│  • Active entities       │  • Recent history   │ • Entities  │
│  • Causal links         │  • Vector index     │ • Reflections│
│  <1ms latency           │  ~10ms latency     │ ~50ms      │
├─────────────────────────────────────────────────────────────┤
│  Backend: Redis/Mem0    │  pgvector          │ Filesystem  │
│  (optional)             │  + hybrid search   │ + metadata  │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation Plan

### Phase 1: Core Infrastructure

#### 1.1 Vector Embeddings

**Add pgvector support for semantic search:**

```rust
// In apex-memory
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE memory_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content TEXT NOT NULL,
    embedding vector(1536),  -- OpenAI ada-002 dimension
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    accessed_at TIMESTAMP DEFAULT NOW(),
    access_count INTEGER DEFAULT 0
);

CREATE INDEX ON memory_embeddings 
USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);
```

**Embedding providers supported:**
- OpenAI (text-embedding-ada-002)
- Local (optional)
- Configurable in unified_config

#### 1.2 Hybrid Search

Combine vector similarity with keyword matching:

```rust
// Hybrid query: α × vector_sim + (1-α) × keyword_sim
struct HybridSearchConfig {
    vector_weight: f32 = 0.7,      // 70% semantic
    text_weight: f32 = 0.3,       // 30% keyword
    candidate_multiplier: usize = 4, // Expand pool for better results
    max_results: usize = 8,
}
```

**Features:**
- Max Marginal Relevance (MMR) for diverse results
- Temporal decay (newer memories weighted higher)
- Configurable half-life (default: 30 days)

#### 1.3 Memory API Endpoints

```yaml
POST /api/v1/memory/embeddings    # Store embedding
POST /api/v1/memory/search       # Hybrid search
GET  /api/v1/memory/entities      # List entities
POST /api/v1/memory/entities     # Create entity
GET  /api/v1/memory/graph        # Entity relationships
```

---

### Phase 2: Working Memory

#### 2.1 Scratchpad

Fast in-memory storage for active task context:

```rust
struct WorkingMemory {
    task_id: String,
    active_entities: HashMap<String, Entity>,
    causal_links: Vec<CausalLink>,
    scratchpad: String,  // Active reasoning
}
```

**Features:**
- Sub-millisecond read/write
- Auto-persisted on task completion
- Causal link tracking

#### 2.2 Entity Extraction

Automatically extract entities from task context:

```rust
struct Entity {
    name: String,
    entity_type: String,  // person, project, tool, concept
    attributes: HashMap<String, Value>,
    first_seen: DateTime,
    last_updated: DateTime,
}
```

---

### Phase 3: Graph Memory

#### 3.1 Entity Graph

Track relationships between entities:

```rust
struct EntityGraph {
    nodes: HashMap<String, Entity>,
    edges: Vec<GraphEdge>,  // relationships
}

struct GraphEdge {
    source: String,
    target: String,
    relationship: String,  // "depends_on", "uses", "created_by"
    strength: f32,
}
```

**Queries supported:**
- "What tools did I use for X?"
- "Who depends on Y?"
- "Show all related to Z"

#### 3.2 Causality Tracking (Future)

Like Memclawz v3.0 causality graph:

```rust
struct CausalityNode {
    event: String,
    cause: Option<String>,
    effects: Vec<String>,
    timestamp: DateTime,
}
```

---

## Configuration

### Unified Config Extensions

```rust
pub struct MemoryConfig {
    // Embeddings
    pub embedding_provider: String,  // "openai", "local"
    pub embedding_model: String,      // "text-embedding-ada-002"
    
    // Hybrid search
    pub vector_weight: f32,
    pub text_weight: f32,
    pub max_results: usize,
    pub temporal_decay_days: u32,
    
    // Working memory
    pub working_memory_enabled: bool,
    pub working_memory_backend: String,  // "memory", "redis"
    
    // Graph
    pub entity_graph_enabled: bool,
    
    // Retention
    pub short_term_retention_days: u32,
    pub long_term_retention_days: u32,
}
```

### Environment Variables

```bash
# Memory
APEX_MEMORY_EMBEDDING_PROVIDER=openai
APEX_MEMORY_EMBEDDING_MODEL=text-embedding-ada-002
APEX_MEMORY_VECTOR_WEIGHT=0.7
APEX_MEMORY_TEXT_WEIGHT=0.3
APEX_MEMORY_TEMPORAL_DECAY_DAYS=30
APEX_MEMORY_WORKING_ENABLED=true
APEX_MEMORY_ENTITY_GRAPH=true

# Optional: Redis for working memory
APEX_REDIS_URL=redis://localhost:6379
```

---

## Migration Path

### Step 1: Add pgvector (Non-breaking)
- New tables for embeddings
- Existing narrative files remain unchanged
- New API endpoints for search

### Step 2: Background Indexing
- Scan existing `journal/`, `knowledge/` directories
- Generate embeddings in background
- Update index incrementally

### Step 3: Enable Working Memory
- Add scratchpad to task context
- Entity extraction from new tasks
- Causal link tracking

### Step 4: Graph Layer
- Build entity relationships
- Enable graph queries

---

## Comparison with OpenClaw

| Feature | OpenClaw | APEX (Target) |
|---------|----------|---------------|
| Storage | Markdown files | pgvector + files |
| Search | Hybrid (vector+keyword) | Hybrid |
| Working memory | Memclawz (QMD) | Built-in |
| Entity graph | None | SQLite-backed |
| Causality | Memclawz v3.0 | Future |
| Providers | OpenAI/Gemini/Voyage | OpenAI |

---

## Files to Modify

### New Files
- `core/memory/src/embeddings.rs` - Vector operations
- `core/memory/src/hybrid_search.rs` - Search logic
- `core/memory/src/entity_graph.rs` - Graph layer
- `core/memory/src/working_memory.rs` - Scratchpad

### Modified Files
- `core/memory/src/lib.rs` - Export new modules
- `core/memory/migrations/` - Add pgvector migration
- `core/router/src/api/memory.rs` - New endpoints
- `core/router/src/unified_config.rs` - Memory config

---

## Testing Plan

1. **Unit tests** for embedding generation, hybrid search, graph operations
2. **Integration tests** for API endpoints
3. **Performance tests**:
   - Embedding generation latency (<500ms)
   - Search latency (<50ms)
   - Working memory latency (<1ms)
4. **Memory benchmarks**: Compare with OpenClaw memorySearch

---

## References

- OpenClaw memorySearch (2026): Hybrid vector + keyword search
- Memclawz (2026): 3-tier memory (QMD/Zvec/Causality)
- Mem0: Self-improving memory layer
- 0GMem: Graph-based memory (96% LoCoMo)
- pgvector: PostgreSQL vector operations

---

## Open Questions

1. **Embedding provider**: Use OpenAI API or local model?
2. **Working memory backend**: In-memory only or Redis?
3. **Entity extraction**: LLM-based or rule-based?
4. **Graph storage**: SQLite or continue with pgvector?

---

*Last Updated: 2026-03-06*
