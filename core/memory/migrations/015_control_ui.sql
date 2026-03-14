-- Migration: 015_control_ui
-- Control UI Dashboard: Layout preferences, pinned messages, chat bookmarks, session metadata
-- See docs/OPENCLAW_IMPLEMENTATION_PLAN.md

-- 1. dashboard_layout: User dashboard layout preferences
CREATE TABLE IF NOT EXISTS dashboard_layout (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL DEFAULT 'default',
    layout_config   TEXT NOT NULL DEFAULT '{}',  -- JSON: { sidebar: {}, panels: [], order: [] }
    theme           TEXT DEFAULT 'agentzero',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_dashboard_layout_user ON dashboard_layout(user_id);

-- 2. pinned_messages: Pinned message storage
CREATE TABLE IF NOT EXISTS pinned_messages (
    id              TEXT PRIMARY KEY,
    message_id      TEXT NOT NULL,
    channel         TEXT NOT NULL,
    thread_id       TEXT,
    task_id         TEXT,
    pinned_by       TEXT NOT NULL DEFAULT 'user',
    pin_note        TEXT,
    pinned_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_pinned_channel ON pinned_messages(channel);
CREATE INDEX IF NOT EXISTS idx_pinned_task ON pinned_messages(task_id);
CREATE INDEX IF NOT EXISTS idx_pinned_message ON pinned_messages(message_id);

-- 3. chat_bookmarks: User bookmarks in conversations
CREATE TABLE IF NOT EXISTS chat_bookmarks (
    id              TEXT PRIMARY KEY,
    message_id      TEXT NOT NULL,
    channel         TEXT NOT NULL,
    thread_id       TEXT,
    bookmark_note   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_bookmarks_channel ON chat_bookmarks(channel);
CREATE INDEX IF NOT EXISTS idx_bookmarks_thread ON chat_bookmarks(thread_id);
CREATE INDEX IF NOT EXISTS idx_bookmarks_message ON chat_bookmarks(message_id);

-- 4. session_metadata: Extended session information
CREATE TABLE IF NOT EXISTS session_metadata (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL UNIQUE,
    model           TEXT,
    thinking_level  TEXT DEFAULT 'medium',
    verbose_level   TEXT DEFAULT 'default',
    fast_mode       INTEGER DEFAULT 0,
    send_policy     TEXT DEFAULT 'async',
    activation_mode TEXT DEFAULT 'always',
    group_policy    TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_session_model ON session_metadata(model);
CREATE INDEX IF NOT EXISTS idx_session_fast ON session_metadata(fast_mode);

-- 5. dashboard_chat_history: Chat history for search/export
CREATE TABLE IF NOT EXISTS dashboard_chat_history (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL,
    message_id      TEXT NOT NULL,
    channel         TEXT NOT NULL,
    thread_id       TEXT,
    author          TEXT NOT NULL,
    content         TEXT NOT NULL,
    role            TEXT NOT NULL,  -- 'user', 'assistant', 'system', 'tool'
    metadata        TEXT,  -- JSON: { attachments: [], reactions: [] }
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_dashboard_chat_session ON dashboard_chat_history(session_id);
CREATE INDEX IF NOT EXISTS idx_dashboard_chat_channel ON dashboard_chat_history(channel);
CREATE INDEX IF NOT EXISTS idx_dashboard_chat_created ON dashboard_chat_history(created_at DESC);

-- 6. command_palette_history: User command history for quick access
CREATE TABLE IF NOT EXISTS command_palette_history (
    id              TEXT PRIMARY KEY,
    command         TEXT NOT NULL,
    command_type    TEXT NOT NULL,  -- 'action', 'skill', 'navigation', 'search'
    frequency       INTEGER DEFAULT 1,
    last_used       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_palette_frequency ON command_palette_history(frequency DESC);
CREATE INDEX IF NOT EXISTS idx_palette_type ON command_palette_history(command_type);

-- 7. dashboard_exports: Export history
CREATE TABLE IF NOT EXISTS dashboard_exports (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL,
    export_format   TEXT NOT NULL,  -- 'json', 'markdown', 'pdf', 'txt'
    export_range    TEXT NOT NULL,  -- 'all', 'selection', 'date_range'
    date_from       TEXT,
    date_to         TEXT,
    file_path       TEXT,
    status          TEXT NOT NULL,  -- 'pending', 'completed', 'failed'
    error_message   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at    TEXT
);

CREATE INDEX IF NOT EXISTS idx_exports_session ON dashboard_exports(session_id);
CREATE INDEX IF NOT EXISTS idx_exports_status ON dashboard_exports(status);
