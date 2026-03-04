-- Migration: Add skill version history table
-- This enables tracking skill version changes over time

CREATE TABLE IF NOT EXISTS skill_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    skill_name TEXT NOT NULL,
    version TEXT NOT NULL,
    tier TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    health_status TEXT NOT NULL DEFAULT 'unknown',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000)
);

CREATE INDEX IF NOT EXISTS idx_skill_versions_name ON skill_versions(skill_name);
CREATE INDEX IF NOT EXISTS idx_skill_versions_created ON skill_versions(created_at);
