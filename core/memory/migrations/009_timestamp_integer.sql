-- Migration: Convert timestamps from TEXT to INTEGER (Unix epoch milliseconds)
-- This fixes silent parsing failures in TTL cleanup

-- Create new INTEGER columns
ALTER TABLE tasks ADD COLUMN created_at_ms INTEGER;
ALTER TABLE tasks ADD COLUMN updated_at_ms INTEGER;
ALTER TABLE tasks ADD COLUMN started_at_ms INTEGER;
ALTER TABLE tasks ADD COLUMN completed_at_ms INTEGER;

-- Migrate existing data: convert TEXT timestamps to INTEGER milliseconds
UPDATE tasks SET 
    created_at_ms = CAST((julianday(created_at) - 2440587.5) * 86400000 AS INTEGER),
    updated_at_ms = CAST((julianday(updated_at) - 2440587.5) * 86400000 AS INTEGER),
    started_at_ms = CAST((julianday(started_at) - 2440587.5) * 86400000 AS INTEGER),
    completed_at_ms = CAST((julianday(completed_at) - 2440587.5) * 86400000 AS INTEGER)
WHERE created_at IS NOT NULL;

-- Messages table
ALTER TABLE messages ADD COLUMN created_at_ms INTEGER;
UPDATE messages SET created_at_ms = CAST((julianday(created_at) - 2440587.5) * 86400000 AS INTEGER)
WHERE created_at IS NOT NULL;

-- Channels table
ALTER TABLE channels ADD COLUMN created_at_ms INTEGER;
ALTER TABLE channels ADD COLUMN updated_at_ms INTEGER;
UPDATE channels SET 
    created_at_ms = CAST((julianday(created_at) - 2440587.5) * 86400000 AS INTEGER),
    updated_at_ms = CAST((julianday(updated_at) - 2440587.5) * 86400000 AS INTEGER)
WHERE created_at IS NOT NULL;

-- Decision journal table
ALTER TABLE decision_journal ADD COLUMN created_at_ms INTEGER;
ALTER TABLE decision_journal ADD COLUMN updated_at_ms INTEGER;
UPDATE decision_journal SET 
    created_at_ms = CAST((julianday(created_at) - 2440587.5) * 86400000 AS INTEGER),
    updated_at_ms = CAST((julianday(updated_at) - 2440587.5) * 86400000 AS INTEGER)
WHERE created_at IS NOT NULL;

-- Skill registry table
ALTER TABLE skill_registry ADD COLUMN created_at_ms INTEGER;
ALTER TABLE skill_registry ADD COLUMN updated_at_ms INTEGER;
ALTER TABLE skill_registry ADD COLUMN last_health_check_ms INTEGER;
UPDATE skill_registry SET 
    created_at_ms = CAST((julianday(created_at) - 2440587.5) * 86400000 AS INTEGER),
    updated_at_ms = CAST((julianday(updated_at) - 2440587.5) * 86400000 AS INTEGER),
    last_health_check_ms = CAST((julianday(last_health_check) - 2440587.5) * 86400000 AS INTEGER)
WHERE created_at IS NOT NULL;

-- Create indexes for faster queries
CREATE INDEX IF NOT EXISTS idx_tasks_created_at_ms ON tasks(created_at_ms);
CREATE INDEX IF NOT EXISTS idx_tasks_updated_at_ms ON tasks(updated_at_ms);
CREATE INDEX IF NOT EXISTS idx_messages_created_at_ms ON messages(created_at_ms);
CREATE INDEX IF NOT EXISTS idx_decision_journal_created_at_ms ON decision_journal(created_at_ms);

-- Note: Application code should use INTEGER comparisons for time-based queries
-- e.g., WHERE created_at_ms > (strftime('%s', 'now') * 1000 - 86400000) for last 24 hours
