-- Migration: 018_pdf_tool
-- PDF document processing and caching

-- 1. pdf_documents: PDF processing cache
CREATE TABLE IF NOT EXISTS pdf_documents (
    id              TEXT PRIMARY KEY,
    file_name       TEXT NOT NULL,
    file_hash       TEXT NOT NULL,  -- SHA256 for deduplication
    file_size       INTEGER NOT NULL,
    page_count      INTEGER,
    extracted_text  TEXT,
    metadata        TEXT,  -- JSON: { author, created, title, etc. }
    provider        TEXT NOT NULL DEFAULT 'fallback',  -- 'anthropic', 'google', 'fallback'
    model_used      TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at      TEXT  -- TTL for cache
);

CREATE INDEX IF NOT EXISTS idx_pdf_hash ON pdf_documents(file_hash);
CREATE INDEX IF NOT EXISTS idx_pdf_provider ON pdf_documents(provider);

-- 2. pdf_extraction_jobs: Background job tracking
CREATE TABLE IF NOT EXISTS pdf_extraction_jobs (
    id              TEXT PRIMARY KEY,
    document_id     TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'processing', 'completed', 'failed'
    provider        TEXT NOT NULL DEFAULT 'fallback',
    error_message   TEXT,
    started_at      TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at    TEXT,
    FOREIGN KEY (document_id) REFERENCES pdf_documents(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_pdf_job_status ON pdf_extraction_jobs(status);
CREATE INDEX IF NOT EXISTS idx_pdf_job_document ON pdf_extraction_jobs(document_id);
