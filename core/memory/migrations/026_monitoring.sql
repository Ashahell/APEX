-- M1: Background Process Monitoring
-- Watch patterns, monitor events, and notification tracking

-- Watch patterns table
CREATE TABLE IF NOT EXISTS watch_patterns (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    pattern TEXT NOT NULL,
    watch_scope TEXT NOT NULL,              -- JSON: All | Project | TaskIds
    notify_on TEXT NOT NULL,                -- Match | Completion | Error | Timeout | Threshold
    notification_mode TEXT NOT NULL DEFAULT 'Result',
    threshold_count INTEGER DEFAULT 1,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Monitor events table for event history
CREATE TABLE IF NOT EXISTS monitor_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    task_id TEXT,
    session_id TEXT,
    payload TEXT NOT NULL,
    matched_watcher_id TEXT,
    created_at TEXT NOT NULL
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_watch_patterns_enabled ON watch_patterns(enabled);
CREATE INDEX IF NOT EXISTS idx_monitor_events_task ON monitor_events(task_id);
CREATE INDEX IF NOT EXISTS idx_monitor_events_type ON monitor_events(event_type);
CREATE INDEX IF NOT EXISTS idx_monitor_events_created ON monitor_events(created_at DESC);

-- Notification log for audit trail
CREATE TABLE IF NOT EXISTS notification_log (
    id TEXT PRIMARY KEY,
    watcher_id TEXT,
    event_id TEXT,
    notification_type TEXT NOT NULL,
    channel TEXT NOT NULL,
    recipient TEXT,
    message TEXT NOT NULL,
    status TEXT NOT NULL,                   -- sent | failed | skipped
    error_message TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_notification_log_created ON notification_log(created_at DESC);
