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
| **Security-first** | Hardened beyond both — T0-T3 tiers, HMAC auth, TOTP, VM isolation |

## 6-Layer Architecture

```
L6 ┌─────────────────────────────────────────────────────────────┐
   │  Web UI (React SPA)                                         │
   │  Real-time chat · Skill marketplace · File browser          │
L5 └──────────────────────┬────────────────────────────────────┘
                          │  WebSocket
L4 ┌──────────────────────▼─────────────────────────────────────┐
   │  L1 · Messaging Gateway (TypeScript)                        │
   │  Slack · Discord · Telegram · WhatsApp · Email             │
L3 └──────────────────────┬──────────────────────────────────────┘
                          │  TaskRequest → NATS
L2 ┌──────────────────────▼─────────────────────────────────────┐
   │  L2 · Task Router (Rust)                                    │
   │  Intent classification · Permission enforcement            │
L1 └──────────┬────────────────────────────┬──────────────────┘
              │ apex.tasks.shallow       │ apex.tasks.deep
L0 ┌──────────▼──────────┐    ┌───────────▼───────────────────┐
   │  L4 · Skill Runner  │    │  L5 · Execution Engine         │
   │  (TypeScript)       │    │  (Python in Firecracker microVM)│
   │  Curated ~33 skills │    │  Agent Zero loop               │
   └─────────────────────┘    └────────────────────────────────┘
```

## Key Components

### L2 Task Router
- **Intent Classification**: Instant (<100ms) / Shallow (<3s) / Deep (async)
- **Permission Enforcement**: T0-T3 tier gates
- **Cost Estimation**: Model selection, budget tracking

### L3 Memory & State
- SQLite database: `~/.apex/data/apex.db`
- Vector search via sqlite_vec
- Append-only audit log (tamper-evident)
- Memory tiers: Working → Session → Project → Long-term

### L4 Skill Registry
- 33 built-in skills across categories
- SKILL.md standard (hot-reload capable)
- Permission tiers: T0 (silent) → T1 (tap) → T2 (type) → T3 (TOTP)

### L5 Execution Engine
- Firecracker micro-VMs for isolation
- Agent Zero loop: plan → act → observe → reflect
- 125ms cold start, 512MB RAM default
- gVisor fallback on non-KVM machines

## Security Model

### Permission Tiers
| Tier | Actions | Gate |
|------|---------|------|
| T0 | Read-only queries, search | None |
| T1 | File writes, drafts | Tap confirm |
| T2 | External API calls, git push | Type to confirm |
| T3 | Destructive ops, cost >$10 | TOTP + 5-min delay |

### VM Isolation
- Dedicated Linux kernel (no host sharing)
- Network blocked by default (allowlist required)
- Ephemeral storage (destroyed with VM)
- Resource limits enforced

## Implementation Status

| Component | Version | Status |
|-----------|---------|--------|
| Core (Rust) | L2/L3 Router | Built |
| Gateway (TypeScript) | L1 Messaging | Built |
| Skills (TypeScript) | L4 Registry | Built (33 skills) |
| Execution (Python) | L5 Engine | Built (Docker) |
| UI (React) | L6 Web UI | Built |
| MCP | Model Context Protocol | Built |

## Hermes Agent Features (v1.5.0+)
- Bounded curated memory (2,200 agent / 1,375 user chars)
- Auto-created skills after 5+ tool calls
- Session search with FTS5
- User profile modeling

## Sapphire Features (v1.6.0)
- Tool Maker runtime validation
- Persona assembly
- Context scope isolation
- Continuity scheduler
- Plugin signing
- Privacy toggle
- Story engine

## Files
- Design spec: [raw/APEX-Design.md](raw/APEX-Design.md)
- Architecture: [ARCHITECTURE.md](raw/ARCHITECTURE.md)
- Security: [SECURITY.md](raw/SECURITY.md)