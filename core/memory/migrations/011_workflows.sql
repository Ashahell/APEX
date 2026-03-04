-- Migration: Add workflows table for YAML workflow storage
-- Enables storing and executing predefined workflows

-- Create workflows table
CREATE TABLE IF NOT EXISTS workflows (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    definition TEXT NOT NULL,  -- YAML workflow definition
    category TEXT,
    version INTEGER DEFAULT 1,
    is_active INTEGER DEFAULT 1,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    last_executed_at_ms INTEGER,
    execution_count INTEGER DEFAULT 0,
    avg_duration_secs REAL,
    success_rate REAL
);

-- Indexes for workflow queries
CREATE INDEX IF NOT EXISTS idx_workflows_category ON workflows(category);
CREATE INDEX IF NOT EXISTS idx_workflows_is_active ON workflows(is_active);
CREATE INDEX IF NOT EXISTS idx_workflows_last_executed ON workflows(last_executed_at_ms);

-- Create workflow_execution_logs table for tracking runs
CREATE TABLE IF NOT EXISTS workflow_execution_logs (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    status TEXT NOT NULL,  -- pending, running, completed, failed, cancelled
    started_at_ms INTEGER NOT NULL,
    completed_at_ms INTEGER,
    duration_secs REAL,
    input_data TEXT,  -- JSON
    output_data TEXT,  -- JSON
    error_message TEXT,
    triggered_by TEXT,
    FOREIGN KEY (workflow_id) REFERENCES workflows(id)
);

CREATE INDEX IF NOT EXISTS idx_wf_executions_workflow_id ON workflow_execution_logs(workflow_id);
CREATE INDEX IF NOT EXISTS idx_wf_executions_status ON workflow_execution_logs(status);
CREATE INDEX IF NOT EXISTS idx_wf_executions_started ON workflow_execution_logs(started_at_ms);
