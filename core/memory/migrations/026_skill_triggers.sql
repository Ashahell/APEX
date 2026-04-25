-- Migration 026: Skill triggers for lexical matching fallback
-- Enables keyword-based skill matching when vector search is unavailable

CREATE TABLE IF NOT EXISTS skill_triggers (
    id TEXT PRIMARY KEY NOT NULL,
    skill_name TEXT NOT NULL,
    keyword TEXT NOT NULL,
    weight INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(skill_name, keyword)
);

CREATE INDEX IF NOT EXISTS idx_skill_triggers_keyword ON skill_triggers(keyword);
CREATE INDEX IF NOT EXISTS idx_skill_triggers_skill ON skill_triggers(skill_name);
