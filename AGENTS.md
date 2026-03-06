# AGENTS.md - APEX Development Guide

> ⚠️ **WARNING: PRE-ALPHA** - This is an experimental research project. Not production ready.

## Project Overview

APEX is a **pre-alpha** single-user autonomous agent platform combining messaging interfaces with secure code execution. Multi-tenancy is explicitly out of scope.

- **Architecture**: 6-layer system (L1-L6) with Rust core, TypeScript gateway/skills, Python execution, React UI
- **Status**: Pre-Alpha (Experimental) ⚠️
- **Version**: v1.3.0
- **Repository Structure**: See design doc `docs/APEX-Design.md`

---

## ⚠️ Pre-Alpha Warnings

- **No security audit** - Do not use with sensitive data
- **Limited testing** - Many features are proof-of-concept
- **API instability** - Breaking changes expected
- **No production support** - Use at your own risk
- **Firecracker/VM isolation** - Requires kernel/rootfs configuration
- **Missing features** - Dynamic tool generation, subagent pool not implemented

---

## Current Status

### Implemented Components (Proof-of-Concept)

| Layer | Component | Status | Location |
|-------|-----------|--------|----------|
| L2 | Task Router | ✅ POC | `core/router/` |
| L3 | Memory Service | ✅ POC | `core/memory/` |
| L1 | Gateway | ✅ Built | `gateway/` |
| L4 | Skills Framework | ✅ Built | `skills/` |
| L6 | React UI | ✅ Built | `ui/` |
| L5 | Execution Engine | ✅ Docker | `execution/` |
| LLM | Qwen3-4B | ✅ Optional | llama-server |

### Update Plan Progress
- **Phase 1: Security & Permissions** ✅ Complete (v0.1.1 - HMAC auth, TOTP)
- **Phase 2: Core Skills** ✅ Complete (28 skills)
- **Phase 3: Messaging Adapters** ✅ Complete
- **Phase 4: Execution Engine** ✅ Complete
- **Phase 5: UI Enhancements** ✅ Complete (WebSocket, TaskSidebar, ProcessGroup)
- **Phase 6: Advanced Features** ✅ Complete
- **Phase 7: UI Overhaul** ✅ Complete (Settings tabs, Memory tabs, Workflows, Audit)
- **Phase 8: Future Features** ✅ Complete (v0.1.2 - Channels, Journal, WebSocket Server, NATS)
- **Phase 9: v0.2.0 Upgrade** ✅ Complete (Firecracker, Agent Zero loop, SKILL.md plugins, PostgreSQL, Config files)
- **Phase 10: Social Context** ✅ Complete (Moltbook Integration)
- **Phase 11: Governance & Constitution** ✅ Complete

### Recent Optimizations
- **API Modularization** ✅ Complete - Split 1556-line monolithic `api.rs` into 9 modular files in `core/router/src/api/`
- **Database Optimization** ✅ Complete - Added composite indexes (012 migration) for common filter queries
- **Startup Config Validation** ✅ Complete - Added validation at router startup
- **Worker Supervision** ✅ Complete - Added supervised restart loop to all workers (skill_worker, deep_task_worker, t3_confirm_worker)
- **Transaction Boundaries** ✅ Complete - Added atomic task update + decision journal writes in deep_task_worker

### v0.3.0 New Features
- **Real-time Agent Thoughts Streaming** - Execution events stream to UI via WebSocket
- **Consequence Preview** - Blast radius shown before T2/T3 actions
- **Runtime Tool Generation** - Agent generates custom Python tools via LLM when needed
- **TIR (Tool-Integrated Reasoning)** - LLM returns interleaved Thought/Action/Observation
- **Subagent Pool** - Complex tasks split into parallelizable subtasks
- **SOUL.md Identity System** - Agent reads identity file on wake
- **Heartbeat Daemon** - Autonomous wake cycles with configurable intervals

### Skills Registry (33 Total)
- T0 (Read-only): 3 skills
- T1 (Tap confirm): 11 skills
- T2 (Type confirm): 8 skills
- T3 (TOTP verification): 1 skill (shell.execute)
- Note: shell.execute moved from T2 to T3 per security audit

### API Endpoints

**Tasks:**
- `POST /api/v1/tasks` - Create task (auto-tiers: Instant→response, Shallow→skill, Deep→LLM)
  - Optional fields: `max_steps`, `budget_usd`, `time_limit_secs`, `project`, `priority`, `category`
- `GET /api/v1/tasks` - List tasks (supports `project`, `status`, `priority`, `category`, `limit`, `offset` filters)
- `GET /api/v1/tasks/filter-options` - Get available filter options (projects, categories, priorities, statuses)
- `GET /api/v1/tasks/:id` - Get task status
- `PUT /api/v1/tasks/:id` - Update task (project, priority, category, status)
- `POST /api/v1/tasks/:id/cancel` - Cancel task
- `POST /api/v1/tasks/:id/confirm` - Confirm task (for T1-T3 permission tiers)

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

**TOTP (T3 Verification):**
- `POST /api/v1/totp/setup` - Generate TOTP secret for user
- `POST /api/v1/totp/verify` - Verify TOTP token
- `GET /api/v1/totp/status` - Check if TOTP is configured

**Events:**
- `GET /api/v1/events` - Server-Sent Events stream (for real-time updates)
- `GET /api/v1/ws` - WebSocket endpoint for real-time task updates

**Channels:**
- `GET /api/v1/channels` - List all channels
- `POST /api/v1/channels` - Create a channel
- `GET /api/v1/channels/:id` - Get channel details
- `PUT /api/v1/channels/:id` - Update channel
- `DELETE /api/v1/channels/:id` - Delete channel

**Decision Journal:**
- `GET /api/v1/journal` - List journal entries (supports `limit`, `offset`)
- `POST /api/v1/journal` - Create journal entry
- `GET /api/v1/journal/:id` - Get journal entry
- `PUT /api/v1/journal/:id` - Update journal entry
- `DELETE /api/v1/journal/:id` - Delete journal entry
- `GET /api/v1/journal/search?q=query` - Search journal entries

**SOUL Identity:**
- `GET /api/v1/soul` - Get SOUL identity
- `PUT /api/v1/soul` - Update SOUL identity (with auto-backup)
- `GET /api/v1/soul/fragments` - Get modular identity fragments

**Heartbeat/Autonomy:**
- `GET /api/v1/heartbeat/config` - Get heartbeat configuration
- `POST /api/v1/heartbeat/config` - Update heartbeat configuration
- `GET /api/v1/heartbeat/stats` - Get heartbeat statistics
- `POST /api/v1/heartbeat/trigger` - Trigger manual wake cycle
- `POST /api/v1/heartbeat/toggle` - Enable/disable heartbeat daemon

**Narrative Memory:**
- `GET /api/v1/memory/stats` - Get memory statistics
- `GET /api/v1/memory/journal` - Get journal entries
- `GET /api/v1/memory/entities` - Get entities
- `GET /api/v1/memory/knowledge` - Get knowledge items
- `GET /api/v1/memory/reflections` - Get reflections
- `POST /api/v1/memory/reflections` - Add a reflection

**Moltbook Social:**
- `GET /api/v1/moltbook/status` - Get Moltbook connection status
- `GET /api/v1/moltbook/agents` - Get agent directory
- `POST /api/v1/moltbook/connect` - Connect to Moltbook
- `POST /api/v1/moltbook/disconnect` - Disconnect from Moltbook
- `GET /api/v1/social/profile` - Get social profile
- `POST /api/v1/social/post` - Create social post
- `GET /api/v1/social/notifications` - Get notifications
- `GET /api/v1/social/agents/search?q=query` - Search agents
- `GET /api/v1/social/agents/directory` - Get agent directory
- `GET /api/v1/social/trust?agent_id=id` - Assess trust

**Governance:**
- `GET /api/v1/governance/policy` - Get governance policy
- `POST /api/v1/governance/check` - Check action allowed
- `GET /api/v1/governance/immutable` - Get immutable values
- `GET /api/v1/governance/emergency` - Get emergency protocols
- `POST /api/v1/governance/oracle` - Toggle oracle mode
- `GET /api/v1/memory/reflections` - Get reflections
- `POST /api/v1/memory/reflections` - Add a reflection

**System:**
- `GET /api/v1/metrics` - Get metrics (includes total cost)
- `GET /api/v1/system/health` - Get system health
- `GET /api/v1/system/cache` - Get cache statistics
- `DELETE /api/v1/system/cache` - Clear cache
- `GET /api/v1/system/ratelimit` - Get rate limit stats
- `GET /api/v1/vm/stats` - Get VM pool stats
- `GET /api/v1/skills/pool/stats` - Get Skill Pool stats (latency, errors, slot availability)
- `GET /health` - Health check
- `GET /` - Root info

### Authentication

**HMAC Request Signing:**
- All API requests require `X-APEX-Signature` and `X-APEX-Timestamp` headers
- Signature = HMAC-SHA256(timestamp + method + path + body)
- Timestamp must be within 5 minutes to prevent replay attacks
- Set `APEX_AUTH_DISABLED=1` for local development
- Set `APEX_SHARED_SECRET` environment variable for production

**Environment Variables:**
- `APEX_SHARED_SECRET` - Secret key for HMAC signing
- `APEX_AUTH_DISABLED` - Disable authentication (dev only)
- `APEX_NATS_ENABLED` - Enable NATS for distributed deployment
- `APEX_NATS_URL` - NATS server URL (default: 127.0.0.1:4222)
- `APEX_NATS_SUBJECT_PREFIX` - NATS subject prefix (default: apex)

### Unified Configuration System

APEX v0.2.0 uses a unified configuration system via `AppConfig` in `core/router/src/unified_config.rs`. All configuration goes through `AppConfig::global()`.

**Configuration API Endpoints:**
- `GET /api/v1/config` - Get all configuration variables
- `GET /api/v1/config/summary` - Get configuration summary with validation

**View Configuration in UI:**
- Settings → Config tab shows all runtime configuration

### Complete Environment Variables Reference

| Variable | Description | Default |
-|---------|
||----------|------------ **Server** | | |
| `APEX_PORT` | Router HTTP port | 3000 |
| `APEX_HOST` | Router host | 0.0.0.0 |
| **Authentication** | | |
| `APEX_SHARED_SECRET` | HMAC signing secret | dev-secret-change-in-production |
| `APEX_AUTH_DISABLED` | Disable auth (set to 1) | false |
| **LLM/Agent** | | |
| `APEX_USE_LLM` | Enable LLM (set to 1) | false (development mode) |
| `LLAMA_SERVER_URL` | llama-server URL | http://localhost:8080 |
| `LLAMA_MODEL` | Model name | qwen3-4b |

> **Development Mode**: By default, APEX runs in development mode where the local LLM is disabled to avoid unnecessary LLM usage during development. Enable LLM via the Settings → LLM tab in the UI, or set `APEX_USE_LLM=1` environment variable when testing LLM-powered features.

> **Embedding Server**: For semantic memory search, run a separate llama-server instance on port 8081 with nomic-embed-text model loaded:
> 
> Using LM Studio (recommended):
> ```
> LM Studio → Select model → nomic-embed-text-v1.5.Q4_K_M.gguf → Start Server → Enable Embeddings
> ```
> 
> Or using llama-server directly:
> ```
> llama-server --model "C:\Program Files\LM Studio\resources\app\.webpack\bin\bundled-models\nomic-ai\omic-embed-text-v1.5-GGUF\omic-embed-text-v1.5.Q4_K_M.gguf" --embedding --port 8081
> ```

| **Execution** | | |
| `APEX_EXECUTION_ISOLATION` | Isolation backend: docker, firecracker, gvisor, mock | docker |
| `APEX_USE_DOCKER` | Enable Docker execution (legacy) | true (if isolation=docker) |
| `APEX_DOCKER_IMAGE` | Docker image | apex-execution:latest |
| `APEX_USE_FIRECRACKER` | Enable Firecracker VMs (Linux only) | false |
| `APEX_FIRECRACKER_PATH` | firecracker binary path | system PATH |
| `APEX_VM_VCPU` | VM vCPU count | 2 |
| `APEX_VM_MEMORY_MIB` | VM memory in MiB | 2048 |
| `APEX_VM_KERNEL` | Linux kernel path | - |
| `APEX_VM_ROOTFS` | Root filesystem path | - |
| `APEX_VM_NETWORK_ISOLATION` | Network isolation mode | none |
| `APEX_VM_FAST_BOOT` | Enable fast boot | false |
| `APEX_USE_JAILER` | Use jailer with Firecracker | false |
| `APEX_USE_GVISOR` | Enable gVisor sandbox (Linux only) | false |
| `APEX_RUNSC_PATH` | runsc binary path | system PATH |
| **Database** | | |
| `APEX_DATABASE_URL` | Database connection string | sqlite:apex.db |
| `DATABASE_URL` | Fallback DB URL | sqlite:apex.db |
| `APEX_DB_MAX_CONNECTIONS` | Max pool connections | 10 |
| `APEX_DB_MIN_CONNECTIONS` | Min pool connections | 1 |
| **NATS** | | |
| `APEX_NATS_ENABLED` | Enable NATS | false |
| `APEX_NATS_URL` | NATS server URL | 127.0.0.1:4222 |
| `APEX_NATS_SUBJECT_PREFIX` | Subject prefix | apex |
| **Logging** | | |
| `APEX_JSON_LOGS` | JSON formatted logs (set to 1) | false |
| `APEX_LOG_LEVEL` | Log level | info |
| **Skills** | | |
| `APEX_SKILLS_CLI` | Skills CLI path | - |
| `APEX_SKILLS_DIR` | Skills directory | ./skills |
| **Skill Pool** | | |
| `APEX_SKILL_POOL_ENABLED` | Enable skill pool (set to 0 to disable) | true |
| `APEX_SKILL_POOL_SIZE` | Number of pre-warmed workers | 4 |
| `APEX_SKILL_POOL_WORKER` | Path to Bun dispatcher script | ./skills/pool_worker.ts |
| `APEX_SKILL_POOL_TIMEOUT` | Request timeout (ms) | 30000 |
| `APEX_SKILL_POOL_ACQUIRE` | Slot acquire timeout (ms) | 5000 |
| **Soul/Identity** | | |
| `APEX_SOUL_DIR` | Soul directory | ~/.apex/soul |
| `APEX_SOUL_BACKUP` | Enable soul backups (set to 1) | false |
| **Heartbeat** | | |
| `APEX_HEARTBEAT_ENABLED` | Enable heartbeat daemon | false |
| `APEX_HEARTBEAT_INTERVAL` | Interval in minutes | 60 |
| `APEX_HEARTBEAT_JITTER` | Jitter percentage | 10 |
| `APEX_HEARTBEAT_COOLDOWN` | Cooldown in seconds | 300 |
| `APEX_HEARTBEAT_MAX_ACTIONS` | Max actions per wake | 3 |
| **Memory (Enhanced)** | | |
| `APEX_MEMORY_EMBEDDING_PROVIDER` | Embedding provider: local \| openai | local |
| `APEX_MEMORY_EMBEDDING_MODEL` | Embedding model | nomic-embed-text |
| `APEX_MEMORY_EMBEDDING_URL` | Embedding server URL | http://localhost:8081 |
| `APEX_MEMORY_EMBEDDING_DIM` | Embedding dimension (768 local, 1536 OpenAI) | 768 |
| `APEX_MEMORY_RRF_K` | RRF constant | 60 |
| `APEX_MEMORY_MAX_RESULTS` | Max search results | 8 |
| `APEX_MEMORY_MMR_LAMBDA` | MMR lambda (0-1) | 0.7 |
| `APEX_MEMORY_HALF_LIFE_DAYS` | Temporal decay half-life | 30 |
| `APEX_MEMORY_CHUNK_SIZE` | Chunk size in tokens | 256 |
| `APEX_MEMORY_CHUNK_OVERLAP` | Chunk overlap in tokens | 32 |
| `APEX_MEMORY_EMBED_RATE_LIMIT_MS` | Embedding rate limit (ms) | 50 |
| `APEX_MEMORY_INDEXER_BATCH_SIZE` | Indexer batch size | 16 |
| **Moltbook** | | |
| `APEX_MOLTBOOK_AGENT_ID` | Moltbook agent ID | - |

### Permission Tiers

APEX implements T0-T3 permission tiers for security:
- **T0**: Read-only queries - no confirmation needed
- **T1**: File writes, drafts - tap to confirm
- **T2**: External API calls, git push - type to confirm
- **T3**: Destructive operations - TOTP verification required

### Kanban Board

The UI includes a **Task Board** (Kanban) for managing tasks visually:
- **Columns**: Pending, Running, Completed, Failed, Cancelled
- **Features**:
  - Filter by project
  - Click task to view details
  - Click → buttons to move tasks between columns
  - Click ▶ Run to execute pending tasks
  - Auto-refresh every 5 seconds
  - Edit project, priority, and category inline
  - "+ New Task" button to create tasks directly
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

# Start all services INCLUDING embedding server (for memory search)
.\apex.bat start-full

# Stop all services
.\apex.bat stop

# Restart
.\apex.bat restart

# Build all
.\apex.bat build
```

**Individual Services:**

```powershell
.\apex.bat llama          # Start LLM server (port 8080)
.\apex.bat embed         # Start embedding server (port 8081)
.\apex.bat embed-test    # Test embedding server
.\apex.bat router         # Start router (no LLM)
.\apex.bat router-llm     # Start router with LLM
.\apex.bat ui             # Start UI dev server
.\apex.bat ui-serve       # Serve built UI
.\apex.bat status         # Show all service status
```

**Manual (Alternative)**

```powershell
# Terminal 1 - Start llama-server for LLM (requires b8185+, -c 4096 limits context)
D:\Users\ashah\Documents\llama.cpp\llama-server.exe -m "C:\Users\ashah\.ollama\models\Qwen3-4B-Instruct-2507-Q4_K_M.gguf" --port 8080 -c 4096

# Terminal 1b - Start llama-server for embeddings (nomic-embed-text)
# Only needed for memory search feature
"C:\Program Files\LM Studio\resources\app\.webpack\bin\bundled-models\nomic-ai\omic-embed-text-v1.5-GGUF\omic-embed-text-v1.5.Q4_K_M.gguf" --embedding --port 8081

# Terminal 2 - Start Router (with LLM)
cd core
set APEX_USE_LLM=1
set APEX_MEMORY_EMBEDDING_URL=http://localhost:8081
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

# Start router with Docker isolation (without LLM)
.\apex.bat router-docker

# Start router with LLM and Docker execution
.\apex.bat router-llm
```

**Important**: On Windows, use `--privileged=false` (with equals) not `--privileged false`.

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
- **Authentication**: HMAC-signed requests between Gateway/Router/UI

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
- Use `@tanstack/react-query` for server state
- Use `socket.io-client` for WebSocket
- Use `framer-motion` for animations

---

## Project Structure

```
apex/
├── core/                    # Rust (L2/L3)
│   ├── router/              # Task Router (HTTP API)
│   │   ├── src/
│   │   │   ├── api/        # Modular API endpoints
│   │   │   │   ├── mod.rs       # Router composer
│   │   │   │   ├── tasks.rs      # Task endpoints (6)
│   │   │   │   ├── skills.rs    # Skill endpoints (6)
│   │   │   │   ├── workflows.rs # Workflow endpoints (6)
│   │   │   │   ├── notifications.rs # Notification endpoints (7)
│   │   │   │   ├── webhooks.rs  # Webhook endpoints (5)
│   │   │   │   ├── adapters.rs  # Adapter endpoints (4)
│   │   │   │   ├── memory.rs    # Memory endpoints (4)
│   │   │   │   └── system.rs    # System endpoints (4)
│   │   │   ├── auth.rs      # HMAC authentication middleware
│   │   │   ├── totp.rs      # TOTP verification
│   │   │   ├── classifier.rs # Task classification
│   │   │   ├── metrics.rs   # Prometheus metrics
│   │   │   ├── message_bus.rs # Internal message bus
│   │   │   ├── llama.rs     # LLM client (llama-server)
│   │   │   ├── vm_pool.rs   # VM pool manager (Docker/Firecracker)
│   │   │   ├── agent_loop.rs # Agent execution loop
│   │   │   ├── deep_task_worker.rs # Deep task worker
│   │   │   ├── skill_worker.rs # Skill execution worker
│   │   │   └── t3_confirm_worker.rs # T3 confirmation handler
│   │   ├── tests/
│   │   │   ├── integration.rs # Integration tests (41)
│   │   │   └── e2e.rs      # E2E tests (2, #[ignore])
│   │   └── Cargo.toml
│   ├── memory/              # Memory Service (SQLite)
│   │   ├── src/
│   │   │   ├── db.rs        # Database connection
│   │   │   ├── tasks.rs     # Task models
│   │   │   └── task_repo.rs # Task repository
│   │   └── migrations/      # SQL migrations (012)
│   └── security/            # Capability tokens
│
├── gateway/                 # TypeScript (L1)
│   ├── src/
│   │   ├── index.ts        # Gateway service with HMAC signing
│   │   └── index.test.ts   # Tests
│   └── tsconfig.json
│
├── skills/                  # TypeScript (L4)
│   ├── src/
│   │   ├── types.ts        # Skill interface
│   │   ├── loader.ts       # Skill loader
│   │   ├── utils.ts       # Shared utilities
│   │   └── loader.test.ts  # Tests
│   └── skills/              # Built-in skills
│       ├── shell.execute/  # T3 - Shell execution
│       ├── code.generate/
│       ├── code.review/    # T0 - Code review
│       ├── git.commit/
│       └── ...
│
├── ui/                      # React (L6)
│   └── src/
│       ├── App.tsx         # Main app with header, task count, budget
│       ├── stores/
│       │   └── appStore.ts # Zustand store with WebSocket state
│       ├── lib/
│       │   ├── api.ts      # Signed fetch utilities
│       │   └── websocket.ts # WebSocket client
│       └── components/
│           ├── chat/
│           │   ├── Chat.tsx        # Main chat with TaskSidebar
│           │   ├── TaskSidebar.tsx # Active tasks panel
│           │   ├── ProcessGroup.tsx # Task execution trace
│           │   └── ConfirmationGate.tsx # T1-T3 inline confirmation
│           ├── kanban/
│           ├── skills/
│           ├── memory/
│           ├── workflows/
│           ├── audit/
│           └── settings/
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
| **Rust unit tests** | 70 | `core/*/src/*_test.rs` or `mod tests` |
| **Rust integration tests** | 41 | `core/router/tests/` |
| **Rust e2e tests** | 2 | `core/router/tests/e2e.rs` (run with `-- --ignored`) |
| **Gateway tests** | 8 | `gateway/src/*.test.ts` |
| **Skills tests** | 8 | `skills/src/*.test.ts` |
| **UI tests** | 23 | `ui/src/**/*.test.tsx` |
| **Total** | **150** | |

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

# UI tests
cd ui && pnpm test
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
- **Security**: shell.execute is T3 (requires TOTP), not T2

---

## Security

- **HMAC Authentication**: All API requests signed with shared secret
- **TOTP Verification**: T3 tasks require TOTP code before execution
- **No secrets in code**: Use environment variables or encrypted SQLite
- **Firecracker isolation**: For L5 execution (gVisor fallback in dev)
- **Input validation**: On all external boundaries
- **Single-user**: localhost-only binding by default

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `APEX_SHARED_SECRET` | HMAC signing secret | `dev-secret-change-in-production` |
| `APEX_AUTH_DISABLED` | Disable auth (dev only) | not set |
| `APEX_USE_FIRECRACKER` | Enable Firecracker VMs | false |
| `APEX_USE_GVISOR` | Enable gVisor | false |
| `APEX_USE_DOCKER` | Enable Docker execution | false |

---

## Git Conventions

- Conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`
- No force pushes to main/master
- Keep commits atomic and small

---

## UI Features (v0.1.1)

### Real-Time Updates
- WebSocket client with automatic reconnection
- Polling fallback when WebSocket unavailable
- Connection state indicator: Connected / Degraded / Disconnected

### Task Sidebar
- Right panel showing active and recent tasks
- Status icons with color coding
- Elapsed time and cost display

### Process Groups
- Collapsible task execution traces
- Step badges: GEN (LLM), USE (skill), EXE (code), WWW (web), SUB (subagent), MEM (memory), AUD (audit)
- Expandable step details with input/output

### Inline Confirmation Gates
- T1: Tap to confirm
- T2: Type to confirm (type action text)
- T3: TOTP verification (6-digit code from authenticator app)

### Budget Ticker
- Live session cost in header
- Click to view cost details

### Navigation Tabs
- Chat, Skills, Memory, Files, Board (Kanban), Workflows, Audit, **Channels**, **Journal**, Settings
- Keyboard shortcuts: Ctrl+1-10 for navigation

### Channel Management
- Create, edit, delete conversation channels
- List view with descriptions
- Default channels: default, general

### Decision Journal
- Document and track decisions
- Fields: title, context, decision, rationale, outcome, tags
- Link decisions to tasks
- Search functionality

### Responsive Design
- Desktop: Full sidebar + main content
- Mobile: Bottom navigation bar
- Collapsible sidebar

### Theme Support
- Dark/Light mode toggle in header
- OS preference detection on first load
- Persisted to localStorage

---

## Important Notes

- All core features implemented and tested
- Security audit fixes applied (HMAC auth, TOTP, shell.execute T3)
- Gateway → Router calls require HMAC signature
- UI → Router calls require HMAC signature  
- T3 tasks require TOTP verification
- All subsystems build and pass linting
- Test suite: 150 tests (70 Rust unit + 41 Rust integration + 8 Gateway + 8 Skills + 23 UI)
- E2E tests spawn router binary and verify HTTP endpoints
- Session context: see `docs/SESSION.md`
- Task limits (max_steps, budget_usd, time_limit_secs) configured in Settings, stored in localStorage
- Time limits NOT enforced when using local LLM (APEX_USE_LLM=1)
- llama-server requires `-c 4096` flag to limit context size and reduce memory usage
- Prompt injection defense: User input is sanitized before sending to LLM
- Logging: Use APEX_JSON_LOGS=1 for JSON formatted logs
- Unified config: All settings managed via `AppConfig::global()`, see Settings → Config tab
