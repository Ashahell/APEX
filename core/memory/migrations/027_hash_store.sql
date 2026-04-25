-- Migration 027: Memory Integrity Hash Store
-- Stores SHA-256 hashes for memory_chunks to detect corruption
-- Sidecar for sqlite_vec index integrity verification

CREATE TABLE IF NOT EXISTS hash_store (
    chunk_id TEXT PRIMARY KEY NOT NULL,
    content_hash TEXT NOT NULL,
    word_count INTEGER NOT NULL,
    chunk_index INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    verified_at TEXT,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK(status IN ('pending', 'valid', 'corrupted', 'repaired'))
);

CREATE INDEX IF NOT EXISTS idx_hash_store_status ON hash_store(status);
CREATE INDEX IF NOT EXISTS idx_hash_store_verified ON hash_store(verified_at);

-- Populate from existing memory_chunks
INSERT OR IGNORE INTO hash_store (chunk_id, content_hash, word_count, chunk_index, status)
SELECT
    mc.id,
    LOWER(HEX(RANDOMBLOB(32))),  -- placeholder, will be computed by integrity service
    mc.word_count,
    mc.chunk_index,
    'pending'
FROM memory_chunks mc
WHERE mc.id NOT IN (SELECT chunk_id FROM hash_store);

-- Integrity check metadata
CREATE TABLE IF NOT EXISTS integrity_meta (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    last_check TEXT,
    total_chunks INTEGER DEFAULT 0,
    valid_chunks INTEGER DEFAULT 0,
    corrupted_chunks INTEGER DEFAULT 0,
    repaired_chunks INTEGER DEFAULT 0,
    status TEXT DEFAULT 'unknown' CHECK(status IN ('unknown', 'checking', 'valid', 'corrupted', 'error'))
);

INSERT OR IGNORE INTO integrity_meta (id) VALUES (1);