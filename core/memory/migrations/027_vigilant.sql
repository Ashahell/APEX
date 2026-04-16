-- M2: Vigilant Mode - Alert Monitoring
-- Alert rules, active alerts, and alert history

-- Alert rules table
CREATE TABLE IF NOT EXISTS alert_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    alert_type TEXT NOT NULL,               -- JSON serialized AlertType
    severity TEXT NOT NULL,                 -- Info | Warning | Critical
    cooldown_secs INTEGER NOT NULL DEFAULT 0,
    actions TEXT NOT NULL,                  -- JSON array of AlertAction
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Active alerts table
CREATE TABLE IF NOT EXISTS alerts (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL,
    alert_type TEXT NOT NULL,               -- JSON serialized AlertType
    severity TEXT NOT NULL,
    task_id TEXT,
    message TEXT NOT NULL,
    payload TEXT,                           -- Additional JSON data
    status TEXT NOT NULL DEFAULT 'Active',  -- Active | Acknowledged | Dismissed | Resolved
    acknowledged_at TEXT,
    acknowledged_by TEXT,
    created_at TEXT NOT NULL,
    resolved_at TEXT,
    FOREIGN KEY (rule_id) REFERENCES alert_rules(id)
);

-- Alert history for audit trail
CREATE TABLE IF NOT EXISTS alert_history (
    id TEXT PRIMARY KEY,
    alert_id TEXT NOT NULL,
    action TEXT NOT NULL,                   -- created | acknowledged | dismissed | resolved
    performed_by TEXT,
    metadata TEXT,                          -- JSON additional data
    created_at TEXT NOT NULL,
    FOREIGN KEY (alert_id) REFERENCES alerts(id)
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_alert_rules_enabled ON alert_rules(enabled);
CREATE INDEX IF NOT EXISTS idx_alert_rules_severity ON alert_rules(severity);
CREATE INDEX IF NOT EXISTS idx_alerts_status ON alerts(status);
CREATE INDEX IF NOT EXISTS idx_alerts_severity ON alerts(severity);
CREATE INDEX IF NOT EXISTS idx_alerts_task ON alerts(task_id);
CREATE INDEX IF NOT EXISTS idx_alerts_created ON alerts(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_history_alert ON alert_history(alert_id);

-- Insert default alert rules
INSERT OR IGNORE INTO alert_rules (id, name, alert_type, severity, cooldown_secs, actions, enabled, created_at, updated_at) VALUES
('builtin-loop-detection', 'Infinite Loop Detection', 
 '{"type":"InfiniteLoop","task_id":"","iterations":100}', 
 'Critical', 300, 
 '[{"type":"Notify"},{"type":"CancelTask"}]', 
 1, datetime('now'), datetime('now'));

INSERT OR IGNORE INTO alert_rules (id, name, alert_type, severity, cooldown_secs, actions, enabled, created_at, updated_at) VALUES
('builtin-no-progress', 'No Progress Warning', 
 '{"type":"NoProgress","task_id":"","steps":10}', 
 'Warning', 60, 
 '[{"type":"Notify"}]', 
 1, datetime('now'), datetime('now'));

INSERT OR IGNORE INTO alert_rules (id, name, alert_type, severity, cooldown_secs, actions, enabled, created_at, updated_at) VALUES
('builtin-timeout-warning', 'Timeout Warning', 
 '{"type":"TimeoutWarning","task_id":"","remaining_secs":60}', 
 'Warning', 0, 
 '[{"type":"Notify"}]', 
 1, datetime('now'), datetime('now'));
