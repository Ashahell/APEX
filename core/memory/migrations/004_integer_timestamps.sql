-- Migration: Use INTEGER timestamps (Unix epoch) instead of TEXT
-- This improves query performance and ensures consistent sorting

-- For new installations, we use INTEGER columns
-- Existing TEXT data would need to be migrated separately

-- Note: This migration is for fresh installs
-- Existing databases should use a data migration script

-- Example migration for existing databases:
-- ALTER TABLE tasks ADD COLUMN created_at_int INTEGER;
-- UPDATE tasks SET created_at_int = strftime('%s', created_at) * 1000;
-- ALTER TABLE tasks ADD COLUMN updated_at_int INTEGER;
-- UPDATE tasks SET updated_at_int = strftime('%s', updated_at) * 1000;
