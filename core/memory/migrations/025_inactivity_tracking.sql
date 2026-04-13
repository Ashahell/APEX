-- M2: Inactivity-Based Timeout
-- Add last_activity_at column to track activity for smart timeouts

ALTER TABLE tasks ADD COLUMN last_activity_at INTEGER;

-- Index for efficient queries on inactive tasks
CREATE INDEX IF NOT EXISTS idx_tasks_last_activity ON tasks(last_activity_at) WHERE status = 'running';
