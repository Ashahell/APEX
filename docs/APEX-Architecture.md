# APEX Architecture Overview

> **Status**: Pre-Alpha (Experimental) ⚠️
> **Version**: v1.3.1
> **Last Updated**: 2026-03-10

---

## Vision

APEX combines the **best of OpenClaw and AgentZero** with **significantly stronger security**.

| Reference | What We Take |
|-----------|-------------|
| **OpenClaw** | Open architecture, extensibility, community-driven plugin ecosystem, messaging adapters |
| **AgentZero** | Dark navy/cyan aesthetic, polished UI, smooth UX patterns, agent loop logic |
| **Security-first** | Hardened beyond both — T0-T3 permission tiers, HMAC auth, TOTP verification, input sanitization, connection pooling |

---

## Six Architectural Layers

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         L6: React UI                                       │
│  Chat │ Skills │ Board │ Memory │ Settings │ Metrics │ + more            │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    L1: Gateway (TypeScript)                                │
│  REST Adapter │ HMAC Signing │ WebSocket                                  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    L2: Router (Rust)                                        │
│  Task Router │ Auth │ MCP │ Skills │ Subagent Pool │ Dynamic Tools       │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
┌─────────────────────────────────┐    ┌─────────────────────────────────────┐
│   L4: Skills (TypeScript)      │    │    L5: Execution (Docker)          │
│   33 skills with T0-T3 tiers   │    │    Hardened isolation              │
└─────────────────────────────────┘    └─────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    L3: Memory (Rust - SQLite)                              │
│  Tasks │ Messages │ Skills │ Audit │ Vector Store (sqlite_vec)          │
│  Narrative Memory │ Journal │ Reflections                                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Key Components

### L2 Router (Rust)
| Component | Status | Description |
|-----------|--------|-------------|
| Task Router | ✅ | Task classification, routing |
| HMAC Auth | ✅ | Request signing |
| TOTP | ✅ | T3 verification |
| MCP Client | ✅ | Connection pooling, resources, prompts |
| **Subagent Pool** | ✅ | Parallel task execution |
| **Dynamic Tools** | ✅ | LLM-generated tools |
| **SystemComponent** | ✅ NEW | Unified lifecycle management |
| Execution Stream | ✅ | Thought/tool streaming |
| Rate Limiter | ✅ | Enhanced with progressive throttle |

### L3 Memory (Rust)
| Component | Status | Description |
|-----------|--------|-------------|
| SQLite | ✅ | Task/message persistence |
| sqlite_vec | ✅ | Vector search |
| Embedder | ✅ | Local (llama-server) or OpenAI |
| Hybrid Search | ✅ | Keyword + vector |
| Narrative | ✅ | Task journaling |
| Reflections | ✅ | Agent self-reflection |
| **Memory Export** | ✅ NEW | JSON export/import |

### L4 Skills (TypeScript)
| Component | Status | Description |
|-----------|--------|-------------|
| Skill Loader | ✅ | Dynamic loading |
| Skill Pool | ✅ | Worker pool (34 skills) |
| **Skill SDK** | ✅ Enhanced | skill.yaml, CLI |
| **Hot Reload** | ✅ | File watching |
| **Cache Invalidation** | ✅ NEW | API endpoints for reload |

### L5 Execution (Docker)
| Component | Status | Description |
|-----------|--------|-------------|
| Docker | ✅ Default | Hardened config |
| Firecracker | 🔧 WSL2 | Ready (needs testing) |
| gVisor | 🔧 | Not configured |

### L6 UI (React)
| Component | Status | Description |
|-----------|--------|-------------|
| Chat | ✅ | Real-time messaging |
| ProcessGroup | ✅ | Step visualization |
| TaskSidebar | ✅ | Active tasks |
| SkillMarketplace | ✅ | MCP tools |
| **MCP Marketplace** | ✅ NEW | Server templates, install modal |
| MemoryViewer | ✅ | Timeline view |
| MetricsPanel | ✅ | Cost tracking |
| **Keyboard Shortcuts** | ✅ | Ctrl+1-5, Ctrl+K |
| TaskClassifier | ✅ NEW | Instant/Shallow/Deep tiers |

---

## Security Controls

| Control | Implementation |
|---------|----------------|
| Authentication | HMAC-SHA256 request signing |
| Authorization | T0-T3 permission tiers |
| TOTP | Time-based OTP for T3 actions |
| Rate Limiting | Enhanced with progressive throttling |
| Audit Log | Hash chain verification |
| Input Validation | Zod + MCP sanitization |
| Execution Isolation | Docker: `--network none`, `--cap-drop ALL`, `--read-only` |
| **Secret Storage** | ✅ NEW - AES-256-GCM encrypted |
| **TOTP Persistence** | ✅ NEW - Encrypted storage |

---

## API Endpoints

### Core
- Tasks: CRUD, confirm, cancel
- Skills: List, execute, pool stats
- MCP: Servers, tools, registries
- Memory: Search, journal, reflections

### New (v1.3.0)
- **Subagent Pool**: `/api/v1/subagent/*`
- **Dynamic Tools**: `/api/v1/dynamic-tools/*`

---

## Documentation

| Document | Description |
|----------|-------------|
| `VISION_REALIZATION_PLAN.md` | Full implementation roadmap |
| `SECURITY_THREAT_MODEL.md` | Threat model |
| `KEYBOARD_SHORTCUTS.md` | Shortcuts reference |
| `SKILL-SDK.md` | Skill development |
| `VM_BACKEND_WINDOWS.md` | Windows deployment |
| `FIRECRACKER_WSL2.md` | WSL2 setup |

---

## Development Status

- **Pre-Alpha**: Experimental, not production ready
- **Tests**: 158+ passing
- **Build**: Compiles successfully
- **Platform**: Windows 11 + WSL2

---

*This document is updated as the architecture evolves.*
