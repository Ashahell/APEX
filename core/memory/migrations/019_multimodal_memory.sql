-- Migration: 019_multimodal_memory
-- Multimodal memory embeddings for images and audio

-- 1. memory_embeddings: Multimodal embeddings
CREATE TABLE IF NOT EXISTS memory_embeddings (
    id              TEXT PRIMARY KEY,
    memory_id       TEXT NOT NULL,
    memory_type     TEXT NOT NULL,  -- 'entity', 'knowledge', 'reflection', 'journal'
    modality        TEXT NOT NULL,  -- 'text', 'image', 'audio'
    embedding       TEXT NOT NULL,  -- Vector as JSON array
    embedding_model TEXT NOT NULL,
    original_data   TEXT,  -- Base64 for image/audio
    mime_type       TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_embedding_modality ON memory_embeddings(modality);
CREATE INDEX IF NOT EXISTS idx_embedding_model ON memory_embeddings(embedding_model);
CREATE INDEX IF NOT EXISTS idx_embedding_memory ON memory_embeddings(memory_id);
CREATE INDEX IF NOT EXISTS idx_embedding_memory_type ON memory_embeddings(memory_type);

-- 2. memory_indexing_jobs: Background indexing
CREATE TABLE IF NOT EXISTS memory_indexing_jobs (
    id              TEXT PRIMARY KEY,
    memory_id       TEXT NOT NULL,
    modality        TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'processing', 'completed', 'failed'
    error_message   TEXT,
    started_at      TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at    TEXT
);

CREATE INDEX IF NOT EXISTS idx_indexing_status ON memory_indexing_jobs(status);
CREATE INDEX IF NOT EXISTS idx_indexing_memory ON memory_indexing_jobs(memory_id);

-- 3. memory_multimodal_config: Multimodal settings
CREATE TABLE IF NOT EXISTS memory_multimodal_config (
    id              TEXT PRIMARY KEY,
    image_indexing  INTEGER DEFAULT 1,
    audio_indexing  INTEGER DEFAULT 1,
    embedding_model TEXT DEFAULT 'gemini-embedding-2-preview',
    embedding_dim   INTEGER DEFAULT 1536,
    enabled         INTEGER DEFAULT 1,
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default config
INSERT OR IGNORE INTO memory_multimodal_config (id, image_indexing, audio_indexing, embedding_model, embedding_dim, enabled)
VALUES ('default', 1, 1, 'gemini-embedding-2-preview', 1536, 1);
