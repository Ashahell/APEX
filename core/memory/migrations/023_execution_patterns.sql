-- Migration: 023_execution_patterns
-- Death Spiral Detection - Execution pattern storage

-- Execution patterns table (for death spiral detection)
CREATE TABLE IF NOT EXISTS execution_patterns (
    id              TEXT PRIMARY KEY,
    task_id         TEXT NOT NULL,
    pattern_type    TEXT NOT NULL,  -- 'file_creation_burst', 'tool_call_loop', 'no_side_effects', 'error_cascade'
    severity        TEXT NOT NULL,  -- 'low', 'medium', 'high', 'critical'
    tool_calls      TEXT,  -- JSON array of tool calls
    file_ops        TEXT,  -- JSON array of file operations
    error_count     INTEGER DEFAULT 0,
    details         TEXT,  -- JSON: additional context
    detected_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_patterns_task ON execution_patterns(task_id);
CREATE INDEX IF NOT EXISTS idx_patterns_type ON execution_patterns(pattern_type);
CREATE INDEX IF NOT EXISTS idx_patterns_severity ON execution_patterns(severity);
CREATE INDEX IF NOT EXISTS idx_patterns_detected ON execution_patterns(detected_at);

-- Pre-built pattern alert templates
CREATE TABLE IF NOT EXISTS pattern_alert_templates (
    id              TEXT PRIMARY KEY,
    pattern_type    TEXT NOT NULL UNIQUE,
    title           TEXT NOT NULL,
    description     TEXT,
    severity        TEXT NOT NULL,
    remediation     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default pattern alert templates
INSERT OR IGNORE INTO pattern_alert_templates (id, pattern_type, title, description, severity, remediation) VALUES
    ('file_burst', 'file_creation_burst', 'File Creation Burst Detected', 
     'Multiple files created in short succession - possible infinite generation loop', 
     'high', 'Review generated files, implement file count limits'),
    ('tool_loop', 'tool_call_loop', 'Tool Call Loop Detected',
     'Same tool called multiple times in succession - possible infinite recursion',
     'critical', 'Cancel task immediately, review tool selection logic'),
    ('no_side_effects', 'no_side_effects', 'No Side Effects Detected',
     'Multiple tool calls with no observable state changes - possible dead loop',
     'medium', 'Check tool outputs, verify file writes are working'),
    ('error_cascade', 'error_cascade', 'Error Cascade Detected',
     'Multiple sequential errors - task likely failing repeatedly',
     'critical', 'Cancel task, review error logs, check credentials/permissions');
