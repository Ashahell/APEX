# APEX Memory System Specification

**Status**: Final Draft  
**Version**: 2.0  
**Architecture ref**: APEX v1.0.0  
**Supersedes**: MEMORY-ENHANCEMENT.md v1.0 (rejected — see design rationale throughout)  
**Last Updated**: 2026-03-06

---

## 1. Executive Summary

APEX already has the right memory structure: files organised into `journal/`, `entities/`, `knowledge/`, and `reflections/` directories. What it lacks is a retrieval layer on top of those files. This spec adds exactly that — no more, no less.

The three core additions are:

1. **Semantic search** via `sqlite-vec` — vector embeddings stored in the existing SQLite database, no additional services
2. **Hybrid scoring** via SQLite FTS5 — BM25 keyword search merged with vector similarity using Reciprocal Rank Fusion, with temporal decay applied before final ranking
3. **Working memory** via in-process Rust `HashMap` — per-task scratchpad, write-through persisted, zero external dependencies

Redis is not used. PostgreSQL is not used. pgvector is not used. The entire memory system runs in-process against a single SQLite file, consistent with APEX's existing single-binary deployment model.

**Target latencies (measured, not aspirational):**

| Operation | Target | Method |
|---|---|---|
| Hybrid search (top-8 results) | < 30ms | sqlite-vec KNN + FTS5 BM25 + RRF |
| Working memory read/write | < 1ms | In-process HashMap |
| Embedding generation | < 200ms | Local `nomic-embed-text` via llama-server |
| Background indexing | non-blocking | Tokio task, rate-limited |

---

## 2. Design Rationale

### 2.1 Why not pgvector

pgvector requires PostgreSQL. APEX defaults to SQLite. Switching to PostgreSQL for vector search while keeping SQLite for everything else creates a two-database deployment that breaks the single-binary model, adds an operational dependency, and forces every APEX user to run a PostgreSQL instance for what is a single-user application. `sqlite-vec` eliminates the need for additional infrastructure entirely and keeps all data in the existing `apex.db` file.

### 2.2 Why sqlite-vec over sqlite-vector

Two production-ready SQLite vector extensions exist as of early 2026. `sqlite-vector` insert time is ~50% faster than `sqlite-vec` and plain query time is ~16% faster; with quantization, queries run in under 4ms with perfect recall. However, `sqlite-vector` is under the Elastic License 2.0 for production use, which may conflict with APEX's license. `sqlite-vec` is MIT licensed and maintained by Alex Garcia with broad community adoption. **`sqlite-vec` is the default choice. If APEX is released under a license compatible with the Elastic License 2.0, revisit `sqlite-vector` for its quantization performance advantage.**

### 2.3 Why nomic-embed-text over OpenAI ada-002

The original spec hardcodes `vector(1536)` for OpenAI `text-embedding-ada-002`. This bakes an external API dependency into the schema at the byte level. `nomic-embed-text` surpasses OpenAI `text-embedding-ada-002` and `text-embedding-3-small` performance on both short and long context tasks, runs locally via llama-server (already in APEX's stack), produces **768-dimension** embeddings, and costs nothing per call. The schema is defined at 768 dimensions. Changing this later requires a full re-index — this decision is locked in at migration time.

### 2.4 Why RRF over weighted alpha blend

The original spec proposes `score = α × vector_sim + (1-α) × keyword_sim`. This requires normalising both score distributions to a common range before blending, which is non-trivial when BM25 scores are unbounded. RRF is the best starting point for hybrid search because of its simplicity and resilience to mismatched score scales. It produces strong results without extensive tuning, making it ideal for prototyping or when retrievers overlap. RRF is simpler, more robust, and requires no calibration. It is the default. The weighted blend is available as a config option for users who want to tune.

### 2.5 Why agents naturally use keyword search

BM25 indexes in milliseconds and queries in microseconds. No embedding model to call, no vector comparisons to run, no chunking strategy to tune. Agents search with keywords and BM25 matches keywords — the fit is natural rather than approximate. Every millisecond of latency is a millisecond added to the thinking loop, and it compounds across a session with dozens of searches. BM25 via SQLite FTS5 is therefore not merely a component of hybrid search — it is the fast path for keyword-heavy agent queries and should be exposed independently.

### 2.6 Why defer graph memory

Entity graphs are the right long-term direction. They are not the right next step. The retrieval gap (no semantic search) is the primary problem. Graph construction requires entity extraction, which either requires rule-based heuristics (brittle) or an LLM call per chunk (expensive during indexing). The correct order is: build the search layer first, measure what queries actually fail, then determine whether graph traversal would fix those failures. Graph memory is specified here as Phase 3 with explicit prerequisite gates.

---

## 3. Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        APEX Router Process                          │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                    MemoryService                             │  │
│  │                                                              │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │  │
│  │  │  Working    │  │   Search     │  │   Embedder       │   │  │
│  │  │  Memory     │  │   Engine     │  │   Client         │   │  │
│  │  │  (in-proc)  │  │  (hybrid)    │  │  (llama-server)  │   │  │
│  │  └─────────────┘  └──────┬───────┘  └────────┬─────────┘   │  │
│  │                          │                    │              │  │
│  └──────────────────────────┼────────────────────┼──────────────┘  │
│                             │                    │                  │
│                    ┌────────▼──────────┐         │                  │
│                    │     apex.db       │◄────────┘                  │
│                    │                   │  (writes embeddings)       │
│                    │  memory_chunks    │                            │
│                    │  memory_fts       │                            │
│                    │  memory_vec       │                            │
│                    │  memory_entities  │                            │
│                    └───────────────────┘                            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
                             │
                    existing filesystem
                    journal/ entities/
                    knowledge/ reflections/
```

The filesystem is the **source of truth**. The database is the **search index**. If the database is deleted, it is fully reconstructable from the filesystem via the background indexer.

---

## 4. Embedding Provider

### 4.1 Model Selection

**Default: `nomic-embed-text` via llama-server**

| Property | Value |
|---|---|
| Model | `nomic-embed-text-v1` or `nomic-embed-text-v1.5` |
| Dimensions | **768** (fixed — determines schema) |
| Context length | 8192 tokens |
| Serving | llama-server (already in APEX stack) |
| License | Apache 2.0 |
| API compatibility | OpenAI `/v1/embeddings` endpoint |

Load in llama-server:

```bash
llama-server \
  --model nomic-embed-text-v1.5.Q4_K_M.gguf \
  --embedding \
  --port 8081 \        # separate port from LLM server
  --ctx-size 8192
```

**Fallback: OpenAI `text-embedding-3-small`**  
Available when `APEX_EMBEDDING_PROVIDER=openai`. **Produces 1536-dimension vectors — requires a separate schema migration.** Both providers cannot coexist in the same database. The provider is a deployment-time decision, not a runtime switch.

### 4.2 Embedding Client (`embedder.rs`)

```rust
// core/memory/src/embedder.rs

#[derive(Debug, Clone)]
pub enum EmbeddingProvider {
    Local { url: String, model: String },   // llama-server
    OpenAI { api_key: String, model: String },
}

pub struct Embedder {
    provider: EmbeddingProvider,
    client:   reqwest::Client,
    dim:      usize,  // 768 for nomic, 1536 for OpenAI — validated at startup
}

impl Embedder {
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        let prefixed = format!("search_document: {}", text); // nomic requires prefix
        match &self.provider {
            EmbeddingProvider::Local { url, model } => {
                self.embed_local(url, model, &prefixed).await
            }
            EmbeddingProvider::OpenAI { api_key, model } => {
                self.embed_openai(api_key, model, text).await // OpenAI doesn't use prefix
            }
        }
    }

    pub async fn embed_query(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        // Query prefix differs from document prefix for nomic
        let prefixed = match &self.provider {
            EmbeddingProvider::Local { .. } => format!("search_query: {}", text),
            EmbeddingProvider::OpenAI { .. } => text.to_string(),
        };
        self.embed_raw(&prefixed).await
    }

    /// Called at startup — verifies the provider returns vectors of the expected dimension.
    /// Hard-fails if dimension mismatch: a 1536-dim vector stored in a 768-dim schema
    /// silently corrupts the index.
    pub async fn validate_dimension(&self, expected: usize) -> Result<(), EmbedError> {
        let test_vec = self.embed("dimension validation probe").await?;
        if test_vec.len() != expected {
            return Err(EmbedError::DimensionMismatch {
                expected,
                actual: test_vec.len(),
            });
        }
        Ok(())
    }
}
```

---

## 5. Database Schema

### 5.1 Migration: `memory_chunks`

```sql
-- migration: add_memory_chunks
-- Stores chunked text from all memory files.
-- The filesystem file is the source of truth; this table is the searchable index.

CREATE TABLE IF NOT EXISTS memory_chunks (
    id          TEXT PRIMARY KEY,           -- UUID
    file_path   TEXT NOT NULL,              -- relative to APEX_SOUL_DIR, e.g. journal/2026/03/task-abc.md
    chunk_index INTEGER NOT NULL,           -- 0-based index within the file
    content     TEXT NOT NULL,              -- raw chunk text
    word_count  INTEGER NOT NULL,           -- for BM25 length normalisation context
    memory_type TEXT NOT NULL,              -- 'journal' | 'knowledge' | 'entity' | 'reflection'
    task_id     TEXT,                       -- if this chunk originated from a task
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    accessed_at TEXT NOT NULL DEFAULT (datetime('now')),
    access_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(file_path, chunk_index)
);

CREATE INDEX IF NOT EXISTS idx_memory_chunks_type     ON memory_chunks(memory_type);
CREATE INDEX IF NOT EXISTS idx_memory_chunks_task     ON memory_chunks(task_id);
CREATE INDEX IF NOT EXISTS idx_memory_chunks_accessed ON memory_chunks(accessed_at DESC);
```

### 5.2 Migration: `memory_fts` (BM25 via FTS5)

```sql
-- FTS5 virtual table for BM25 keyword search.
-- Content table mirrors memory_chunks — deletes and updates must be manually synced.

CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
    content,
    memory_type UNINDEXED,
    content='memory_chunks',
    content_rowid='rowid',
    tokenize='porter unicode61'   -- Porter stemming: "running" matches "run"
);

-- Triggers to keep FTS index in sync with memory_chunks
CREATE TRIGGER memory_chunks_ai AFTER INSERT ON memory_chunks BEGIN
    INSERT INTO memory_fts(rowid, content, memory_type)
    VALUES (new.rowid, new.content, new.memory_type);
END;

CREATE TRIGGER memory_chunks_ad AFTER DELETE ON memory_chunks BEGIN
    INSERT INTO memory_fts(memory_fts, rowid, content, memory_type)
    VALUES ('delete', old.rowid, old.content, old.memory_type);
END;

CREATE TRIGGER memory_chunks_au AFTER UPDATE ON memory_chunks BEGIN
    INSERT INTO memory_fts(memory_fts, rowid, content, memory_type)
    VALUES ('delete', old.rowid, old.content, old.memory_type);
    INSERT INTO memory_fts(rowid, content, memory_type)
    VALUES (new.rowid, new.content, new.memory_type);
END;
```

### 5.3 Migration: `memory_vec` (sqlite-vec)

```sql
-- sqlite-vec virtual table for KNN vector search.
-- Dimension is 768 — matches nomic-embed-text output.
-- THIS VALUE IS LOCKED IN. Changing it requires dropping and recreating
-- this table and re-indexing all chunks.

CREATE VIRTUAL TABLE IF NOT EXISTS memory_vec USING vec0(
    chunk_id    TEXT PARTITION KEY,    -- matches memory_chunks.id
    embedding   float[768]             -- nomic-embed-text-v1 / v1.5 dimension
);
```

### 5.4 Migration: `memory_entities`

```sql
-- Lightweight entity store.
-- No graph edges in this phase — entity relationship tracking is Phase 3.

CREATE TABLE IF NOT EXISTS memory_entities (
    id           TEXT PRIMARY KEY,         -- UUID
    name         TEXT NOT NULL,
    entity_type  TEXT NOT NULL,            -- 'person' | 'project' | 'tool' | 'concept' | 'file'
    attributes   TEXT NOT NULL DEFAULT '{}',  -- JSON blob
    first_seen   TEXT NOT NULL DEFAULT (datetime('now')),
    last_updated TEXT NOT NULL DEFAULT (datetime('now')),
    mention_count INTEGER NOT NULL DEFAULT 1
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_memory_entities_name_type
    ON memory_entities(name, entity_type);
```

### 5.5 Migration: `memory_index_state`

```sql
-- Tracks which files have been indexed and at what mtime.
-- The background indexer uses this to skip unchanged files.

CREATE TABLE IF NOT EXISTS memory_index_state (
    file_path   TEXT PRIMARY KEY,
    mtime_unix  INTEGER NOT NULL,
    chunk_count INTEGER NOT NULL,
    indexed_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

---

## 6. Chunking Strategy

Files are split into overlapping chunks before indexing. The chunker runs in the background indexer and on write for new memory entries.

```rust
// core/memory/src/chunker.rs

pub struct ChunkerConfig {
    pub chunk_size_tokens:    usize,   // default: 256
    pub overlap_tokens:       usize,   // default: 32
    pub min_chunk_tokens:     usize,   // default: 20  — skip tiny trailing chunks
    pub respect_headings:     bool,    // default: true — never split at a markdown heading
    pub respect_code_blocks:  bool,    // default: true — never split inside ``` blocks
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            chunk_size_tokens:   256,
            overlap_tokens:      32,
            min_chunk_tokens:    20,
            respect_headings:    true,
            respect_code_blocks: true,
        }
    }
}

/// Split text into overlapping chunks.
/// Returns Vec<(chunk_index, chunk_text)>.
pub fn chunk_text(text: &str, config: &ChunkerConfig) -> Vec<(usize, String)> {
    // 1. Identify hard split boundaries (headings, code fence open/close)
    // 2. Split into sentences using unicode sentence boundary detection
    // 3. Greedily accumulate sentences up to chunk_size_tokens
    // 4. Overlap: retain last `overlap_tokens` worth of sentences
    //    for the start of the next chunk
    // ... implementation omitted for brevity — see chunker.rs
}
```

**Why 256 tokens with 32 overlap?**
- Local Granite embedding model produces 768-dimension embeddings effective at the 256-token chunk size in production RAG — consistent with nomic's optimal input range
- 32-token overlap prevents context loss at chunk boundaries
- Smaller chunks (128) increase recall but double index size
- Larger chunks (512) are faster to index but reduce retrieval precision

---

## 7. Hybrid Search Engine

### 7.1 Algorithm

```
query
  │
  ├── embed_query(query)  →  query_vec (768-dim)
  │
  ├── sqlite-vec KNN(query_vec, k=32)  →  vec_results[(chunk_id, distance)]
  │
  ├── FTS5 BM25(query, limit=32)       →  bm25_results[(chunk_id, bm25_score)]
  │
  ├── RRF merge(vec_results, bm25_results, k=60)  →  fused_scores[(chunk_id, rrf_score)]
  │
  ├── × temporal_decay(chunk.accessed_at)  →  decayed_scores
  │
  ├── MMR diversity filter(decayed_scores, λ=0.7)  →  diverse_top_N
  │
  └── fetch chunk content + metadata  →  SearchResult[]
```

### 7.2 Reciprocal Rank Fusion

```rust
// core/memory/src/hybrid_search.rs

/// RRF score for a document appearing at rank r in a result list.
/// k=60 is the standard constant — empirically validated, rarely needs tuning.
fn rrf_score(rank: usize, k: usize) -> f64 {
    1.0 / (k + rank) as f64
}

/// Merge two ranked lists using RRF.
pub fn reciprocal_rank_fusion(
    vec_ranks:  &[(String, usize)],   // (chunk_id, rank) — rank is 0-based
    bm25_ranks: &[(String, usize)],
    k:          usize,                // default: 60
) -> Vec<(String, f64)> {
    let mut scores: HashMap<String, f64> = HashMap::new();

    for (chunk_id, rank) in vec_ranks {
        *scores.entry(chunk_id.clone()).or_default() += rrf_score(*rank, k);
    }
    for (chunk_id, rank) in bm25_ranks {
        *scores.entry(chunk_id.clone()).or_default() += rrf_score(*rank, k);
    }

    let mut merged: Vec<(String, f64)> = scores.into_iter().collect();
    merged.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    merged
}
```

### 7.3 Temporal Decay

Applied after RRF, before MMR. Newer memories and frequently accessed memories rank higher.

```rust
/// Exponential decay multiplier.
/// score × decay(age) where age is days since last access.
/// half_life_days = 30: a memory accessed 30 days ago scores at 50% of a fresh memory.
/// half_life_days = 90: more gradual decay — better for long-term knowledge.
pub fn temporal_decay(accessed_at: chrono::DateTime<Utc>, half_life_days: f64) -> f64 {
    let age_days = (Utc::now() - accessed_at).num_seconds() as f64 / 86_400.0;
    2.0_f64.powf(-age_days / half_life_days)
}

/// Access frequency boost: log(1 + access_count) — diminishing returns.
pub fn frequency_boost(access_count: u64) -> f64 {
    (1.0 + access_count as f64).ln()
}

/// Combined score applied to each candidate after RRF.
pub fn apply_temporal_score(
    rrf_score:    f64,
    accessed_at:  chrono::DateTime<Utc>,
    access_count: u64,
    half_life_days: f64,
) -> f64 {
    rrf_score * temporal_decay(accessed_at, half_life_days) * frequency_boost(access_count).max(1.0)
}
```

### 7.4 Max Marginal Relevance (MMR)

Prevents the top-8 results from being 8 paraphrases of the same memory chunk.

```rust
/// MMR selection. λ=1.0 is pure relevance; λ=0.0 is pure diversity.
/// λ=0.7 is the practical default — prioritises relevance but avoids redundancy.
pub fn mmr_select(
    candidates:   &[(String, f64, Vec<f32>)],  // (chunk_id, score, embedding)
    query_vec:    &[f32],
    n:            usize,
    lambda:       f64,
) -> Vec<String> {
    let mut selected: Vec<(String, Vec<f32>)> = Vec::with_capacity(n);
    let mut remaining = candidates.to_vec();

    while selected.len() < n && !remaining.is_empty() {
        let best_idx = remaining.iter().enumerate().max_by(|(_, a), (_, b)| {
            let a_relevance = cosine_similarity(&a.2, query_vec);
            let a_redundancy = selected.iter()
                .map(|(_, s_emb)| cosine_similarity(&a.2, s_emb))
                .fold(0.0_f32, f32::max);
            let a_mmr = lambda * a_relevance as f64 - (1.0 - lambda) * a_redundancy as f64;

            let b_relevance = cosine_similarity(&b.2, query_vec);
            let b_redundancy = selected.iter()
                .map(|(_, s_emb)| cosine_similarity(&b.2, s_emb))
                .fold(0.0_f32, f32::max);
            let b_mmr = lambda * b_relevance as f64 - (1.0 - lambda) * b_redundancy as f64;

            a_mmr.partial_cmp(&b_mmr).unwrap_or(std::cmp::Ordering::Equal)
        }).map(|(i, _)| i).unwrap();

        let chosen = remaining.remove(best_idx);
        selected.push((chosen.0.clone(), chosen.2.clone()));
    }

    selected.into_iter().map(|(id, _)| id).collect()
}
```

### 7.5 Full Search Call

```rust
pub struct SearchQuery {
    pub text:            String,
    pub memory_types:    Option<Vec<String>>,  // filter to specific types
    pub max_results:     usize,                // default: 8
    pub lambda_mmr:      f64,                  // default: 0.7
    pub half_life_days:  f64,                  // default: 30.0
    pub min_score:       Option<f64>,          // filter out low-confidence results
}

pub struct SearchResult {
    pub chunk_id:      String,
    pub file_path:     String,
    pub content:       String,
    pub memory_type:   String,
    pub score:         f64,
    pub accessed_at:   chrono::DateTime<Utc>,
    pub access_count:  u64,
}

impl HybridSearchEngine {
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        // 1. Embed the query
        let query_vec = self.embedder.embed_query(&query.text).await?;

        // 2. Vector KNN — top 32 candidates
        let vec_results = self.knn_search(&query_vec, 32, &query.memory_types).await?;

        // 3. BM25 keyword — top 32 candidates
        let bm25_results = self.bm25_search(&query.text, 32, &query.memory_types).await?;

        // 4. RRF merge
        let vec_ranks:  Vec<(String, usize)> = vec_results.iter()
            .enumerate().map(|(i, (id, _))| (id.clone(), i)).collect();
        let bm25_ranks: Vec<(String, usize)> = bm25_results.iter()
            .enumerate().map(|(i, (id, _))| (id.clone(), i)).collect();

        let fused = reciprocal_rank_fusion(&vec_ranks, &bm25_ranks, 60);

        // 5. Fetch metadata for fused candidates
        let chunk_ids: Vec<String> = fused.iter().map(|(id, _)| id.clone()).collect();
        let chunks = self.fetch_chunks(&chunk_ids).await?;

        // 6. Apply temporal decay
        let mut scored: Vec<(String, f64, Vec<f32>)> = fused.into_iter()
            .filter_map(|(id, rrf)| {
                let chunk = chunks.get(&id)?;
                let embedding = self.fetch_embedding(&id)?;
                let final_score = apply_temporal_score(
                    rrf,
                    chunk.accessed_at,
                    chunk.access_count,
                    query.half_life_days,
                );
                Some((id, final_score, embedding))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 7. MMR diversity
        let selected_ids = mmr_select(&scored, &query_vec, query.max_results, query.lambda_mmr);

        // 8. Update access metadata
        self.update_access_stats(&selected_ids).await?;

        // 9. Build results
        let results = selected_ids.iter()
            .filter_map(|id| {
                let chunk = chunks.get(id)?;
                let score = scored.iter().find(|(s_id, _, _)| s_id == id)?.1;
                Some(SearchResult {
                    chunk_id:     id.clone(),
                    file_path:    chunk.file_path.clone(),
                    content:      chunk.content.clone(),
                    memory_type:  chunk.memory_type.clone(),
                    score,
                    accessed_at:  chunk.accessed_at,
                    access_count: chunk.access_count,
                })
            })
            .collect();

        Ok(results)
    }
}
```

---

## 8. Working Memory

Per-task scratchpad held in process memory. Persisted write-through to SQLite so task context survives process restart.

```rust
// core/memory/src/working_memory.rs

pub struct WorkingMemory {
    task_id:         String,
    scratchpad:      String,                     // active reasoning text
    active_entities: HashMap<String, Entity>,    // entity name → entity
    causal_links:    Vec<CausalLink>,            // (cause_event, effect_event)
    created_at:      chrono::DateTime<Utc>,
    db:              Arc<DatabaseManager>,       // for write-through persistence
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Entity {
    pub name:        String,
    pub entity_type: String,
    pub attributes:  serde_json::Value,
    pub first_seen:  chrono::DateTime<Utc>,
    pub last_updated: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CausalLink {
    pub cause:     String,
    pub effect:    String,
    pub timestamp: chrono::DateTime<Utc>,
}

impl WorkingMemory {
    pub async fn new(task_id: &str, db: Arc<DatabaseManager>) -> Result<Self, MemoryError> {
        // Try to restore from a previous session for this task_id
        if let Some(saved) = Self::restore(task_id, &db).await? {
            return Ok(saved);
        }
        Ok(Self {
            task_id:         task_id.to_string(),
            scratchpad:      String::new(),
            active_entities: HashMap::new(),
            causal_links:    Vec::new(),
            created_at:      Utc::now(),
            db,
        })
    }

    pub async fn update_scratchpad(&mut self, text: &str) -> Result<(), MemoryError> {
        self.scratchpad = text.to_string();
        self.persist().await  // write-through: every mutation persists immediately
    }

    pub async fn add_entity(&mut self, entity: Entity) -> Result<(), MemoryError> {
        self.active_entities.insert(entity.name.clone(), entity);
        self.persist().await
    }

    pub async fn add_causal_link(&mut self, cause: &str, effect: &str) -> Result<(), MemoryError> {
        self.causal_links.push(CausalLink {
            cause:     cause.to_string(),
            effect:    effect.to_string(),
            timestamp: Utc::now(),
        });
        self.persist().await
    }

    /// Flush working memory to long-term store on task completion.
    /// Generates a memory narrative and schedules embedding.
    pub async fn flush_to_longterm(
        &self,
        indexer: &BackgroundIndexer,
    ) -> Result<(), MemoryError> {
        let narrative = self.generate_narrative();
        let file_path = format!(
            "journal/{}/{}.md",
            Utc::now().format("%Y/%m"),
            self.task_id
        );
        tokio::fs::write(&file_path, &narrative).await?;
        indexer.queue_file(file_path).await;
        Ok(())
    }

    async fn persist(&self) -> Result<(), MemoryError> {
        sqlx::query(
            "INSERT OR REPLACE INTO working_memory
             (task_id, scratchpad, entities_json, causal_links_json, created_at)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&self.task_id)
        .bind(&self.scratchpad)
        .bind(serde_json::to_string(&self.active_entities)?)
        .bind(serde_json::to_string(&self.causal_links)?)
        .bind(self.created_at.to_rfc3339())
        .execute(self.db.pool())
        .await?;
        Ok(())
    }
}
```

Add this table to migrations:

```sql
CREATE TABLE IF NOT EXISTS working_memory (
    task_id           TEXT PRIMARY KEY,
    scratchpad        TEXT NOT NULL DEFAULT '',
    entities_json     TEXT NOT NULL DEFAULT '{}',
    causal_links_json TEXT NOT NULL DEFAULT '[]',
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
);
```

---

## 9. Background Indexer

Indexes existing memory files on startup, and new files as they are written. Never blocks the main task path.

```rust
// core/memory/src/background_indexer.rs

pub struct BackgroundIndexer {
    queue:    mpsc::Sender<IndexJob>,
    embedder: Arc<Embedder>,
    db:       Arc<DatabaseManager>,
    config:   IndexerConfig,
}

pub struct IndexerConfig {
    pub batch_size:          usize,         // default: 16 — files per batch
    pub embed_rate_limit_ms: u64,           // default: 50 — min ms between embed calls
    pub chunk_config:        ChunkerConfig,
    pub embedding_dim:       usize,         // must match schema — default: 768
}

enum IndexJob {
    File(PathBuf),
    Directory(PathBuf),
    Shutdown,
}

impl BackgroundIndexer {
    /// Queue a single file for (re-)indexing.
    pub async fn queue_file(&self, path: impl Into<PathBuf>) {
        let _ = self.queue.send(IndexJob::File(path.into())).await;
    }

    /// Scan entire memory directory on startup.
    pub async fn initial_scan(&self, memory_dir: &Path) {
        let _ = self.queue.send(IndexJob::Directory(memory_dir.to_path_buf())).await;
    }

    async fn run(mut rx: mpsc::Receiver<IndexJob>, embedder: Arc<Embedder>, db: Arc<DatabaseManager>, config: IndexerConfig) {
        while let Some(job) = rx.recv().await {
            match job {
                IndexJob::File(path)      => Self::index_file(&path, &embedder, &db, &config).await,
                IndexJob::Directory(path) => Self::scan_directory(&path, &embedder, &db, &config).await,
                IndexJob::Shutdown        => break,
            }
        }
    }

    async fn index_file(path: &Path, embedder: &Embedder, db: &DatabaseManager, config: &IndexerConfig) {
        // 1. Check mtime against memory_index_state — skip if unchanged
        let mtime = fs::metadata(path).await.map(|m| m.modified()).ok().flatten();
        if Self::is_up_to_date(path, mtime, db).await { return; }

        // 2. Read file
        let text = match fs::read_to_string(path).await {
            Ok(t)  => t,
            Err(e) => { tracing::warn!("Indexer: failed to read {:?}: {}", path, e); return; }
        };

        // 3. Chunk
        let chunks = chunk_text(&text, &config.chunk_config);

        // 4. Remove old chunks for this file
        Self::delete_chunks_for_file(path, db).await;

        // 5. Embed and store each chunk
        for (idx, chunk_text) in &chunks {
            // Rate limit: don't hammer the embedding server
            tokio::time::sleep(Duration::from_millis(config.embed_rate_limit_ms)).await;

            let embedding = match embedder.embed(chunk_text).await {
                Ok(e)  => e,
                Err(e) => { tracing::warn!("Indexer: embed failed for chunk {}: {}", idx, e); continue; }
            };

            let chunk_id = Uuid::new_v4().to_string();
            let memory_type = Self::classify_path(path);

            // Insert into memory_chunks (triggers FTS5 update automatically)
            sqlx::query(
                "INSERT OR REPLACE INTO memory_chunks
                 (id, file_path, chunk_index, content, word_count, memory_type)
                 VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(&chunk_id)
            .bind(path.to_string_lossy().as_ref())
            .bind(*idx as i64)
            .bind(chunk_text)
            .bind(chunk_text.split_whitespace().count() as i64)
            .bind(&memory_type)
            .execute(db.pool()).await.ok();

            // Insert embedding into sqlite-vec
            sqlx::query(
                "INSERT OR REPLACE INTO memory_vec (chunk_id, embedding) VALUES (?, ?)"
            )
            .bind(&chunk_id)
            .bind(serde_json::to_string(&embedding).unwrap())
            .execute(db.pool()).await.ok();
        }

        // 6. Update index state
        Self::update_index_state(path, mtime, chunks.len(), db).await;
    }
}
```

---

## 10. API Endpoints

Consistent with existing APEX API patterns in `core/router/src/api/memory.rs`.

```yaml
# Memory search
POST /api/v1/memory/search
  body: { query: string, types?: string[], max_results?: int, half_life_days?: float }
  response: { results: SearchResult[], latency_ms: int }

# Store a new memory entry (also triggers background indexing)
POST /api/v1/memory/entries
  body: { content: string, memory_type: string, task_id?: string }
  response: { file_path: string, chunk_count: int }

# Working memory for a task
GET  /api/v1/memory/working/:task_id
POST /api/v1/memory/working/:task_id/scratchpad
POST /api/v1/memory/working/:task_id/entity
POST /api/v1/memory/working/:task_id/causal
DELETE /api/v1/memory/working/:task_id    # explicit flush to long-term + cleanup

# Entity management
GET  /api/v1/memory/entities
GET  /api/v1/memory/entities/:name
POST /api/v1/memory/entities

# Index management
GET  /api/v1/memory/index/status          # { total_chunks, indexed_files, queue_depth }
POST /api/v1/memory/index/reindex         # trigger full re-index (admin operation)
```

### 10.1 Search Response Shape

```typescript
interface SearchResult {
  chunk_id:     string;
  file_path:    string;        // relative path in memory directory
  content:      string;        // chunk text — ready to inject into context
  memory_type:  'journal' | 'knowledge' | 'entity' | 'reflection';
  score:        number;        // final score after RRF + temporal + MMR
  accessed_at:  string;        // ISO 8601
  access_count: number;
}
```

---

## 11. Integration with Agent Loop

The agent loop (`src/agent_loop.rs`) gains two memory operations injected into the Plan step:

```rust
// In agent_loop.rs — Plan step, before LLM call

// 1. Search long-term memory for context relevant to current task
let memory_results = memory_service
    .search(SearchQuery {
        text:           &task_content,
        memory_types:   None,  // search all types
        max_results:    6,     // inject top 6 chunks into context
        ..Default::default()
    })
    .await?;

// 2. Read working memory scratchpad for this task
let working = memory_service
    .working_memory(&context.task_id)
    .await?;

// 3. Build memory context string for LLM prompt
let memory_context = format!(
    "## Relevant Memory\n{}\n\n## Working Notes\n{}",
    memory_results.iter()
        .map(|r| format!("- [{}] {}", r.memory_type, r.content))
        .collect::<Vec<_>>()
        .join("\n"),
    working.scratchpad
);

// 4. Prepend to system prompt (before task content)
```

The Heartbeat daemon (`src/heartbeat/mod.rs`) calls `memory_service.flush_all_working()` on each fire, consolidating any abandoned in-flight task memory to long-term storage.

---

## 12. Configuration

Consistent with APEX's unified config pattern:

```rust
pub struct MemoryConfig {
    // Embedding
    pub embedding_provider:    String,   // "local" | "openai"
    pub embedding_model:       String,   // "nomic-embed-text" | "text-embedding-3-small"
    pub embedding_url:         String,   // local: llama-server URL; openai: api base
    pub embedding_dim:         usize,    // 768 (local) or 1536 (openai) — MUST match schema
    pub embedding_api_key:     Option<String>,

    // Search
    pub rrf_k:                 usize,    // default: 60
    pub max_results:           usize,    // default: 8
    pub lambda_mmr:            f64,      // default: 0.7
    pub half_life_days:        f64,      // default: 30.0
    pub min_score_threshold:   Option<f64>,

    // Chunking
    pub chunk_size_tokens:     usize,    // default: 256
    pub chunk_overlap_tokens:  usize,    // default: 32

    // Indexer
    pub embed_rate_limit_ms:   u64,      // default: 50
    pub indexer_batch_size:    usize,    // default: 16
}
```

### Environment Variables

```bash
# Embedding provider (decide at deployment — cannot change without re-indexing)
APEX_MEMORY_EMBEDDING_PROVIDER=local          # or: openai
APEX_MEMORY_EMBEDDING_MODEL=nomic-embed-text  # or: text-embedding-3-small
APEX_MEMORY_EMBEDDING_URL=http://localhost:8081
APEX_MEMORY_EMBEDDING_DIM=768                 # must match provider output

# Search tuning
APEX_MEMORY_RRF_K=60
APEX_MEMORY_MAX_RESULTS=8
APEX_MEMORY_MMR_LAMBDA=0.7
APEX_MEMORY_HALF_LIFE_DAYS=30.0

# Chunking
APEX_MEMORY_CHUNK_SIZE=256
APEX_MEMORY_CHUNK_OVERLAP=32

# Indexer
APEX_MEMORY_EMBED_RATE_LIMIT_MS=50
APEX_MEMORY_INDEXER_BATCH_SIZE=16

# Embedding server (separate port from LLM server)
APEX_EMBEDDING_SERVER_PORT=8081
```

---

## 13. Files to Create / Modify

### New files

```
core/memory/src/
├── embedder.rs              Vector embedding client (local + OpenAI)
├── chunker.rs               Text chunking with markdown awareness
├── hybrid_search.rs         RRF, temporal decay, MMR
├── working_memory.rs        Per-task scratchpad
├── background_indexer.rs    Async file watcher + embedding queue

core/memory/migrations/
├── 005_memory_chunks.sql    memory_chunks + triggers
├── 006_memory_fts.sql       FTS5 virtual table
├── 007_memory_vec.sql       sqlite-vec virtual table (768-dim)
├── 008_memory_entities.sql  entity store
├── 009_memory_index_state.sql
├── 010_working_memory.sql
```

### Modified files

```
core/memory/src/lib.rs              — export new modules
core/memory/Cargo.toml              — add sqlite-vec, reqwest, uuid
core/router/src/api/memory.rs       — add new endpoints
core/router/src/agent_loop.rs       — inject memory context into Plan step
core/router/src/heartbeat/mod.rs    — call memory_service.flush_all_working()
core/router/src/unified_config.rs   — add MemoryConfig
core/router/src/api/mod.rs          — add memory_service to AppState
AGENTS.md                           — document new env vars
ARCHITECTURE.md                     — update memory section
```

### Cargo.toml additions

```toml
[dependencies]
sqlite-vec   = "0.1"          # check latest version at github.com/asg017/sqlite-vec
reqwest      = { version = "0.12", features = ["json"] }
uuid         = { version = "1", features = ["v4"] }
chrono       = { version = "0.4", features = ["serde"] }
```

---

## 14. Implementation Plan

### Phase 1 — Foundation (Weeks 1–3)

**Goal:** Semantic search working on existing memory files. Nothing fancy. Just working.

**Week 1: Schema and embedding client**

- [ ] Write migrations 005–010
- [ ] Implement `embedder.rs` — local provider only first; OpenAI is fallback
- [ ] Run `dimension_validate()` at router startup — hard fail if mismatch
- [ ] Add `APEX_MEMORY_EMBEDDING_*` vars to unified config
- [ ] Spin up llama-server with `nomic-embed-text` on port 8081
- [ ] Manual test: `POST /api/v1/embeddings` returns 768-dim vector

**Week 2: Chunker and indexer**

- [ ] Implement `chunker.rs` — markdown-aware, heading-respecting
- [ ] Unit test: verify no chunks cross headings or code fences
- [ ] Implement `background_indexer.rs` — startup scan only (no file watcher yet)
- [ ] Run initial index against existing `journal/`, `knowledge/` directories
- [ ] Verify `memory_chunks` and `memory_vec` populate correctly
- [ ] Verify FTS5 triggers populate `memory_fts`

**Week 3: Search and API**

- [ ] Implement `hybrid_search.rs` — RRF + temporal decay (skip MMR initially)
- [ ] Implement `POST /api/v1/memory/search`
- [ ] Integration test: search returns relevant results from indexed files
- [ ] Add MMR
- [ ] Performance test: search latency < 30ms on 10K chunks
- [ ] Expose `GET /api/v1/memory/index/status`

**Phase 1 exit gate:** `POST /api/v1/memory/search` returns semantically relevant results from existing memory files in under 30ms.

---

### Phase 2 — Working Memory and Agent Integration (Weeks 4–6)

**Goal:** Agent loop uses memory. Memory survives task restarts.

**Week 4: Working memory**

- [ ] Implement `working_memory.rs` — write-through to `working_memory` table
- [ ] Unit test: simulate process restart — verify working memory restores
- [ ] Implement working memory API endpoints
- [ ] Add `WorkingMemory` creation to task startup in `agent_loop.rs`

**Week 5: Agent loop integration**

- [ ] Inject memory search results into Plan step system prompt
- [ ] Inject working memory scratchpad into Plan step system prompt
- [ ] Implement `WorkingMemory::flush_to_longterm()`
- [ ] Call flush on task completion in `DeepTaskWorker`
- [ ] Integration test: task completion creates indexed journal entry

**Week 6: Heartbeat integration and file watcher**

- [ ] Wire `memory_service.flush_all_working()` into `heartbeat/mod.rs`
- [ ] Add filesystem watcher to `background_indexer.rs` (notify crate)
- [ ] Test: write a new `.md` file to `knowledge/` — verify it indexes within 30s
- [ ] Performance test: background indexing does not affect search latency

**Phase 2 exit gate:** Agent tasks read from and write to memory. New memory files are indexed automatically. Heartbeat consolidates orphaned task memory.

---

### Phase 3 — Entity Graph (Weeks 7–10, conditional)

**Gate before starting Phase 3:** Analyse 4 weeks of Phase 2 search logs. If > 20% of searches return poor results that graph traversal would improve, proceed. If not, defer.

**Week 7: Entity extraction**

- [ ] Implement rule-based entity extractor (regex patterns for common types)
- [ ] LLM-based extractor as a fallback (calls agent LLM — expensive, rate-limited)
- [ ] Run extractor over existing journal entries
- [ ] Populate `memory_entities` table
- [ ] Verify `GET /api/v1/memory/entities` returns meaningful entities

**Week 8: Graph edges**

- [ ] Add `memory_entity_edges` table:

```sql
CREATE TABLE memory_entity_edges (
    id           TEXT PRIMARY KEY,
    source_id    TEXT NOT NULL REFERENCES memory_entities(id),
    target_id    TEXT NOT NULL REFERENCES memory_entities(id),
    relationship TEXT NOT NULL,   -- "uses", "created_by", "depends_on", "related_to"
    strength     REAL NOT NULL DEFAULT 1.0,
    evidence     TEXT,            -- chunk_id that established this relationship
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_id, target_id, relationship)
);

CREATE INDEX IF NOT EXISTS idx_edges_source ON memory_entity_edges(source_id);
CREATE INDEX IF NOT EXISTS idx_edges_target ON memory_entity_edges(target_id);
```

- [ ] Implement edge extraction from entity co-occurrence in chunks
- [ ] Implement `GET /api/v1/memory/graph` with adjacency list response

**Week 9: Graph-augmented search**

- [ ] Post-search graph expansion: for each search result, fetch 1-hop entity neighbours
- [ ] Add graph score to final ranking (configurable weight — default: 0.1)
- [ ] A/B test: measure result quality with/without graph augmentation

**Week 10: Evaluation**

- [ ] Build an evaluation set from real APEX task history
- [ ] Measure: Recall@8 with Phase 1 search vs Phase 3 graph-augmented search
- [ ] Document: if graph augmentation does not improve Recall@8 by > 5%, disable by default

**Phase 3 exit gate:** Measurable improvement in search recall. Graph is disabled by default if improvement is < 5%.

---

## 15. Testing Plan

### Unit tests

```
core/memory/src/chunker_test.rs
  - chunk_text: heading boundaries respected
  - chunk_text: code fence boundaries respected
  - chunk_text: overlap is correct
  - chunk_text: min_chunk_tokens filter works

core/memory/src/hybrid_search_test.rs
  - rrf_score: correct values at standard ranks
  - reciprocal_rank_fusion: documents in both lists score higher
  - temporal_decay: 30-day decay at day 0 = 1.0, day 30 = 0.5
  - mmr_select: returned chunks are not near-duplicates
  - apply_temporal_score: combined score is monotone in recency
```

### Integration tests

```
core/router/tests/memory_search_test.rs
  - POST /api/v1/memory/search: relevant chunks rank above irrelevant ones
  - POST /api/v1/memory/search: memory_type filter works
  - POST /api/v1/memory/search: returns in < 30ms on test corpus of 5K chunks
  - GET /api/v1/memory/index/status: reflects actual chunk count
```

### Performance benchmarks

```
cargo bench memory_search
  - 1K chunks: p50 < 10ms, p99 < 20ms
  - 10K chunks: p50 < 20ms, p99 < 50ms
  - 50K chunks: p50 < 50ms, p99 < 100ms
  (Trigger re-architecture review if 50K p99 > 100ms)
```

---

## 16. What This Spec Deliberately Excludes

| Excluded | Reason |
|---|---|
| Redis / external cache | Single-user app; in-process HashMap is sufficient |
| pgvector | Requires PostgreSQL; breaks single-binary deployment |
| Multiple simultaneous embedding providers | Schema dimension is fixed at deploy time |
| OpenAI as default embedding provider | External API dependency; local model is superior default |
| Memclawz QMD | Python 3.10+ Linux-only; incompatible with Windows deployment |
| Causality graph (Phase 3+) | Requires measuring Phase 2 gaps first |
| Redis working memory backend | In-memory is sufficient; adds operational complexity |
| Automatic entity extraction on every search | Too expensive; runs in background on write only |

---

## 17. Open Questions (Resolved)

All questions from v1.0 are resolved:

| Question | Decision |
|---|---|
| Embedding provider | **Local `nomic-embed-text` via llama-server** — no external API |
| Vector dimension | **768** — fixed at migration time |
| Working memory backend | **In-process HashMap with write-through SQLite** |
| Entity extraction | **Rule-based first; LLM fallback; graph deferred to Phase 3** |
| Graph storage | **SQLite, not pgvector** — `memory_entity_edges` table, Phase 3 only |
| RRF vs weighted blend | **RRF default; weighted blend available via config** |

---

*APEX Memory System Specification · v2.0 · Architecture ref: APEX v1.0.0*
