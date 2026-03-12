# APEX - Autonomous Agent Platform

> ⚠️ **Status: PRE-ALPHA** - This is an experimental research project. Not production ready.

APEX combines the **best of OpenClaw and AgentZero** with **significantly stronger security**. A single-user autonomous agent platform with messaging interfaces and secure code execution.

## Vision

| Reference | What We Take |
|-----------|-------------|
| **OpenClaw** | Open architecture, extensibility, community-driven plugin ecosystem, messaging adapters |
| **AgentZero** | Dark navy/cyan aesthetic, polished UI, smooth UX patterns, agent loop logic |
| **Security-first** | Hardened beyond both — T0-T3 permission tiers, HMAC auth, TOTP verification, input sanitization, connection pooling |

APEX is **more secure than both** by design:
- Single-user architecture (no multi-tenancy attack surface)
- Hardened MCP with connection pooling and input validation
- Firecracker/gVisor isolation for code execution
- Audit trail with decision journal and reflection tracking

## 🚨 Pre-Alpha Warnings

- **Security-first but unaudited** — Built with strong security principles, but not formally audited yet
- **Limited testing** — Many features are proof-of-concept
- **API instability** — Breaking changes expected
- **No production support** — Use at your own risk
- **Execution isolation** — Firecracker/gVisor/Docker/Mock backends available

## Status

**Version**: v1.3.2 (Pre-Alpha)

## Architecture

6-layer system (Proof-of-Concept):
- **L1**: TypeScript Gateway (REST API adapters)
- **L2**: Rust Router (Task routing & classification)
- **L3**: Rust Memory Service (SQLite persistence)
- **L4**: TypeScript Skills Framework
- **L5**: Python Execution Engine (Docker)
- **L6**: React UI

## Features Implemented (Experimental)

### Core (POC)
- Task routing with automatic tier classification (Instant/Shallow/Deep)
- HMAC authentication between components (basic)
- SQLite database with migrations
- WebSocket real-time updates (basic)
- Execution streaming with consequence preview (POC)

### Security Features
- **Encrypted Secret Storage** - AES-256-GCM encrypted key-value store
- **Enhanced Rate Limiting** - Per-endpoint with progressive throttling
- **TOTP Persistence** - Time-based OTP with encrypted storage
- **Input Validation** - MCP sanitization tests (31 tests)
- **Audit Chain** - Hash chain verification tests (12 tests)
- **Permission Tiers** - T0-T3 enforcement tests (14 tests)

### Skills System (34 skills) - Experimental
- **T0 (Read-only)**: code.review, repo.search, deps.check, file.search, docs.read
- **T1 (Tap confirm)**: file.delete, git.commit, code.generate, code.format, code.test, code.refactor, code.document, api.test, api.design, db.migrate, db.schema, copy.generate, script.draft, script.outline, seo.optimize, music.extend, music.generate, music.remix, video.edit, video.generate
- **T2 (Type confirm)**: db.drop, deploy.kubectl, docker.build, docker.run, git.branch, git.force_push, aws.lambda, ci.configure
- **T3 (TOTP)**: shell.execute

### Advanced Features (Research/POC)
- **SOUL.md Identity** - Agent reads identity file on wake (POC)
- **Heartbeat Daemon** - Autonomous wake cycles (experimental)
- **Narrative Memory** - Tracks agent decisions (basic)
- **Moltbook Social** - Federated agent network integration (POC)
- **Governance Engine** - Constitution enforcement (basic)
- **Execution Streaming** - Real-time thought/action/observation to UI (POC)
- **Dynamic Tool Generation** - LLM generates custom Python tools (POC)
- **Subagent Pool** - Parallel task splitting (POC)

### UI Features (AgentZero Theme)
- Real-time chat with task sidebar
- **AgentZero Styling**: Indigo (#4248f1) primary, CSS variables, SVG icons
- **Message Reactions**: Copy, edit, regenerate on hover
- **Toast Notifications**: success/error/warning/info with auto-dismiss
- **Attachment Support**: File upload with preview
- **Speech Input**: Voice recording via Web Speech API
- **Enhanced Welcome Screen**: Quick action cards
- Kanban board for task management
- Workflow visualizer (basic)
- Memory/narrative viewer (basic)
- Skills marketplace
- System health monitoring
- Quick command bar (Ctrl+P)
- Skill quick launcher (Ctrl+K)

## Tech Stack (Development)

- **Backend**: Rust (Axum, Tokio, Sqlx)
- **Frontend**: React 18, TypeScript, Tailwind CSS, Zustand
- **Database**: SQLite
- **LLM**: llama.cpp (Qwen3-4B) - requires local setup (disabled in development mode by default)

## Getting Started (Development)

> **Development Mode**: By default, APEX runs with local LLM disabled to avoid unnecessary LLM usage during development. Enable LLM via Settings → LLM in the UI, or set `APEX_USE_LLM=1` when testing LLM features.

```powershell
# Clone and setup
cd apex

# Install dependencies
cargo install  # Rust
cd ui && pnpm install  # Frontend

# Start all services (with LLM + Docker)
.\apex.bat start

# Start all services INCLUDING embedding server (for memory search)
.\apex.bat start-full

# Or with different isolation backends:
.\apex.bat router-docker       # Docker container (default)
.\apex.bat router-gvisor       # gVisor sandbox (Linux only)
.\apex.bat router-firecracker # Firecracker VM (Linux only)  
.\apex.bat router-mock        # No real execution

# Individual services:
.\apex.bat llama    # LLM server (port 8080)
.\apex.bat embed   # Embedding server (port 8081)
.\apex.bat router  # Router

# Or manually:
# Terminal 1: llama-server (local LLM)
# Terminal 2: cargo run --release --bin apex-router
# Terminal 3: cd ui && pnpm dev
```

## API

See `docs/ARCHITECTURE.md` for API documentation (subject to change).

## Roadmap

- [x] Formal security audit (Phases 1-2 complete)
- [x] Dynamic tool generation (POC)
- [x] Subagent pool (POC)
- [x] Comprehensive testing (212+ tests)
- [x] Task classification rules (Instant/Shallow/Deep tiers)
- [x] Capability enforcement (fail-closed for unknown skills)
- [x] Gateway optional (auth can be disabled)
- [x] MCP marketplace UI (templates, install modal, registries)
- [x] Production hardening (seccomp, AppArmor, SIEM docs)
- [x] SystemComponent trait (unified lifecycle management)

## License

MIT
