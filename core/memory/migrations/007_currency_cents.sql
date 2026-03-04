-- Migration: Fix currency precision (INTEGER cents)
-- Fixes floating-point precision issues with currency

-- Add INTEGER columns for cents (backward compatible with old REAL columns)
ALTER TABLE tasks ADD COLUMN cost_estimate_cents INTEGER;
ALTER TABLE tasks ADD COLUMN actual_cost_cents INTEGER;

-- Create index for cost queries
CREATE INDEX IF NOT EXISTS idx_tasks_actual_cost_cents ON tasks(actual_cost_cents);

-- Note: Existing data will have NULL cents values
-- Application code should populate cents columns from USD columns on read
-- Or run: UPDATE tasks SET actual_cost_cents = CAST(actual_cost_usd * 100 AS INTEGER) WHERE actual_cost_usd IS NOT NULL;
