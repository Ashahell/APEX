-- Add TTL configuration table
CREATE TABLE IF NOT EXISTS ttl_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL UNIQUE,
    retention_days INTEGER NOT NULL DEFAULT 90,
    enabled INTEGER NOT NULL DEFAULT 1,
    last_cleanup TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default TTL configurations
INSERT OR IGNORE INTO ttl_config (entity_type, retention_days, enabled) VALUES 
    ('tasks', 90, 1),
    ('messages', 90, 1),
    ('audit_log', 365, 1),
    ('vector_store', 30, 1);

-- Add index for TTL queries
CREATE INDEX IF NOT EXISTS idx_tasks_created_at_ttl ON tasks(created_at);
CREATE INDEX IF NOT EXISTS idx_messages_created_at_ttl ON messages(created_at);
