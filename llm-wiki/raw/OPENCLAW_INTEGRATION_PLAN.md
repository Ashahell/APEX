# APEX Enhancement Plan: OpenClaw Features Integration

> **Date**: 2026-03-10
> **Version**: v1.3.2
> **Status**: Planned

---

## Overview

This document outlines the implementation plan for integrating three key OpenClaw features into APEX:

1. **Death Spiral Detection** - Enhanced pattern detection for agent failure modes
2. **External Notifications** - Discord/Telegram notifications for task completion
3. **Workspace .env Loading** - Environment variable injection at skill execution time

---

## 1. Death Spiral Detection

### Problem
Agents can get stuck in failure loops without proper detection. OpenClaw identifies 4 common death spiral patterns.

### Implementation

#### Files Modified
- `core/router/src/security/anomaly_detector.rs` - Add new detection patterns

#### New Patterns to Add

| Pattern | Detection Logic | Severity |
|---------|----------------|----------|
| **File Creation Bursts** | >10 file creates in 5 seconds | Warning |
| **Tool Call Loops** | Same tool called >5 times in a row | Critical |
| **No Side Effects** | >10 tool calls with no state change | Warning |
| **Error Cascades** | >3 sequential errors | Critical |

#### Database Schema
```sql
-- Add to migration 015
CREATE TABLE IF NOT EXISTS execution_patterns (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    pattern_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    tool_calls JSON,
    file_ops JSON,
    error_count INTEGER DEFAULT 0,
    detected_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_patterns_task ON execution_patterns(task_id);
CREATE INDEX idx_patterns_type ON execution_patterns(pattern_type);
```

---

## 2. External Notifications

### Problem
Sub-agents complete but user doesn't know when using async channels (Discord/Telegram).

### Implementation

#### Architecture
```
Task Completion
    │
    ▼
NotificationManager
    │
    ├──► In-app notifications (existing)
    │
    └──► External channels
            │
            ├──► Discord webhook
            │
            └──► Telegram bot
```

#### Files Modified
- `core/router/src/notification.rs` - Add external notification support
- `core/router/src/api/notifications.rs` - Add external config endpoints
- `core/router/src/main.rs` - Initialize notification clients

#### New Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| PUT | `/api/v1/notifications/external` | Configure external notification |
| POST | `/api/v1/notifications/test` | Send test notification |
| GET | `/api/v1/notifications/channels` | List configured channels |

#### Configuration Schema
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalNotificationConfig {
    pub enabled: bool,
    pub discord_webhook: Option<String>,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub notify_on: Vec<NotificationTrigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationTrigger {
    TaskCompleted,
    TaskFailed,
    TaskCancelled,
    AnomalyDetected,
    ConstitutionViolation,
}
```

---

## 3. Workspace .env Loading

### Problem
Skills need access to workspace-specific environment variables at execution time.

### Implementation

#### Architecture
```
Skill Execution Request
    │
    ▼
WorkspaceResolver
    │
    ├──► Read .env from workspace directory
    │
    ├──► Merge with skill default env
    │
    └──► Inject into skill process
```

#### Files Modified
- `core/router/src/skill_pool.rs` - Add workspace env resolution
- `core/router/src/skill_pool_ipc.rs` - Pass env to worker
- `skills/pool_worker.ts` - Read and inject env vars

#### Implementation Details

1. **Workspace Detection**
   - Get workspace from task context (project field)
   - Default workspace: `./workspace/`
   - Custom: `./workspace/{project}/`

2. **Env Loading**
   - Load `.env` file if exists
   - Merge with system env
   - Skill env overrides workspace env

3. **Security**
   - Block dangerous vars (APEX_* prefix)
   - Log env access for audit

---

## 4. UI Elements

### Settings → Notifications Tab

**Current**: In-app notifications only

**New**:
- External channels section
- Discord webhook URL input
- Telegram bot token + chat ID inputs
- Test notification button
- Toggle switches for each trigger type

### Settings → Workspace Tab

**New**:
- Workspace directory path config
- .env file management
- View loaded environment variables
- Clear cache button

### Dashboard → Anomalies Panel

**New**:
- Death spiral alerts
- Pattern breakdown charts
- Severity indicators

---

## Implementation Order

| Phase | Tasks | Files |
|-------|-------|-------|
| 1 | Death Spiral Detection patterns | anomaly_detector.rs, migration 015 |
| 2 | External Notifications - Core | notification.rs, api/notifications.rs |
| 3 | External Notifications - Discord | discord webhook client |
| 4 | External Notifications - Telegram | telegram bot client |
| 5 | Workspace .env - Backend | skill_pool.rs, skill_pool_ipc.rs |
| 6 | Workspace .env - Worker | pool_worker.ts |
| 7 | UI - Notifications Tab | React components |
| 8 | UI - Workspace Tab | React components |
| 9 | Integration Tests | Test suite |

---

## Testing Strategy

### Unit Tests
- Death spiral pattern detection (4 tests)
- Notification routing (3 tests)
- Env loading/merging (5 tests)

### Integration Tests
- Full notification flow (Discord/Telegram mock)
- Workspace env injection
- Pattern detection in real execution

---

## Backward Compatibility

- All existing APIs unchanged
- Default notifications unchanged
- Workspace env opt-in only
- No breaking changes to skill execution
