# OpenFang to APEX Crosswalk (Template)

Owner: [Name]
Last Updated: [Date]

Purpose
- Map OpenFang primitives to APEX equivalents focusing on telemetry, governance, and streaming parity.
- Provide data contracts, ownership, and notes.

OpenFang Primitive | APEX Equivalent | Data Contracts / Interfaces | Ownership / Notes
--- | --- | --- | ---
- Hands bundles | Streaming + Tools surface | Data: HandsSurface { streams, tool_invocation } | Surface Owner
- Memory/Embeddings | Hermes-style memory + embeddings surface | Data: MemoryIndex, EmbeddingsIndex | Hermes memory owner
- MCP tooling surface | MCP surface with tooling discovery | Data: ToolRegistry, DiscoveryQuery | MCP surface owner
- Telemetry dashboards | Telemetry surface + dashboards | Data: TelemetryPlan, Metrics | Telemetry Owner
- Telemetry enrichment surface | Telemetry enrichment surface | Data: TelemetrySignal, Latency | Telemetry Owner
- Orchestration surface parity | Orchestration surface | Data: OrchestrationConfig | Orchestration Owner
- Governance surface parity | Governance surface in APEX | Data: GovernanceContracts | Governance Owner
- MPC governance interface | Governance/Policy surface | Data: GovernancePolicy | Governance Owner
- Governance surface | Governance surface in APEX | Data: GovernanceContracts | Governance Owner
- Telemetry enrichment surface | Telemetry enrichment surface | Data: TelemetrySignal, Latency | Telemetry Owner
- MCP governance interface | Governance/Policy surface | Data: GovernancePolicy | Governance Owner
- Telemetry enrichment surface | Telemetry enrichment surface | Data: TelemetrySignal, Latency | Telemetry Owner
- MPC governance interface | Governance/Policy surface | Data: GovernancePolicy | Governance Owner
- Governance surface | Governance surface in APEX | Data: GovernanceContracts | Governance Owner

Notes
- Start with MVP surface; expand incrementally to avoid risk.

 
