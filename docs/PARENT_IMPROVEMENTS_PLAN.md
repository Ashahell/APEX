# Parent Project Improvements Implementation Plan

> **Plan Date:** 2026-04-12  
> **Version:** v1.7.1  
> **Status:** Proposed  
> **Owner:** APEX Development Team  

---

## Overview

This plan details adoption of key features from the three parent projects (OpenClaw, AgentZero, Hermes Agent) into APEX. Each item includes scope, design, tasks, milestones, and acceptance criteria.

---

## Priority Matrix

| Priority | Count | Timeline |
|----------|-------|----------|
| **High** | 4 | 2-3 sprints |
| **Medium** | 4 | 1-2 sprints |
| **Low** | 3 | Backlog |

---

## High Priority Items

### H1: Early Tool Dispatch via Streaming JSON

**Parent:** AgentZero v1.7  
**Objective:** Dispatch tools as soon as first complete JSON detected in stream, rather than waiting for all tokens. Reduces perceived latency.

**Scope:** Execution stream, tool execution layer, streaming parser

**Design:**
- Add `completed` flag to DirtyJson parser for early extraction
- Stop stream at first closed top-level JSON object
- Route to tool executor immediately

**Tasks:**
- [ ] Update streaming parser with early-completion detection
- [ ] Modify execution stream to emit tool-ready event before EOF
- [ ] Wire tool dispatcher to receive early events
- [ ] Add streaming tests for early dispatch scenarios
- [ ] Update SSE endpoints to support early emission

**Milestones:**
1. Parser modification (0.5 sprint)
2. Integration + tests (0.5 sprint)
3. QA + polish (0.5 sprint)

**Acceptance Criteria:**
- [ ] Tools dispatch within 500ms of JSON closure (vs. full completion)
- [ ] All existing streaming tests pass
- [ ] No regression in tool execution correctness

**Files:**
- `core/router/src/execution_stream.rs` - Modify parser
- `core/router/src/streaming.rs` - Update event emission
- `core/router/src/tool_executor.rs` - Wire early dispatch

---

### H2: Subagent Progress Reporting

**Parent:** OpenClaw #13990  
**Objective:** Visible progress updates from subagent/task work - parents see milestone notifications during long runs.

**Scope:** Task execution, deep task worker, streaming events

**Design:**
- Add `subagent_progress` event type with `percent`, `stage`, `message`
- Emit progress events from deep_task_worker at key milestones
- Display in UI ProcessGroup component

**Tasks:**
- [ ] Define subagent progress event schema
- [ ] Add progress emission points in deep_task_worker
- [ ] Update streaming SSE to carry progress events
- [ ] Add progress display to UI (ProcessGroup badges)
- [ ] Add progress tests

**Milestones:**
1. Backend schema + emission (0.5 sprint)
2. Streaming + tests (0.5 sprint)
3. UI display (0.5 sprint)

**Acceptance Criteria:**
- [ ] Progress events visible in streaming dashboard
- [ ] UI shows progress badges during task execution
- [ ] At least 3 progress points per deep task

**Files:**
- `core/router/src/streaming_types.rs` - Event schema
- `core/router/src/deep_task_worker.rs` - Emission points
- `ui/src/components/chat/ProcessGroup.tsx` - UI display

---

### H3: Expanded Secrets Targets

**Parent:** OpenClaw v2026.3.2 (64 targets)  
**Objective:** Support more secret reference targets for credentials, APIs, keys.

**Scope:** Secrets system, config, adapters

**Design:**
- Add new target categories to secrets system
- Support dynamic target registration
- Update API keys storage layer

**Tasks:**
- [ ] Audit current 32 targets vs. OpenClaw's 64
- [ ] Add missing targets (GitHub, AWS, Azure, GCP, etc.)
- [ ] Add target discovery/registration API
- [ ] Update secrets UI with expanded categories
- [ ] Add target validation tests

**Milestones:**
1. Gap analysis (0.25 sprint)
2. Backend expansion (0.5 sprint)
3. UI + tests (0.5 sprint)

**Acceptance Criteria:**
- [ ] Minimum 50 supported targets
- [ ] UI shows all available categories
- [ ] Target validation in config

**Files:**
- `core/router/src/secrets_repo.rs` - Targets
- `core/router/src/api/secrets.rs` - API
- `ui/src/components/settings/SecretsManager.tsx` - UI

---

### H4: Supply Chain Security Hardening

**Parent:** Hermes v0.5.0  
**Objective:** Comprehensive security audit - dependency pinning, CVE scanning, supply chain verification.

**Scope:** Dependencies, CI/CD, security

**Design:**
- Pin all dependency versions
- Add supply chain audit CI
- Implement CVE scanning workflow
- Regular security audits

**Tasks:**
- [ ] Audit all Cargo.toml, package.json, pyproject.toml
- [ ] Pin exact versions (no ranges)
- [ ] Add trivy/dependency-track to CI
- [ ] Add security audit workflow (weekly)
- [ ] Document security policy

**Milestones:**
1. Dependency audit (0.25 sprint)
2. CI integration (0.5 sprint)
3. Documentation (0.25 sprint)

**Acceptance Criteria:**
- [ ] All dependencies pinned
- [ ] Weekly CVE scan passes
- [ ] Security policy documented

**Files:**
- `core/router/Cargo.toml` - Rust deps
- `gateway/package.json` - JS deps
- `.github/workflows/security.yml` - CI

---

## Medium Priority Items

### M1: Response Caching Layer

**Parent:** AgentZero v1.5 (API/WebSocket caching)  
**Objective:** Cache expensive API responses for repeated queries.

**Scope:** API layer, cache system, responses

**Design:**
- Use existing ResponseCache module
- Wire into API state  
- Add cache headers
- Support cache invalidation

**Tasks:**
- [ ] Wire ResponseCache into AppState
- [ ] Add cache decorators to GET endpoints
- [ ] Add cache headers (ETag, Cache-Control)
- [ ] Add cache invalidation hooks
- [ ] Add caching tests

**Milestones:**
1. Integration (0.5 sprint)
2. Headers + tests (0.25 sprint)

**Acceptance Criteria:**
- [ ] ResponseCache actively used
- [ ] Cache hit/miss metrics visible
- [ ] TTL Configurable per endpoint

**Files:**
- `core/router/src/response_cache.rs` - Module (exists, unwired)
- `core/router/src/main.rs` - Wiring

---

### M2: Inactivity-Based Timeout

**Parent:** Hermes v0.8.0  
**Objective:** Smart timeouts based on activity, not wall-clock.

**Scope:** Task execution, heartbeat, workers

**Design:**
- Track last activity timestamp per task
- Reset on any event (tool call, message, etc.)
- Timeout only when no activity for N minutes

**Tasks:**
- [ ] Add activity tracking to task state
- [ ] Update timeout logic to check activity
- [ ] Add inactivity config to AppConfig
- [ ] Add inactivity tests

**Milestones:**
1. Backend tracking (0.5 sprint)
2. Config + tests (0.25 sprint)

**Acceptance Criteria:**
- [ ] Tasks timeout on inactivity, not clock time
- [ ] Configurable per-task inactivity limit

**Files:**
- `core/router/src/task_repo.rs` - Activity tracking
- `core/router/src/unified_config.rs` - Config

---

### M3: Skills System Alignment

**Parent:** AgentZero v0.9.8 (SKILL.md standard)  
**Objective:** Align APEX skills with AgentZero's structured SKILL.md format.

**Scope:** Skills framework, skill loader

**Design:**
- Adopt SKILL.md frontmatter format
- Add skill version history
- Add skill dependencies
- Improve skill discovery

**Tasks:**
- [ ] Update SKILL.md schema with frontmatter
- [ ] Add version field to skill loader
- [ ] Add skill dependencies support
- [ ] Add skill search/discovery API

**Milestones:**
1. Schema update (0.25 sprint)
2. Loader updates (0.5 sprint)
3. API + tests (0.25 sprint)

**Acceptance Criteria:**
- [ ] Skills use SKILL.md format
- [ ] Skill versioning works
- [ ] Skill search returns results

**Files:**
- `skills/src/types.ts` - Schema
- `skills/src/loader.ts` - Loader

---

### M4: MCP OAuth 2.1

**Parent:** Hermes v0.8.0  
**Objective:** Update MCP authentication to OAuth 2.1 spec.

**Scope:** MCP server, auth system

**Design:**
- Add OAuth 2.1 provider support
- Add token refresh handling
- Support device flow

**Tasks:**
- [ ] Add OAuth 2.1 types
- [ ] Implement token refresh
- [ ] Add device flow support
- [ ] Add MCP OAuth tests

**Milestones:**
1. OAuth types (0.25 sprint)
2. Implementation (0.5 sprint)

**Acceptance Criteria:**
- [ ] OAuth 2.1 auth for MCP tools
- [ ] Token refresh works
- [ ] Pass MCP compliance tests

**Files:**
- `core/router/src/mcp/server.rs` - MCP server
- `core/router/src/auth.rs` - Auth

---

## Low Priority Items

### L1: Interactive Model Picker UI

**Parent:** Hermes v0.8.0 (interactive model picker for Telegram/Discord)  
**Objective:** In-app model selection UI.

**Scope:** Settings UI, model config

**Tasks:**
- [ ] Add model picker component
- [ ] Wire to model config API
- [ ] Add model pricing display (optional)

**Milestones:** 0.5 sprint

---

### L2: Nix Flake Support

**Parent:** Hermes v0.5.0  
**Objective:** Add Nix flake for NixOS deployment.

**Scope:** Deployment, Nix

**Tasks:**
- [ ] Create flake.nix
- [ ] Add NixOS module
- [ ] Add deployment docs

**Milestones:** 0.5 sprint

---

### L3: WhatsApp Adapter

**Parent:** AgentZero v1.6  
**Objective:** Add WhatsApp via Baileys integration.

**Scope:** Gateway adapters

**Tasks:**
- [ ] Create WhatsApp adapter
- [ ] Add QR code pairing
- [ ] Test group chat

**Milestones:** 1 sprint

---

## Implementation Order

```
Sprint 1-2: H1 (Early Tool Dispatch), H2 (Progress Reporting)
Sprint 2-3: H3 (Secrets Expansion), M1 (Response Caching)
Sprint 3-4: M2 (Inactivity Timeout), M3 (Skills Alignment)
Sprint 4-5: H4 (Security Hardening), M4 (MCP OAuth)
Sprint 5+: L1, L2, L3 (if time permits)
```

---

## Dependencies

| Item | Depends On |
|------|-------------|
| H1: Early Tool Dispatch | None |
| H2: Progress Reporting | Streaming, deep task worker |
| H3: Secrets Expansion | None |
| H4: Security | None |
| M1: Response Caching | H1 (streaming improvements help) |
| M2: Inactivity Timeout | None |
| M3: Skills Alignment | None |
| M4: MCP OAuth | H4 (security hardening) |

---

## Testing Strategy

| Category | Coverage |
|----------|----------|
| Unit Tests | Each new feature |
| Integration Tests | API endpoints, streaming |
| E2E Tests | Full task workflows |
| Security Tests | CVE scanning, dependency audit |

---

## Rollback Plan

For each item:
1. Feature flag for enable/disable
2. Keep old code path until new verified
3. Quick revert if critical failure

---

## Success Metrics

- [ ] All tests pass (target: 500+ tests)
- [ ] No security CVEs (30-day window)
- [ ] Response latency reduced 20%+ (early dispatch)
- [ ] Subagent visibility improved (progress reporting)
- [ ] 50+ secrets targets supported
- [ ] Supply chain audit passes weekly

---

## References

- OpenClaw: https://github.com/openclaw/openclaw
- AgentZero: https://github.com/agent0ai/agent-zero  
- Hermes: https://github.com/NousResearch/hermes-agent
- This plan: `docs/PARENT_IMPROVEMENTS_PLAN.md`

---

*Generated from parent project analysis - April 2026*