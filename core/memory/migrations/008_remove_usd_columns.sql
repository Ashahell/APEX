-- Migration: Remove REAL cost columns (keep only INTEGER cents)
-- Completed migration 007, now removing deprecated USD columns

-- First, populate cents columns from USD if not already set
UPDATE tasks SET cost_estimate_cents = CAST(cost_estimate_usd * 100 AS INTEGER) 
WHERE cost_estimate_usd IS NOT NULL AND cost_estimate_cents IS NULL;

UPDATE tasks SET actual_cost_cents = CAST(actual_cost_usd * 100 AS INTEGER) 
WHERE actual_cost_usd IS NOT NULL AND actual_cost_cents IS NULL;

-- Drop the deprecated REAL columns
ALTER TABLE tasks DROP COLUMN cost_estimate_usd;
ALTER TABLE tasks DROP COLUMN actual_cost_usd;
