-- Migration: 017_subagent_control
-- sessions_yield & sessions_resume: Enhanced subagent control flow

-- 1. session_yield_log: Track yield events
CREATE TABLE IF NOT EXISTS session_yield_log (
    id              TEXT PRIMARY KEY,
    parent_session  TEXT NOT NULL,
    child_session   TEXT NOT NULL,
    yield_reason    TEXT,  -- 'completed', 'yielded', 'error', 'timeout'
    yield_payload   TEXT,  -- JSON: hidden payload for next turn
    resumed_at       TEXT,
    created_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_yield_parent ON session_yield_log(parent_session);
CREATE INDEX IF NOT EXISTS idx_yield_child ON session_yield_log(child_session);

-- 2. session_resume_history: Resume conversation history
CREATE TABLE IF NOT EXISTS session_resume_history (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL,
    resumed_from    TEXT NOT NULL,  -- original session_id
    resume_type     TEXT NOT NULL,  -- 'acp', 'codex', 'manual'
    context_summary TEXT,  -- Summary of previous context
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_resume_session ON session_resume_history(session_id);

-- 3. session_attachments: File attachments for sessions
CREATE TABLE IF NOT EXISTS session_attachments (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL,
    task_id         TEXT,
    file_name       TEXT NOT NULL,
    file_type       TEXT NOT NULL,
    file_size       INTEGER NOT NULL,
    file_path       TEXT NOT NULL,  -- Local path or base64 for small files
    encoding        TEXT DEFAULT 'binary',  -- 'binary', 'base64', 'utf8'
    uploaded_by     TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_attachment_session ON session_attachments(session_id);
CREATE INDEX IF NOT EXISTS idx_attachment_task ON session_attachments(task_id);

-- 4. session_state: Persist session state for resume
CREATE TABLE IF NOT EXISTS session_state (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL UNIQUE,
    state_data      TEXT NOT NULL,  -- JSON: { messages: [], context: {}, tools: [] }
    checkpoint_id   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_session_state_session ON session_state(session_id);

-- 5. session_checkpoints: Named checkpoints for quick resume
CREATE TABLE IF NOT EXISTS session_checkpoints (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL,
    checkpoint_name TEXT NOT NULL,
    checkpoint_data TEXT NOT NULL,  -- JSON snapshot
    description     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_checkpoints_session ON session_checkpoints(session_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_checkpoints_name ON session_checkpoints(session_id, checkpoint_name);
