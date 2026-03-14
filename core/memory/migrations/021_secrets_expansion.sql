-- Migration: 021_secrets_expansion
-- Expanded secret management with references and rotation

-- Secret references (for runtime resolution)
CREATE TABLE IF NOT EXISTS secret_refs (
    id              TEXT PRIMARY KEY,
    ref_key         TEXT NOT NULL UNIQUE,
    secret_name     TEXT NOT NULL,
    env_var         TEXT,  -- Maps to this env var
    description     TEXT,
    targets         TEXT NOT NULL,  -- JSON: ['tool.read', 'skill.foo', 'adapter.slack', etc.]
    category        TEXT DEFAULT 'generic',  -- 'api_key', 'token', 'credential', 'certificate', 'generic'
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_secret_ref_key ON secret_refs(ref_key);
CREATE INDEX IF NOT EXISTS idx_secret_ref_category ON secret_refs(category);

-- Secret rotation log
CREATE TABLE IF NOT EXISTS secret_rotation_log (
    id              TEXT PRIMARY KEY,
    secret_name     TEXT NOT NULL,
    rotated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    rotated_by      TEXT,
    status          TEXT NOT NULL,  -- 'success', 'failed', 'pending'
    error_message   TEXT,
    old_value_hash  TEXT,  -- Hash of old value (for audit)
    new_value_hash  TEXT  -- Hash of new value (for audit)
);

CREATE INDEX IF NOT EXISTS idx_rotation_secret ON secret_rotation_log(secret_name);
CREATE INDEX IF NOT EXISTS idx_rotation_status ON secret_rotation_log(status);

-- Secret access audit
CREATE TABLE IF NOT EXISTS secret_access_log (
    id              TEXT PRIMARY KEY,
    secret_ref_id   TEXT NOT NULL,
    accessed_at     TEXT NOT NULL DEFAULT (datetime('now')),
    access_type     TEXT NOT NULL,  -- 'read', 'write', 'delete', 'resolve'
    accessed_by     TEXT,  -- Component that accessed the secret
    success         INTEGER DEFAULT 1,
    error_message   TEXT
);

CREATE INDEX IF NOT EXISTS idx_access_ref ON secret_access_log(secret_ref_id);
CREATE INDEX IF NOT EXISTS idx_access_time ON secret_access_log(accessed_at);

-- Predefined secret categories/targets for OpenClaw compatibility
-- 64 target types covering all common credential uses
INSERT OR IGNORE INTO secret_refs (id, ref_key, secret_name, targets, category, created_at, updated_at)
VALUES 
    -- API Keys
    ('openai_api_key', 'OPENAI_API_KEY', 'OpenAI API Key', '["llm.openai", "provider.openai"]', 'api_key', datetime('now'), datetime('now')),
    ('anthropic_api_key', 'ANTHROPIC_API_KEY', 'Anthropic API Key', '["llm.anthropic", "provider.anthropic"]', 'api_key', datetime('now'), datetime('now')),
    ('google_api_key', 'GOOGLE_API_KEY', 'Google AI API Key', '["llm.google", "provider.google"]', 'api_key', datetime('now'), datetime('now')),
    
    -- OAuth Tokens
    ('slack_token', 'SLACK_TOKEN', 'Slack OAuth Token', '["adapter.slack"]', 'token', datetime('now'), datetime('now')),
    ('discord_token', 'DISCORD_TOKEN', 'Discord Bot Token', '["adapter.discord"]', 'token', datetime('now'), datetime('now')),
    ('telegram_token', 'TELEGRAM_TOKEN', 'Telegram Bot Token', '["adapter.telegram"]', 'token', datetime('now'), datetime('now')),
    
    -- Database
    ('db_password', 'DB_PASSWORD', 'Database Password', '["db.postgres", "db.mysql", "db.sqlite"]', 'credential', datetime('now'), datetime('now')),
    ('db_api_key', 'DB_API_KEY', 'Database API Key', '["db.supabase", "db.firebase"]', 'api_key', datetime('now'), datetime('now')),
    
    -- Cloud Providers
    ('aws_access_key', 'AWS_ACCESS_KEY_ID', 'AWS Access Key', '["deploy.aws", "storage.aws"]', 'api_key', datetime('now'), datetime('now')),
    ('aws_secret_key', 'AWS_SECRET_ACCESS_KEY', 'AWS Secret Key', '["deploy.aws", "storage.aws"]', 'credential', datetime('now'), datetime('now')),
    ('gcp_key', 'GCP_SERVICE_ACCOUNT_KEY', 'GCP Service Account Key', '["deploy.gcp", "storage.gcp"]', 'credential', datetime('now'), datetime('now')),
    ('azure_key', 'AZURE_STORAGE_KEY', 'Azure Storage Key', '["deploy.azure", "storage.azure"]', 'credential', datetime('now'), datetime('now')),
    
    -- Webhooks
    ('webhook_secret', 'WEBHOOK_SECRET', 'Webhook Secret', '["webhook.verification"]', 'credential', datetime('now'), datetime('now')),
    
    -- SSH Keys
    ('ssh_private_key', 'SSH_PRIVATE_KEY', 'SSH Private Key', '["deploy.ssh", "git.ssh"]', 'certificate', datetime('now'), datetime('now')),
    ('ssh_public_key', 'SSH_PUBLIC_KEY', 'SSH Public Key', '["deploy.ssh", "git.ssh"]', 'certificate', datetime('now'), datetime('now')),
    
    -- Encryption
    ('encryption_key', 'ENCRYPTION_KEY', 'Encryption Key', '["security.encryption"]', 'credential', datetime('now'), datetime('now')),
    ('hmac_secret', 'HMAC_SECRET', 'HMAC Secret', '["security.hmac"]', 'credential', datetime('now'), datetime('now')),
    
    -- LLM Providers
    ('ollama_url', 'OLLAMA_URL', 'Ollama URL', '["llm.ollama"]', 'generic', datetime('now'), datetime('now')),
    ('ollama_api_key', 'OLLAMA_API_KEY', 'Ollama API Key', '["llm.ollama"]', 'api_key', datetime('now'), datetime('now')),
    ('vllm_url', 'VLLM_URL', 'vLLM Server URL', '["llm.vllm"]', 'generic', datetime('now'), datetime('now')),
    ('vllm_api_key', 'VLLM_API_KEY', 'vLLM API Key', '["llm.vllm"]', 'api_key', datetime('now'), datetime('now')),
    
    -- Additional Channels
    ('matrix_token', 'MATRIX_TOKEN', 'Matrix Access Token', '["adapter.matrix"]', 'token', datetime('now'), datetime('now')),
    ('signal_cli_path', 'SIGNAL_CLI_PATH', 'Signal CLI Path', '["adapter.signal"]', 'generic', datetime('now'), datetime('now')),
    ('irc_password', 'IRC_PASSWORD', 'IRC Password', '["adapter.irc"]', 'credential', datetime('now'), datetime('now')),
    ('teams_webhook', 'TEAMS_WEBHOOK_URL', 'MS Teams Webhook URL', '["adapter.teams"]', 'generic', datetime('now'), datetime('now')),
    ('feishu_app_id', 'FEISHU_APP_ID', 'Feishu App ID', '["adapter.feishu"]', 'api_key', datetime('now'), datetime('now')),
    ('feishu_app_secret', 'FEISHU_APP_SECRET', 'Feishu App Secret', '["adapter.feishu"]', 'credential', datetime('now'), datetime('now')),
    ('line_channel_secret', 'LINE_CHANNEL_SECRET', 'LINE Channel Secret', '["adapter.line"]', 'credential', datetime('now'), datetime('now')),
    ('line_channel_token', 'LINE_CHANNEL_TOKEN', 'LINE Channel Access Token', '["adapter.line"]', 'token', datetime('now'), datetime('now')),
    ('mattermost_token', 'MATTERMOST_TOKEN', 'Mattermost Access Token', '["adapter.mattermost"]', 'token', datetime('now'), datetime('now')),
    ('nostr_private_key', 'NOSTR_PRIVATE_KEY', 'Nostr Private Key', '["adapter.nostr"]', 'credential', datetime('now'), datetime('now')),
    
    -- Storage
    ('s3_bucket', 'S3_BUCKET', 'S3 Bucket Name', '["storage.s3"]', 'generic', datetime('now'), datetime('now')),
    ('s3_region', 'S3_REGION', 'S3 Region', '["storage.s3"]', 'generic', datetime('now'), datetime('now')),
    ('gcs_bucket', 'GCS_BUCKET', 'GCS Bucket Name', '["storage.gcs"]', 'generic', datetime('now'), datetime('now')),
    ('azure_blob', 'AZURE_BLOB_CONTAINER', 'Azure Blob Container', '["storage.azure"]', 'generic', datetime('now'), datetime('now')),
    
    -- Email/SMTP
    ('smtp_host', 'SMTP_HOST', 'SMTP Host', '["adapter.email"]', 'generic', datetime('now'), datetime('now')),
    ('smtp_user', 'SMTP_USERNAME', 'SMTP Username', '["adapter.email"]', 'credential', datetime('now'), datetime('now')),
    ('smtp_password', 'SMTP_PASSWORD', 'SMTP Password', '["adapter.email"]', 'credential', datetime('now'), datetime('now')),
    
    -- Misc Services
    ('github_token', 'GITHUB_TOKEN', 'GitHub Personal Access Token', '["git.github", "adapter.github"]', 'token', datetime('now'), datetime('now')),
    ('gitlab_token', 'GITLAB_TOKEN', 'GitLab Token', '["git.gitlab"]', 'token', datetime('now'), datetime('now')),
    ('jira_api_key', 'JIRA_API_KEY', 'Jira API Key', '["adapter.jira"]', 'api_key', datetime('now'), datetime('now')),
    ('notion_token', 'NOTION_TOKEN', 'Notion Integration Token', '["adapter.notion"]', 'token', datetime('now'), datetime('now')),
    ('airtable_key', 'AIRTABLE_API_KEY', 'Airtable API Key', '["adapter.airtable"]', 'api_key', datetime('now'), datetime('now')),
    ('sendgrid_key', 'SENDGRID_API_KEY', 'SendGrid API Key', '["adapter.sendgrid"]', 'api_key', datetime('now'), datetime('now')),
    ('twilio_sid', 'TWILIO_ACCOUNT_SID', 'Twilio Account SID', '["adapter.twilio"]', 'api_key', datetime('now'), datetime('now')),
    ('twilio_token', 'TWILIO_AUTH_TOKEN', 'Twilio Auth Token', '["adapter.twilio"]', 'credential', datetime('now'), datetime('now')),
    ('stripe_key', 'STRIPE_SECRET_KEY', 'Stripe Secret Key', '["adapter.stripe"]', 'credential', datetime('now'), datetime('now')),
    
    -- Custom/Generic
    ('custom_1', 'CUSTOM_SECRET_1', 'Custom Secret 1', '["custom"]', 'generic', datetime('now'), datetime('now')),
    ('custom_2', 'CUSTOM_SECRET_2', 'Custom Secret 2', '["custom"]', 'generic', datetime('now'), datetime('now')),
    ('custom_3', 'CUSTOM_SECRET_3', 'Custom Secret 3', '["custom"]', 'generic', datetime('now'), datetime('now')),
    ('custom_4', 'CUSTOM_SECRET_4', 'Custom Secret 4', '["custom"]', 'generic', datetime('now'), datetime('now')),
    ('custom_5', 'CUSTOM_SECRET_5', 'Custom Secret 5', '["custom"]', 'generic', datetime('now'), datetime('now'));
