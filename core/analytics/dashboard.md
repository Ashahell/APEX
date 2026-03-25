Analytics Dashboard Prototype (ACME MVP)

Overview
- Lightweight analytics surface to visualize adoption progress for the ACME MVP OpenFang adoption.
- Goal: provide a simple, testable blueprint for dashboards that can be wired to live data sources during later phases.

Metrics (core MVP set)
- Hands: started, running, completed, failed, total
- Security: gate_ok_count, gate_warning_count, gate_blocked_count, audit_events_count
- Memory: embeddings_count, memory_usage_mib (high level)
- API: requests_total, throughput_rps, error_rate
- Latency: avg_api_latency_ms

Data sources (initial)
- In-memory MVP stores and placeholder metrics (to be wired to real runtime signals later)
- UI components will consume this data via API or WebSocket streams

Data format example (JSON)
{
  "hands": {"started": 1, "running": 1, "completed": 0, "failed": 0, "total": 1},
  "security": {"gate_ok": 0, "gate_warning": 0, "gate_blocked": 0, "audit_events": 0},
  "memory": {"embeddings": 1, "memory_usage_mib": 32},
  "api": {"requests_total": 5, "throughput_rps": 2, "error_rate": 0},
  "latency": {"avg_api_latency_ms": 120}
}

Next steps
- Implement live data bindings to populate these metrics from runtime signals (Hands, MCP, memory, security events)
- Create a small UI widget or page to render these metrics in the existing frontend
- Expand to a fuller dashboard with filtering, time ranges, and per-hand drill-downs
