-- Migration: 020_messaging_channels
-- Extended channel configuration for additional messaging platforms

-- Channel settings (per-channel specific configuration)
CREATE TABLE IF NOT EXISTS channel_settings (
    id              TEXT PRIMARY KEY,
    channel_type    TEXT NOT NULL,  -- 'signal', 'irc', 'matrix', 'teams', 'feishu', 'line', 'mattermost', 'nostr', 'synology', 'webchat'
    channel_id      TEXT NOT NULL,
    settings        TEXT NOT NULL,  -- JSON: channel-specific config
    credentials_encrypted TEXT,  -- Encrypted credentials JSON
    is_enabled      INTEGER DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_channel_type ON channel_settings(channel_type);
CREATE INDEX IF NOT EXISTS idx_channel_enabled ON channel_settings(is_enabled);

-- Channel message templates
CREATE TABLE IF NOT EXISTS channel_templates (
    id              TEXT PRIMARY KEY,
    channel_type    TEXT NOT NULL,
    template_name   TEXT NOT NULL,
    template_content TEXT NOT NULL,  -- JSON template
    is_default      INTEGER DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_template_channel ON channel_templates(channel_type);

-- Channel webhooks
CREATE TABLE IF NOT EXISTS channel_webhooks (
    id              TEXT PRIMARY KEY,
    channel_type    TEXT NOT NULL,
    channel_id      TEXT NOT NULL,
    webhook_url    TEXT NOT NULL,
    events         TEXT NOT NULL,  -- JSON array: ['message', 'reaction', etc.]
    is_enabled     INTEGER DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_webhook_channel ON channel_webhooks(channel_type);

-- Insert default templates for common channels
INSERT OR IGNORE INTO channel_templates (id, channel_type, template_name, template_content, is_default)
VALUES 
    ('matrix_default', 'matrix', 'default', '{"format": "html"}', 1),
    ('teams_default', 'teams', 'default', '{"format": "adaptive_card"}', 1),
    ('slack_default', 'slack', 'default', '{"format": "block_kit"}', 1);
