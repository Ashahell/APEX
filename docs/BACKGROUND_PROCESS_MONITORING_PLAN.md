# Background Process Monitoring & Vigilant Mode Implementation Plan

> **Inspired by**: NousResearch Hermes Agent v0.9.0
> **Target Version**: v1.9.0 (Future Release)
> **Status**: Planning

---

## Executive Summary

This plan covers two complementary features from Hermes Agent:

1. **Background Process Monitoring** - Watch patterns in agent execution, auto-notify on completion
2. **Vigilant Mode** - Local dashboard + alert system for proactive agent oversight

Both features enhance **observability** and **control** over autonomous agent execution.

---

## Feature 1: Background Process Monitoring

### Overview

Monitor background task execution with configurable notification modes and structured logging. Allows users to watch for specific patterns without constantly monitoring the UI.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Background Monitor Service                   │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐   │
│  │  Watcher    │  │  Notifier   │  │  Notification Dispatcher│   │
│  │  Registry   │  │  Service    │  │  (WS/SSE/Push/Telegram) │   │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                     Hook System                                  │
│  agent:start │ agent:step │ agent:end │ session:start │ session:end │
└─────────────────────────────────────────────────────────────────┘
```

### Data Model

```rust
// core/router/src/monitoring/models.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchPattern {
    pub id: String,
    pub name: String,
    pub pattern: Regex,              // Pattern to watch for
    pub watch_scope: WatchScope,     // Which events to watch
    pub notify_on: NotifyOn,         // Match, completion, error, etc.
    pub notification_mode: NotifyMode,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatchScope {
    All,                     // Watch all tasks
    Project(String),         // Watch specific project
    TaskIds(Vec<String>),    // Watch specific task IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotifyOn {
    Match,           // Pattern matched
    Completion,      // Task completed
    Error,          // Task failed
    Timeout,        // Task timed out
    Threshold,      // N matches reached
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotifyMode {
    All,      // All events
    Result,   // Only final result
    Error,    // Only errors
    Off,      // Disabled
}
```

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/monitor/watchers` | List all watch patterns |
| POST | `/api/v1/monitor/watchers` | Create watch pattern |
| GET | `/api/v1/monitor/watchers/:id` | Get watcher details |
| PUT | `/api/v1/monitor/watchers/:id` | Update watcher |
| DELETE | `/api/v1/monitor/watchers/:id` | Delete watcher |
| GET | `/api/v1/monitor/events` | Get recent events (SSE stream) |
| GET | `/api/v1/monitor/stats` | Get monitoring statistics |

### Hook System

```rust
// Hook event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MonitorEvent {
    AgentStart { task_id: String, prompt: String },
    AgentStep { task_id: String, step: u32, action: String },
    AgentEnd { task_id: String, result: Result<String> },
    SessionStart { session_id: String },
    SessionEnd { session_id: String },
    PatternMatched { watcher_id: String, task_id: String, match: String },
}

// Hook registration in AppState
pub struct MonitorState {
    pub watcher_registry: Arc<RwLock<WatcherRegistry>>,
    pub hook_emitter: Arc<HookEmitter>,
    pub event_log: Arc<Mutex<Vec<MonitorEvent>>>,
}
```

### Implementation Steps

#### Phase 1: Core Infrastructure (Day 1)

**1.1 Create monitoring module**
```
core/router/src/monitoring/
├── mod.rs
├── models.rs        # WatchPattern, MonitorEvent, etc.
├── watcher.rs       # WatcherRegistry implementation
├── hooks.rs         # Hook system
├── notifier.rs      # Notification dispatcher
└── api.rs           # API endpoints
```

**1.2 Add database migration**
```sql
-- 026_monitoring.sql
CREATE TABLE watch_patterns (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    pattern TEXT NOT NULL,          -- Regex pattern
    watch_scope TEXT NOT NULL,      -- JSON: All | Project | TaskIds
    notify_on TEXT NOT NULL,        -- Match | Completion | Error | Timeout | Threshold
    notification_mode TEXT NOT NULL DEFAULT 'result',
    threshold_count INTEGER DEFAULT 1,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE monitor_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    task_id TEXT,
    session_id TEXT,
    payload TEXT NOT NULL,          -- JSON event data
    matched_watcher_id TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_monitor_events_task ON monitor_events(task_id);
CREATE INDEX idx_monitor_events_type ON monitor_events(event_type);
CREATE INDEX idx_monitor_events_created ON monitor_events(created_at DESC);
```

**1.3 Integrate hooks into existing workers**
```rust
// In deep_task_worker.rs - emit hooks
async fn run_agent_loop(&self, task: &mut Task) -> Result<AgentResult> {
    self.monitor.emit(MonitorEvent::AgentStart {
        task_id: task.id.clone(),
        prompt: task.prompt.clone(),
    }).await;

    for step in 0..task.max_steps {
        self.monitor.emit(MonitorEvent::AgentStep {
            task_id: task.id.clone(),
            step,
            action: format!("Executing step {}", step),
        }).await;

        // ... existing logic

        if let Some(watcher_match) = self.check_watcher_patterns(&output) {
            self.monitor.handle_match(watcher_match).await;
        }
    }

    self.monitor.emit(MonitorEvent::AgentEnd {
        task_id: task.id.clone(),
        result: Ok(output),
    }).await;
}
```

#### Phase 2: Notification System (Day 2)

**2.1 Notification dispatcher**
```rust
pub enum NotificationChannel {
    WebSocket,     // Real-time push to UI
    SSE,           // Server-Sent Events stream
    Webhook(String), // HTTP webhook URL
    Telegram { bot_token: String, chat_id: String },
    Email { smtp_config: SmtpConfig, recipients: Vec<String> },
}

pub struct NotificationDispatcher {
    channels: Vec<NotificationChannel>,
}

impl NotificationDispatcher {
    pub async fn send(&self, notification: Notification) -> Result<()> {
        let mode = notification.mode;
        
        for channel in &self.channels {
            match channel {
                NotificationChannel::WebSocket => {
                    if matches!(mode, NotifyMode::All | NotifyMode::Result) {
                        self.send_ws(notification).await?;
                    }
                }
                NotificationChannel::Webhook(url) => {
                    self.send_webhook(url, notification).await?;
                }
                // ... other channels
            }
        }
        Ok(())
    }
}
```

**2.2 WebSocket integration**
```rust
// Emit to connected WebSocket clients
pub async fn broadcast_notification(&self, notification: Notification) {
    let msg = serde_json::to_string(&notification).unwrap();
    for (client_id, sender) in &self.clients.read().await {
        if sender.send(Ok(msg.clone())).await.is_err() {
            // Client disconnected
        }
    }
}
```

#### Phase 3: UI Components (Day 3)

**3.1 Monitoring Dashboard**
```
ui/src/components/monitoring/
├── MonitoringDashboard.tsx    # Main monitoring view
├── WatcherList.tsx            # List of watch patterns
├── WatcherEditor.tsx          # Create/edit watch patterns
├── EventLog.tsx               # Real-time event log
├── NotificationSettings.tsx   # Configure notifications
└── AlertBanner.tsx            # Alert notifications in UI
```

**3.2 Real-time event stream**
```typescript
// useMonitorEvents.ts
export function useMonitorEvents() {
  const [events, setEvents] = useState<MonitorEvent[]>([]);
  
  useEffect(() => {
    const eventSource = new EventSource('/api/v1/monitor/events');
    
    eventSource.onmessage = (e) => {
      const event = JSON.parse(e.data);
      setEvents(prev => [event, ...prev].slice(0, 100));
    };
    
    return () => eventSource.close();
  }, []);
  
  return { events };
}
```

**3.3 Watcher creation UI**
```typescript
interface WatcherEditorProps {
  watcher?: WatchPattern;
  onSave: (watcher: WatchPattern) => void;
}

// Fields:
// - Name (text input)
// - Pattern (regex with validation)
// - Scope (dropdown: All / Project / Specific Tasks)
// - Notify On (multi-select: Match / Completion / Error / Timeout)
// - Mode (radio: All / Result / Error / Off)
// - Enable immediately (checkbox)
```

### File Structure

```
core/router/src/
├── monitoring/
│   ├── mod.rs
│   ├── models.rs           # 150 lines
│   ├── watcher.rs          # 200 lines
│   ├── hooks.rs            # 150 lines
│   ├── notifier.rs         # 200 lines
│   └── api.rs              # 300 lines
├── workers/
│   └── deep_task_worker.rs  # Add hook emissions
├── lib.rs                   # Add monitoring module
└── main.rs                  # Initialize MonitorState

core/memory/migrations/
└── 026_monitoring.sql

ui/src/components/monitoring/
├── MonitoringDashboard.tsx
├── WatcherList.tsx
├── WatcherEditor.tsx
├── EventLog.tsx
└── NotificationSettings.tsx

ui/src/hooks/
└── useMonitorEvents.ts
```

### Effort Estimate

| Component | Files | Complexity | Time |
|-----------|-------|------------|------|
| Backend models & logic | 5 | Medium | 4h |
| API endpoints | 1 | Medium | 2h |
| Worker integration | 1 | Low | 1h |
| Database migration | 1 | Low | 30m |
| UI components | 5 | Medium | 6h |
| WebSocket/SSE | 2 | Medium | 2h |
| **Total** | **15** | - | **~15.5h** |

---

## Feature 2: Vigilant Mode (Alert Monitoring)

### Overview

Proactive monitoring system with local dashboard and configurable alerts. Watches for agent issues (loops, crashes, resource exhaustion) and notifies immediately.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Vigilant Mode Service                       │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐   │
│  │  Alert      │  │  Threshold  │  │  Alert                  │   │
│  │  Rules      │  │  Monitor    │  │  Dispatcher              │   │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                    Alert Dashboard (Local)                       │
│  http://localhost:3000/vigilant                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Alert Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    // Execution alerts
    InfiniteLoop { task_id: String, iterations: u32 },
    NoProgress { task_id: String, steps: u32 },
    ResourceExhaustion { task_id: String, resource: String },
    TimeoutWarning { task_id: String, remaining: u32 },
    
    // Pattern alerts (from monitoring)
    PatternDetected { pattern: String, task_id: String },
    ErrorSpike { task_id: String, error_count: u32 },
    
    // System alerts
    HighMemoryUsage { percentage: u8 },
    LLMUnavailable,
    ExecutionPoolExhausted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub alert_type: AlertType,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub cooldown_secs: u32,
    pub actions: Vec<AlertAction>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    Immediate,
    After(u32),           // After N occurrences
    Threshold { count: u32, window_secs: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertAction {
    Log,
    Notify,
    PauseTask,
    CancelTask,
    Webhook(String),
    ExecuteCommand(String),
}
```

### Built-in Alert Rules

```rust
pub const DEFAULT_ALERT_RULES: &[AlertRule] = &[
    AlertRule {
        id: "loop-detection",
        name: "Infinite Loop Detection",
        alert_type: AlertType::InfiniteLoop { task_id: String::new(), iterations: 0 },
        condition: AlertCondition::Threshold { count: 100, window_secs: 60 },
        severity: AlertSeverity::Critical,
        cooldown_secs: 300,
        actions: vec![AlertAction::Notify, AlertAction::CancelTask],
        enabled: true,
    },
    AlertRule {
        id: "no-progress",
        name: "No Progress Warning",
        alert_type: AlertType::NoProgress { task_id: String::new(), steps: 0 },
        condition: AlertCondition::After(10),
        severity: AlertSeverity::Warning,
        cooldown_secs: 60,
        actions: vec![AlertAction::Notify],
        enabled: true,
    },
    AlertRule {
        id: "timeout-warning",
        name: "Timeout Warning",
        alert_type: AlertType::TimeoutWarning { task_id: String::new(), remaining: 0 },
        condition: AlertCondition::Immediate,
        severity: AlertSeverity::Warning,
        cooldown_secs: 0,
        actions: vec![AlertAction::Notify],
        enabled: true,
    },
];
```

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/vigilant/rules` | List alert rules |
| POST | `/api/v1/vigilant/rules` | Create alert rule |
| GET | `/api/v1/vigilant/rules/:id` | Get rule details |
| PUT | `/api/v1/vigilant/rules/:id` | Update rule |
| DELETE | `/api/v1/vigilant/rules/:id` | Delete rule |
| GET | `/api/v1/vigilant/alerts` | List active alerts |
| POST | `/api/v1/vigilant/alerts/:id/acknowledge` | Acknowledge alert |
| POST | `/api/v1/vigilant/alerts/:id/dismiss` | Dismiss alert |
| GET | `/api/v1/vigilant/alerts/history` | Alert history |
| GET | `/api/v1/vigilant/stats` | Alert statistics |

### Local Dashboard

```typescript
// ui/src/components/vigilant/
├── VigilantDashboard.tsx      # Main dashboard
├── AlertRulesList.tsx         # Manage rules
├── AlertRuleEditor.tsx        # Create/edit rules
├── ActiveAlerts.tsx           # Current alerts with severity badges
├── AlertHistory.tsx           # Past alerts with filters
├── AlertDetail.tsx            # Single alert details
└── VigilantSettings.tsx       # Global vigilant mode settings

// Access: http://localhost:3000/vigilant
// Or tab in Settings: Settings → Vigilant tab
```

### UI Design

```
┌─────────────────────────────────────────────────────────────────┐
│  ⚠️ VIGILANT MODE                              [Settings] [?]   │
├─────────────────────────────────────────────────────────────────┤
│  Status: ● Active    Alerts: 3    Last 24h: 47                  │
├───────────────────────────────┬─────────────────────────────────┤
│  ACTIVE ALERTS                │  ALERT RULES                    │
│  ┌─────────────────────────┐  │  ┌────────────────────────────┐ │
│  │ 🔴 CRITICAL              │  │  │ Infinite Loop Detection   │ │
│  │ Task abc123 looping      │  │  │ [Edit] [Toggle]           │ │
│  │ 100+ iterations in 60s   │  │  ├────────────────────────────┤ │
│  │ [Pause] [Cancel] [×]    │  │  │ No Progress Warning        │ │
│  └─────────────────────────┘  │  │  [Edit] [Toggle]           │ │
│  ┌─────────────────────────┐  │  ├────────────────────────────┤ │
│  │ 🟡 WARNING                │  │  │ Timeout Warning            │ │
│  │ Task def456 no progress  │  │  │ [Edit] [Toggle]            │ │
│  │ 10 steps without output  │  │  └────────────────────────────┘ │
│  │ [Dismiss] [×]            │  │                                 │
│  └─────────────────────────┘  │  [+ Add Rule]                   │
└───────────────────────────────┴─────────────────────────────────┘
```

### Implementation Steps

#### Phase 1: Core Alert System (Day 1)

**1.1 Create vigilant module**
```
core/router/src/vigilant/
├── mod.rs
├── models.rs        # AlertRule, Alert, AlertType, AlertAction
├── rules.rs         # Rule engine and matching
├── monitor.rs       # Threshold monitoring
├── dispatcher.rs    # Alert action execution
└── api.rs           # API endpoints
```

**1.2 Add database migration**
```sql
-- 027_vigilant.sql
CREATE TABLE alert_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    alert_type TEXT NOT NULL,       -- JSON
    condition TEXT NOT NULL,        -- JSON
    severity TEXT NOT NULL,
    cooldown_secs INTEGER NOT NULL DEFAULT 0,
    actions TEXT NOT NULL,          -- JSON array
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE alerts (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL,
    alert_type TEXT NOT NULL,       -- JSON
    severity TEXT NOT NULL,
    task_id TEXT,
    message TEXT NOT NULL,
    payload TEXT,                    -- JSON additional data
    status TEXT NOT NULL DEFAULT 'active',  -- active | acknowledged | dismissed
    acknowledged_at TEXT,
    acknowledged_by TEXT,
    created_at TEXT NOT NULL,
    resolved_at TEXT,
    FOREIGN KEY (rule_id) REFERENCES alert_rules(id)
);

CREATE TABLE alert_history (
    id TEXT PRIMARY KEY,
    alert_id TEXT NOT NULL,
    action TEXT NOT NULL,           -- created | acknowledged | dismissed | resolved
    performed_by TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_alerts_status ON alerts(status);
CREATE INDEX idx_alerts_created ON alerts(created_at DESC);
CREATE INDEX idx_alerts_task ON alerts(task_id);
```

**1.3 Integrate with execution pattern detection**
```rust
// Connect to existing execution_pattern_repo
// When pattern detected → create alert
pub async fn handle_pattern_detected(&self, pattern: &ExecutionPattern) {
    let alert = Alert {
        id: Uuid::new_v4().to_string(),
        rule_id: "pattern-detection",
        alert_type: AlertType::PatternDetected {
            pattern: pattern.pattern_type.clone(),
            task_id: pattern.task_id.clone(),
        },
        severity: pattern.severity.into(),
        task_id: Some(pattern.task_id.clone()),
        message: format!("Execution pattern detected: {}", pattern.pattern_type),
        payload: Some(serde_json::to_string(pattern).unwrap()),
        status: AlertStatus::Active,
        created_at: Utc::now(),
        ..Default::default()
    };
    
    self.create_alert(alert).await?;
}
```

#### Phase 2: Dashboard & UI (Day 2)

**2.1 Dashboard components**
- Main dashboard with active alerts
- Alert rules management
- History view with filters
- Real-time updates via WebSocket

**2.2 Integration with existing patterns**
```typescript
// Reuse existing anomaly detection patterns from v1.4.0
// execution_pattern_repo already detects:
// - FileCreationBurst
// - ToolCallLoop
// - NoSideEffects

// Vigilant mode extends with proactive alerts
```

### File Structure

```
core/router/src/
├── vigilant/
│   ├── mod.rs
│   ├── models.rs           # 200 lines
│   ├── rules.rs            # 250 lines
│   ├── monitor.rs          # 150 lines
│   ├── dispatcher.rs       # 200 lines
│   └── api.rs              # 250 lines
├── workers/
│   └── deep_task_worker.rs  # Add vigilant checks
├── lib.rs
└── main.rs

core/memory/migrations/
└── 027_vigilant.sql

ui/src/components/vigilant/
├── VigilantDashboard.tsx
├── AlertRulesList.tsx
├── AlertRuleEditor.tsx
├── ActiveAlerts.tsx
├── AlertHistory.tsx
└── VigilantSettings.tsx
```

### Effort Estimate

| Component | Files | Complexity | Time |
|-----------|-------|------------|------|
| Backend models & logic | 5 | Medium | 5h |
| API endpoints | 1 | Medium | 2h |
| Worker integration | 1 | Low | 1h |
| Database migration | 1 | Low | 30m |
| UI dashboard | 6 | Medium | 7h |
| WebSocket integration | 2 | Medium | 2h |
| **Total** | **16** | - | **~17.5h** |

---

## Integration: Unified Monitoring

Both features share a common event bus:

```rust
// core/router/src/observability.rs

pub struct ObservabilityState {
    pub monitoring: MonitorState,
    pub vigilant: VigilantState,
    pub event_bus: Arc<EventBus>,
}

pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<String, Vec<Box<dyn EventHandler>>>>>,
}

impl EventBus {
    pub async fn publish(&self, event: ObservableEvent) {
        // Route to monitoring watchers
        self.monitoring.check_watchers(&event).await;
        
        // Route to vigilant alerts
        self.vigilant.check_rules(&event).await;
        
        // Broadcast to UI via WebSocket
        self.broadcast_to_ui(&event).await;
    }
}
```

### Shared Event Types

```rust
#[derive(Debug, Clone)]
pub enum ObservableEvent {
    // Agent events
    TaskCreated(Task),
    TaskStarted { task_id: String },
    TaskStep { task_id: String, step: u32 },
    TaskCompleted { task_id: String, result: String },
    TaskFailed { task_id: String, error: String },
    TaskCancelled { task_id: String },
    
    // Pattern events
    PatternDetected(ExecutionPattern),
    AnomalyDetected(Anomaly),
    
    // System events
    LLMRequest { duration_ms: u64, success: bool },
    ResourceUsage { cpu: f32, memory: f32 },
}
```

---

## Rollout Strategy

### Phase A: Background Process Monitoring (Week 1)
- [ ] Core infrastructure
- [ ] Watch patterns API
- [ ] Hook system integration
- [ ] Basic UI
- [ ] WebSocket notifications

### Phase B: Vigilant Mode (Week 2)
- [ ] Alert rules engine
- [ ] Alert dispatch system
- [ ] Dashboard UI
- [ ] Integration with patterns

### Phase C: Integration & Polish (Week 3)
- [ ] Unified event bus
- [ ] Shared WebSocket stream
- [ ] Notification preferences
- [ ] Performance optimization

---

## Dependencies

| Feature | Depends On | Notes |
|---------|------------|-------|
| Background Monitoring | WebSocket/SSE | Reuse existing streaming |
| Vigilant Mode | Execution Patterns (v1.4.0) | Extend existing detection |
| Unified Dashboard | Both above | Phase C |

---

## Testing Plan

```rust
#[cfg(test)]
mod monitoring_tests {
    #[test]
    fn test_watch_pattern_matching() {
        let pattern = WatchPattern {
            pattern: Regex::new(r"error|failed|exception").unwrap(),
            notify_on: NotifyOn::Match,
            ..Default::default()
        };
        
        assert!(pattern.matches("Operation failed"));
        assert!(!pattern.matches("Success"));
    }
    
    #[tokio::test]
    async fn test_hook_emission() {
        // Test hook fires and reaches subscribers
    }
}

#[cfg(test)]
mod vigilant_tests {
    #[test]
    fn test_alert_rule_evaluation() {
        let rule = AlertRule {
            condition: AlertCondition::Threshold { count: 5, window_secs: 60 },
            severity: AlertSeverity::Warning,
            actions: vec![AlertAction::Notify],
            ..Default::default()
        };
        
        // Should not alert on 4 occurrences
        assert!(!rule.should_fire(4));
        
        // Should alert on 5th occurrence
        assert!(rule.should_fire(5));
    }
}
```

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Watch pattern creation | < 100ms |
| Alert latency | < 500ms from event |
| Dashboard load time | < 2s |
| WebSocket reconnection | < 1s |
| False positive rate | < 5% |

---

## Future Enhancements

1. **ML-based anomaly detection** - Train model on historical data
2. **A/B testing for alert thresholds** - Auto-tune based on feedback
3. **Integration with PagerDuty/OpsGenie** - Enterprise alerting
4. **Predictive alerts** - Warn before issues occur
5. **Audit log export** - Compliance requirements

---

## References

- Hermes Agent v0.9.0 Release Notes
- Hermes Agent Vigilant Mode Documentation
- Hermes Agent Background Process Monitoring
- APEX Execution Patterns (v1.4.0)
- APEX WebSocket Streaming (v1.7.0)
