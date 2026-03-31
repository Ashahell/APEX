# OpenClaw to APEX Crosswalk (Complete)

Owner: Sisyphus AI Agent
Last Updated: 2026-03-31

Purpose
- Map OpenClaw primitives to APEX equivalents with actual implementation status.
- Provide evidence links and parity scores for each primitive.

## Implementation Status

| OpenClaw Primitive | APEX Equivalent | Status | Evidence | Parity Score |
|---|---|---|---|---|
| Multi-channel surface | Streaming surface (Hands, MCP, Task) | ✅ Complete | `streaming.rs`, `streaming_types.rs` | 9/10 |
| Streaming thoughts surface | Streaming UI events with partial SSE | ✅ Complete | `streaming_mcp_tinysse.rs`, `StreamingDashboard.tsx` | 9/10 |
| Memory surface | Hermes-style bounded memory | ✅ Complete | `bounded_memory.rs`, `BoundedMemory.tsx` | 10/10 |
| Streaming state surface | StreamState with active/history tracking | ✅ Complete | `streaming_types.rs` (SessionStart, SessionEnd) | 9/10 |
| Telemetry surface | Per-endpoint latency + error tracking | ✅ Complete | `metrics.rs`, `telemetry_middleware.rs` | 9/10 |
| Telemetry enrichment | Performance metrics (avg duration, events/sec) | ✅ Complete | `streaming_types.rs` (PerformanceMetrics) | 9/10 |
| Governance surface | T0-T3 tiers, TOTP, HMAC auth | ✅ Complete | `auth.rs`, `totp.rs`, `governance.rs` | 10/10 |
| Memory visibility | 6-tab Memory UI (search, TTL, consolidation) | ✅ Complete | `BoundedMemory.tsx`, `memory_ttl_api.rs` | 10/10 |
| Event correlation | Streaming event types with correlation IDs | ✅ Complete | `streaming_types.rs` (11 event types) | 8/10 |
| Observability dashboards | Monitoring Dashboard + Streaming Dashboard | ✅ Complete | `MonitoringDashboard.tsx`, `StreamingDashboard.tsx` | 9/10 |
| Tooling parity | MCP tool registry + discovery | ✅ Complete | `mcp.rs` (enriched endpoints) | 9/10 |
| Security posture | Injection detection, replay protection, config validation | ✅ Complete | `security_integration.rs` (40 tests) | 10/10 |
| UI parity | 4 themes (modern, amiga, agentzero, high-contrast) | ✅ Complete | `useTheme.tsx`, `high-contrast.ts` | 9/10 |
| Memory indexing | Hybrid search (BM25 + embeddings) | ✅ Complete | `session_search.rs`, `searchMemory()` | 9/10 |
| Plugin ecosystem | Plugin signing + Skills Hub marketplace | ✅ Complete | `skill_signer.rs`, `hub_client.rs` | 9/10 |

## Overall Parity Score: 9.2/10

### Notes
- APEX exceeds OpenClaw in security posture (T0-T3 tiers, injection detection)
- APEX matches OpenClaw in streaming and memory capabilities
- APEX has stronger theming system (4 themes vs OpenClaw's default)
- Minor gap in event correlation (could add more correlation IDs)
