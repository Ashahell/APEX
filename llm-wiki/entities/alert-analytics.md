# Alert Analytics - Historical Trend Analysis

> **Version:** v1.9.0  
> **Entity Type:** Feature Component

---

## Overview

Alert Analytics provides historical trend analysis for alert data, including hourly buckets, severity breakdowns, and average response times.

---

## Data Models

### AlertAnalytics

```rust
pub struct AlertAnalytics {
    pub total_alerts: u32,
    pub by_severity: HashMap<String, u32>,
    pub by_status: HashMap<String, u32>,
    pub by_rule: HashMap<String, u32>,
    pub avg_ack_time_secs: f64,
    pub avg_resolve_time_secs: f64,
    pub top_rules: Vec<(String, u32)>,
    pub hourly_buckets: Vec<HourlyBucket>,
}
```

### HourlyBucket

```rust
pub struct HourlyBucket {
    pub hour: String,      // ISO timestamp (hour precision)
    pub count: u32,        // Total alerts in hour
    pub critical: u32,     // Critical alerts
    pub warning: u32,      // Warning alerts
    pub info: u32,         // Info alerts
}
```

---

## Analytics Calculation

### Time Window

```rust
pub async fn get_analytics(&self, hours: u32) -> AlertAnalytics {
    // Get alerts from last `hours` hours
    let cutoff = Utc::now() - Duration::hours(hours as i64);
    let alerts = self.alerts.values()
        .filter(|a| a.created_at > cutoff)
        .collect::<Vec<_>>();
    
    // Calculate aggregates...
}
```

### Aggregations

1. **Count by Severity**: Group alerts by `AlertSeverity` enum
2. **Count by Status**: Group alerts by `AlertStatus` enum
3. **Top Rules**: Sort rules by alert count, take top 5
4. **Average Times**: Calculate mean time to acknowledge/resolve

---

## API Usage

### Request

```http
GET /api/v1/vigilant/analytics?hours=24
```

### Response

```json
{
  "analytics": {
    "total_alerts": 42,
    "by_severity": {
      "Critical": 5,
      "Warning": 15,
      "Info": 22
    },
    "by_status": {
      "Active": 3,
      "Acknowledged": 12,
      "Resolved": 27
    },
    "avg_ack_time_secs": 180.5,
    "avg_resolve_time_secs": 900.2,
    "top_rules": [
      ["High Error Rate", 15],
      ["Tool Loop Detection", 8],
      ["Timeout Warning", 5]
    ],
    "hourly_buckets": [
      {
        "hour": "2026-04-16T10:00:00Z",
        "count": 5,
        "critical": 1,
        "warning": 2,
        "info": 2
      }
    ]
  },
  "period_hours": 24
}
```

---

## UI Charts

### Hourly Trend Chart

```
Count
  5 ┤    █
  4 ┤    █  █
  3 ┤    █  █  █
  2 ┤ █  █  █  █  █
  1 ┤ █  █  █  █  █  █
  0 ┼─────────────────────
    10:00 11:00 12:00 13:00
         Time (hourly buckets)
```

Rendered as vertical bar chart with:
- Total count (cyan bar)
- Critical alerts (red overlay)
- Hover tooltip with exact counts

---

## Time Range Options

| Option | Hours | Use Case |
|--------|-------|----------|
| 6 hours | 6 | Real-time monitoring |
| 24 hours | 24 | Daily review |
| 48 hours | 48 | Weekend analysis |
| 7 days | 168 | Weekly trends |

---

## Performance Considerations

- Analytics queries are O(n) where n = alerts in time window
- Index on `created_at` for efficient time filtering
- Hourly buckets computed at query time (not pre-aggregated)

---

## Related Documentation

- [vigilant-mode.md](vigilant-mode.md)
- [death-spiral-detection.md](death-spiral-detection.md)
