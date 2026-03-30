# OpenClaw to APEX Crosswalk (Template)

Owner: [Name]
Last Updated: [Date]

Purpose
- Map OpenClaw primitives to APEX equivalents focusing on parity across streaming, memory, and governance aspects.
- Provide data contracts, ownership, and notes.

OpenClaw Primitive | APEX Equivalent | Data Contracts / Interfaces | Ownership / Notes
- Multi-channel surface | Streaming surface (APEX) | Data: Streaming event envelope { type, timestamp, payload }, EventSource; UI consumes streaming endpoints | Core Router / Streaming surface owner
- Streaming thoughts surface | Streaming UI events | Data: Thought segments embedded in streaming events; types and timing standardized | Streaming/UI Owner
- Memory surface | Hermes-like bounded memory | Data: MemoryStore interface (get/set/delete), TTL semantics, indexing hooks | Hermes memory owner
- Streaming state surface | Streaming surface parity surface for live state | Data: StreamState { active, history }, Event hooks | Streaming surface owner
- Telemetry surface parity surface | Telemetry dashboards | Data: TelemetryPlan, Metrics | Telemetry Owner
- Telemetry enrichment surface | Telemetry enrichment surface | Data: TelemetrySignal, Latency | Telemetry Owner
- Governance surface parity | Governance surface in APEX | Data: GovernanceContracts | Governance Owner
- Memory visibility surface | Hermes memory visibility surface | Data: MemorySnapshot, TTL | Hermes memory owner
- Event correlation surface | Streaming UI event correlation surface | Data: CorrelationMap, EventId | Streaming surface owner
- Observability surface parity | Observability dashboards | Data: ObservabilityPlan | Ops Owner
- Tooling parity surface | Tooling parity surface | Data: ToolingConfig | Tooling Owner
- Security posture surface | Security posture parity | Data: SecurityPolicies | Security Owner
- UI parity surface | UI alignment surface | Data: UIConfig | UI/UX Owner
- Memory indexing surface | Hermes Memory Index surface | Data: MemoryIndex, EmbeddingIndex | Hermes indexer owner
- Observability surface parity | Observability dashboards | Data: ObservabilityPlan | Ops Owner
