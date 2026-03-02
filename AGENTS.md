# AGENTS.md - APEX Development Guide

## Project Overview

APEX is a single-user autonomous agent platform combining messaging interfaces with secure code execution. Multi-tenancy is explicitly out of scope.

- **Architecture**: 6-layer system (L1-L6) with Rust core, TypeScript gateway/skills, Python execution, React UI
- **Status**: Phase 6 Complete | Hardening done
- **Repository Structure**: See design doc `docs/APEX-Design.md`

---

## Current Status

### Implemented Components

| Layer | Component | Status | Location |
|-------|-----------|--------|----------|
| L2 | Task Router | ✅ Running | `core/router/` |
| L3 | Memory Service | ✅ Working | `core/memory/` |
| L1 | Gateway | ✅ Built | `gateway/` |
| L4 | Skills Framework | ✅ Built | `skills/` |
| L6 | React UI | ✅ Built | `ui/` |
| L5 | Execution Engine | ✅ Built | `execution/` |
| LLM | Qwen3-4B | ✅ Integrated | llama-server (b8185+) |

### API Endpoints

**Tasks:**
- `POST /api/v1/tasks` - Create task (auto-tiers: Instant→response, Shallow→skill, Deep→LLM)
  - Optional fields: `max_steps`, `budget_usd`, `time_limit_secs`, `project`, `priority`, `category`
- `GET /api/v1/tasks` - List tasks (supports `project`, `status`, `priority`, `category`, `limit`, `offset` filters)
- `GET /api/v1/tasks/filter-options` - Get available filter options (projects, categories, priorities, statuses)
- `GET /api/v1/tasks/:id` - Get task status
- `PUT /api/v1/tasks/:id` - Update task (project, priority, category)
- `POST /api/v1/tasks/:id/cancel` - Cancel task

**Messages:**
- `GET /api/v1/messages` - List messages (supports `limit`, `offset`, `channel` params)
- `GET /api/v1/messages/task/:task_id` - Get messages for a specific task

**Skills:**
- `GET /api/v1/skills` - List all skills
- `GET /api/v1/skills/:name` - Get skill details
- `POST /api/v1/skills` - Register a skill
- `DELETE /api/v1/skills/:name` - Delete a skill
- `PUT /api/v1/skills/:name/health` - Update skill health
- `POST /api/v1/skills/execute` - Execute a skill

**Deep Tasks:**
- `POST /api/v1/deep` - Execute deep task (uses VM pool + LLM)

**System:**
- `GET /api/v1/metrics` - Get metrics (includes total cost)
- `GET /api/v1/vm/stats` - Get VM pool stats
- `GET /health` - Health check
- `GET /` - Root info

### Task Configuration

Task limits (`max_steps`, `budget_usd`, `time_limit_secs`) are configured in **Settings** (not the main screen). They are stored in localStorage and:
- **NOT applied** when using local LLM (`APEX_USE_LLM=1`)
- Time limit enforcement checks `!config.use_llm` in `can_continue()`

### Kanban Board

The UI includes a **Task Board** (Kanban) for managing tasks visually:
- **Columns**: Pending, Running, Completed, Failed, Cancelled
- **Features**:
  - Filter by project
  - Click task to view details
  - Click → buttons to move tasks between columns
  - Auto-refresh every 5 seconds
  - Edit project, priority, and category inline
- **Access**: Click the 📋 icon in the sidebar

### Gateway Adapters

The gateway (`gateway/`) provides adapters for external integrations:
- **REST API** (`src/adapters/rest/`) - Runs on port 3001, proxies to router
- **Slack** (`src/adapters/slack/`) - Slack bot integration
- **Discord** (`src/adapters/discord/`) - Discord bot integration
- **Telegram** (`src/adapters/telegram/`) - Telegram bot integration

---

## Build Commands

### Prerequisites
- Rust 1.93+, Node.js 20+, Python 3.11+, pnpm, Poetry, Docker

### Running the Application

**Recommended: Use apex.bat**

```powershell
# Start all services (llama-server, router with LLM, UI)
.\apex.bat start

# Stop all services
.\apex.bat stop

# Restart
.\apex.bat restart

# Build all
.\apex.bat build
```

**Manual (Alternative)**

```powershell
# Terminal 1 - Start llama-server (requires b8185+, -c 4096 limits context to reduce memory)
D:\Users\ashah\Documents\llama.cpp\llama-server.exe -m "C:\Users\ashah\.ollama\models\Qwen3-4B-Instruct-2507-Q4_K_M.gguf" --port 8080 -c 4096

# Terminal 2 - Start Router (with LLM)
cd core
set APEX_USE_LLM=1
cargo run --release --bin apex-router

# Terminal 3 - Start UI
cd ui && pnpm dev

# Or serve built:
cd ui && npx serve dist -l 8083
```

### Rust (Core Daemon - L2/L3)
```bash
cd core

# Build all workspace crates
cargo build

# Run tests
cargo test

# Run single test
cargo test <test_name>

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

### TypeScript (Gateway/Skills - L1/L4)
```bash
cd gateway  # or skills/

# Install dependencies
pnpm install

# Build
pnpm build

# Run tests
pnpm test
```

### Python (Execution Engine - L5)
```bash
cd execution/

# Install dependencies
poetry install

# Run tests
poetry run pytest

# Lint
poetry run ruff check .
poetry run mypy .
```

### Docker Execution (L5)
```bash
# Build Docker execution image
.\apex.bat docker-build

# Test Docker execution
.\apex.bat docker-test

# Enable Docker execution (set APEX_USE_DOCKER=1)
.\apex.bat router-llm
```

### React UI (L6)
```bash
cd ui/

# Development
pnpm dev

# Build
pnpm build
```

### Full Stack Build
```bash
cargo build && pnpm run build
```

---

## Code Style Guidelines

### General Principles
- **Single-user context**: No user ID fields, no authentication between layers
- **English-only**: All code, comments, error messages in English
- **No multi-tenancy**: Reject any PRs adding multi-user features

### Rust (Core)
- Follow `rustfmt` defaults
- Use `tokio` for async, `axum` for HTTP, `sqlx` for SQLite
- Error handling: Use `thiserror` + `anyhow` pattern
- Naming: `snake_case` functions, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants

### TypeScript (Gateway/Skills)
- Use strict mode (`"strict": true` in tsconfig)
- Use `zod` for runtime validation
- Use `fastify` for HTTP servers
- Naming: `camelCase` variables/functions, `PascalCase` types/classes

### Python (Execution)
- Use `poetry` for dependency management
- Type hints required (strict mypy)
- Use `ruff` for linting

### React UI (L6)
- Use React 18 with TypeScript
- Use `zustand` for state management
- Use Tailwind CSS + Radix UI components
- Use `vite` for bundling

---

## Project Structure

```
apex/
├── core/                    # Rust (L2/L3)
│   ├── router/              # Task Router (HTTP API)
│   │   ├── src/
│   │   │   ├── api.rs      # HTTP endpoints
│   │   │   ├── classifier.rs # Task classification
│   │   │   ├── metrics.rs  # Prometheus metrics
│   │   │   ├── message_bus.rs # Internal message bus
│   │   │   ├── llama.rs    # LLM client (llama-server)
│   │   │   ├── vm_pool.rs  # VM pool manager
│   │   │   ├── agent_loop.rs # Agent execution loop
│   │   │   ├── deep_task_worker.rs # Deep task worker
│   │   │   └── skill_worker.rs # Skill execution worker
│   │   ├── tests/
│   │   │   ├── integration.rs # Integration tests (14)
│   │   │   └── e2e.rs      # E2E tests (2, #[ignore])
│   │   ├── Cargo.toml
│   │   └── run-with-llm.bat # Start router with LLM enabled
│   ├── memory/              # Memory Service (SQLite)
│   │   ├── src/
│   │   │   ├── db.rs       # Database connection
│   │   │   ├── tasks.rs    # Task models
│   │   │   └── task_repo.rs # Task repository
│   │   └── migrations/      # SQL migrations
│   └── security/            # Capability tokens
│
├── gateway/                 # TypeScript (L1)
│   ├── src/
│   │   ├── index.ts        # Gateway service
│   │   └── index.test.ts   # Tests (8)
│   └── tsconfig.json
│
├── skills/                  # TypeScript (L4)
│   ├── src/
│   │   ├── types.ts        # Skill interface
│   │   ├── loader.ts       # Skill loader
│   │   └── loader.test.ts  # Tests (8)
│   └── skills/              # Built-in skills
│       ├── code.generate/
│       ├── code.review/
│       ├── shell.execute/
│       ├── docs.read/
│       └── git.commit/
│
├── ui/                      # React (L6)
│   └── src/
│       ├── App.tsx
│       ├── stores/appStore.ts
│       └── components/
│
└── execution/               # Python (L5)
    └── src/apex_agent/
        └── agent.py         # Agent Zero fork
```

---

## Error Handling

- **Rust**: `Result<T, Error>` with `thiserror` enums. Never panic in production.
- **TypeScript**: Use custom error classes, never throw raw errors
- **Python**: Use custom exceptions inheriting from `Exception`

---

## Testing

### Test Suite

| Component | Tests | Location |
|-----------|-------|----------|
| **Rust unit tests** | 15 | `core/*/src/*_test.rs` or `mod tests` |
| **Rust integration tests** | 14 | `core/router/tests/integration.rs` |
| **Rust e2e tests** | 2 | `core/router/tests/e2e.rs` (run with `-- --ignored`) |
| **Memory tests** | 3 | `core/memory/src/*_test.rs` |
| **Gateway tests** | 8 | `gateway/src/*.test.ts` |
| **Skills tests** | 8 | `skills/src/*.test.ts` |
| **Total** | **48** | |

### Running Tests

```bash
# All Rust tests (unit + integration)
cd core && cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration

# E2E tests (requires stopping any running router)
cargo test --test e2e

# TypeScript tests
cd gateway && pnpm test
cd skills && pnpm test
```

### Test Categories

- **Unit tests**: Test individual functions and modules in isolation
- **Integration tests**: Test API endpoints using in-memory SQLite
- **E2E tests**: Spawn router binary, make real HTTP requests (slow, marked `#[ignore]`)

- Unit tests co-located with source (`*.test.ts`, `*_test.py`)
- Integration tests in `tests/` directory
- Minimum 80% coverage for core modules

---

## Skill System

Skills follow the standard:
```
skill-name/
├── package.json
└── src/index.ts    # Exports: name, version, tier, inputSchema, outputSchema, execute(), healthCheck()
```

- Skills are typed (T0-T3) per permission tier
- All skills require input/output schema validation via zod
- Health checks required for each skill

---

## Security

- No secrets in code; use environment variables or encrypted SQLite
- Firecracker isolation for L5 execution (gVisor fallback in dev)
- Input validation on all external boundaries
- Single-user: localhost-only binding by default

---

## Git Conventions

- Conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`
- No force pushes to main/master
- Keep commits atomic and small

---

## Important Notes

- Phase 1-5 Complete - All core features implemented
- Phase 6 (Hardening) Complete - Security audit, prompt injection defense, CSP headers
- Gateway calls Router API over HTTP (NATS optional)
- Message bus abstraction ready for NATS integration
- All subsystems build and pass linting
- Test suite: 48 tests (15 Rust unit + 14 Rust integration + 3 memory + 8 Gateway + 8 Skills)
- E2E tests spawn router binary and verify HTTP endpoints (run manually with `cargo test --test e2e -- --ignored`)
- Session context: see `docs/SESSION.md`
- Note: `paseto` crate was removed from router (pulled in OpenSSL). Capability tokens use simple base64 encoding in `apex-security`.
- Task limits (max_steps, budget_usd, time_limit_secs) configured in Settings, stored in localStorage
- Time limits NOT enforced when using local LLM (APEX_USE_LLM=1)
- llama-server requires `-c 4096` flag to limit context size and reduce memory usage
- Prompt injection defense: User input is sanitized before sending to LLM
- Logging: Use APEX_JSON_LOGS=1 for JSON formatted logs
