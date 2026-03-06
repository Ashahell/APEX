# APEX - Autonomous Agent Platform

> ⚠️ **Status: PRE-ALPHA** - This is an experimental research project. Not production ready.

APEX is a **pre-alpha** autonomous agent platform combining messaging interfaces with secure code execution. Multi-tenancy is explicitly out of scope.

## 🚨 Pre-Alpha Warnings

- **No security audit** - Do not use with sensitive data
- **Limited testing** - Many features are proof-of-concept
- **API instability** - Breaking changes expected
- **No production support** - Use at your own risk
- **Execution isolation** - Firecracker/gVisor/Docker/Mock backends available

## Status

**Version**: v1.3.0 (Pre-Alpha)

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

### Skills System (33 skills) - Experimental
- T0 (Read-only): code.review, repo.search, deps.check, file.search
- T1 (Tap confirm): file.delete, git.commit, code.generate, code.format, api.test, etc.
- T2 (Type confirm): db.drop, deploy.kubectl, docker.build, git.branch, docker.run
- T3 (TOTP): shell.execute

### Advanced Features (Research/POC)
- **SOUL.md Identity** - Agent reads identity file on wake (POC)
- **Heartbeat Daemon** - Autonomous wake cycles (experimental)
- **Narrative Memory** - Tracks agent decisions (basic)
- **Moltbook Social** - Federated agent network integration (POC)
- **Governance Engine** - Constitution enforcement (basic)
- **Execution Streaming** - Real-time thought/action/observation to UI (POC)
- **Dynamic Tool Generation** - LLM generates custom Python tools (not implemented)
- **Subagent Pool** - Parallel task splitting (not implemented)

### UI Features (Basic)
- Real-time chat with task sidebar
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

# Or with different isolation backends:
.\apex.bat router-docker       # Docker container (default)
.\apex.bat router-gvisor       # gVisor sandbox (Linux only)
.\apex.bat router-firecracker # Firecracker VM (Linux only)  
.\apex.bat router-mock        # No real execution

# Or manually:
# Terminal 1: llama-server (local LLM)
# Terminal 2: cargo run --release --bin apex-router
# Terminal 3: cd ui && pnpm dev
```

## API

See `docs/ARCHITECTURE.md` for API documentation (subject to change).

## Roadmap

- [ ] Security audit
- [ ] Dynamic tool generation
- [ ] Subagent pool
- [ ] Comprehensive testing
- [ ] Production hardening

## License

MIT
