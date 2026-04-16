# Death Spiral Detection - Pattern-Based Anomaly Detection

> **Version:** v1.9.0  
> **Entity Type:** Security Feature

---

## Overview

Death Spiral Detection identifies dangerous execution patterns (infinite loops, error cascades, resource exhaustion) and automatically suggests alert rules to prevent future occurrences.

---

## Pattern Types

### 1. Tool Call Loop (`tool_call_loop`)

**Detection:** Same tool called N times in succession without progress.

```rust
// Detection logic
if tool_call_count > threshold && no_state_change() {
    record_pattern(PatternType::ToolCallLoop {
        tool_name: tool.name.clone(),
        iterations: tool_call_count,
    });
}
```

**Severity:** Critical  
**Remediation:** Cancel task immediately, review tool selection logic

### 2. No Progress (`no_progress`)

**Detection:** No observable state change after N tool calls.

```rust
// Detection logic
if steps_since_last_change > threshold {
    record_pattern(PatternType::NoProgress {
        steps: steps_since_last_change,
        last_state_hash: current_hash,
    });
}
```

**Severity:** High  
**Remediation:** Check tool outputs, verify file writes

### 3. Error Cascade (`error_cascade`)

**Detection:** Multiple sequential errors without successful operations.

```rust
// Detection logic
if sequential_errors > threshold {
    record_pattern(PatternType::ErrorCascade {
        error_count: sequential_errors,
        error_types: recent_errors.clone(),
    });
}
```

**Severity:** Critical  
**Remediation:** Cancel task, review error logs

### 4. File Creation Burst (`file_creation_burst`)

**Detection:** Multiple files created in short succession.

```rust
// Detection logic
if file_creations_in_window > threshold {
    record_pattern(PatternType::FileCreationBurst {
        count: file_creations_in_window,
        directory: current_dir,
    });
}
```

**Severity:** High  
**Remediation:** Review generated files, implement limits

### 5. No Side Effects (`no_side_effects`)

**Detection:** Tool calls with no observable state changes.

```rust
// Detection logic
if empty_tool_outputs > threshold {
    record_pattern(PatternType::NoSideEffects {
        consecutive: empty_tool_outputs,
    });
}
```

**Severity:** Medium  
**Remediation:** Check tool outputs, verify writes

---

## Data Models

### ExecutionPattern

```rust
pub struct ExecutionPattern {
    pub id: String,
    pub task_id: String,
    pub pattern_type: String,      // e.g., "tool_call_loop"
    pub severity: String,           // "critical", "high", "medium"
    pub tool_calls: Option<String>, // JSON array of tool names
    pub file_ops: Option<String>,  // JSON array of file operations
    pub error_count: i32,
    pub details: Option<String>,   // JSON object with specifics
    pub detected_at: String,      // ISO timestamp
}
```

### PatternAlertTemplate

```rust
pub struct PatternAlertTemplate {
    pub id: String,
    pub pattern_type: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub remediation: String,
    pub created_at: String,
}
```

### DetectedPattern (Aggregated)

```rust
pub struct DetectedPattern {
    pub pattern_type: String,
    pub severity: String,
    pub occurrences: u32,
    pub last_occurrence: String,
    pub affected_tasks: Vec<String>,
}
```

---

## Auto-Rule Creation

### Suggestion Algorithm

```rust
pub fn to_rule_suggestion(&self) -> AlertRuleSuggestion {
    AlertRuleSuggestion {
        pattern_type: self.pattern_type.clone(),
        suggested_name: format!("Auto: {} Detection", self.pattern_type),
        suggested_severity: severity_from_string(&self.severity),
        suggested_actions: default_actions_for(&self.pattern_type),
        cooldown_secs: 300,
        confidence: min(self.occurrences * 10, 90), // Max 90%
        reason: format!("Detected {} times, affecting {} tasks", 
            self.occurrences, self.affected_tasks.len()),
    }
}
```

### Confidence Score

| Occurrences | Confidence |
|-------------|------------|
| 1 | 10% |
| 2 | 20% |
| 5 | 50% |
| 10 | 90% |
| 20+ | 90% (capped) |

---

## API Endpoints

### Get Pattern Suggestions

```http
GET /api/v1/vigilant/patterns/suggestions
```

Response:
```json
{
  "suggestions": [
    {
      "pattern_type": "tool_call_loop",
      "suggested_name": "Auto: tool_call_loop Detection",
      "suggested_severity": "Critical",
      "suggested_actions": [{"type": "Notify"}, {"type": "CancelTask"}],
      "cooldown_secs": 300,
      "confidence": 80,
      "reason": "Detected 8 times, affecting 5 tasks"
    }
  ],
  "total_patterns": 12
}
```

### Create Rule from Pattern

```http
POST /api/v1/vigilant/patterns/create-rule
Content-Type: application/json

{
  "pattern_type": "tool_call_loop",
  "name": "Tool Loop Detection",
  "severity": "Critical",
  "threshold": 10,
  "cooldown_secs": 300,
  "actions": [{"type": "Notify"}, {"type": "CancelTask"}]
}
```

---

## Integration with Vigilant Mode

```
┌──────────────────┐     ┌─────────────────┐
│ Execution Engine │────▶│ Pattern Detector │
└──────────────────┘     └────────┬────────┘
                                  │
                                  ▼
                         ┌─────────────────┐
                         │ Pattern Store   │
                         │ (execution_     │
                         │  pattern_repo)  │
                         └────────┬────────┘
                                  │
          ┌────────────────────────┼────────────────────────┐
          │                        │                        │
          ▼                        ▼                        ▼
   ┌─────────────┐        ┌─────────────┐        ┌─────────────┐
   │ Suggestions │        │ Alert       │        │ Analytics   │
   │ API         │        │ Dispatcher  │        │             │
   └─────────────┘        └─────────────┘        └─────────────┘
```

---

## Related Documentation

- [vigilant-mode.md](vigilant-mode.md)
- [alert-analytics.md](alert-analytics.md)
- [security-system.md](security-system.md)
