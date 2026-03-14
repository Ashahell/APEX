-- Migration: 016_fast_mode_providers
-- Fast Mode & Provider Plugins: Modular LLM providers, session fast mode, model fallbacks

-- 1. provider_plugins: Modular LLM provider configuration
CREATE TABLE IF NOT EXISTS provider_plugins (
    id              TEXT PRIMARY KEY,
    provider_type   TEXT NOT NULL,  -- 'ollama', 'vllm', 'sglang', 'minimax', 'openrouter'
    name            TEXT NOT NULL,
    base_url        TEXT NOT NULL,
    api_key         TEXT,  -- encrypted
    default_model   TEXT,
    config          TEXT,  -- JSON: provider-specific settings
    enabled         INTEGER DEFAULT 1,
    priority        INTEGER DEFAULT 100,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_provider_type ON provider_plugins(provider_type);
CREATE INDEX IF NOT EXISTS idx_provider_enabled ON provider_plugins(enabled);

-- Default: Ollama provider (if available locally)
INSERT OR IGNORE INTO provider_plugins (id, provider_type, name, base_url, default_model, config, enabled, priority) VALUES
    (ulid_generate(), 'ollama', 'Ollama (Local)', 'http://localhost:11434', 'qwen2.5:7b', '{"timeout": 60}', 1, 100);

-- 2. session_fast_mode: Per-session fast mode state
CREATE TABLE IF NOT EXISTS session_fast_mode (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL UNIQUE,
    fast_enabled    INTEGER DEFAULT 0,
    fast_model      TEXT,  -- model to use in fast mode
    fast_config     TEXT,  -- JSON: { temperature: 0.0, max_tokens: 1024 }
    toggles         TEXT,  -- JSON: { thinking: false, verbose: false }
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT idx_fast_session ON session_fast_mode(session_id);

-- 3. model_fallbacks: Fallback chain for models
CREATE TABLE IF NOT EXISTS model_fallbacks (
    id              TEXT PRIMARY KEY,
    primary_model   TEXT NOT NULL,
    fallback_model  TEXT NOT NULL,
    provider        TEXT,
    priority        INTEGER DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(primary_model, fallback_model)
);

CREATE INDEX IF NOT idx_fallback_primary ON model_fallbacks(primary_model);

-- 4. provider_models: Available models per provider
CREATE TABLE IF NOT EXISTS provider_models (
    id              TEXT PRIMARY KEY,
    provider_id     TEXT NOT NULL,
    model_id        TEXT NOT NULL,
    model_name      TEXT NOT NULL,
    context_length  INTEGER,
    supports_vision INTEGER DEFAULT 0,
    supports_tools  INTEGER DEFAULT 1,
    pricing_input   REAL,
    pricing_output  REAL,
    last_verified   TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(provider_id, model_id)
);

CREATE INDEX IF NOT idx_provider_models ON provider_models(provider_id);

-- 5. provider_health: Health status per provider
CREATE TABLE IF NOT EXISTS provider_health (
    id              TEXT PRIMARY KEY,
    provider_id     TEXT NOT NULL UNIQUE,
    status          TEXT NOT NULL,  -- 'healthy', 'degraded', 'unhealthy', 'unknown'
    latency_ms      INTEGER,
    last_check      TEXT NOT NULL DEFAULT (datetime('now')),
    error_message   TEXT,
    consecutive_failures INTEGER DEFAULT 0
);

CREATE INDEX IF NOT idx_health_provider ON provider_health(provider_id);
