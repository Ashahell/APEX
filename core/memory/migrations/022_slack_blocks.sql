-- Migration: 022_slack_blocks
-- Slack Block Kit message templates

-- Slack message templates table
CREATE TABLE IF NOT EXISTS slack_block_templates (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    template        TEXT NOT NULL,  -- JSON: Block Kit template
    description     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_slack_template_name ON slack_block_templates(name);

-- Pre-built templates
INSERT OR IGNORE INTO slack_block_templates (id, name, template, description) VALUES
    ('task_complete', 'task_complete', 
     '{"blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "✅ Task Complete: {{task_name}}\\n\\n{{summary}}"}}]}', 
     'Task completion notification'),
    ('error_alert', 'error_alert', 
     '{"blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "⚠️ Error Alert\\n\\n{{error_message}}"}}, {"type": "context", "elements": [{"type": "mrkdwn", "text": "Task: {{task_id}}"}]}]}', 
     'Error notification'),
    ('task_started', 'task_started', 
     '{"blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "🔄 Task Started: {{task_name}}\\n\\n{{description}}"}}]}', 
     'Task started notification'),
    ('confirmation_request', 'confirmation_request', 
     '{"blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "🤔 Confirmation Required\\n\\n{{message}}"}}, {"type": "actions", "block_id": "confirm_actions", "elements": [{"type": "button", "text": {"type": "plain_text", "text": "Approve"}, "style": "primary", "action_id": "approve"}, {"type": "button", "text": {"type": "plain_text", "text": "Deny"}, "style": "danger", "action_id": "deny"}]}]}', 
     'T1-T3 confirmation request'),
    ('budget_alert', 'budget_alert', 
     '{"blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "💰 Budget Alert\\n\\nCurrent spend: ${{current_cost}}\\nBudget limit: ${{budget_limit}}"}}, {"type": "context", "elements": [{"type": "mrkdwn", "text": "{{percentage}}% of budget used"}]}]}', 
     'Budget threshold notification'),
    ('session_resume', 'session_resume', 
     '{"blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "▶️ Session Resumed\\n\\n{{session_info}}"}}]}', 
     'Session resume notification');
