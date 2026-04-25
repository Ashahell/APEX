-- Migration 025: Cancellation tracking for persistent stop-button
-- Allows cancellation requests to persist across reconnects

-- Add cancellation_requested flag to tasks table
ALTER TABLE tasks ADD COLUMN cancellation_requested INTEGER NOT NULL DEFAULT 0;

-- Add cancellation_requested_at timestamp
ALTER TABLE tasks ADD COLUMN cancellation_requested_at TEXT;

-- Create cancellation_requests table for tracking cancel signals
CREATE TABLE IF NOT EXISTS cancellation_requests (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL,
    requested_at TEXT NOT NULL DEFAULT (datetime('now')),
    source TEXT NOT NULL DEFAULT 'user',
    fulfilled INTEGER NOT NULL DEFAULT 0,
    fulfilled_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_cancel_task ON cancellation_requests(task_id);
CREATE INDEX IF NOT EXISTS idx_cancel_pending ON cancellation_requests(fulfilled, requested_at);