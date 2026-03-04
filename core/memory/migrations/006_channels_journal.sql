-- Create channels table
CREATE TABLE IF NOT EXISTS channels (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Create decision_journal table
CREATE TABLE IF NOT EXISTS decision_journal (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT,
    title TEXT NOT NULL,
    context TEXT,
    decision TEXT NOT NULL,
    rationale TEXT,
    outcome TEXT,
    tags TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- Create indexes for channels
CREATE INDEX IF NOT EXISTS idx_channels_name ON channels(name);

-- Create indexes for decision_journal
CREATE INDEX IF NOT EXISTS idx_decision_journal_task ON decision_journal(task_id);
CREATE INDEX IF NOT EXISTS idx_decision_journal_created ON decision_journal(created_at);

-- Insert default channels
INSERT OR IGNORE INTO channels (id, name, description) VALUES 
    ('default', 'default', 'Default conversation channel'),
    ('general', 'general', 'General discussions');
