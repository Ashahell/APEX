# Memory System

## Overview

APEX implements a Hermes-style bounded memory system with multiple storage types and search capabilities.

## Components

### Bounded Memory
- Character-limited stores (2,200 agent / 1,375 user chars)
- Automatic consolidation when approaching limits
- Frozen snapshot for system prompts

### Memory Stores
| Store | Purpose |
|-------|---------|
| Working Memory | Short-term task context |
| Consolidated | Long-term knowledge |
| Narrative | Story/memory persistence |
| TTL Store | Time-based expiration |

### Search
- **Semantic Search**: Hybrid BM25 + embeddings with MMR reranking
- **FTS5**: Full-text search with LIKE fallback
- **BM25**: Ranking algorithm for relevance

### Features
- TTL semantics (configurable per-store)
- Consolidation workflow (AI-suggested merging)
- File-based persistence in ~/.apex/memory/

## API Endpoints
- `/api/v1/memory/bounded/*` - Bounded memory operations
- `/api/v1/memory/stats` - Memory statistics
- `/api/v1/memory/search` - Hybrid search