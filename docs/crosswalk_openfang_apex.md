# OpenFang to APEX Crosswalk (Complete)

Owner: Sisyphus AI Agent
Last Updated: 2026-03-31

Purpose
- Map OpenFang primitives to APEX equivalents focusing on telemetry, governance, and streaming parity.
- Provide implementation status, evidence links, and parity scores.

## Implementation Status

| OpenFang Primitive | APEX Equivalent | Status | Evidence | Parity Score |
|---|---|---|---|---|
| Hands bundles | Streaming surface (Hands, MCP, Task) | ✅ Complete | `streaming.rs`, `HandMonitor.tsx` | 9/10 |
| Memory/Embeddings | Hybrid search with embeddings | ✅ Complete | `session_search.rs`, embedding support | 10/10 |
| MCP tooling surface | MCP registry + discovery | ✅ Complete | `mcp.rs` (enriched endpoints) | 9/10 |
| Telemetry dashboards | Monitoring + Streaming dashboards | ✅ Complete | `MonitoringDashboard.tsx`, `StreamingDashboard.tsx` | 9/10 |
| Telemetry enrichment | Per-endpoint latency + error rates | ✅ Complete | `telemetry_middleware.rs`, `metrics.rs` | 9/10 |
| Orchestration surface | Task router + agent loop | ✅ Complete | `api/mod.rs`, `agent_loop.rs` | 10/10 |
| Governance surface | T0-T3 tiers + governance engine | ✅ Complete | `governance.rs`, `GovernanceControls.tsx` | 10/10 |
| MPC governance interface | Policy checks + immutable values | ✅ Complete | `/api/v1/governance/*` endpoints | 9/10 |
| Plugin ecosystem | Plugin signing + Skills Hub | ✅ Complete | `skill_signer.rs`, `hub_client.rs` | 9/10 |
| Security posture | Injection detection + replay protection | ✅ Complete | `security_integration.rs` (40 tests) | 10/10 |

## Overall Parity Score: 9.4/10

### Notes
- APEX matches OpenFang in telemetry and orchestration capabilities
- APEX exceeds OpenFang in security (injection detection, replay protection)
- APEX has stronger governance (T0-T3 tiers, TOTP verification)
- APEX has more comprehensive plugin ecosystem (signing, marketplace)
- Minor gap in Hands bundle visualization (could enhance HandMonitor)

 
