# Escalation Patterns - Alert Response Automation

> **Version:** v1.9.0  
> **Concept Type:** Pattern

---

## Overview

Escalation patterns define how alerts progress through increasingly severe response actions when left unacknowledged.

---

## Escalation Chain

```
Alert Triggered
      │
      ▼
┌─────────────┐     Not acknowledged     ┌─────────────┐
│   Level 1   │ ───────────────────────▶ │   Level 2   │
│   (5 min)    │                          │   (10 min)  │
└─────────────┘                          └─────────────┘
                                                │
                                                ▼
                                        ┌─────────────┐
                                        │   Level 3   │
                                        │ (Immediate) │
                                        └─────────────┘
```

---

## Default Configuration

```rust
let default_escalation_config = EscalationConfig {
    enabled: true,
    max_level: 3,
    default_wait_secs: 300,
    levels: vec![
        EscalationLevel {
            level: 1,
            wait_secs: 300,  // 5 minutes
            actions: vec![
                AlertAction::Notify,
                AlertAction::Email { 
                    to: "alerts@example.com".into(),
                    subject: "Alert Unacknowledged".into(),
                },
            ],
        },
        EscalationLevel {
            level: 2,
            wait_secs: 600,  // 10 minutes
            actions: vec![
                AlertAction::ExecuteCommand {
                    command: "echo 'ALERT: Level 2 escalation'".into(),
                },
            ],
        },
        EscalationLevel {
            level: 3,
            wait_secs: 0,  // Immediate
            actions: vec![
                AlertAction::CancelTask,
            ],
        },
    ],
};
```

---

## Escalation Decision Logic

```rust
impl Alert {
    pub fn should_escalate(&self, wait_secs: u32) -> bool {
        // Only escalate active alerts
        if self.status != AlertStatus::Active {
            return false;
        }
        
        // Check if wait time exceeded
        let elapsed = self.time_since_created_secs();
        elapsed >= wait_secs
    }
    
    pub fn escalate(&mut self, new_level: u32) {
        self.escalation_level = new_level;
        self.last_escalation_at = Some(Utc::now());
        if self.escalated_at.is_none() {
            self.escalated_at = Some(Utc::now());
        }
    }
}
```

---

## Action Types

### Log
```rust
AlertAction::Log
// Logs to audit trail
```

### Notify
```rust
AlertAction::Notify
// Sends in-app notification
```

### Email
```rust
AlertAction::Email {
    to: "admin@example.com".into(),
    subject: "Critical Alert".into(),
}
// Sends email notification
```

### Webhook
```rust
AlertAction::Webhook {
    url: "https://hooks.example.com/alerts".into(),
}
// POSTs to webhook endpoint
```

### ExecuteCommand
```rust
AlertAction::ExecuteCommand {
    command: "PagerDuty trigger".into(),
}
// Executes system command
```

### PauseTask
```rust
AlertAction::PauseTask
// Pauses task execution
```

### CancelTask
```rust
AlertAction::CancelTask
// Cancels task immediately
```

---

## Best Practices

1. **Start with Notify**: Always start escalation with non-destructive actions
2. **Gradual Severity**: Increase severity with each level
3. **Meaningful Intervals**: Allow time for human response before escalation
4. **Maximum 3-5 Levels**: Beyond that, automate resolution
5. **Include Context**: Actions should include alert details in notifications

---

## Example: Production Escalation Chain

| Level | Time | Actions | Rationale |
|-------|------|---------|-----------|
| 1 | 5 min | Notify + Email | First response window |
| 2 | 15 min | Slack + Escalation Email | Broader visibility |
| 3 | 30 min | PagerDuty + SMS | Urgent attention |
| 4 | 60 min | Auto-remediation | Last resort before impact |

---

## Related Documentation

- [vigilant-mode.md](../entities/vigilant-mode.md)
- [alert-analytics.md](../entities/alert-analytics.md)
