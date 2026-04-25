# APEX Architecture & Design

**Status:** Implemented (v1.6.0)
**Date:** 2026-04-25
**Source:** [docs/APEX-Design.md](raw/APEX-Design.md)

## Overview

APEX (Autonomous Platform for Execution & Communication) is a 6-layer single-user autonomous agent platform combining OpenClaw architecture with AgentZero UI patterns and security-first design.

## Parent Systems

| System | What APEX Takes |
|--------|----------------|
| **OpenClaw** | Messaging adapters (Slack, Discord, Telegram), open architecture, plugin ecosystem |
| **AgentZero** | Dark navy/cyan aesthetic, agent loop logic, SKILL.md standard |
| **Hermes** | Bounded memory, auto-created skills, session search, user profiling |
| **Security-first** | Hardened beyond both — T0-T3 tiers, HMAC auth, TOTP, VM isolation |

## 6-Layer Architecture

```
L6 ┌─────────────────────────────────────────────────────────────┐
   │  Web UI (React SPA)                                         │
   │  Real-time chat · Skill marketplace · File browser          │
L5 └──────────────────────┬──────────────────────────────────────┘
                          │  WebSocket / HTTP
L4 ┌──────────────────────▼──────────────────────────────────────┐
   │  L1 · Messaging Gateway (TypeScript)                        │
   │  Slack · Discord · Telegram · WhatsApp · Email             │
L3 └──────────────────────┬───────────────────────────────────────┘
                          │  TaskRequest → NATS / internal bus
L2 ┌──────────────────────▼──────────────────────────────────────┐
   │  L2 · Task Router (Rust)                                    │
   │  Intent classification · Permission enforcement            │
L1 └──────────┬────────────────────────────┬────────────────────┘
              │ apex.tasks.shallow          │ apex.tasks.deep
L0 ┌──────────▼──────────┐   ┌──────────────▼─────────────────────┐
   │  L4 · Skill Runner  │   │  L5 · Execution Engine             │
   │  (TypeScript)       │   │  (Python in Firecracker microVM)   │
   │  Curated ~33 skills │   │  Agent Zero loop                   │
   └─────────────────────┘   └────────────────────────────────────┘
```

## Key Components

### L2 Task Router (`core/router/`)
- **Intent Classification**: Instant (<100ms) / Shallow (<3s) / Deep (async)
- **Permission Enforcement**: T0-T3 tier gates
- **Cost Estimation**: Model selection, budget tracking
- **Workers**: `skill_worker.rs`, `deep_task_worker.rs`, `t3_confirm_worker.rs`
- **API**: 60+ endpoints (tasks, skills, channels, journal, memory, MCP, etc.)

### L3 Memory & State (`core/memory/`)
- SQLite database: `~/.apex/data/apex.db`
- Vector search via sqlite_vec + embedder
- Append-only audit log with hash chain (tamper-evident)
- Memory tiers: Working → Session → Project → Long-term
- FTS5 full-text search with BM25 ranking
- 27 migrations (000-026)

### L4 Skill Registry (`skills/`)
- 33 built-in skills across categories
- SKILL.md standard (hot-reload capable)
- Permission tiers: T0 (silent) → T1 (tap) → T2 (type) → T3 (TOTP)
- Lexical matching fallback (60+ keyword triggers)
- Auto-created skills after 5+ tool calls

### L5 Execution Engine (`execution/`)
- Firecracker micro-VMs for isolation (Linux)
- Docker fallback on Windows
- Agent Zero loop: plan → act → observe → reflect
- 125ms cold start, 512MB RAM default
- gVisor fallback on non-KVM machines
- SSRF protection for web.fetch tool

### L6 Web UI (`ui/`)
- React 18 + TypeScript + Tailwind CSS
- Real-time via WebSocket + SSE
- Kanban task board
- Process group execution traces
- Theme system (Modern 2026, Amiga, AgentZero)
- Toast notifications

### L1 Gateway (`gateway/`)
- TypeScript/Fastify messaging adapters
- REST API proxy to router
- HMAC request signing

## API Surface

**Core:**
- Tasks: create, list, filter, cancel, confirm
- Messages: list, by task
- Skills: registry, execute, triggers, auto-created
- Deep Tasks: execute with VM pool
- TOTP: setup, verify, status

**Session Control:**
- Sessions: yield, resume, compact, checkpoints, attachments
- Channels: CRUD
- Journal: CRUD + search

**Memory:**
- Narrative memory: entities, knowledge, reflections
- Bounded memory: agent/user with frozen snapshots
- Session search: FTS5 + BM25 + context extraction
- Hub: marketplace skills

**System:**
- MCP servers and registries
- LLM config and providers
- Secrets (64 targets)
- Slack block kit templates
- Execution patterns (death spiral detection)
- Moltbook social integration
- Governance and policy

## Unified Configuration

All settings via `AppConfig::global()` in `core/router/src/unified_config.rs`:
- Server: port, host
- Auth: shared secret, disabled flag
- LLM: provider, model, llama-server URL
- Execution: isolation backend, VM memory/vCPU
- Memory: embedding provider, RRF k, decay
- Skill Pool: size, timeout, acquire
- Heartbeat: interval, jitter, cooldown

## Files

- Design spec: [raw/APEX-Design.md](raw/APEX-Design.md)
- Architecture: [raw/ARCHITECTURE.md](raw/ARCHITECTURE.md)
- Security: [raw/SECURITY.md](raw/SECURITY.md)
- Memory spec: [raw/APEX_Memory_System_Spec_v2.md](raw/APEX_Memory_System_Spec_v2.md)

## Related

- [project-overview.md](project-overview.md)
- [skills.md](concepts/skills.md)
- [phase-1-ux-improvements.md](concepts/phase-1-ux-improvements.md)
- [security.md](concepts/security.md)