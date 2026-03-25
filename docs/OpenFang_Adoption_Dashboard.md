OpenFang Adoption Dashboard (MVP)

Overview
- This dashboard presents executive and developer-friendly views into the OpenFang adoption progress within ACME MVP.

Key Metrics (MVP)
- Hands status: counts of hands started, running, completed
- Active tasks: currently executing tasks and their progress
- Gate status: aggregate status of security gates (OK / Warning / Blocked)
- Audit events: count of security/audit events logged
- Memory/Embedding signals: high-level indicators of embedding usage and memory pressure
- API health: basic health/throughput indicators for the MVP API surface

Data sources (MVP)
- In-memory stores and placeholders used for MVP (to be wired to real sources in subsequent patches)
- UI components will hook into endpoints and the memory layer as we evolve

Layout (idea)
- Summary banner with KPI cards
- Hands detail section with per-hand status and progress
- Security panel summarizing gate status and last events
- Logs and events feed for audit trails

Next steps
- Connect to actual MVP data sources as embedding/memory wiring solidifies
- Integrate with UI pages to render live data
- Expand dashboards with drill-down capabilities as adoption grows
