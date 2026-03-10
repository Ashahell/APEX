-- Migration: 014_skill_security
-- Skill Security: Content hashes, validation, and integrity tracking
-- See docs/APEX_Security_Implementation_Plan.md

-- 1. skill_integrity: Track content hashes for skill files
CREATE TABLE IF NOT EXISTS skill_integrity (
    id              TEXT PRIMARY KEY,
    skill_name      TEXT NOT NULL,
    file_path       TEXT NOT NULL,
    content_hash    TEXT NOT NULL,
    file_size       INTEGER NOT NULL,
    last_verified   TEXT NOT NULL DEFAULT (datetime('now')),
    status          TEXT NOT NULL DEFAULT 'valid',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(skill_name, file_path)
);

CREATE INDEX IF NOT EXISTS idx_skill_integrity_name ON skill_integrity(skill_name);
CREATE INDEX IF NOT EXISTS idx_skill_integrity_status ON skill_integrity(status);
CREATE INDEX IF NOT EXISTS idx_skill_integrity_verified ON skill_integrity(last_verified DESC);

-- 2. skill_validation_log: Audit trail for skill validations
CREATE TABLE IF NOT EXISTS skill_validation_log (
    id              TEXT PRIMARY KEY,
    skill_name      TEXT NOT NULL,
    validation_type TEXT NOT NULL,
    status          TEXT NOT NULL,
    error_message   TEXT,
    details         TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_validation_log_name ON skill_validation_log(skill_name);
CREATE INDEX IF NOT EXISTS idx_validation_log_type ON skill_validation_log(validation_type);
CREATE INDEX IF NOT EXISTS idx_validation_log_created ON skill_validation_log(created_at DESC);

-- 3. skill_execution_sandbox: Track sandbox isolation for executions
CREATE TABLE IF NOT EXISTS skill_execution_sandbox (
    id                  TEXT PRIMARY KEY,
    task_id             TEXT NOT NULL,
    skill_name          TEXT NOT NULL,
    execution_mode      TEXT NOT NULL,  -- 'bun_pool', 'vm_pool', 'docker', 'firecracker'
    isolation_level     TEXT NOT NULL,   -- 'none', 'process', 'container', 'vm'
    vm_instance_id      TEXT,
    started_at          TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at        TEXT,
    exit_code           INTEGER,
    memory_used_mb      INTEGER,
    cpu_time_ms         INTEGER,
    status              TEXT NOT NULL DEFAULT 'running'
);

CREATE INDEX IF NOT EXISTS idx_sandbox_task ON skill_execution_sandbox(task_id);
CREATE INDEX IF NOT EXISTS idx_sandbox_skill ON skill_execution_sandbox(skill_name);
CREATE INDEX IF NOT EXISTS idx_sandbox_status ON skill_execution_sandbox(status);
CREATE INDEX IF NOT EXISTS idx_sandbox_started ON skill_execution_sandbox(started_at DESC);

-- 4. anomaly_log: Security anomaly detection
CREATE TABLE IF NOT EXISTS anomaly_log (
    id              TEXT PRIMARY KEY,
    anomaly_type    TEXT NOT NULL,
    severity        TEXT NOT NULL,  -- 'low', 'medium', 'high', 'critical'
    source          TEXT NOT NULL,
    description     TEXT NOT NULL,
    task_id         TEXT,
    skill_name      TEXT,
    metadata        TEXT,
    resolved        INTEGER NOT NULL DEFAULT 0,
    resolved_at     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_anomaly_type ON anomaly_log(anomaly_type);
CREATE INDEX IF NOT EXISTS idx_anomaly_severity ON anomaly_log(severity);
CREATE INDEX IF NOT EXISTS idx_anomaly_resolved ON anomaly_log(resolved);
CREATE INDEX IF NOT EXISTS idx_anomaly_created ON anomaly_log(created_at DESC);

-- 5. path_traversal_whitelist: Allowed paths for file operations
CREATE TABLE IF NOT EXISTS path_traversal_whitelist (
    id              TEXT PRIMARY KEY,
    path_pattern    TEXT NOT NULL UNIQUE,
    description     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    enabled         INTEGER NOT NULL DEFAULT 1
);

-- Default whitelist entries
INSERT OR IGNORE INTO path_traversal_whitelist (id, path_pattern, description) VALUES
    (ulid_generate(), '%/skills/%', 'All skill directories'),
    (ulid_generate(), '%/workspace/%', 'User workspace'),
    (ulid_generate(), '%/temp/%', 'Temporary files'),
    (ulid_generate(), '%/.apex/%', 'APEX config and data');

-- 6. injection_patterns: Known injection patterns for detection
CREATE TABLE IF NOT EXISTS injection_patterns (
    id              TEXT PRIMARY KEY,
    pattern_type    TEXT NOT NULL,  -- 'llm_prompt', 'command', 'file_path', 'sql', 'template'
    pattern         TEXT NOT NULL,
    description     TEXT,
    severity        TEXT NOT NULL,
    enabled         INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Default injection patterns
INSERT OR IGNORE INTO injection_patterns (id, pattern_type, pattern, description, severity) VALUES
    (ulid_generate(), 'llm_prompt', '(?i)(system|prompt|ignore.*instructions)', 'Prompt injection attempt', 'high'),
    (ulid_generate(), 'llm_prompt', '(?i)(forget.*previous|new.*instructions)', 'Instruction override attempt', 'high'),
    (ulid_generate(), 'command', '.*\\|\\s*rm\\s+-rf', 'Destructive shell command', 'critical'),
    (ulid_generate(), 'command', '.*\\&\\&\\s*curl.*\\|\\s*sh', 'Pipe to shell execution', 'critical'),
    (ulid_generate(), 'file_path', '\\.\\./\\.\\./', 'Path traversal attempt', 'high'),
    (ulid_generate(), 'file_path', '(?i)(/etc/passwd|/etc/shadow|\\\\windows\\\\system32)', 'Sensitive file access', 'critical'),
    (ulid_generate(), 'sql', "(?i)(union.*select|drop.*table|insert.*into|delete.*from)", 'SQL injection attempt', 'high'),
    (ulid_generate(), 'template', '(?i)(\\{\\{.*\\}\\}|<%.*%>)', 'Template injection attempt', 'medium');

-- 7. skill_execution_allowlist: Skills allowed to execute in each mode
CREATE TABLE IF NOT EXISTS skill_execution_allowlist (
    id              TEXT PRIMARY KEY,
    skill_name      TEXT NOT NULL,
    execution_mode  TEXT NOT NULL,  -- 'bun_pool', 'vm_pool', 'docker', 'firecracker'
    tier            TEXT NOT NULL,   -- 'T0', 'T1', 'T2', 'T3'
    enabled         INTEGER NOT NULL DEFAULT 1,
    UNIQUE(skill_name, execution_mode)
);

-- Default allowlist: Only T3 shell.execute gets VM isolation
INSERT OR IGNORE INTO skill_execution_allowlist (id, skill_name, execution_mode, tier) VALUES
    (ulid_generate(), 'shell.execute', 'vm_pool', 'T3'),
    (ulid_generate(), 'shell.execute', 'bun_pool', 'T3');
