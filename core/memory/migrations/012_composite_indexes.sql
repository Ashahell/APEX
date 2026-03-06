-- Migration: Add composite indexes for common query patterns
-- Optimizes filter queries that combine multiple columns

-- Composite indexes for task filtering (most common query pattern)
CREATE INDEX IF NOT EXISTS idx_tasks_project_status ON tasks(project, status);
CREATE INDEX IF NOT EXISTS idx_tasks_status_priority ON tasks(status, priority);
CREATE INDEX IF NOT EXISTS idx_tasks_priority_category ON tasks(priority, category);
CREATE INDEX IF NOT EXISTS idx_tasks_project_status_priority ON tasks(project, status, priority);
CREATE INDEX IF NOT EXISTS idx_tasks_project_category ON tasks(project, category);

-- Composite index for complete filter pattern (all columns)
CREATE INDEX IF NOT EXISTS idx_tasks_filter_all ON tasks(project, status, priority, category);

-- Index for sorting by priority within status
CREATE INDEX IF NOT EXISTS idx_tasks_status_priority_created ON tasks(status, priority, created_at_ms DESC);
