-- Migration: Add adapter configuration to channels table
-- Enables channels table to be the source of truth for adapter configuration

-- Add adapter-specific configuration columns
ALTER TABLE channels ADD COLUMN adapter_type TEXT;
ALTER TABLE channels ADD COLUMN adapter_config TEXT;  -- JSON blob for adapter-specific settings
ALTER TABLE channels ADD COLUMN credentials TEXT;  -- Encrypted credentials
ALTER TABLE channels ADD COLUMN webhook_url TEXT;
ALTER TABLE channels ADD COLUMN status TEXT DEFAULT 'inactive';
ALTER TABLE channels ADD COLUMN last_connected_at_ms INTEGER;
ALTER TABLE channels ADD COLUMN health_status TEXT;

-- Add index for status queries
CREATE INDEX IF NOT EXISTS idx_channels_status ON channels(status);
CREATE INDEX IF NOT EXISTS idx_channels_adapter_type ON channels(adapter_type);

-- Update existing default channel
UPDATE channels SET adapter_type = 'default', status = 'active' WHERE name = 'default';
UPDATE channels SET adapter_type = 'default', status = 'active' WHERE name = 'general';
