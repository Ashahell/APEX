# Session Context - APEX Development

> ⚠️ **Status: PRE-ALPHA** - This is an experimental research project. Not production ready.

**Date**: 2026-03-05
**Session**: v1.3.0 - Quick Command Bar - COMPLETE

---

## Recent Updates

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
- `core/router/src/vm_pool.rs` - VM pool manager
- `core/router/src/deep_task_worker.rs` - Deep task worker
- `core/router/src/agent_loop.rs` - Agent loop
- `core/router/src/llama.rs` - Llama client
- `skills/src/cli.ts` - TypeScript CLI
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
| Rust unit tests | 19 |
| Rust integration tests | 14 |
| Rust e2e tests | 2 (ignored) |
| TypeScript Gateway | 8 |
| TypeScript Skills | 8 |
| **Total** | **49** |

### LLM Test
- `test_llama_server_connectivity` - Verifies llama-server is running and responding (run with `-- --ignored`)

---

## What's Working

- Router on localhost:3000
- Skill registry CRUD
- Circuit breaker
- TypeScript skills via CLI
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

# Stop all services
.\apex.bat stop

# Restart all services
.\apex.bat restart

# Build all components
.\apex.bat build

# Run tests
.\apex.bat test

# Individual services
.\apex.bat llama          # Start llama-server
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
