# APEX Architecture Documentation

> ⚠️ **WARNING: PRE-ALPHA** - This document describes an experimental research system. Not production ready.
> 
> **Production Gaps**: Security implementation complete (Phases 0-7), comprehensive testing (251+ tests), production hardening docs available, no formal load testing, no disaster recovery procedures, single-node deployment only.

**Date**: 2026-03-12
**Version**: v1.3.2
**Status**: Pre-Alpha (Experimental)
**Scope**: All directories (core/, gateway/, skills/, ui/, execution/)

---

> **Note on Versioning**: v1.3.2 includes AgentZero UI migration. Earlier versions (v0.x) were internal development milestones. Version history at bottom explains the progression.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           L6: React UI                                   │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────┐   │
│  │  Chat   │  │ Skills  │  │ Files   │  │ Kanban  │  │ Settings  │   │
│  │   +     │  │         │  │         │  │          │  │           │   │
│  │Taskside │  │         │  │         │  │          │  │           │   │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘  └────┬─────┘   │
└───────┼────────────┼────────────┼────────────┼────────────┼──────────┘
        │            │            │            │            │
        └────────────┴────────────┴────────────┴────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    L1: Gateway (TypeScript)                             │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  REST Adapter  │  HMAC Signing  │  NATS Integration           │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                              │                                           │
│                    HTTP POST /api/v1/tasks                             │
└──────────────────────────────┼────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    L2: Router (Rust)                                    │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  REST API (Axum)  │  WebSocket  │  Workers  │  Auth/Middleware│    │
│  └────────────────────────────────────────────────────────────────┘    │
│                              │                                           │
│         ┌────────────────────┼────────────────────┐                      │
│         ▼                    ▼                    ▼                      │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐            │
│  │    Skill    │    │    Deep     │    │       T3        │            │
│  │   Worker    │    │   Task      │    │  Confirmation   │            │
│  │  (L4 Skill  │    │  Worker     │    │   Worker        │            │
│  │   Executor) │    │             │    │                 │            │
│  └──────┬──────┘    └──────┬──────┘    └─────────────────┘            │
│         │                  │                                          │
└─────────┼──────────────────┼──────────────────────────────────────────────┘
          │                  │
          ▼                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    L3: Memory (Rust - SQLite)                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐            │
│  │  Tasks   │  │ Messages  │  │ Skills   │  │Audit Log  │            │
│  │          │  │           │  │          │  │          │            │
│  │Channels  │  │  Journal  │  │VectorSt. │  │ Prefs    │            │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘            │
│         │                                                                  │
└─────────┼────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    L4: Skills Framework (TypeScript)                    │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  SkillPool (pre-warmed Bun)  │  SkillLoader (TypeScript)      │    │
│  │  SKILL.md plugins           │  Zod schema validation          │    │
│  └────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    L5: Execution (Python/Docker)                       │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │                    Agent Zero Fork                               │    │
│  │         Plan → Act → Observe → Reflect Loop                    │    │
│  │         + Tool Generation + Budget Enforcement                  │    │
│  └────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### Architecture Notes

1. **L2 (Router)** orchestrates all communication - it does NOT flow directly to L5
2. **L2 ↔ L3**: Router queries Memory via sqlx (not L3→L5)
3. **L2 ↔ L4**: Router executes Skills via SkillPool or spawns Bun processes
4. **L2 → L5**: DeepTaskWorker invokes Python agent via subprocess
5. **L4 Skills** are independent of L5 Execution - Skills run in Router's worker pool, not in Execution VMs

---

## Deployment Model

### Single-Process Architecture

**Current Implementation**: APEX is implemented as a **single-process Rust binary** where:

- L2 (Router), L3 (Memory), and workers all run in the same process
- Inter-component communication uses **Tokio broadcast channels** (in-memory)
- For distributed deployment, **NATS JetStream** can replace Tokio channels

**Implications**:
- No network overhead between layers - extremely fast message passing
- If the router process crashes, all in-flight messages are lost
- Can enable NATS for distributed mode

**Distributed Mode (Optional)**:
- Enable with `APEX_NATS_ENABLED=1`
- Requires NATS server running
- Message persistence across restarts
- Horizontal scalability

### Configuration

APEX uses a **unified configuration system** (`AppConfig`) that loads from:
1. Environment variables (highest priority)
2. YAML configuration file (apex.yaml)
3. Default values (lowest priority)

All configuration is accessed via `AppConfig::global()` in the Rust codebase.

**Configuration API:**
- `GET /api/v1/config` - Get all configuration variables
- `GET /api/v1/config/summary` - Get configuration summary with validation
- Settings → Config tab in the UI

```yaml
# apex.yaml
server:
  host: "0.0.0.0"
  port: 3000

auth:
  shared_secret: "${APEX_SHARED_SECRET}"
  disabled: false

database:
  type: "sqlite"  # or "postgresql"
  connection_string: "${APEX_DATABASE_URL}"
```

**Environment Variables:**
All settings can be configured via environment variables. See `AGENTS.md` for the complete reference.

### Authentication

- **Gateway → Router**: HMAC-SHA256 signed requests
  - Requires `APEX_SHARED_SECRET` environment variable
  - Request signature includes: timestamp + method + path + body
  - Timestamp must be within 5 minutes to prevent replay attacks
  - Set `APEX_AUTH_DISABLED=1` for local development only

- **UI → Router**: Same HMAC authentication via signed fetch

- **T3 Tasks**: Require TOTP verification before execution
  - Users must configure TOTP via `/api/v1/totp/setup`
  - 6-digit TOTP token required for destructive operations (shell.execute)

---

## Layer Breakdown

### L1: Gateway (TypeScript)
**Location**: `gateway/`
**Technology**: Node.js, TypeScript, NATS

#### Responsibilities
- Accepts inbound messages from external sources (Slack, Discord, Telegram, REST)
- Normalizes messages into TaskRequest format
- Routes tasks to Router via HTTP with HMAC signing
- Publishes responses back to message sources
- NATS integration for distributed messaging

#### Components
| Component | File | Purpose |
|-----------|------|---------|
| REST Adapter | `src/adapters/rest/index.ts` | HTTP server on port 3001 |
| Slack Adapter | `src/adapters/slack/index.ts` | Slack bot integration |
| Discord Adapter | `src/adapters/discord/index.ts` | Discord bot integration |
| Telegram Adapter | `src/adapters/telegram/index.ts` | Telegram bot integration |
| Gateway Core | `src/index.ts` | Main gateway with HMAC signing |

#### Public API
```typescript
class Gateway {
  async createTask(request: TaskRequest): Promise<TaskResponse>
  async getTask(taskId: string): Promise<TaskResponse>
  async getMetrics(): Promise<unknown>
  registerAdapter(name: string, adapter: unknown): void
}
```

#### Configuration
```typescript
interface GatewayConfig {
  natsUrl?: string;      // NATS server URL (default: localhost:4222)
  routerUrl?: string;   // Router URL (default: http://localhost:3000)
  port?: number;        // Gateway port (default: 3001)
  sharedSecret?: string; // HMAC signing secret
}
```

---

### L2: Router (Rust)
**Location**: `core/router/`
**Technology**: Rust, Axum, Tokio, SQLx

#### Responsibilities
- HTTP API server (port 3000)
- Task classification and routing
- Worker coordination via MessageBus
- Capability token generation
- Metrics collection
- WebSocket for real-time updates
- HMAC authentication
- TOTP verification for T3 tasks

#### Components
| Component | File | Purpose |
|-----------|------|---------|
| API Layer | `src/api/` | Modular HTTP endpoints (43 routes in 8 modules) |
| LLM API | `src/api/llms.rs` | Multi-provider LLM configuration (26 providers) |
| MessageBus | `src/message_bus.rs` | Tokio broadcast channels |
| WebSocket | `src/websocket.rs` | Real-time WebSocket server |
| NATS | `src/nats_integration.rs` | Distributed messaging |
| Auth | `src/auth.rs` | HMAC request verification |
| TOTP | `src/totp.rs` | TOTP-based T3 verification |
| SkillWorker | `src/skill_worker.rs` | Executes TypeScript skills (with SkillPool) |
| SkillPool | `src/skill_pool.rs` | Pre-warmed Bun process pool for low-latency skill execution |
| SkillPoolIpc | `src/skill_pool_ipc.rs` | UUID-based IPC framing for multiplexed communication |
| DeepTaskWorker | `src/deep_task_worker.rs` | Handles LLM-powered tasks |
| T3ConfirmWorker | `src/t3_confirm_worker.rs` | Handles T3 confirmation |
| VmPool | `src/vm_pool.rs` | Docker/Firecracker/gVisor VM management |
| CircuitBreaker | `src/circuit_breaker.rs` | Fault tolerance for skills |
| Llama | `src/llama.rs` | LLM client (llama-server) |
| Classifier | `src/classifier.rs` | Task tier classification |
| CostEstimator | `src/cost_estimator.rs` | Task cost estimation |
| Metrics | `src/metrics.rs` | Prometheus metrics |
| SkillPlugin | `src/skill_plugin.rs` | SKILL.md plugin system (markdown-based skills) |
| ClientAuth | `src/client_auth.rs` | Per-client authentication |
| DatabaseManager | `src/db_manager.rs` | PostgreSQL/SQLite abstraction |
| UnifiedConfig | `src/unified_config.rs` | Unified configuration system |
| AgentLoop | `src/agent_loop.rs` | Agent reasoning loop |
| NarrativeService | `src/narrative.rs` | Narrative memory service |
| Heartbeat | `src/heartbeat/mod.rs` | Periodic autonomy daemon (✅ ACTIVE) |
| Soul | `src/soul/mod.rs` | Identity system (✅ ACTIVE) |
| Governance | `src/governance.rs` | Constitution enforcement (✅ ACTIVE) |
| ExecutionStream | `src/execution_stream.rs` | Real-time execution streaming |
| Moltbook | `src/moltbook/mod.rs` | Social context (✅ ACTIVE) |
| Webhook | `src/webhook.rs` | External integrations |
| Notification | `src/notification.rs` | In-app notifications |

#### Active Components Detail

##### Heartbeat (`src/heartbeat/mod.rs`)
- **Purpose**: Periodic autonomous wake cycle for self-directed tasks
- **Behaviour**: Runs every N minutes (configurable via `APEX_HEARTBEAT_INTERVAL`). On wake:
  1. Checks system health metrics
  2. Scans for pending tasks or suggestions
  3. May execute up to `APEX_HEARTBEAT_MAX_ACTIONS` actions autonomously
  4. Flushes any orphaned working memory to long-term storage
- **Config**: `APEX_HEARTBEAT_ENABLED`, `APEX_HEARTBEAT_INTERVAL`, `APEX_HEARTBEAT_JITTER`, `APEX_HEARTBEAT_COOLDOWN`, `APEX_HEARTBEAT_MAX_ACTIONS`

##### Soul (`src/soul/mod.rs`)
- **Purpose**: Agent identity and persona system
- **Behaviour**: Loads SOUL.md file from `APEX_SOUL_DIR` (~/.apex/soul) at startup. Provides:
  - Agent name, values, and personality
  - Skills and capabilities declaration
  - Relationship mapping to other agents
  - Goals and objectives
- **Config**: `APEX_SOUL_DIR`, `APEX_SOUL_BACKUP`

##### Governance (`src/governance.rs`)
- **Purpose**: Constitution enforcement for agent actions
- **Behaviour**: 
  - Loads constitution rules at startup
  - Intercepts all T2/T3 actions before execution
  - Checks if action violates immutable values
  - Can block action or enable oracle mode (unrestricted)
- **API**: `POST /api/v1/governance/check`, `POST /api/v1/governance/oracle`

##### Moltbook (`src/moltbook/mod.rs`)
- **Purpose**: Social context and agent directory
- **Behaviour**:
  - Connects to Moltbook service (optional)
  - Maintains agent directory with trust scores
  - Enables social posts and notifications
  - Assesses trust between agents before collaboration
- **Config**: `APEX_MOLTBOOK_AGENT_ID`

#### API Endpoints

**Tasks**
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/tasks` | Create task (auto-tiered) |
| GET | `/api/v1/tasks` | List tasks with filters |
| GET | `/api/v1/tasks/filter-options` | Get available filter values |
| GET | `/api/v1/tasks/:id` | Get task status |
| PUT | `/api/v1/tasks/:id` | Update task (project, priority, category) |
| POST | `/api/v1/tasks/:id/cancel` | Cancel task |
| POST | `/api/v1/tasks/:id/confirm` | Confirm T1-T3 task |

**Messages**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/messages` | List messages |
| GET | `/api/v1/messages/task/:task_id` | Get messages for task |

**Skills**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/skills` | List all skills |
| POST | `/api/v1/skills` | Register new skill |
| GET | `/api/v1/skills/:name` | Get skill details |
| DELETE | `/api/v1/skills/:name` | Delete skill |
| PUT | `/api/v1/skills/:name/health` | Update skill health |
| POST | `/api/v1/skills/execute` | Execute skill directly |

**Deep Tasks**
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/deep` | Execute deep task with LLM |

**TOTP (T3 Verification)**
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/totp/setup` | Generate TOTP secret |
| POST | `/api/v1/totp/verify` | Verify TOTP token |
| GET | `/api/v1/totp/status` | Check TOTP status |

**Channels**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/channels` | List channels |
| POST | `/api/v1/channels` | Create channel |
| GET | `/api/v1/channels/:id` | Get channel |
| PUT | `/api/v1/channels/:id` | Update channel |
| DELETE | `/api/v1/channels/:id` | Delete channel |

**Decision Journal**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/journal` | List entries |
| POST | `/api/v1/journal` | Create entry |
| GET | `/api/v1/journal/:id` | Get entry |
| PUT | `/api/v1/journal/:id` | Update entry |
| DELETE | `/api/v1/journal/:id` | Delete entry |
| GET | `/api/v1/journal/search?q=` | Search entries |

**Workflows**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/workflows` | List workflows |
| POST | `/api/v1/workflows` | Create workflow |
| GET | `/api/v1/workflows/filter-options` | Get filter options |
| GET | `/api/v1/workflows/:id` | Get workflow |
| PUT | `/api/v1/workflows/:id` | Update workflow |
| DELETE | `/api/v1/workflows/:id` | Delete workflow |
| GET | `/api/v1/workflows/:id/executions` | Get execution history |

**Adapters**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/adapters` | List adapters |
| GET | `/api/v1/adapters/:name` | Get adapter config |
| PUT | `/api/v1/adapters/:name` | Update adapter |
| POST | `/api/v1/adapters/:name/toggle` | Toggle adapter |

**Webhooks**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/webhooks` | List webhooks |
| POST | `/api/v1/webhooks` | Create webhook |
| GET | `/api/v1/webhooks/:id` | Get webhook |
| DELETE | `/api/v1/webhooks/:id` | Delete webhook |
| POST | `/api/v1/webhooks/:id/toggle` | Toggle webhook |

**Notifications**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/notifications` | List notifications |
| GET | `/api/v1/notifications/unread-count` | Get unread count |
| GET | `/api/v1/notifications/:id` | Get notification |
| POST | `/api/v1/notifications/:id/read` | Mark as read |
| POST | `/api/v1/notifications/read-all` | Mark all as read |
| DELETE | `/api/v1/notifications/:id` | Delete notification |

**Real-time**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/events` | Server-Sent Events |
| GET | `/api/v1/ws` | WebSocket endpoint |

**System**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/metrics` | Get system metrics |
| GET | `/api/v1/vm/stats` | Get VM pool stats |
| GET | `/api/v1/config` | Get all configuration |
| GET | `/api/v1/config/summary` | Get configuration summary |
| GET | `/health` | Health check |
| GET | `/` | Root info |

**Memory (v1.3.0)**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/memory/stats` | Get memory statistics |
| GET | `/api/v1/memory/reflections` | Get reflections |
| GET | `/api/v1/memory/search?q=&limit=` | Search memory (hybrid) |
| GET | `/api/v1/memory/index` | Get index statistics |

**Files**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/files` | List files |
| GET | `/api/v1/files/content?path=` | Get file content |

**LLM Configuration (v1.3.1)**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/llms/providers` | List available LLM providers |
| GET | `/api/v1/llms/config` | Get current LLM configuration |
| PUT | `/api/v1/llms/config` | Update LLM configuration |
| GET | `/api/v1/llms/models?provider=` | List models for a provider |
| POST | `/api/v1/llms/test` | Test LLM connection |

#### Message Bus Types
```rust
// Four broadcast channels:
- TaskMessage           // General task updates
- SkillExecutionMessage // Skill execution requests
- DeepTaskMessage      // LLM task requests
- ConfirmationMessage   // T1-T3 confirmations
```

#### AppState Structure
```rust
pub struct AppState {
    pub config: AppConfig,           // C4: Unified config
    pub pool: sqlx::SqlitePool,
    pub metrics: RouterMetrics,
    pub message_bus: MessageBus,
    pub circuit_breakers: CircuitBreakerRegistry,
    pub vm_pool: Option<VmPool>,
    pub skill_pool: Option<Arc<SkillPool>>,
    pub execution_streams: ExecutionStreamManager,
    pub ws_manager: WebSocketManager,
    pub moltbook: Option<MoltbookClient>,
    pub governance: Arc<Mutex<GovernanceEngine>>,
    pub system_monitor: SystemMonitor,
    pub cache: ResponseCache,
    pub rate_limiter: RateLimiter,
    pub workflow_repo: WorkflowRepository,
    pub webhook_manager: WebhookManager,
    pub notification_manager: NotificationManager,
    pub embedder: Arc<Embedder>,              // v1.3.0: Embedding client
    pub background_indexer: Arc<BackgroundIndexer>, // v1.3.0: File indexer
    Arc<Narrative pub narrative_memory:Memory>,      // v1.3.0: Long-term memory
}
```

> **Note on Component Wiring** (v1.3.0): All memory components are now wired into AppState:
> - `Embedder`: Created at startup from config, stored in AppState
> - `BackgroundIndexer`: Spawned at startup, scans ~/.apex/memory/
> - `NarrativeMemory`: Initialized at startup, manages journal/entities/knowledge/reflections
> - `WorkingMemory`: Created per-task via `WorkingMemory::new()`, passed to AgentLoop, each step recorded to scratchpad, persisted to SQLite, flushed on completion

---

### L3: Memory (Rust)
**Location**: `core/memory/`
**Technology**: Rust, SQLite, SQLx, sqlite-vec

#### Responsibilities
- SQLite database persistence (single `apex.db` file)
- Task, message, skill, and preference storage
- Audit logging with hash chain
- Channel and decision journal management
- **Enhanced Semantic Search** (v1.3.0):
  - sqlite-vec for vector storage (768-dim embeddings)
  - FTS5 for BM25 keyword search  
  - Hybrid search with RRF (Reciprocal Rank Fusion)
  - Temporal decay and MMR (Max Marginal Relevance)
  - Working memory (per-task scratchpad)
  - Background indexer for automatic file indexing

> **Note on VectorStore vs HybridSearch**: `VectorStore` (legacy) stores embeddings as JSON in SQLite. `HybridSearch` uses sqlite-vec for efficient KNN. The legacy `VectorStore` is deprecated; use `HybridSearch` for new development.

#### Database Schema

All tables are in `apex.db`. See `core/memory/migrations/` for complete definitions.

**tasks**
```sql
CREATE TABLE tasks (
  id TEXT PRIMARY KEY,
  status TEXT NOT NULL,
  tier TEXT NOT NULL,
  input_content TEXT,
  output_content TEXT,
  channel TEXT,
  thread_id TEXT,
  author TEXT,
  skill_name TEXT,
  error_message TEXT,
  cost_estimate_cents INTEGER,
  actual_cost_cents INTEGER,
  project TEXT,
  priority TEXT,
  category TEXT,
  created_at TEXT,
  updated_at TEXT
);
```

**messages**
```sql
CREATE TABLE messages (
  id TEXT PRIMARY KEY,
  task_id TEXT,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  channel TEXT,
  thread_id TEXT,
  author TEXT,
  attachments TEXT,
  created_at TEXT
);
```

**channels**
```sql
CREATE TABLE channels (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  description TEXT,
  created_at TEXT,
  updated_at TEXT
);
```

**decision_journal**
```sql
CREATE TABLE decision_journal (
  id TEXT PRIMARY KEY,
  task_id TEXT,
  title TEXT NOT NULL,
  context TEXT,
  decision TEXT NOT NULL,
  rationale TEXT,
  outcome TEXT,
  tags TEXT,
  created_at TEXT,
  updated_at TEXT
);
```

**skill_registry**
```sql
CREATE TABLE skill_registry (
  name TEXT PRIMARY KEY,
  version TEXT NOT NULL,
  tier TEXT NOT NULL,
  enabled INTEGER DEFAULT 1,
  health_status TEXT,
  last_health_check TEXT,
  created_at TEXT,
  updated_at TEXT
);
```

**memory_chunks** (enhanced search - v1.3.0)
```sql
CREATE TABLE memory_chunks (
    id          TEXT PRIMARY KEY,
    file_path   TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    content     TEXT NOT NULL,
    word_count  INTEGER NOT NULL,
    memory_type TEXT NOT NULL,
    task_id     TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    accessed_at TEXT NOT NULL DEFAULT (datetime('now')),
    access_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(file_path, chunk_index)
);
```

**memory_vec** (sqlite-vec virtual table)
```sql
CREATE VIRTUAL TABLE memory_vec USING vec0(
    chunk_id    TEXT PARTITION KEY,
    embedding   float[768]
);
```

**working_memory** (per-task scratchpad)
```sql
CREATE TABLE working_memory (
    task_id           TEXT PRIMARY KEY,
    scratchpad        TEXT NOT NULL DEFAULT '',
    entities_json     TEXT NOT NULL DEFAULT '{}',
    causal_links_json TEXT NOT NULL DEFAULT '[]',
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### Components
| Component | File | Purpose |
|-----------|------|---------|
| Database | `src/db.rs` | SQLite connection pool + migrations |
| TaskRepository | `src/task_repo.rs` | Task CRUD operations |
| MessageRepository | `src/msg_repo.rs` | Message CRUD operations |
| ChannelRepository | `src/channel_repo.rs` | Channel CRUD operations |
| DecisionJournalRepository | `src/decision_journal.rs` | Journal CRUD operations |
| SkillRegistry | `src/skill_registry.rs` | Skill management |
| Tasks | `src/tasks.rs` | Task/TaskStatus/TaskTier types |
| HybridSearch | `src/hybrid_search.rs` | Semantic search (RRF + BM25) |
| Audit | `src/audit.rs` | Audit logging with hash chain |
| Preferences | `src/preferences.rs` | User preferences |
| TtlCleanup | `src/ttl_cleanup.rs` | Data retention |
| Embedder | `src/embedder.rs` | Local (nomic-embed-text) + OpenAI embeddings |
| Chunker | `src/chunker.rs` | Text chunking with markdown awareness |
| WorkingMemory | `src/working_memory.rs` | Per-task scratchpad |
| BackgroundIndexer | `src/background_indexer.rs` | Async file indexing |

#### Public API
```rust
// Task repository
impl TaskRepository {
    pub async fn create(&self, id: &str, input: CreateTask, tier: TaskTier) -> Result<()>
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Task>>
    pub async fn find_by_filter(...) -> Result<Vec<Task>>
    pub async fn update_status(&self, id: &str, status: TaskStatus) -> Result<()>
    pub async fn get_total_cost(&self) -> Result<f64>
    pub async fn cleanup_old_completed(&self, days: i64) -> Result<u64>
}

// Channel repository
impl ChannelRepository {
    pub async fn find_all(&self) -> Result<Vec<Channel>>
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Channel>>
    pub async fn create(&self, id: &str, name: &str, description: Option<&str>) -> Result<()>
    pub async fn update(&self, id: &str, name: &str, description: Option<&str>) -> Result<()>
    pub async fn delete(&self, id: &str) -> Result<()>
}

// Decision journal
impl DecisionJournalRepository {
    pub async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<DecisionJournalEntry>>
    pub async fn find_by_id(&self, id: &str) -> Result<Option<DecisionJournalEntry>>
    pub async fn create(&self, id: &str, entry: CreateDecisionEntry) -> Result<()>
    pub async fn search(&self, query: &str, limit: i64) -> Result<Vec<DecisionJournalEntry>>
}
```

---

### L4: Skills Framework (TypeScript)
**Location**: `skills/`
**Technology**: TypeScript, Zod

#### Responsibilities
- Skill loading and execution
- Input/output schema validation with Zod
- Skill discovery and registry
- 28 built-in skills across T0-T3 tiers

#### Skill Interface
```typescript
interface Skill {
  name: string;           // e.g., "code.generate"
  version: string;       // e.g., "0.1.0"
  tier: 'T0' | 'T1' | 'T2' | 'T3';
  inputSchema: z.ZodType;
  outputSchema: z.ZodType;
  
  execute(ctx: SkillContext, input: unknown): Promise<SkillResult>;
  healthCheck(): Promise<boolean>;
}

interface SkillContext {
  taskId: string;
  capabilityToken: string;
  permissionTier: string;
  config: Record<string, unknown>;
}

interface SkillResult {
  success: boolean;
  output?: string;
  error?: string;
  artifacts?: Array<{ path: string; content: string }>;
}
```

#### Built-in Skills (28 total)

**T0 - Read-only (no confirmation)**
- `code.review` - Code review
- `docs.read` - Documentation reading
- `deps.check` - Dependency vulnerability check
- `repo.search` - Repository search

**T1 - Tap to confirm**
- `code.generate` - Code generation
- `code.refactor` - Code refactoring
- `code.document` - Documentation generation
- `api.design` - API design
- `ci.configure` - CI/CD configuration
- `db.schema` - Database schema
- `copy.generate` - Copy generation
- `script.draft` - Script drafting
- `script.outline` - Script outlining
- `seo.optimize` - SEO optimization

**T2 - Type to confirm**
- `git.commit` - Git commit
- `code.test` - Test generation
- `db.migrate` - Database migration
- `docker.build` - Docker build
- `video.edit` - Video editing
- `video.generate` - Video generation
- `music.generate` - Music generation
- `music.extend` - Music extension
- `music.remix` - Music remix

**T3 - TOTP verification required**
- `shell.execute` - Shell command execution
- `file.delete` - File deletion
- `git.force_push` - Git force push
- `db.drop` - Drop database objects
- `aws.lambda` - AWS Lambda deployment
- `deploy.kubectl` - Kubernetes deployment

#### Skill Loader
```typescript
class SkillLoader {
  async loadSkills(skillsPath: string): Promise<Map<string, Skill>>
  getSkill(name: string): Skill | undefined
  validateInput(skill: Skill, input: unknown): boolean
  async executeSkill(skillName: string, ctx: SkillContext, input: unknown): Promise<SkillResult>
}
```

#### SKILL.md Plugin System
The SkillPlugin component (`src/skill_plugin.rs`) allows skills to be defined via Markdown files (SKILL.md) instead of TypeScript:

**SKILL.md Format:**
```markdown
# Skill Name
version: 0.1.0
tier: T2

## Description
What this skill does

## Input Schema
{JSON schema}

## Output Schema
{JSON schema}

## Implementation
{Python or bash code}
```

**Relationship to TypeScript Skills:**
- SKILL.md skills are loaded by SkillPlugin in the Router (L2)
- TypeScript skills are loaded by SkillLoader in Skills Framework (L4)
- Both paths go through the same security pipeline (capability token verification)
- SKILL.md provides a simpler definition format; TypeScript provides full control
- All skills (regardless of source) must declare their permission tier

---

### L5: Execution Engine (Python)
**Location**: `execution/`
**Technology**: Python, asyncio, Agent Zero fork

#### Purpose
- Agent loop for complex tasks (Plan → Act → Observe → Reflect)
- Firecracker VM isolation
- Tool execution framework
- Budget enforcement in cents

#### L2 → L5 Boundary: Rust orchestrates Python

The Router (L2) invokes Python (L5) via **subprocess**:
1. **DeepTaskWorker** (Rust) receives a deep task message from the MessageBus
2. **VmPool** acquires a VM (Docker/Firecracker/gVisor) from the pool
3. **DeepTaskWorker** spawns Python subprocess inside the VM:
   ```
   python -m apex_agent.agent --task-id <id> --input "<content>"
   ```
4. Python agent executes the agent loop, writing outputs to stdout
5. **DeepTaskWorker** captures stdout, parses results, stores in database
6. VM is released back to the pool

**There are TWO agent loop implementations:**
- **`core/router/src/agent_loop.rs` (Rust)**: Used for simpler orchestration within Router process
- **`execution/src/apex_agent/agent.py` (Python)**: Used for complex tasks requiring full tool suite and VM isolation

The Rust agent loop is invoked directly (no subprocess). The Python agent loop runs in an isolated VM.

#### Agent Loop
```python
class ApexAgent:
    SYSTEM_PROMPT = """You are an autonomous AI agent that executes tasks by planning, acting, observing, and reflecting.
    
Available Tools:
- code.generate: Generate code from natural language
- code.review: Review code for issues
- shell.execute: Execute shell commands (requires T3 verification)
- file.read/file.write: File operations
- web.search/web.fetch: Web operations

Loop Pattern:
1. PLAN: Analyze task and decide next action
2. ACT: Execute the chosen action using a tool
3. OBSERVE: Check the result of the action
4. REFLECT: Determine if task is complete

Respond with "TASK_COMPLETE: <summary>" when done."""

    async def run(self, task: str) -> dict:
        """Execute a deep task using the agent loop."""
        
    async def _execute_loop(self, context: AgentContext) -> str:
        """Main agent loop: plan → act → observe → reflect."""
        for step in range(self.config.max_steps):
            # Budget check
            if context.total_cost_cents >= self.config.max_cost_cents:
                raise BudgetExceededError(context.total_cost_cents)
            
            # Plan
            plan = await self._plan(context)
            
            # Act
            result = await self._act(plan, context)
            
            # Observe & Reflect
            context.add_tool_result(plan.get("tool"), result)
            
            if self._is_complete(result.output):
                return result.output
                
        return "Task did not complete within step limit"

@dataclass
class AgentConfig:
    max_steps: int = 50
    max_cost_usd: float = 5.0
    max_cost_cents: int = 500  # Precise budget in cents
    allowed_domains: list[str] = field(default_factory=list)  # Empty = all allowed, ["*"] = wildcard
    allowed_skills: list[str] = field(default_factory=list)  # Empty = all allowed
    timeout_seconds: int = 300
    llm_url: str = "http://localhost:8080"
    llm_model: str = "qwen3-4b"
    context_window_tokens: int = 32768
```

#### Domain Restriction Enforcement
- **Empty list** (`[]`): All domains allowed (default)
- **Wildcard** (`["*"]`): All domains allowed
- **Specific list** (`["github.com", "api.example.com"]`): Only listed domains allowed
- **Enforcement level**: Python-level check in `web.search` and `web.fetch` tools (soft limit, not kernel-level)

#### Default Tools (8 built-in)
- `code.generate` - Generate code from description
- `code.review` - Review code for issues
- `docs.read` - Read documentation
- `shell.execute` - Execute shell (T3 only)
- `file.read` / `file.write` - File operations
- `web.search` / `web.fetch` - Web operations

#### Additional Python Modules
- `tool_gen.py` - Dynamic tool generation
- `tir.py` - Tool-integrated reasoning
- `curriculum.py` - Learning from execution history

---

### L6: React UI
**Location**: `ui/`
**Technology**: React 18, TypeScript, Tailwind CSS, Zustand, Socket.io

#### Components
| Component | File | Purpose |
|-----------|------|---------|
| App | `src/App.tsx` | Main app with routing, header |
| Chat | `src/components/chat/Chat.tsx` | Main chat interface with message queue |
| TaskSidebar | `src/components/chat/TaskSidebar.tsx` | Active tasks panel |
| ProcessGroup | `src/components/chat/ProcessGroup.tsx` | Execution trace display |
| StepDetailModal | `src/components/chat/StepDetailModal.tsx` | Step debugging modal (double-click) |
| ConfirmationGate | `src/components/chat/ConfirmationGate.tsx` | T1-T3 confirmation UI |
| Skills | `src/components/skills/Skills.tsx` | Skill marketplace |
| MemoryViewer | `src/components/memory/MemoryViewer.tsx` | 3-tab memory viewer |
| KanbanBoard | `src/components/kanban/KanbanBoard.tsx` | Task board |
| Workflows | `src/components/workflows/Workflows.tsx` | Workflow automation |
| AuditLog | `src/components/audit/AuditLog.tsx` | Audit trail with CSV export |
| ChannelManager | `src/components/channels/ChannelManager.tsx` | Channel CRUD |
| DecisionJournal | `src/components/journal/DecisionJournal.tsx` | Decision tracking |
| Settings | `src/components/settings/Settings.tsx` | Full-page settings with tabs |
| ConfigViewer | `src/components/settings/ConfigViewer.tsx` | Runtime configuration viewer |
| LlmManager | `src/components/settings/LlmManager.tsx` | Multi-LLM provider configuration |
| ThemeEditor | `src/components/settings/ThemeEditor.tsx` | Theme customization with preview |
| Sidebar | `src/components/ui/Sidebar.tsx` | Navigation sidebar with 5 top-level items + 7 collapsible submenus |
| ThemeSelector | `src/components/ui/Sidebar.tsx` | Theme switching (Modern/Amiga/AgentZero) |
| WebSocket | `src/lib/websocket.ts` | Real-time client |
| API | `src/lib/api.ts` | HMAC-signed fetch |

#### Theme System

APEX implements a CSS-variable-based theme system with three built-in themes:

| Theme | ID | Description |
|-------|-----|-------------|
| Modern 2026 | `modern-2026` | Default dark theme with cyan accents |
| Amiga Workbench | `amiga` | Classic Amiga-inspired aesthetic |
| AgentZero | `agentzero` | Dark navy with cyan accents, inspired by OpenClaw/AgentZero |

**Theme Architecture:**
- Themes are defined as TypeScript objects with color tokens
- `useTheme` hook injects CSS variables into `:root`
- Theme preference persisted in localStorage (`apex-theme-id`)
- Progressive migration: components can use theme tokens or fallback to Tailwind defaults

**AgentZero UI Migration (v1.3.2):**
- All UI components migrated to use CSS variables for consistent theming
- Primary color: Indigo (#4248f1) for AgentZero theme
- Rounded corners: `rounded-xl` throughout
- SVG icons replacing emoji for status indicators
- Components use `bg-[var(--color-panel)]`, `border-[var(--color-border)]`, `text-[var(--color-text)]`

**Theme Tokens:**
```
--color-bg-base, --color-bg-elevated, --color-bg-overlay
--color-text-primary, --color-text-secondary, --color-text-muted
--color-primary, --color-primary-hover, --color-primary-active
--color-accent-success, --color-accent-warning, --color-accent-error
--color-agent-active, --color-agent-thinking, --color-agent-alert
--color-badge-gen, --color-badge-use, --color-badge-exe, ...
```

**Amiga-Specific Tokens:**
```
--color-chrome-titlebar-active, --color-chrome-titlebar-inactive
--color-chrome-button-raised, --color-chrome-button-depressed
--color-chrome-window-border
```

**Files:**
- `src/themes/types.ts` - Theme type definitions
- `src/themes/modern-2026.ts` - Modern theme tokens
- `src/themes/amiga.ts` - Amiga theme tokens
- `src/themes/agentzero.ts` - AgentZero theme tokens
- `src/hooks/useTheme.ts` - Theme context and injection

#### State Management
```typescript
interface AppState {
  messages: Message[];
  tasks: Task[];
  connectionState: 'connected' | 'degraded' | 'disconnected';
  sessionCost: number;
  totalCost: number;
  pendingConfirmation: PendingConfirmation | null;
  messageQueue: string[];
  isProcessingQueue: boolean;
}
```

---

## Data Flows

### Task Creation Flow
```
1. User sends message via UI/Adapter
   ↓
2. Gateway receives and normalizes
   ↓
3. HMAC signs request
   ↓
4. POST /api/v1/tasks to Router
   ↓
5. TaskClassifier determines tier (Instant/Shallow/Deep)
   ↓
6. Task saved to SQLite via TaskRepository

#### Task Classification Logic

The Classifier (`src/classifier.rs`) determines task tier based on:

| Input Factor | Classification |
|--------------|----------------|
| Empty or greeting | **Instant** (respond immediately, no skill) |
| Skill name detected (e.g., "generate code for...") | **Shallow** (execute skill) |
| Complex/long input, ambiguous intent | **Deep** (LLM-powered) |

**Classification algorithm:**
1. **Intent detection**: If input contains skill keywords (code., git., shell., etc.) → Shallow
2. **Complexity check**: If input > 500 chars or multiple sentences → Deep
3. **Explicit routing**: If `?tier=deep` in request → Deep
4. **Default**: Instant for simple queries, Shallow otherwise

**User tier capping**: The request's permission tier (from auth) caps the maximum task tier. A T1 user cannot trigger T3 operations.

**Memory integration**: The enhanced memory system can influence classification by:
- Searching relevant past tasks in `memory_chunks`
- If similar tasks exist, using their tier as a hint

### Skill Execution Pipeline
```
1. Router receives skill execution request
   ↓
2. Verify capability token and permission tier
   ↓
3. Check circuit breaker for skill
   ↓
4. SkillWorker receives from message bus
   ↓
5. Acquire slot from SkillPool (pre-warmed Bun processes)
   ↓
6. Write IPC request with UUID correlation
   ↓
7. Read IPC response from pool worker (skills/pool_worker.ts)
   ↓
8. Release slot back to pool
   ↓
9. Validate output against skill's outputSchema
   ↓
10. Update task status and output
   ↓
11. Publish completion to message bus
```

**Fallback**: If SkillPool unavailable, SkillWorker spawns fresh Bun process (legacy mode)

**Fallback Observability**:
- **Trigger conditions**: Pool not started (`APEX_SKILL_POOL_ENABLED=0`), all slots busy (timeout after `APEX_SKILL_POOL_ACQUIRE`), or health check failed
- **Latency impact**: Legacy mode adds ~50-100ms per execution (process spawn overhead)
- **Detection**: 
  - Log level `warn` with message "SkillPool unavailable, falling back to spawn mode"
  - Metric `skill_pool_fallback_total` increments on each fallback
- **Recovery**: If pool was disabled, stays in legacy mode. If slots were busy, pool automatically recovers on next acquire attempt.

### SkillPool Architecture

**Purpose**: Reduce skill execution latency from ~100ms to ~10-15ms via pre-warmed processes

**Components**:
- `SkillPool` (`src/skill_pool.rs`): Pool manager with mpsc-based slot management
- `SkillPoolIpc` (`src/skill_pool_ipc.rs`): UUID-based request/response framing for multiplexed IPC
- `pool_worker.ts` (`skills/pool_worker.ts`): Bun REPL dispatcher

**Configuration**:
| Env Variable | Default | Description |
|--------------|---------|-------------|
| `APEX_SKILL_POOL_SIZE` | 4 | Number of pre-warmed workers |
| `APEX_SKILL_POOL_WORKER` | `./skills/pool_worker.ts` | Path to Bun dispatcher |
| `APEX_SKILL_POOL_TIMEOUT` | 30000 | Request timeout (ms) |
| `APEX_SKILL_POOL_ACQUIRE` | 5000 | Slot acquire timeout (ms) |

**Performance Target**: ~10-15ms vs ~100ms (spawn mode)

### Deep Task (LLM) Flow
```
1. DeepTaskMessage published to message bus
   ↓
2. DeepTaskWorker acquires VM from pool
   ↓
3. AgentLoop initialized with:
   - Task content
   - Max steps
   - Budget USD
   - Time limit
   ↓
4. Llama client sends prompts to llama-server
   ↓
5. Agent loop executes steps:
   - Reasoning → Action → Observation
   - Each step within budget/time limits
   ↓
6. Final output stored in database
   ↓
7. VM released back to pool
```

### Permission Tier Flow
```
T0 - Read-only:
  Request → Classify → Execute → Response

T1 - Tap to confirm:
  Request → Classify → Confirm (tap) → Execute → Response

T2 - Type to confirm:
  Request → Classify → Confirm (type action) → Execute → Response

T3 - TOTP verify:
  Request → Classify → Confirm (TOTP code) → Execute → Response
```

---

## Security Model

### Permission Tiers
| Tier | Name | Confirmation | Skills |
|------|------|--------------|--------|
| T0 | Read-only | None | code.review, docs.read, deps.check, repo.search |
| T1 | Tap | Tap button | code.generate, code.refactor, code.document, api.design, ci.configure, db.schema, copy.generate, script.draft, script.outline, seo.optimize |
| T2 | Type | Type action name | git.commit, code.test, db.migrate, docker.build, video.edit, video.generate, music.* |
| T3 | TOTP | 6-digit code | shell.execute, file.delete, git.force_push, db.drop, aws.lambda, deploy.kubectl |

### HMAC Authentication
- All API requests signed with `X-APEX-Signature`
- Timestamp in `X-APEX-Timestamp` (5-minute window)
- Prevents replay attacks

### Capability Tokens
Generated for each task with:
- Task ID
- Permission tier
- Allowed actions
- Budget (USD)
- TTL (seconds)

### VM Security
- Docker with `--network=none` (isolated)
- `--read-only` filesystem
- `--pids-limit=256` (process limit)
- Memory limits enforced

### Additional Security Considerations

#### Skill Pool Trust Boundary
- Pool worker processes run Bun code from `skills/` directory
- **No sandboxing** applied to pool workers beyond OS-level process isolation
- **T3 tier enforcement**: `shell.execute` and other T3 skills check permission tier at Router level before IPC call; pool workers trust the Router's authorization header
- **Module access**: Pool workers can import any module available in the Bun runtime

#### Embedding Server (Port 8081)
- System makes outbound HTTP calls to local llama-server for embeddings
- **Network surface**: Only localhost (127.0.0.1) by default
- **Integrity risk**: If embedding server is compromised, memory index could be poisoned with malicious embeddings
- **Mitigation**: Use localhost-only binding, verify server fingerprints

#### Working Memory Persistence
- `working_memory` table stores scratchpad content and entity data in plaintext SQLite
- **Database file access**: If `apex.db` is accessed directly, all working memory is readable
- **Acceptable for single-user**: Assumes host filesystem permissions restrict access

#### Single-User Assumption
- No user isolation between tasks
- All tasks run with same permissions as the APEX process
- Database and memory files are not encrypted at rest

---

## Integration Points

### Gateway ↔ Router
```
HTTP REST API (port 3000)
├── HMAC-SHA256 signing
├── X-APEX-Signature header
└── X-APEX-Timestamp header (5-min window)
```

### Router ↔ Workers
```
Tokio Broadcast Channels
├── Task messages
├── Skill execution messages
├── Deep task messages
└── Confirmation messages
```

### Router ↔ Memory
```
SQLite via sqlx
├── Task persistence
├── Message storage
├── Skill registry
└── Audit logging
```

### UI ↔ Router
```
HTTP + HMAC + WebSocket
├── REST API for operations
├── WebSocket for real-time updates
└── SSE fallback for events
```

### Distributed Deployment
```
NATS JetStream (optional)
├── Task distribution
├── Result aggregation
└── Cross-node communication
```

---

## Configuration

APEX uses a unified configuration system (`AppConfig`). All settings can be configured via environment variables, YAML config file, or defaults.

### Configuration API
- `GET /api/v1/config` - Get all configuration variables
- `GET /api/v1/config/summary` - Get configuration summary with validation

### Environment Variables
See `AGENTS.md` for the complete reference. Key variables:

| Variable | Layer | Default | Description |
|----------|-------|---------|-------------|
| `APEX_PORT` | Router | 3000 | Router port |
| `APEX_HOST` | Router | 0.0.0.0 | Router host |
| `APEX_SHARED_SECRET` | All | dev-secret | HMAC secret |
| `APEX_AUTH_DISABLED` | All | false | Disable auth |
| `APEX_USE_LLM` | Router | false | Enable LLM |
| `LLAMA_SERVER_URL` | Router | localhost:8080 | LLM endpoint |
| `LLAMA_MODEL` | Router | qwen3-4b | Model name |
| `APEX_USE_DOCKER` | Router | false | Enable Docker |
| `APEX_USE_FIRECRACKER` | Router | false | Enable Firecracker |
| `APEX_USE_GVISOR` | Router | false | Enable gVisor |
| `APEX_DATABASE_URL` | Router | sqlite:apex.db | Database connection |
| `APEX_NATS_ENABLED` | Router | false | Enable NATS |
| `APEX_NATS_URL` | Router | 127.0.0.1:4222 | NATS URL |
| `APEX_NATS_SUBJECT_PREFIX` | Router | apex | NATS prefix |
| `APEX_JSON_LOGS` | Router | false | JSON logging |
| `APEX_LOG_LEVEL` | Router | info | Log level |
| `APEX_HEARTBEAT_ENABLED` | Router | false | Enable heartbeat |
| `APEX_HEARTBEAT_INTERVAL` | Router | 60 | Heartbeat interval (minutes) |
| `APEX_SOUL_DIR` | Router | ~/.apex/soul | Soul directory |
| `APEX_SKILLS_DIR` | Router | ./skills | Skills directory |

---

## Technology Stack

| Layer | Technology | Key Libraries |
|-------|------------|---------------|
| L1 Gateway | TypeScript | NATS, fastify |
| L2 Router | Rust | axum, tokio, sqlx |
| L3 Memory | Rust/SQLite | sqlx |
| L4 Skills | TypeScript | zod |
| L5 Execution | Python | asyncio, loguru |
| L6 UI | React/TypeScript | React 18, zustand, socket.io |

---

## Testing

| Component | Tests | Location |
|-----------|-------|----------|
| Router (unit) | 158 | `core/router/src/*_test.rs` |
| Router (integration) | 59 | `core/router/tests/` |
| Python (execution) | 20 | `execution/tests/` |
| Gateway | 8 | `gateway/src/*.test.ts` |
| Skills | 8 | `skills/src/*.test.ts` |
| UI | 23 | `ui/src/**/*.test.tsx` |
| **Total** | **276** | |

> Note: 1 test ignored (requires llama-server running)

---

## Version History

> **Note on versioning**: v1.x marks production-readiness milestones. v0.x were internal development versions. v1.3.1 represents the current pre-alpha state with full feature set including SystemComponent trait.

- **v1.3.1** (2026-03-10): SystemComponent trait, MCP Marketplace UI, TaskClassifier rules, Capability enforcement (fail-closed), Production hardening docs
- **v1.3.0** (2026-03-06): Architecture fixes - Migration 013, WAL mode, atomic writes, Bun pool tier enforcement, cache invalidation, C4 config injection
- **v1.3.0** (2026-03-06): Enhanced Memory System - vector similarity search, FTS5, working memory (per-task scratchpad), background indexer, narrative integration
- **v1.2.0** (2026-03-05): UI overhaul, settings tabs, memory viewer, workflow visualizer
- **v1.1.0** (2026-03-04): Skill quick-launch, command bar, task board
- **v1.0.0** (2026-03-03): Core feature freeze - Firecracker VM, Agent Zero loop, SKILL.md plugins, PostgreSQL support
- **v0.1.2** (2026-03-03): Added Channels, Decision Journal, WebSocket server, NATS integration
- **v0.1.1** (2026-01-XX): Added HMAC auth, TOTP verification, shell.execute → T3
- **v0.1.0** (2026-01-XX): Initial release

---

## Architecture Fixes Applied (v1.3.1)

### Correctness Fixes
- **Migration 013**: Added memory_chunks, memory_fts, memory_entities, memory_index_state, working_memory tables
- **WAL Mode**: SQLite now uses WAL mode with optimized pragma settings for concurrency
- **Atomic Writes**: NarrativeService writes to .tmp then renames - prevents corrupt index entries

### Security Fixes  
- **B1 - Capability Enforcement**: Bun pool validates tier permissions before executing skills
- **B2 - Cache Invalidation**: Added `__cache_bust__` message to clear skill cache

### Complexity Reduction
- **C4 - Config Injection**: Thread-local config override for test isolation, config in AppState

---

*Documentation generated: 2026-03-06*
