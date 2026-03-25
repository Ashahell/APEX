-- Migration: 024_performance_indexes
-- Performance indexes for common query patterns

-- Messages table indexes for efficient channel/task queries
CREATE INDEX IF NOT EXISTS idx_messages_channel_created ON messages(channel, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_task_created ON messages(task_id, created_at ASC);

-- Audit log index for timestamp-based pagination
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp DESC);

-- Memory embeddings composite index for modality + time queries
CREATE INDEX IF NOT EXISTS idx_embedding_modality_created ON memory_embeddings(modality, created_at DESC);

-- Memory indexing jobs composite index for status queue ordering
CREATE INDEX IF NOT EXISTS idx_indexing_status_started ON memory_indexing_jobs(status, started_at ASC);
