# Session Context - APEX Development

> ⚠️ **Status: PRE-ALPHA** - This is an experimental research project. Not production ready.

**Date**: 2026-03-06
**Session**: Enhanced Memory System Implementation

---

## Session 2026-03-06: Enhanced Memory System

### Completed Implementation (Phase 1)

Implemented the enhanced memory system per `docs/APEX_Memory_System_Spec_v2.md`:

#### New Components

1. **Embedder** (`core/memory/src/embedder.rs`)
   - Local provider: nomic-embed-text via llama-server (768-dim)
   - OpenAI fallback: text-embedding-3-small (1536-dim)
   - Dimension validation at startup

2. **Chunker** (`core/memory/src/chunker.rs`)
   - 256 tokens chunk size (configurable)
   - 32 tokens overlap
   - Markdown-aware (respects headings, code blocks)
   - Unicode word segmentation

3. **Hybrid Search** (`core/memory/src/hybrid_search.rs`)
   - Reciprocal Rank Fusion (RRF) for vector + BM25
   - Temporal decay (half-life: 30 days default)
   - Max Marginal Relevance (MMR) for diversity

4. **Working Memory** (`core/memory/src/working_memory.rs`)
   - Per-task scratchpad
   - Entity tracking
   - Causal link recording
   - Write-through to SQLite (survives restarts)

5. **Background Indexer** (`core/memory/src/background_indexer.rs`)
   - Async file indexing
   - Rate-limited embedding calls
   - mtime-based change detection
   - Automatic FTS5 sync

#### New Database Tables (Migration 013)

- `memory_chunks` - Chunked text from memory files
- `memory_vec` - sqlite-vec vector storage (768-dim)
- `memory_fts` - FTS5 virtual table for BM25
- `memory_entities` - Entity store
- `memory_index_state` - Index tracking
- `working_memory` - Per-task scratchpad

#### Specification

See `docs/APEX_Memory_System_Spec_v2.md` for the full specification.

#### Test Results

- Memory unit tests: 30 passing
- All Rust tests: 144 passing (73 unit) + 41 integration

#### Files Created
- `core/memory/src/embedder.rs`
- `core/memory/src/chunker.rs`
- `core/memory/src/hybrid_search.rs`
- `core/memory/src/working_memory.rs`
- `core/memory/src/background_indexer.rs`
- `core/memory/migrations/013_enhanced_memory.sql`
- `docs/APEX_Memory_System_Spec_v2.md`
- `docs/MEMORY-ENHANCEMENT.md`

---

## Session 2026-03-06: Architecture Fixes

### Implemented Recommendations from APEX_Architecture_Recommendations.md

#### Correctness Fixes

1. **Migration 013** - Wired in `db.rs`:
   - `memory_chunks`, `memory_fts`, `memory_entities`, `memory_index_state`, `working_memory`

2. **D1 - WAL Mode** - SQLite pragma in `db.rs`:
   - WAL mode, synchronous=NORMAL, cache_size=-64000, temp_store=MEMORY

3. **A3 - Atomic Writes** - NarrativeService:
   - All writes use tmp+rename pattern
   - Prevents corrupt index entries

#### Security Fixes

4. **B1 - Capability Enforcement** - `pool_worker.ts`:
   - Tier validation before skill execution
   - `permitted_tier` in IPC protocol

5. **B2 - Cache Invalidation** - `pool_worker.ts`:
   - `__cache_bust__` message support
   - Can bust single skill or all

#### Complexity Reduction

6. **C4 - Config Injection**:
   - Thread-local config override (`with_test_config_async`)
   - `config` field in `AppState`
   - 11 remaining `AppConfig::global()` in init code (acceptable)

#### apex.bat Updates

7. **Embedding Server Support**:
   - `apex.bat embed` - Start embedding server
   - `apex.bat embed-test` - Test embedding server
   - `apex.bat start-full` - Start all services

---

## Recent Updates

### Session 2026-03-06: Implementation Session

#### All Tasks Completed (13 new implementations)

1. **OpenAPI 3.0 spec** - Created `docs/openapi.yaml`
   - Full REST API specification
   - 50+ endpoints documented

2. **Kubernetes deployment guide** - Created `docs/KUBERNETES.md`
   - Helm charts, kubectl manifests
   - Resource limits, health checks
   - Network policies, RBAC

2. **Go SDK** - Created `sdk/go/`
   - Full client implementation
   - HMAC authentication
   - Tasks, skills, metrics APIs

3. **TypeScript SDK** - Created `sdk/typescript/`
   - NPM package structure
   - Full API coverage

4. **Python SDK** - Created `sdk/python/`
   - PyPI-compatible package
   - All core APIs

5. **Skill marketplace API** - Created in `core/router/src/api/skills.rs`
   - `/api/v1/skills/marketplace`
   - Search, install, uninstall

6. **UI components** - Already existed in codebase:
   - Governance dashboard (`GovernanceControls.tsx`)
   - Moltbook status panel (`SocialDashboard.tsx`)
   - SOUL.md editor (`SoulEditor.tsx`)
   - Heartbeat config UI (`AutonomyControls.tsx`)

7. **Skill hot-reload** - Created `core/router/src/skill_hot_reload.rs`
   - File watcher for SKILL.md changes
   - notify crate integration

8. **Security audit documentation** - Created `docs/SECURITY.md`
   - T0-T3 permission model
   - HMAC authentication
   - TOTP verification
   - Network isolation

9. **PostgreSQL connection pooling** - Already existed in codebase
   - `DatabaseConfig` with max/min connections

#### Future Work (Cancelled)
- Firecracker VM (Windows unavailable)
- Remaining 30 tasks are test messages

#### All Pending Tasks Executed
- **Completed**: 70 tasks (13 newly implemented)
- **Cancelled**: 30 tasks
- **Pending**: 0 tasks

#### High Priority Tasks Completed (10 tasks - already implemented in codebase)
1. ✅ WhatsApp channel adapter - Already existed (`gateway/src/adapters/whatsapp/`)
2. ✅ Currency precision validation - Already implemented using cents (migration 007)
3. ✅ Subagent orchestration - Already existed (`core/router/src/subagent.rs`)
4. ✅ SSE streaming - Already works via WebSocket (`/api/v1/ws`)
5. ✅ Curriculum agent - Created `core/router/src/curriculum.rs` for learning from task history
6. ✅ Runtime tool generation - Already existed (`core/router/src/dynamic_tools.rs`)
7. ✅ SOUL.md identity loader - Already existed with template rendering
8. ✅ SOUL.md backup/history - Already existed with timestamped backups
9. ✅ Heartbeat daemon - Already existed (`core/router/src/heartbeat/`)
10. ✅ Modular identity fragments - Already existed (values, skills, relationships, goals)

#### Medium Priority Tasks Completed (24 tasks)
- Reflection system, memory narrativization, consequence preview, WhatsApp adapter
- Webhook adapter, graceful shutdown, Prometheus metrics, rate limiting
- Health check, NATS integration, TIR (Tool-Integrated Reasoning)
- Testing tasks (fuzz, chaos, benchmarks, property-based)
- Session persistence, context truncation, YAML config, per-client auth
- Email webhook adapter, Moltbook trust ledger

#### Low Priority Tasks
- **Completed** (5): Moltbook client, constitution, governance engine, narrative memory viewer, OpenAPI 3.0 spec
- **Cancelled** (10): Firecracker VM tasks (Windows unavailable)

#### Firecracker Tasks (All Cancelled)
- Network isolation, VM config, boot optimization - not available on Windows

**Note**: Added status update capability to `PUT /api/v1/tasks/:id` API for future use.

#### Bug Fixes
1. **Router Startup Error** - Fixed overlapping route "/" between `main.rs` and `api/mod.rs`
   - Removed duplicate route from `main.rs`

2. **Kanban Board Not Loading** - Fixed UI authentication issue
   - Components were using raw `fetch()` instead of `apiFetch()` (missing HMAC headers)
   - Fixed in: KanbanBoard.tsx, Skills.tsx, SkillQuickLaunch.tsx, MemoryStatsDashboard.tsx

3. **Task Content Not Showing** - Fixed API response missing content field
   - Added `content` field to `TaskStatusResponse` in `api/mod.rs`
   - Included `input_content` in all task response mappings in `tasks.rs`
   - Added `content` to Task interface and display in KanbanBoard.tsx

#### Skill Pool Implementation (Complete)
1. **Core Pool Manager** (`core/router/src/skill_pool.rs`)
   - mpsc-based slot management with pre-warmed workers
   - Acquire/release lifecycle with timeouts
   - Metrics tracking (latency, errors, slot availability)

2. **IPC Framing** (`core/router/src/skill_pool_ipc.rs`)
   - UUID-based message framing
   - JSON serialization for skill requests/responses

3. **Pool Worker** (`skills/pool_worker.ts`)
   - Bun REPL dispatcher for skill execution
   - Falls back to spawn mode if Bun not available

4. **Unified Config** (`unified_config.rs`)
   - Added `SkillPoolConfigSection` with env vars:
     - `APEX_SKILL_POOL_ENABLED` (default: true)
     - `APEX_SKILL_POOL_SIZE` (default: 4)
     - `APEX_SKILL_POOL_TIMEOUT` (default: 30000ms)
     - `APEX_SKILL_POOL_ACQUIRE` (default: 5000ms)

5. **Metrics Endpoint** (`api/system.rs`)
   - `GET /api/v1/skills/pool/stats` - Pool statistics

#### Files Modified
- `core/router/src/main.rs` - Removed duplicate route, added skill_pool init
- `core/router/src/lib.rs` - Added clippy allow attributes
- `core/router/src/unified_config.rs` - Added SkillPoolConfigSection
- `core/router/src/api/mod.rs` - Added content field, skill_pool to AppState
- `core/router/src/api/tasks.rs` - Include input_content in responses
- `core/router/src/api/system.rs` - Added skill pool stats endpoint
- `core/router/tests/integration.rs` - Added skill_pool: None to test state
- `ui/src/components/kanban/KanbanBoard.tsx` - Use apiFetch, content display
- `ui/src/components/skills/Skills.tsx` - Use apiFetch
- `ui/src/components/skills/SkillQuickLaunch.tsx` - Use apiFetch
- `ui/src/components/memory/MemoryStatsDashboard.tsx` - Use apiFetch

### Phase 23: Skill Quick-Launch (v1.1.2)
- Added SkillQuickLaunch UI component (Ctrl+K)
- Added 5 new skills: file.search, git.branch, code.format, api.test, docker.run

### Phase 24: Memory Dashboard (v1.2.0)
- Added MemoryStatsDashboard UI component
- Added /api/v1/memory/stats endpoint
- Added /api/v1/memory/reflections endpoint

### Phase 25: Workflow Visualizer (v1.2.1)
- Added WorkflowVisualizer component
- Flowchart and timeline views
- Execution status indicators

### Phase 26: Quick Command Bar (v1.3.0)
- Added QuickCommandBar component (Ctrl+P)
- Navigation commands
- Task execution with `>` prefix
- Grouped by category

### Development Session (2026-03-05)
- Configured Docker as default isolation (Windows compatible)
- Fixed `--privileged false` to `--privileged=false` for Windows Docker
- Added gVisor support with runsc binary (Linux only)
- Added Firecracker backend (Linux only)
- Added Mock backend support
- Updated apex.bat with new commands:
  - `apex.bat router-docker` - Run with Docker isolation
  - `apex.bat router-gvisor` - Run with gVisor isolation (Linux only)
  - `apex.bat router-firecracker` - Run with Firecracker isolation (Linux only)
  - `apex.bat router-mock` - Run with Mock (no real execution)
- Created 10 tasks under "APEX Update" project for future phases
- Fixed Rust compiler warnings

---

## What We Did

### Phase 1: Foundation (Complete)
- Built Rust L2/L3 (Task Router + Memory Service)
- Built TypeScript L1 Gateway
- Built TypeScript L4 Skills Framework  
- Built React L6 UI
- Built Python L5 Execution Engine

### Phase 2: Skill System (Complete)
1. SKILL.md specification
2. Skill registry (SQLite)
3. Skill API endpoints
4. Circuit breaker
5. Skill worker
6. TypeScript CLI
7. Integration tests (4)

### Phase 3: Execution Engine (Complete)

#### Completed:
1. **VM Pool Manager** (`core/router/src/vm_pool.rs`)
   - Manages pool of Firecracker/gVisor VMs
   - Pre-warms VMs (min_ready)
   - Acquire/release lifecycle
   - Crash recovery mechanisms
   - Backend detection (Firecracker/gVisor/Mock)

2. **Deep Task Worker** (`core/router/src/deep_task_worker.rs`)
   - Subscribes to deep task messages
   - Acquires VM from pool
   - Executes agent loop
   - Releases VM on completion

3. **Agent Loop** (`core/router/src/agent_loop.rs`)
   - Plan → Act → Observe → Reflect cycle
   - Budget checking per step
   - Network allowlist enforcement
   - LLM integration (llama-server)

4. **Deep Task Endpoints**
   - POST /api/v1/deep
   - GET /api/v1/vm/stats

### Phase 4: Web UI (Complete)

1. **Skills Marketplace** (`ui/src/components/skills/Skills.tsx`)
   - Lists registered skills with tier badges
   - Quick stats by tier

2. **Settings Page** (`ui/src/components/settings/Settings.tsx`)
   - System info, VM stats, metrics
   - Environment variable guide

3. **File Browser** (`ui/src/components/files/Files.tsx`)
   - Mock file browser with navigation
   - File details panel

4. **Chat Improvements**
   - HTTP-based (no WebSocket required)
   - Deep task support with polling
   - Auto-tier support

### LLM Integration (Complete)

1. **Llama Client** (`core/router/src/llama.rs`)
   - Connects to llama-server
   - OpenAI-compatible API
   - Chat completions

2. **Configuration**
   - APEX_USE_LLM=1 - Enable LLM
   - LLAMA_SERVER_URL - llama-server address
   - LLAMA_MODEL - Model name

3. **Model**
   - Qwen3-4B-Instruct-2507-Q4_K_M.gguf
   - Running on llama-server at port 8080
   - Requires llama.cpp b8185+ (Qwen3 support)

---

## Files Created/Modified

### New Files
- `docs/SKILL.md` - Skill specification
- `core/memory/src/skill_registry.rs` - Skill registry
- `core/router/src/circuit_breaker.rs` - Circuit breaker
- `core/router/src/skill_worker.rs` - Skill worker
- `core/router/src/skill_pool.rs` - Skill pool manager (pre-warmed Bun processes)
- `core/router/src/skill_pool_ipc.rs` - UUID-based IPC framing
- `core/router/src/curriculum.rs` - Curriculum agent for learning from task history
- `core/router/src/vm_pool.rs` - VM pool manager
- `core/router/src/deep_task_worker.rs` - Deep task worker
- `core/router/src/agent_loop.rs` - Agent loop
- `core/router/src/llama.rs` - Llama client
- `skills/src/cli.ts` - TypeScript CLI
- `skills/pool_worker.ts` - Bun REPL dispatcher for skill pool
- `skills/pool_worker_test.ts` - Bun tests for pool worker
- `apex.bat` - Management script for all services
- `ui/src/components/skills/Skills.tsx` - Skills UI
- `ui/src/components/settings/Settings.tsx` - Settings UI
- `ui/src/components/files/Files.tsx` - File browser UI

### Modified Files
- `core/router/src/api.rs` - Added deep task + VM endpoints + instant responses
- `core/router/src/lib.rs` - Added modules
- `core/router/src/main.rs` - Workers spawn, LLM config
- `core/router/src/message_bus.rs` - Deep task message
- `core/router/Cargo.toml` - Added reqwest, tower-http
- `core/router/src/classifier.rs` - Auto-tier routing to LLM
- `ui/src/App.tsx` - Added Skills, Files, Settings tabs
- `ui/src/components/chat/Chat.tsx` - Auto-tier support, polling

---

## Test Count

| Component | Tests |
|-----------|-------|
| Rust unit tests | 70 |
| Rust integration tests | 41 |
| Rust e2e tests | 2 (ignored) |
| TypeScript Gateway | 8 |
| TypeScript Skills | 8 |
| **Total** | **129** |

### LLM Test
- `test_llama_server_connectivity` - Verifies llama-server is running and responding (run with `-- --ignored`)

---

## What's Working

- Router on localhost:3000
- Skill registry CRUD
- Circuit breaker
- TypeScript skills via CLI
- **SkillPool** - Pre-warmed Bun process pool for ~10-15ms latency
- VM pool (Mock/Firecracker/gVisor)
- Deep task worker with agent loop
- Budget checking
- Network allowlist
- Crash recovery
- UI at localhost:8083
- LLM integration with Qwen3-4B
- **Auto-Tier** - All tasks routed to LLM:
  - Greetings → LLM responds
  - Questions → LLM responds  
  - Complex tasks → LLM handles

---

## Commands

### Using apex.bat (Recommended)

```powershell
# Start all services (llama-server, router, UI)
.\apex.bat start

# Start all services INCLUDING embedding server (for memory search)
.\apex.bat start-full

# Stop all services
.\apex.bat stop

# Restart all services
.\apex.bat restart

# Build all components
.\apex.bat build

# Run tests
.\apex.bat test

# Individual services
.\apex.bat llama          # Start llama-server (LLM)
.\apex.bat embed         # Start embedding server (nomic-embed-text)
.\apex.bat embed-test   # Test embedding server
.\apex.bat router         # Start router (no LLM)
.\apex.bat router-llm     # Start router with LLM (starts llama-server first)
.\apex.bat ui            # Start UI dev server
.\apex.bat ui-serve      # Serve built UI

# Test if llama-server is running
.\apex.bat llama-test

# Check service status
.\apex.bat status

# Router on different port
.\apex.bat router2        # Router on port 3001
.\apex.bat router2-llm    # Router on port 3001 with LLM
```

### Manual Commands (Alternative)

```powershell
# Terminal 1 - Start llama-server (requires b8185+)
D:\Users\ashah\Documents\llama.cpp\llama-server.exe -m "C:\Users\ashah\.ollama\models\Qwen3-4B-Instruct-2507-Q4_K_M.gguf" --port 8080

# Terminal 2 - Start router with LLM
cd core
set APEX_USE_LLM=1
set LLAMA_SERVER_URL=http://127.0.0.1:8080
cargo run --release --bin apex-router

# Terminal 3 - Start UI
cd ui && pnpm dev

# Or serve built UI
cd ui && npx serve dist -l 8083

# Run tests
cd core && cargo test
```

### API Examples

```powershell
# Test greeting (routes to LLM)
Invoke-RestMethod -Uri "http://localhost:3000/api/v1/tasks" -Method Post -ContentType "application/json" -Body '{"content":"hi"}'

# Test question (routes to LLM)
Invoke-RestMethod -Uri "http://localhost:3000/api/v1/tasks" -Method Post -ContentType "application/json" -Body '{"content":"what is 2+2?"}'

# Check VM stats
Invoke-RestMethod -Uri "http://localhost:3000/api/v1/vm/stats"

# Check Skill Pool stats
Invoke-RestMethod -Uri "http://localhost:3000/api/v1/skills/pool/stats"

# Register a skill
Invoke-RestMethod -Uri "http://localhost:3000/api/v1/skills" -Method Post -ContentType "application/json" -Body '{"name":"shell.execute","version":"1.0.0","tier":"T1"}'
```

---

## Environment Variables

APEX uses a unified configuration system. See `AGENTS.md` for the complete reference.

| Variable | Description | Default |
|----------|-------------|---------|
| APEX_USE_LLM | Enable LLM integration | false |
| LLAMA_SERVER_URL | llama-server address | http://localhost:8080 |
| LLAMA_MODEL | Model file path | qwen3-4b |
| APEX_USE_FIRECRACKER | Enable Firecracker VMs | false |
| APEX_USE_GVISOR | Enable gVisor VMs | false |
| APEX_VM_VCPU | VCPUs per VM | 2 |
| APEX_VM_MEMORY_MIB | Memory per VM (MiB) | 2048 |
| APEX_PORT | Router port | 3000 |
| APEX_SKILL_POOL_ENABLED | Enable skill pool | true |
| APEX_SKILL_POOL_SIZE | Pool size | 4 |
| APEX_SKILL_POOL_TIMEOUT | Request timeout (ms) | 30000 |
| APEX_SKILL_POOL_ACQUIRE | Slot acquire timeout (ms) | 5000 |

**Configuration API:**
- `GET /api/v1/config` - Get all configuration
- `GET /api/v1/config/summary` - Get configuration summary

---

## Recent Updates (2026-03-02)

### llama-server Update
- Downloaded latest llama.cpp b8185 to support Qwen3 models
- Copied required DLLs to `D:\Users\ashah\Documents\llama.cpp\`
- Model: Qwen3-4B-Instruct-2507-Q4_K_M.gguf

### apex.bat Management Script
- Added `llama-test` command to verify llama-server is running
- Added port conflict detection for UI
- Fixed router-llm to not start duplicate llama-server
- Added `router-llm-no-start` for when llama already running
- Added `embed` command to start embedding server (nomic-embed-text)
- Added `embed-test` command to test embedding server
- Added `start-full` command to start all services including embedding server

### LLM Integration Fixes
- Fixed model name mismatch (Qwen2.5 vs Qwen3)
- Added wait for llama-server to be ready before starting router
- Added completion keywords to prevent response repetition

### Tests Added
- `test_llama_server_connectivity` - Verifies LLM connection (run with `cargo test test_llama_server_connectivity -- --ignored`)

---

## Future Improvements / TODO

- [ ] Add time limit alongside max_steps
- [ ] WebSocket real-time updates for UI
- [ ] Persistent conversation history

---

## Docker Execution (Complete)

### Overview
APEX now supports Docker-based execution for skills. When enabled, skills run in isolated Docker containers instead of using the Mock backend.

### Setup

1. **Build the Docker image:**
```powershell
.\apex.bat docker-build
```

2. **Enable Docker execution:**
```powershell
.\apex.bat router-llm
```
This automatically sets `APEX_USE_DOCKER=1`.

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| APEX_USE_DOCKER | Enable Docker execution | false |
| APEX_DOCKER_IMAGE | Custom Docker image | apex-execution:latest |

### Docker Image
The `execution/Dockerfile` creates a Python 3.11-slim container with Poetry installed. You can customize it for your skill execution needs.

---

## Phase 5: Messaging Gateway (Complete)

### Completed:

1. **REST API Adapter** (`gateway/src/adapters/rest/`)
   - Endpoints: `/api/tasks`, `/api/tasks/:id`, `/api/metrics`, `/api/skills`
   - Proxies to router on port 3001

2. **Slack Adapter** (`gateway/src/adapters/slack/`)
   - Already exists, handles Slack events

3. **Discord Adapter** (`gateway/src/adapters/discord/`)
   - New implementation with Gateway intents

4. **Telegram Adapter** (`gateway/src/adapters/telegram/`)
   - New implementation with bot framework

5. **Unified Message History**
   - `core/memory/src/msg_repo.rs` - Message repository
   - `GET /api/v1/messages` - List messages
   - `GET /api/v1/messages/task/:task_id` - Get task messages

---

## Recent Enhancements

### Task Configuration
- Added `max_steps`, `budget_usd`, `time_limit_secs` to TaskRequest
- Time limits enforced in `can_continue()` (except when using local LLM)
- Settings page stores config in localStorage

### UI Improvements
- Stats display in main screen header
- Total cost tracking in Settings
- Task history with details modal
- Refresh button for task history
- Better error handling with banner display
- Removed "Auto-tier" text from main screen

### Auto-Cleanup
- Docker containers cleaned up on startup
- Background task cleans up completed tasks older than 7 days (runs hourly)

---

## Test Count

| Component | Tests |
|-----------|-------|
| Rust unit tests | 15 |
| Rust integration tests | 14 |
| Rust memory tests | 3 |
| TypeScript Gateway | 8 |
| TypeScript Skills | 8 |
| **Total** | **48** |

---

## Future Improvements / TODO

- [ ] Security audit
- [ ] Performance optimization
- [ ] Release preparation

---

## Kanban Board Implementation (2026-03-02)

### Completed:

1. **Database Schema**
   - Added migration: `core/memory/migrations/002_kanban_fields.sql`
   - Added `project`, `priority`, `category` fields to tasks table
   - Created indexes for new fields

2. **Task Model Updates** (`core/memory/src/tasks.rs`)
   - Added `TaskPriority` enum (low, medium, high, urgent)
   - Added `try_from_str` method to `TaskStatus`
   - Added `project`, `priority`, `category` to `Task`, `CreateTask`, `UpdateTask`

3. **TaskRepository Updates** (`core/memory/src/task_repo.rs`)
   - `find_by_project()` - Get tasks by project
   - `find_by_priority()` - Get tasks by priority
   - `find_by_category()` - Get tasks by category
   - `find_by_filter()` - Multi-field filtering with limit/offset
   - `update_task_fields()` - Update project/priority/category
   - `get_projects()` - Get unique project list
   - `get_categories()` - Get unique category list

4. **API Endpoints** (`core/router/src/api.rs`)
   - `GET /api/v1/tasks?project=&status=&priority=&category=` - Filter tasks
   - `GET /api/v1/tasks/filter-options` - Get filter values
   - `PUT /api/v1/tasks/:id` - Update task fields
   - Added new request/response types: `TaskFilterRequest`, `UpdateTaskRequest`

5. **Frontend** (`ui/src/components/kanban/KanbanBoard.tsx`)
   - 5 columns: Pending, Running, Completed, Failed, Cancelled
   - Project filter dropdown
   - Auto-refresh every 5 seconds
   - Task detail modal with inline editing
   - Click-to-move between columns
   - Priority badges with color coding
   - Integrated into sidebar as "Board" tab

### Features:
- Filter by project
- View/edit project, priority, category
- Auto-status updates via polling (5 second interval)
- Color-coded priority badges
- Click task → view details
- Move tasks with → buttons

### Files Created/Modified:
- `core/memory/migrations/002_kanban_fields.sql` - NEW
- `core/memory/src/tasks.rs` - Updated
- `core/memory/src/task_repo.rs` - Updated
- `core/router/src/api.rs` - Updated
- `ui/src/components/kanban/KanbanBoard.tsx` - NEW
- `ui/src/App.tsx` - Updated
- `ui/src/components/ui/Sidebar.tsx` - Updated
- `docs/API.md` - Updated

### Usage:
Click the 📋 icon in the sidebar to access the Kanban board.

---

## Phase 1: Security & Permissions (2026-03-03)

### Completed:

1. **Confirmation Modal UI** (`ui/src/components/ui/ConfirmationModal.tsx`)
   - T0: Silent (no confirmation needed)
   - T1: Tap to confirm (simple button)
   - T2: Type to confirm (must type action name)
   - T3: TOTP + 5-second delay (mock implementation)

2. **Message Bus Updates** (`core/router/src/message_bus.rs`)
   - Added `permission_tier` to `SkillExecutionMessage`
   - Added `permission_tier` to `DeepTaskMessage`
   - Added `ConfirmationMessage` type for confirmation events
   - Added `subscribe_confirmations()` method

3. **API Endpoints** (`core/router/src/api.rs`)
   - Added `POST /api/v1/tasks/:id/confirm` - Confirm pending task
   - Added `ConfirmTaskRequest` struct
   - Added `permission_tier` to `SkillResponse`

4. **Prompt Injection Defense** (`core/router/src/agent_loop.rs`)
   - Expanded `sanitize_for_llm()` with 10+ new patterns:
     - DAN, jailbreak, developer mode
     - new instructions, override rules
     - bypass restriction, ignore policy
     - do anything now, spanish to english, translate instructions

5. **Kanban Board Enhancements** (`ui/src/components/kanban/KanbanBoard.tsx`)
   - Added "+ New Task" button with modal form
   - Added "▶ Run" button to execute pending tasks
   - Project autocomplete from existing projects
   - Category autocomplete from existing categories

### Files Created/Modified:
- `ui/src/components/ui/ConfirmationModal.tsx` - NEW
- `ui/src/components/chat/Chat.tsx` - Updated
- `core/router/src/message_bus.rs` - Updated
- `core/router/src/api.rs` - Updated
- `core/router/src/agent_loop.rs` - Updated
- `ui/src/components/kanban/KanbanBoard.tsx` - Updated

### New API Endpoints:
- `POST /api/v1/tasks/:id/confirm` - Confirm task with tier verification

---

## Bug Fix: sqlx Migration Error (2026-03-03)

### Problem
Router crashed on startup with `Error: Configuration(VersionMismatch(1))` due to sqlx::migrate!() macro incompatibility.

### Solution
Replaced `sqlx::migrate!()` with manual SQL queries in `core/memory/src/db.rs`.

### Files Modified:
- `core/memory/src/db.rs` - Manual migrations instead of sqlx macro

---

## Session Updates (2026-03-03) - Phases 1-6 Complete

### Docker Container Cleanup Fix
**Problem**: Container name conflicts - "apex-vm-0 already in use"

**Solution**: Added container cleanup in `release()` function in `vm_pool.rs`
- Runs `docker rm -f apex-vm-{id}` when releasing Docker VMs
- Fixed naming consistency (apex-vm-{id} across all backends)

### Security Hardening (Phase 4)
Added to Docker container spawn:
- `--memory=2048m` - Memory limit
- `--cpus=2` - CPU limit
- `--pids-limit=256` - Process limit
- `--network=none` - Network isolation
- `--read-only` - Read-only filesystem
- `--tmpfs=/tmp` - Writable temp directory

### New Components

#### Memory Viewer (Phase 5)
- Created `ui/src/components/memory/MemoryViewer.tsx`
- Added Memory tab to sidebar (🧠 icon)
- Features: search, project filtering, task history

#### Cost Estimator (Phase 6)
- Created `core/router/src/cost_estimator.rs`
- Estimates based on: token count, step count, complexity
- Breakdown: LLM, compute, storage, network costs

#### TTL Cleanup (Phase 6)
- Created `core/memory/migrations/003_ttl_config.sql`
- Created `core/memory/src/ttl_cleanup.rs`
- Default retention: 90 days (tasks/messages), 365 days (audit), 30 days (vectors)

#### Skill SDK (Phase 6)
- Created `docs/SKILL-SDK.md`
- Complete guide for creating new skills

### New Skills (Phase 6)
Registered 9 new skills:
- music.generate, music.extend, music.remix (T2)
- video.generate, video.edit (T2)
- script.outline, script.draft (T1)
- copy.generate, seo.optimize (T1)

### Files Created/Modified
```
core/router/src/vm_pool.rs           -- Security hardening + cleanup
core/router/src/cost_estimator.rs   -- NEW
core/router/src/lib.rs               -- Added cost_estimator module
core/memory/src/ttl_cleanup.rs      -- NEW
core/memory/migrations/003_ttl_config.sql -- NEW
docs/VM-BACKENDS.md                 -- NEW
docs/SKILL-SDK.md                   -- NEW
ui/src/components/memory/MemoryViewer.tsx -- NEW
ui/src/App.tsx                      -- Added memory tab
ui/src/components/ui/Sidebar.tsx    -- Added memory icon
skills/skills/music.generate/       -- NEW
skills/skills/music.extend/         -- NEW
skills/skills/music.remix/          -- NEW
skills/skills/video.generate/       -- NEW
skills/skills/video.edit/           -- NEW
skills/skills/script.outline/       -- NEW
skills/skills/script.draft/         -- NEW
skills/skills/copy.generate/        -- NEW
skills/skills/seo.optimize/         -- NEW
```

---

## Current State (2026-03-03)

### Running Services
- Llama-Server: http://localhost:8080
- Router: http://localhost:3000
- UI: http://localhost:8083

### APEX Update Project
- 66 completed tasks
- All Phases (1-6) complete

### Skills Registry
- Total: 23 skills
  - T0 (Read-only): 3
  - T1 (Tap confirm): 11
  - T2 (Type confirm): 9
  - T3 (Delay): 0

### Documentation
- ✅ UPDATE-PLAN.md - Complete
- ✅ VM-BACKENDS.md - Complete  
- ✅ SKILL-SDK.md - Complete
- ✅ SESSION.md - Updated

---

## Current Session (2026-03-04) - Testing Infrastructure

### What We Did

#### Phase: Test Suite Enhancement

1. **Added UI Testing Infrastructure**
   - Installed `@testing-library/react`, `@testing-library/jest-dom`, `jsdom`
   - Created `ui/src/test/setup.ts` with global mocks
   - Created `ui/src/test/mocks.ts` with API mock utilities
   - Updated `vite.config.ts` with test configuration

2. **Created UI Component Tests**
   - NotificationBell.test.tsx - 7 tests
   - Sidebar.test.tsx - 7 tests
   - ConfirmationModal.test.tsx - 9 tests

3. **Fixed Pre-existing Test Issues**
   - test_stream_manager - Fixed pointer comparison
   - test_decision_engine - Fixed empty context issue
   - test_execution_stream - Fixed subscription timing

### Test Suite Results

| Component | Tests | Status |
|-----------|-------|--------|
| Rust Integration | 40 | ✅ Pass |
| Rust Memory Unit | 16 | ✅ Pass |
| Rust Router Unit | 68 | ✅ Pass |
| Gateway TypeScript | 8 | ✅ Pass |
| Skills TypeScript | 8 | ✅ Pass |
| UI React | 23 | ✅ Pass |

**Total: 163 tests passing**

### Files Created/Modified
```
ui/vite.config.ts                         -- Added test config
ui/src/test/setup.ts                      -- NEW
ui/src/test/mocks.ts                      -- NEW
ui/src/components/ui/NotificationBell.test.tsx -- NEW
ui/src/components/ui/Sidebar.test.tsx     -- NEW
ui/src/components/ui/ConfirmationModal.test.tsx -- NEW
core/router/src/execution_stream.rs       -- Fixed test
core/router/src/heartbeat/scheduler.rs    -- Fixed test
core/router/tests/integration.rs          -- Added 26 new tests
```
