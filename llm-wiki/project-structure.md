# Project Structure

## Overview

APEX is organized into multiple layers with clear separation of concerns.

## Directory Structure

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
├── llm-wiki/                # LLM Wiki (this directory)
├── docs/                    # Documentation
├── scripts/                 # Deployment scripts
├── ci/                      # CI/CD scripts
├── infra/                   # Infrastructure configs
└── .github/workflows/       # GitHub Actions
```

## Layer Architecture

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **L1** | TypeScript (Fastify) | Gateway with HMAC-signed requests |
| **L2** | Rust (Axum, Tokio) | Task routing, classification, agent loop |
| **L3** | Rust (SQLite, sqlx) | Memory service, bounded memory, search |
| **L4** | TypeScript | Skills framework, MCP client/server |
| **L5** | Python (Docker) | Secure execution engine with sandboxing |
| **L6** | React 18 + TypeScript | UI with 4 themes, real-time streaming |

## Key Files

### Core (Rust)
- `core/router/src/main.rs` - Entry point
- `core/router/src/agent_loop.rs` - Agent execution
- `core/router/src/api/mod.rs` - API router composition
- `core/memory/src/db.rs` - Database operations

### Gateway (TypeScript)
- `gateway/src/index.ts` - Gateway service with HMAC signing

### Skills (TypeScript)
- `skills/src/loader.ts` - Skill loader
- `skills/skills/` - Built-in skills directory

### UI (React)
- `ui/src/App.tsx` - Main application
- `ui/src/components/chat/` - Chat components
- `ui/src/stores/appStore.ts` - Zustand store

### Execution (Python)
- `execution/src/apex_agent/agent.py` - Agent execution

## Build Commands

### Rust
```bash
cd core && cargo build
cd core && cargo test
cd core && cargo clippy -- -D warnings
cd core && cargo bench
```

### TypeScript
```bash
cd gateway && pnpm build
cd skills && pnpm build
cd ui && pnpm build
```

### Python
```bash
cd execution && poetry install
cd execution && poetry run pytest
```

### Full Stack
```bash
cargo build && pnpm run build
```

## Testing

- 583+ tests across all layers
- See [testing.md](testing.md) for details

## Documentation

- [AGENTS.md](../AGENTS.md) - Development guide
- [docs/](../docs/) - Full documentation directory
- [llm-wiki/](../llm-wiki/) - This wiki