# APEX - Autonomous Agent Platform

> **Status: v2.0.0** - Parity Complete (OpenClaw + AgentZero + Hermes + OpenFang)

APEX combines the **best of OpenClaw, AgentZero, Hermes, and OpenFang** with **significantly stronger security**. A single-user autonomous agent platform with messaging interfaces, secure code execution, and comprehensive observability.

## Vision

| Reference | What We Take |
|-----------|-------------|
| **OpenClaw** | Open architecture, extensibility, community-driven plugin ecosystem, messaging adapters |
| **AgentZero** | Dark navy/cyan aesthetic, polished UI, smooth UX patterns, agent loop logic |
| **Hermes** | Bounded memory, auto-created skills, session search, user profile |
| **OpenFang** | Telemetry dashboards, MCP governance, orchestration surface |
| **Security-first** | Hardened beyond all — T0-T3 permission tiers, HMAC auth, TOTP verification, input sanitization, connection pooling |

## Parity Scores

| Platform | Score | Status |
|----------|-------|--------|
| OpenClaw | 9.2/10 | ✅ Complete |
| AgentZero | 9.4/10 | ✅ Complete |
| Hermes | 9.8/10 | ✅ Complete |
| OpenFang | 9.4/10 | ✅ Complete |
| **Overall** | **9.45/10** | ✅ **Complete** |

## 🚨 Pre-Alpha Warnings

- **Security-first but unaudited** — Built with strong security principles, but not formally audited yet
- **API instability** — Breaking changes expected until v3.0
- **No production support** — Use at your own risk
- **Execution isolation** — Firecracker/gVisor/Docker/Mock backends available

---

## Quick Start

```powershell
# Clone and setup
git clone https://github.com/Ashahell/APEX.git
cd APEX

# Start all services (with LLM + Docker)
.\apex.bat start

# Start all services INCLUDING embedding server (for memory search)
.\apex.bat start-full

# Check status
.\apex.bat status
```

---

## Architecture

6-layer system with Rust core, TypeScript gateway/skills, Python execution, and React UI:

```
User → UI (React) → Gateway (Fastify) → Router (Axum) → Memory (SQLite)
                                           ↓
                                      Skills (TS) → Execution (Python/Docker)
```

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **L1** | TypeScript (Fastify) | Gateway with HMAC-signed requests |
| **L2** | Rust (Axum, Tokio) | Task routing, classification, agent loop |
| **L3** | Rust (SQLite, sqlx) | Memory service, bounded memory, search |
| **L4** | TypeScript | Skills framework, MCP client/server |
| **L5** | Python (Docker) | Secure execution engine with sandboxing |
| **L6** | React 18 + TypeScript | UI with 4 themes, real-time streaming |

---

## Features

### Core
- **Task Routing** — Automatic tier classification (Instant/Shallow/Deep)
- **Agent Loop** — Plan/act/observe cycle with deep task worker
- **Streaming** — TinySSE-based real-time events (Hands, MCP, Task, Stats)
- **WebSocket** — Real-time updates with polling fallback
- **SQLite** — 22+ migrations with atomic transactions

### Security
- **HMAC Authentication** — Request signing with shared secret
- **TOTP Verification** — T3 tier requires 6-digit authenticator code
- **Permission Tiers** — T0 (read) → T1 (tap) → T2 (type) → T3 (TOTP)
- **Input Validation** — MCP sanitization, injection detection (50+ tests)
- **Audit Chain** — SHA-256 hash chain with tamper detection (17 tests)
- **Replay Protection** — In-memory signature store (9 tests)
- **Rate Limiting** — Per-endpoint with progressive throttling
- **Anomaly Detection** — Runtime behavior monitoring, death spiral detection
- **Encrypted Secrets** — AES-256-GCM encrypted key-value store

### Memory (Hermes-style)
- **Bounded Memory** — Character-limited stores (2,200 agent / 1,375 user)
- **Semantic Search** — Hybrid BM25 + embeddings with MMR reranking
- **TTL Semantics** — Configurable per-store expiration
- **Consolidation** — AI-suggested entry merging with approval workflow
- **Frozen Snapshots** — System prompt-ready memory snapshots

### MCP & Tools
- **Tool Registry** — Discovery, versioning, health checks
- **Marketplace** — Plugin signing (ed25519), trust levels, installation
- **Governance** — Submission, review, revocation policies

### UI
- **4 Themes** — Modern 2026, Amiga Workbench, AgentZero, High Contrast (WCAG AAA)
- **Real-time Streaming** — Hands, MCP, Task panels with live events
- **Monitoring Dashboard** — Per-endpoint latency, error rates, SLO tracking
- **Memory Viewer** — 6-tab interface (Memory, User, Search, TTL, Consolidation, Snapshot)
- **Kanban Board** — Task management with drag-and-drop
- **Settings** — 15+ tabs (Chat, LLM, Memory, User Profile, etc.)

### Skills System
- **34 Built-in Skills** — T0-T3 permission tiers
- **Auto-created Skills** — Generated after complex tasks (5+ tool calls)
- **Skills Hub** — Marketplace with trust levels (Verified > Trusted > Community)
- **Dynamic Tool Generation** — LLM-generated Python with secure sandbox

---

## Project Structure

```
apex/
├── core/                    # Rust (L2/L3)
│   ├── router/              # Task Router (HTTP API, agent loop, streaming)
│   ├── memory/              # Memory Service (SQLite, bounded memory)
│   └── security/            # Secret store, anomaly detection
├── gateway/                 # TypeScript (L1)
├── skills/                  # TypeScript (L4)
├── ui/                      # React (L6)
├── execution/               # Python (L5)
└── docs/                    # Documentation
    ├── PHASE0-10_RUNBOOK.md # Incident response per phase
    ├── crosswalk_*.md       # Platform parity crosswalks
    ├── GOVERNANCE_CHARTER.md
    ├── MIGRATION_PLAN.md
    └── HANDOVER.md
```

---

## Testing & Quality

| Metric | Value |
|--------|-------|
| **Total Tests** | 583 passing |
| **Rust Unit** | 348 |
| **Rust Integration** | 59 + 40 (security) + 16 (streaming) + 9 (telemetry) + 10 (auth) |
| **Python** | 53 |
| **UI** | 20 |
| **Clippy** | 0 warnings (`-D warnings`) |
| **Benchmarks** | 4 groups (Criterion) |

```bash
# Run all tests
cd core && cargo test

# Run clippy
cd core && cargo clippy -- -D warnings

# Run benchmarks
cd core && cargo bench

# Build UI
cd ui && npm run build
```

---

## Documentation

| Document | Purpose |
|----------|---------|
| [AGENTS.md](AGENTS.md) | Development guide |
| [docs/HANDOVER.md](docs/HANDOVER.md) | Project handover |
| [docs/MIGRATION_PLAN.md](docs/MIGRATION_PLAN.md) | Migration strategy |
| [docs/GOVERNANCE_CHARTER.md](docs/GOVERNANCE_CHARTER.md) | Governance framework |
| [docs/RUNBOOK_INDEX.md](docs/RUNBOOK_INDEX.md) | Runbook directory |
| [docs/CODEBASE_AUDIT_REPORT.md](docs/CODEBASE_AUDIT_REPORT.md) | Codebase audit |
| [docs/parity-scorecard.md](docs/parity-scorecard.md) | Parity tracking |
| [docs/crosswalk_openclaw_apex.md](docs/crosswalk_openclaw_apex.md) | OpenClaw crosswalk (9.2/10) |
| [docs/crosswalk_agentzero_apex.md](docs/crosswalk_agentzero_apex.md) | AgentZero crosswalk (9.4/10) |
| [docs/crosswalk_hermes_apex.md](docs/crosswalk_hermes_apex.md) | Hermes crosswalk (9.8/10) |
| [docs/crosswalk_openfang_apex.md](docs/crosswalk_openfang_apex.md) | OpenFang crosswalk (9.4/10) |

---

## Tech Stack

- **Backend**: Rust 1.93+ (Axum, Tokio, sqlx)
- **Frontend**: React 18, TypeScript, Tailwind CSS, Zustand
- **Database**: SQLite (with FTS5 for search)
- **LLM**: llama.cpp (Qwen3-4B) — disabled in dev mode by default
- **Execution**: Docker / Firecracker / gVisor / Mock

---

## Roadmap

### Completed ✅
- [x] 10-Phase Parity Project (OpenClaw, AgentZero, Hermes, OpenFang)
- [x] Streaming MVP (TinySSE, 11 event types, performance metrics)
- [x] Telemetry (per-endpoint latency, error rates, SLO tracking)
- [x] Security (40+ tests, injection detection, replay protection)
- [x] MCP/Tools (discovery, marketplace, plugin signing)
- [x] Memory (bounded, TTL, consolidation, search, snapshots)
- [x] UI/Theming (4 themes, accessibility, monitoring dashboard)
- [x] Ecosystem (plugin signing, skills hub, governance)
- [x] Governance (charter, cadence, crosswalks, audit trail)
- [x] Migration Plan (4 pilot phases, rollback procedures)
- [x] Technical Debt (benchmarks, clippy clean, audit report)

### Next 🚧
- [ ] GitHub Actions CI/CD pipeline
- [ ] Docker Compose for production deployment
- [ ] External security audit
- [ ] Performance benchmarking suite
- [ ] Chaos engineering tests
- [ ] Production hardening (seccomp, AppArmor, SIEM)

---

## License

MIT
