# APEX - Autonomous Agent Platform

APEX is an autonomous agent platform combining messaging interfaces with secure code execution. 

## Status

**Version**: v1.0.0

## Architecture

6-layer system:
- **L1**: TypeScript Gateway (REST API adapters)
- **L2**: Rust Router (Task routing & classification)
- **L3**: Rust Memory Service (SQLite persistence)
- **L4**: TypeScript Skills Framework
- **L5**: Python Execution Engine
- **L6**: React UI

## Features Implemented

### Core
- Task routing with automatic tier classification (Instant/Shallow/Deep)
- HMAC authentication between components
- SQLite database with migrations
- WebSocket real-time updates
- Execution streaming with consequence preview

### Skills System (28 skills)
- T0 (Read-only): code.review, repo.search, deps.check
- T1 (Tap confirm): file.delete, git.commit, code.generate, etc.
- T2 (Type confirm): db.drop, deploy.kubectl, docker.build, etc.
- T3 (TOTP): shell.execute

### Advanced Features
- **SOUL.md Identity** - Agent reads identity file on wake
- **Heartbeat Daemon** - Autonomous wake cycles with configurable intervals
- **Narrative Memory** - Tracks agent decisions and reflections
- **Moltbook Social** - Federated agent network integration
- **Governance Engine** - Constitution enforcement for action control
- **Execution Streaming** - Real-time thought/action/observation to UI
- **Dynamic Tool Generation** - LLM generates custom Python tools at runtime
- **Subagent Pool** - Parallel task splitting for complex operations

### UI Features
- Real-time chat with task sidebar
- Kanban board for task management
- Workflow visualizer
- Memory/narrative viewer
- Skills marketplace
- System health monitoring
- Governance controls

## Tech Stack

- **Backend**: Rust (Axum, Tokio, Sqlx)
- **Frontend**: React 18, TypeScript, Tailwind CSS, Zustand
- **Database**: SQLite
- **LLM**: llama.cpp (Qwen3-4B)

## Getting Started

```powershell
# Start all services
.\apex.bat start

# Or manually:
# Terminal 1: llama-server
# Terminal 2: cargo run --release --bin apex-router
# Terminal 3: cd ui && pnpm dev
```

## API

See `docs/ARCHITECTURE.md` for full API documentation.

## License

MIT
