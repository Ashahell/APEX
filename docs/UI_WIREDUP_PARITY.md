UI Wiring, Full Parity Plan, and Wiring Guidance for APEX Streaming (v1.x)

Overview
- This document defines the full UI wiring for APEX streaming MVP plus concrete parity goals versus OpenClaw, Agent Zero, Hermes, and OpenFang.
- It covers: data contracts, authentication strategies for browser-based SSE, UI component architecture, end-to-end flows, testing, rollout, and a 4-platform parity plan.

Goals
- Provide a wired, production-like UI path that consumes the four streaming endpoints: /stream/stats, /stream/hands/:task_id, /stream/mcp/:task_id, /stream/task/:task_id.
- Ensure authentication is browser-friendly (EventSource) and secure (no secret exposure in the UI).
- Establish UI components that render real-time Hands/MCP/Task streams, plus a Stats KPI panel and a ProcessGroup viewer.
- Define a parity roadmap to match the capabilities of OpenClaw, Agent Zero, Hermes, and OpenFang for streaming UX and adoption.

UI Contract & Data Contracts
- SSE Envelope (serialized as JSON) for all events:
  - type: one of [connected, heartbeat, hands, mcp, task, stats, error, disconnected]
  - timestamp: number (ms since UNIX epoch)
  - trace_id: string | null
  - payload: generic payload depending on event type

- Hands payloads: array of events with fields such as thought, action, target, etc.
- MCP payloads: events describing tool discovery, start, progress, results, errors.
- Task payloads: lifecycle steps, status, progress, and any outputs.
- Stats payloads: active_connections, total_connections, events, errors, and derived metrics.
- Heartbeat payloads: server_time, active_connections.

Authentication Strategy for UI
- Problem: EventSource cannot set custom headers. Therefore we must rely on a browser-friendly auth path.
-Recommended approaches (order of preference):
  1) Streaming URL with query-signed token
     - UI obtains a short-lived token via a back-end endpoint (e.g., /api/v1/streams/sign) after login.
     - Token encodes timestamp and signature; the SSE URL includes sig and ts: /stream/task/:task_id?sig=...&ts=...
     - Server validates query-based signature using the same shared secret via a fallback path in StreamingAuth (optional) or a dedicated query-parameter validator.
  2) Streaming proxy endpoint
     - A tiny server-side proxy signs each connection and attaches the required headers on the fly before streaming to the UI.
     - Keeps secrets on the server, never exposing in the JS client.
  3) Cookie-based session with signed cookie
     - The browser stores a short-lived cookie; SSE client uses same-origin cookies to authenticate.
  4) WebSocket fallback (if essential for UI) with token-based auth
     - If SSE proves insufficient, consider a WS path with a similar signing flow.

- Recommendation: Implement (1) first, and add (2) as an engineering safety bump. Avoid exposing shared secrets to clients.
- If you adopt (1), update streaming.rs to accept query params (sig and ts) as a fallback path for authentication, while preserving header-based auth for non-browser clients.

UI Architecture & Components (React)
- StreamingDashboard (top-level container)
  - Tabs: Stats, Hands, MCP, Task
  - KPIBar: shows live streaming metrics (active, total, throughput)
  - ProcessGroupView: collapsible timeline for current/active tasks and steps
  - StreamViewer: generic SSE viewer that subscribes to the selected stream and renders events in real-time, grouped by type
  - TaskDetailPane: shows per-task Hands/MCP/Task streams and a decision journal link
  - MemoryPanel: a lightweight memory viewer (Hermes-like) showing bounded memory state related to the streaming session
  - SessionSearchPanel: allow quick navigation to streams by task/session

- HandsPanel: shows LLM Hands events (thoughts, actions, etc.)
- McpPanel: shows MCP tool discovery, progress, and results
- TaskPanel: shows lifecycle events and current step
- StatsPanel: live metrics & heartbeat status
- Common UI patterns:
  - real-time updates via SSE
  - optimistic updates for UI skeletons where appropriate
  - keyboard shortcuts for quick navigation (Ctrl+1/2/3 for top-level sections)

Data Models (TypeScript)
- Interfaces (examples):
  - interface SSEEnvelope<T> { type: string; timestamp: number; trace_id?: string; payload: T }
  - interface HandsPayload { thought?: string; action?: string; target?: string; timestamp?: number; }
  - interface McpEventPayload { type: string; tool?: string; id?: string; input?: any; progress?: any; result?: any; error?: string }
  - interface TaskPayload { status: string; step: number; max_steps?: number; output?: any }
  - interface StatsPayload { active_connections: number; total_connections: number; events: { total: number; } ; errors: { total: number } }
  - type StreamEvent = SSEEnvelope<HandsPayload | McpEventPayload | TaskPayload | StatsPayload | HeartbeatPayload | string>
- Reuse existing Axios/Fetch utility types if available (use existing api layer for signing requests).

UI Wiring: End-to-End Flows & Interactions
- User logs in and navigates to Streaming Dashboard
- On Task creation, the UI opens the Task Streams across Hands/MCP/Task and displays heartbeat
- Hands stream feeds real-time “thoughts” and actions, showing a live narrative timeline
- MCP stream reveals tool discovery, progress, tool results, and possible failures
- Task stream shows lifecycle timeline and current state; Stats shows live cost, messages per second, etc.
- The UI integrates with the existing memory/session features to show relevant bound memory for the active task
- UI responds to disconnects with clear re-try guidance & heartbeat status in the KPI bar

Wire-up Plan (Phased)
- Phase 0 (Week 0-1): Add query-based auth fallback to streaming endpoints; implement a minimal UI wiring in code comments and a TS skeleton (StreamingDashboard.tsx).
- Phase 1 (Week 2-4): Build StreamingDashboard components wiring to EventSource or proxy; implement a mock backend for testing; add UI tests.
- Phase 2 (Week 5-8): Add real backend sign-in token API and implement a streaming proxy path; integrate with UI contract docs; finalize parity plan.
- Phase 3 (Week 9-12): E2E UI tests, performance tests, final docs, rollout plan, governance updates.

Prototypical UI Code Snippet (High-Level)
- Example React hook to subscribe to an SSE endpoint using signed URL (no headers on EventSource):
```ts
import { useEffect, useState } from 'react';

type SSEEnvelope<T> = { type: string; timestamp: number; trace_id?: string; payload: T };

function useStreaming(endpoint: string, signedUrl: string) {
  const [events, setEvents] = useState<SSEEnvelope<any>[]>([]);

  useEffect(() => {
    const es = new EventSource(`${endpoint}?sig=${signedUrl}`); // or a proper signed URL
    es.onmessage = (ev) => {
      try {
        const data = JSON.parse(ev.data) as SSEEnvelope<any>;
        setEvents((e) => [...e, data]);
      } catch (err) {
        // handle parsing error
      }
    };
    es.onerror = () => {
      es.close();
    };
    return () => es.close();
  }, [endpoint]);
  return events;
}
```

- This approach requires a signed URL or a proxy layer (as discussed) to handle auth for SSE in browsers.

Migration & Rollout
- Feature flags: enable Streaming UI wiring behind a feature flag in settings
- Migration steps: ensure streaming endpoints can fallback to header-based or query-based auth; ensure no breaking changes to existing non-UI clients
- Rollout plan: staged rollout with monitoring and a rollback path

Tests & QA
- UI tests: end-to-end tests for streaming UI (Playwright/Cypress)
- Accessibility tests for streaming panels
- UI performance tests to ensure streaming updates render within acceptable frames

Risks & Mitigations
- Risk: Browser SSE auth limitations
  Mitigation: Implement query-based auth fallback and streaming proxy as described
- Risk: Complexity of UI integration with streaming
  Mitigation: Start with a minimal, testable UI wiring and progressively enhance with end-to-end tests
- Risk: Security exposure of tokens
  Mitigation: Use server-side signing, short-lived tokens, and never expose secrets to the client

Deliverables
- docs/UI_WIREDUP_PARITY.md (this document)
- Streaming API changes (proxy/signature) if chosen
- Minimal TSX skeleton: StreamingDashboard.tsx (as a follow-up patch)
- UI integration tests plan

Appendix: Parity Roadmap mapping to platforms
- OpenClaw: Expand marketplace-like UI, plugin support for streaming visuals
- Agent Zero: Real-time stream consumption in UI; integrate with ProcessGroups UI
- Hermes: Tie streaming events to bounded memory/session memory flows; live memory indicators in UI
- OpenFang: Telemetry integration (Prometheus, dashboards) and SLO-oriented UI metrics

Appendix: Risks & Mitigations (quick references)
- SSE auth with browsers → implement query-based auth fallback or proxy
- Security across UI interactions → always use server-controlled signing
- Rollout gating → feature flag and quick rollback plan
- OpenFang: Telemetry ready; dashboards for streaming latency, throughput, and reliability; SLO-oriented metrics.
+ OpenFang: Telemetry ready; dashboards for streaming latency, throughput, and reliability; SLO-oriented metrics.

- Phase 0 (Week 0-1): Add query-based auth fallback to streaming endpoints; implement a minimal UI wiring in code comments and a TS skeleton (StreamingDashboard.tsx).
+ Phase 0 (Week 0-1): Add query-based auth fallback to streaming endpoints; implement a minimal UI wiring in code comments and a TS skeleton (StreamingDashboard.tsx).
