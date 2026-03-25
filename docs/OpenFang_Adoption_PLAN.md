# OpenFang Adoption Plan for APEX

Date: 2026-03-25
Status: Draft / Executable Plan

Executive summary
- This document outlines a pragmatic, executable plan to adopt the OpenFang feature set into the current APEX implementation. The plan prioritizes Hands (autonomous capability bundles), memory/embeddings, a minimal MCP tool-discovery protocol, security hardening, and a scalable API/streaming surface. The goal is to deliver a measurable increase in autonomy, security, and extensibility while maintaining safe incremental changes to avoid disruption.

Scope and guiding principles
- Start with a small, auditable MVP (Hands-based Computer Use) and progressively broaden to MCP, Memory, and additional Hands.
- Use a phased rollout with strong governance, well-defined gates, and a ready-to-review documentation set.
- Preserve security-first posture by applying a 16-layer security model incrementally.
- Build for testability: unit, integration, and MVP-style end-to-end tests early and often.

Strategic mapping: OpenFang features to APEX components
- Hands => APEX Hands-like capability packaging within Hermes/Skills framework
- MCP => Cross-component tool discovery, routing, and orchestration interface
- Memory/Embeddings => Hermes-like memory stores and embedding indices for retrieval
- Channel Adapters => UI and API-driven event streams, cross-channel communication
- Security => Merkle audit, prompt-safety, gates, sandboxing, and secure execution
- API/Streaming => Consistent REST/WS streaming for task progress
- Marketplace concepts => internal governance, signing and versioning for Hands/Skills

Phases and milestones (high-level)
- Phase 0: Governance and scoping (2 weeks)
- Phase 1: Hands MVP + MCP scaffolding (4–6 weeks)
- Phase 2: Memory, embeddings, API expansion (4–6 weeks)
- Phase 3: Security hardening and streaming (4–6 weeks)
- Phase 4: Production readiness, rollout, and governance (2–4 weeks)

Phase 0: Governance and scoping
- Define adoption goals and success metrics (reliability, speed, autonomy, security)
- Establish OpenFang Adoption Steering Committee and ownership model
- Create a shared risk and security plan (policy, threat model, mitigations)
- Create a lightweight migration plan for project teams to map current flows to OpenFang patterns

Phase 1: Hands MVP + MCP scaffolding (Weeks 1–6)
- Create first Hand: Computer Use MVP (Hand.toml-like manifest, System Prompt, SKILL.md) and a minimal Hand runner
- Build a minimal Hand runner in the APEX kernel that can invoke a Hand, monitor lifecycle, and emit an execution trace
- Define a minimal MCP protocol for tool discovery (registry) and create a two-tool MVP registry (computer-use, browser)
- Implement a small Merkle-like audit trail scaffold for action histories
- Introduce gating for T0–T2 actions in the Hand flow (permissions and approvals)
- Document Hands lifecycle (start/stop/status) and add an internal Hands dashboard entry point
- Add a small memory hook to record Hand activity and outcomes (temporary index)
- Create a minimal REST endpoint: /api/v1/hands and /api/v1/hands/{name} for basic lifecycle queries
- Begin streaming of trace events (SSE) for Hands (initial surface)
- UI scaffolding: a simple Hands monitor panel to visualize status and progress

Phase 2: Memory, embeddings, API expansion (Weeks 7–12)
- Expand memory: add bounded episodic memory and a basic vector index; enable retrieval to inform planning
- Expand MCP: formalize a small registry for tools and hands, add versioning and health checks
- Security hardening: complete Merkle audit trail, implement prompt safety checks, and gate all risky actions
- API expansion: API surface for hands lifecycle, plan/observe endpoints, and streaming
- Streamlining: upgrade streaming to support live action reasoning steps and final state
- UI integration: connect Hands/Computer Use UI to stream events and status updates
- Add 1–2 additional adapters (Slack/Discord) for Hands delivery of updates

Phase 3: Production readiness (Weeks 13–20+)
- Introduce production-grade feature flags; gating for adoption enabling
- Expand test coverage: unit, integration, e2e for Hands, MCP, memory and security components
- Documentation completion: API docs, Hands docs, onboarding docs
- Rollout plan with cohorts and rollback strategy
- Establish ongoing governance cadence and a roadmap refresh process

Metrics and success criteria
- Autonomy: time-to-task completion improves by X% on MVP tasks vs baseline
- Reliability: tasks complete within timeout/budget constraints with fewer retries
- Security: audit trails are verifiable and tamper-evident; prompt-safety gates are enforced
- Observability: streaming and dashboards provide actionable insights into Hand runs
- Adoption: number of Hands deployed in the field increases over time

Riski assessment and mitigations
- Risk: scope creep; Mitigation: fixed quarterly scope windows and strict gating
- Risk: integration risk; Mitigation: incremental integration and high-coverage tests
- Risk: performance overhead; Mitigation: profiling and phased performance targets

Appendix: Deliverables per phase
- Phase 0: Charter, risk plan, migration map, governance docs
- Phase 1: Hand MVP, MCP scaffold, audit trail scaffolds, gating rules, in-process MVP API
- Phase 2: Memory/index, extended API, streaming, UI wiring, 2nd adapters
- Phase 3: Production readiness artifacts, runbooks, dashboards, controller-level monitoring
