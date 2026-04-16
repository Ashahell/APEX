# Vigilant Mode - Alert Monitoring System

> **Version:** v1.9.0  
> **Entity Type:** Core Feature

---

## Overview

Vigilant Mode is an alert monitoring system that detects anomalies and automatically escalates unacknowledged alerts based on configurable time thresholds. It integrates with the Hermes-style background monitoring system.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    VigilantMode                          │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │ RuleEngine │  │ThresholdMonit│  │   Dispatcher   │  │
│  │             │  │     or       │  │               │  │
│  │ - list()    │  │ - track()    │  │ - dispatch()  │  │
│  │ - check()   │  │ - metrics    │  │ - acknowledge()│ │
│  │ - add()     │  │              │  │ - escalate()   │  │
│  └─────────────┘  └──────────────┘  └───────────────┘  │
│         │                 │                  │           │
│         └─────────────────┼──────────────────┘           │
│                           │                              │
│                    ┌──────┴───────┐                     │
│                    │  Alert Store  │                     │
│                    │   (RwLock)    │                     │
│                    └───────────────┘                     │
└─────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. RuleEngine

Manages alert rules with threshold configurations.

```rust
pub struct RuleEngine {
    rules: RwLock<Vec<AlertRule>>,
}

impl RuleEngine {
    pub async fn list(&self) -> Vec<AlertRule>;
    pub async fn check(&self, alert_type: &AlertType) -> Vec<Alert>;
    pub async fn add(&self, rule: AlertRule) -> Result<(), VigilantError>;
    pub async fn remove(&self, id: &str);
}
```

### 2. ThresholdMonitor

Tracks metrics and detects threshold violations.

```rust
pub struct ThresholdMonitor {
    metrics: RwLock<HashMap<String, TaskMetrics>>,
}

pub struct TaskMetrics {
    pub step_count: u32,
    pub error_count: u32,
    pub last_activity: DateTime<Utc>,
    pub action_history: Vec<ThresholdAction>,
}
```

### 3. Dispatcher

Handles alert dispatch, acknowledgment, and escalation.

```rust
impl AlertDispatcher {
    pub async fn dispatch(&self, alert: Alert) -> Result<(), VigilantError>;
    pub async fn acknowledge(&self, id: &str, by: Option<String>) -> Result<Alert, VigilantError>;
    pub async fn escalate(&self, id: &str) -> Result<(), VigilantError>;
    pub async fn get_analytics(&self, hours: u32) -> AlertAnalytics;
    pub async fn get_pending_escalation(&self, wait_secs: u32) -> Vec<Alert>;
}
```

---

## Alert Types

| Type | Description | Parameters |
|------|-------------|-----------|
| InfiniteLoop | Same tool called repeatedly | `iterations: u32` |
| NoProgress | No state change after N steps | `steps: u32` |
| ResourceExhaustion | Resource limits exceeded | `resource: String` |
| TimeoutWarning | Task approaching timeout | `remaining_secs: u32` |
| ErrorSpike | Multiple errors detected | `error_count: u32` |
| PatternDetected | Execution pattern match | `pattern: String` |

---

## Alert Actions

```rust
pub enum AlertAction {
    Log,                    // Log to audit trail
    Notify,                 // Send notification
    PauseTask,             // Pause task execution
    CancelTask,            // Cancel task
    Webhook { url: String }, // HTTP webhook
    ExecuteCommand { command: String }, // Shell command
    Email { to: String, subject: String }, // Email alert
}
```

---

## Severity Levels

| Level | Color | Description |
|-------|-------|-------------|
| Info | Blue | Informational alerts |
| Warning | Yellow | Attention required |
| Critical | Red | Immediate action needed |

---

## Escalation System

### EscalationConfig

```rust
pub struct EscalationConfig {
    pub enabled: bool,
    pub max_level: u32,
    pub levels: Vec<EscalationLevel>,
    pub default_wait_secs: u32,
}

pub struct EscalationLevel {
    pub level: u32,
    pub wait_secs: u32,
    pub actions: Vec<AlertAction>,
}
```

### Default Escalation Chain

| Level | Wait Time | Actions |
|-------|----------|---------|
| 1 | 5 min | Notify + Email |
| 2 | 10 min | ExecuteCommand |
| 3 | Immediate | CancelTask |

---

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/vigilant/rules` | List alert rules |
| POST | `/api/v1/vigilant/rules` | Create rule |
| GET | `/api/v1/vigilant/alerts` | List alerts |
| POST | `/api/v1/vigilant/alerts/:id/acknowledge` | Acknowledge alert |
| GET | `/api/v1/vigilant/escalation/pending` | Get pending escalations |
| POST | `/api/v1/vigilant/escalation/process` | Process escalations |
| GET | `/api/v1/vigilant/analytics` | Get analytics |
| GET | `/api/v1/vigilant/patterns/suggestions` | Get pattern suggestions |
| POST | `/api/v1/vigilant/patterns/create-rule` | Create rule from pattern |

---

## State Management

```rust
pub struct VigilantState {
    pub rule_engine: Arc<RuleEngine>,
    pub threshold_monitor: Arc<ThresholdMonitor>,
    pub dispatcher: Arc<AlertDispatcher>,
}
```

Integrated into `AppState.vigilant_state` for API access.

---

## Related Documentation

- [alert-analytics.md](alert-analytics.md)
- [death-spiral-detection.md](death-spiral-detection.md)
- [monitoring-dashboard.md](monitoring-dashboard.md)
