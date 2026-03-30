# AgentZero to APEX Crosswalk (Template)

Owner: [Name]
Last Updated: [Date]

Purpose
- Map AgentZero UI/UX patterns and agent loop concepts to APEX equivalents.
- Provide data contracts, ownership, and notes for parity work.

- Agent loop visuals | Agent loop surface and plan/act/observe | Data: AgentLoopPayload { plan, act, observe, timestamp } | UI/Streaming surface owner
- Tool generation UI | Dynamic tool generation surface | Data: ToolMeta, GenerationParams | Tooling Surface Owner
- Memory viewer integration | Hermes-like memory surface | Data: MemoryStore interface (get/set/delete) | Hermes memory owner
- Command Center UI | Governance and control surface | Data: GovernanceConfig | UI/UX Owner
 - Theming | UI Theme surface (APEX themes) | Data: ThemeConfig | UI/Theme Owner
- Governance surface parity | Governance surface in APEX | Data: GovernanceContracts | Governance Owner
- Observability surface parity | Observability dashboards | Data: ObservabilityPlan | Ops Owner

Notes
- Maintain alignment with APEX's security model and phase gates.
