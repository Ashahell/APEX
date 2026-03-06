-- Migration: 013_enhanced_memory
-- Enhanced Memory System with sqlite-vec, FTS5, and working memory
-- See docs/APEX_Memory_System_Spec_v2.md

-- Load sqlite-vec extension
-- This must be loaded at runtime before any vec0 operations

-- 1. memory_chunks: stores chunked text from all memory files
CREATE TABLE IF NOT EXISTS memory_chunks (
    id          TEXT PRIMARY KEY,
    file_path   TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    content     TEXT NOT NULL,
    word_count  INTEGER NOT NULL,
    memory_type TEXT NOT NULL,
    task_id     TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    accessed_at TEXT NOT NULL DEFAULT (datetime('now')),
    access_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(file_path, chunk_index)
);

CREATE INDEX IF NOT EXISTS idx_memory_chunks_type ON memory_chunks(memory_type);
CREATE INDEX IF NOT EXISTS idx_memory_chunks_task ON memory_chunks(task_id);
CREATE INDEX IF NOT EXISTS idx_memory_chunks_accessed ON memory_chunks(accessed_at DESC);

-- 2. memory_fts: FTS5 virtual table for BM25 keyword search
CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
    content,
    memory_type UNINDEXED,
    content='memory_chunks',
    content_rowid='rowid',
    tokenize='porter unicode61'
);

-- Triggers to keep FTS index in sync with memory_chunks
CREATE TRIGGER IF NOT EXISTS memory_chunks_ai AFTER INSERT ON memory_chunks BEGIN
    INSERT INTO memory_fts(rowid, content, memory_type)
    VALUES (new.rowid, new.content, new.memory_type);
END;

CREATE TRIGGER IF NOT EXISTS memory_chunks_ad AFTER DELETE ON memory_chunks BEGIN
    INSERT INTO memory_fts(memory_fts, rowid, content, memory_type)
    VALUES ('delete', old.rowid, old.content, old.memory_type);
END;

CREATE TRIGGER IF NOT EXISTS memory_chunks_au AFTER UPDATE ON memory_chunks BEGIN
    INSERT INTO memory_fts(memory_fts, rowid, content, memory_type)
    VALUES ('delete', old.rowid, old.content, old.memory_type);
    INSERT INTO memory_fts(rowid, content, memory_type)
    VALUES (new.rowid, new.content, new.memory_type);
END;

-- 3. memory_entities: lightweight entity store
CREATE TABLE IF NOT EXISTS memory_entities (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    entity_type  TEXT NOT NULL,
    attributes   TEXT NOT NULL DEFAULT '{}',
    first_seen   TEXT NOT NULL DEFAULT (datetime('now')),
    last_updated TEXT NOT NULL DEFAULT (datetime('now')),
    mention_count INTEGER NOT NULL DEFAULT 1
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_memory_entities_name_type
    ON memory_entities(name, entity_type);

-- 4. memory_index_state: tracks which files have been indexed
CREATE TABLE IF NOT EXISTS memory_index_state (
    file_path   TEXT PRIMARY KEY,
    mtime_unix  INTEGER NOT NULL,
    chunk_count INTEGER NOT NULL,
    indexed_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 5. working_memory: per-task scratchpad (write-through persisted)
CREATE TABLE IF NOT EXISTS working_memory (
    task_id           TEXT PRIMARY KEY,
    scratchpad        TEXT NOT NULL DEFAULT '',
    entities_json     TEXT NOT NULL DEFAULT '{}',
    causal_links_json TEXT NOT NULL DEFAULT '[]',
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
);
