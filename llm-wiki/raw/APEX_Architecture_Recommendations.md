# APEX Architecture — Recommendations

**Based on**: Architecture critique of APEX v1.3.0  
**Scope**: Fixes that reduce complexity, improve security, and improve performance  
**Constraint**: No recommendation removes existing functionality  
**Date**: 2026-03-06

---

## How to read this document

Each recommendation is self-contained. They are grouped by category and ordered by impact-to-effort ratio within each group. Every recommendation states what changes, what stays the same, and why it is worth doing. Recommendations that touch the same system are explicitly cross-referenced.

**Impact key:**  
🔴 Correctness risk — can produce wrong behaviour silently  
🟠 Security gap — can be exploited or bypassed  
🟡 Architectural debt — correct today, costly tomorrow  
🟢 Performance/maintainability improvement

---

## Group A: Correctness Fixes

These are the highest priority. They address silent failure modes that corrupt data or produce wrong behaviour without surfacing an error.

---

### A1 — Drop the dual cost columns; make cents authoritative 🔴

**Problem**: The `tasks` table has both `cost_estimate_usd REAL` / `actual_cost_usd REAL` and `cost_estimate_cents INTEGER` / `actual_cost_cents INTEGER`. Two representations of the same value with no designated authority. Any code path that writes one and reads the other produces silent budget enforcement errors. Floating-point REAL columns accumulate rounding error; 99 cents stored as 0.99 REAL is 0.9899999... in IEEE 754.

**Fix**: Write a migration that drops the REAL columns and makes cents the sole representation. Expose USD only as a computed value at the API layer.

```sql
-- Migration: drop_cost_usd_columns
ALTER TABLE tasks DROP COLUMN cost_estimate_usd;
ALTER TABLE tasks DROP COLUMN actual_cost_usd;
```

```rust
// In TaskRepository or API layer — compute USD only at read time
pub fn cents_to_display_usd(cents: i64) -> String {
    format!("${:.2}", cents as f64 / 100.0)
}
```

All budget checks in `AgentConfig`, `CostEstimator`, and `DeepTaskWorker` operate on `i64` cents only. Never store or compare USD floats.

**What stays the same**: The API response can still return a `cost_usd` display field — it just becomes a derived value, not a stored one.  
**Effort**: Small — one migration, grep-and-replace on all cost comparisons.

---

### A2 — Replace broadcast channels with typed mpsc channels 🔴

**Problem**: `MessageBus` uses Tokio broadcast channels. Broadcast delivers every message to every subscriber — `SkillWorker`, `DeepTaskWorker`, and `T3ConfirmWorker` all receive every message type and discard irrelevant ones. More critically, broadcast channels have a fixed ring buffer. If any subscriber is slow, the buffer fills and **messages are silently dropped**. For a system with audit logging and budget enforcement, silent message loss is a correctness failure.

**Fix**: Replace with typed `mpsc` channels — one sender per message type, one receiver per worker. No subscriber ever receives a message it does not own.

```rust
// message_bus.rs — before
pub struct MessageBus {
    pub tx: broadcast::Sender<BusMessage>,
}

// message_bus.rs — after
pub struct MessageBus {
    pub skill_tx:   mpsc::Sender<SkillExecutionMessage>,
    pub deep_tx:    mpsc::Sender<DeepTaskMessage>,
    pub confirm_tx: mpsc::Sender<ConfirmationMessage>,
    pub task_tx:    broadcast::Sender<TaskMessage>,  // broadcast is correct here —
                                                     // task updates ARE fan-out to UI
}
```

`TaskMessage` (status updates broadcast to WebSocket clients) is the one legitimate use of broadcast. Keep it. Route execution messages through `mpsc`.

Add bounded channel sizes with explicit backpressure:

```rust
let (skill_tx, skill_rx)   = mpsc::channel::<SkillExecutionMessage>(256);
let (deep_tx, deep_rx)     = mpsc::channel::<DeepTaskMessage>(64);
let (confirm_tx, confirm_rx) = mpsc::channel::<ConfirmationMessage>(64);
```

When a channel is full, the send returns `Err` immediately — the caller gets a proper error and the task fails visibly, rather than silently vanishing.

**What stays the same**: All three workers keep their existing message types and logic. The routing topology is unchanged. NATS integration still replaces channels in distributed mode.  
**Effort**: Medium — touch `message_bus.rs`, three worker `recv` loops, and startup wiring.

---

### A3 — Coordinate NarrativeService and BackgroundIndexer writes 🔴

**Problem**: `NarrativeService` writes markdown files that `BackgroundIndexer` reads and indexes. No coordination exists between them. Two failure modes:

1. The indexer reads a file that `NarrativeService` is mid-write — partial content gets chunked and embedded. The resulting index entry is corrupt.
2. On some filesystems (Windows NTFS with low-resolution mtime, network shares), mtime granularity is 2 seconds. A write followed by an index check within the same window appears as "unchanged" — the new content is never indexed.

**Fix**: Two changes.

**First**, `NarrativeService` notifies the indexer after every write via a Tokio channel rather than the indexer polling mtime:

```rust
// narrative.rs
impl NarrativeService {
    pub async fn write_entry(&self, path: &Path, content: &str) -> Result<()> {
        // Atomic write: write to .tmp, then rename
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, content).await?;
        fs::rename(&tmp, path).await?;  // atomic on POSIX; near-atomic on Windows

        // Notify indexer
        self.indexer_tx.send(IndexJob::File(path.to_path_buf())).await.ok();
        Ok(())
    }
}
```

The atomic rename (write-to-temp, rename) ensures the indexer never reads a partial file. The notification means the indexer does not rely on mtime polling for freshly written files.

**Second**, remove mtime-based deduplication for files written within the last 10 seconds — treat them as always dirty. This handles the rare case where an external tool writes a file without going through `NarrativeService`.

**What stays the same**: All existing NarrativeService write paths. The indexer continues to do a full mtime scan on startup for pre-existing files.  
**Effort**: Small — atomic write wrapper, one channel send, one indexer dedup rule change.

---

### A4 — Designate `AgentLoop` (Rust) as the sole agent loop; document L5 relationship 🔴

**Problem**: `AgentLoop` (`src/agent_loop.rs`, Rust) and `ApexAgent` (`execution/`, Python) both implement `Plan → Act → Observe → Reflect`. The relationship is undocumented. If they both run during a deep task, memory injection must happen in both or context is inconsistently available. If only one runs, the other is dead code that will diverge silently.

**Fix**: Make the relationship explicit in code, not just documentation.

**Option A (recommended)**: `AgentLoop` (Rust) is the authoritative loop. L5 Python is an execution *substrate* — it provides the VM sandbox and tool execution runtime, not reasoning. The Rust loop calls Python only to execute individual tool actions inside the VM. Rename `ApexAgent` to `ToolExecutor` to reflect this.

```
AgentLoop (Rust) — owns Plan/Reflect
    │
    └── for each Act step:
        └── ToolExecutor (Python, in VM) — owns sandboxed tool execution
```

**Option B**: Python is the authoritative loop, Rust `AgentLoop` is a lightweight orchestration wrapper that handles budget/auth/database before delegating to Python. In this case, `AgentLoop` in Rust should contain no planning logic — only pre/post hooks.

Pick one. The choice does not matter as much as making it explicit. Whichever is chosen: memory injection, budget enforcement, and audit logging live in the authoritative loop only, not split across both.

**What stays the same**: Firecracker VM isolation. Python tool execution. All existing tool implementations.  
**Effort**: Clarification + targeted refactor. No functional change if Option B is chosen.

---

## Group B: Security Fixes

---

### B1 — Enforce capability tokens inside the Bun pool worker 🟠

**Problem**: T0–T3 tier enforcement is described as happening in Rust before the IPC call. The Bun pool worker receives `{ skill, input }` and executes unconditionally. If the Rust-side check is bypassed — crafted IPC payload, logic bug, direct stdin write during debugging — there is no second enforcement layer. More concretely: any TypeScript skill can call `Bun.spawn()`, `node:child_process`, or `node:fs` directly with no restriction, regardless of its declared tier.

**Fix**: Two layers.

**Layer 1 — Pass the capability token through the IPC protocol:**

```typescript
// Updated IPC request
interface PoolRequest {
  id:             string;
  skill:          string;
  input:          unknown;
  timeout_ms:     number;
  capability_token: string;  // ← add this
  permitted_tier: 'T0' | 'T1' | 'T2' | 'T3';  // ← add this
}
```

The pool worker validates the token before executing. It does not need to do full HMAC verification — that happens in Rust. It checks that `permitted_tier` is consistent with the skill's declared tier:

```typescript
const SKILL_TIERS: Record<string, string> = {
  'code.review': 'T0',
  'shell.execute': 'T3',
  // ... loaded from skills registry at startup
};

async function handleRequest(req: PoolRequest): Promise<PoolResponse> {
  const declaredTier = SKILL_TIERS[req.skill];
  if (!declaredTier) {
    return error(req.id, `Unknown skill: ${req.skill}`);
  }
  if (!tierPermits(req.permitted_tier, declaredTier)) {
    return error(req.id, `Tier violation: ${req.skill} requires ${declaredTier}, got ${req.permitted_tier}`);
  }
  // ... execute
}
```

**Layer 2 — Restrict Bun process capabilities at spawn time:**

```rust
// skill_pool.rs — spawn with restricted permissions
Command::new("bun")
    .arg("run")
    .arg("--allow-read=./skills")     // read skills directory only
    .arg("--allow-net=localhost")      // localhost only — for skills that need HTTP
    .arg(&config.worker_script)
    // No --allow-write, no --allow-env, no --allow-run
```

Bun's `--allow-*` flags (similar to Deno's permission model) are enforced at the runtime level, not just convention. A skill that calls `Bun.spawn()` will receive a runtime permission error, not a silent success.

**What stays the same**: All existing skill execution paths. Skills that need network access (e.g. `seo.optimize`) get `--allow-net` explicitly.  
**Effort**: Medium — IPC protocol extension, pool spawn args, skill tier registry in pool worker.

---

### B2 — Add skill module cache invalidation 🟠

**Problem**: The `skillCache` Map in `pool_worker.ts` is never cleared. A skill updated on disk after the pool starts will not be reloaded until all pool processes restart. This means a security patch to a skill takes effect only on pool restart — creating a window where the old (potentially vulnerable) version continues executing. It also means skill updates during development silently have no effect, leading to debugging confusion.

**Fix**: Add a cache-bust IPC message type:

```typescript
// pool_worker.ts
if (req.skill === '__cache_bust__') {
  const target = req.input as { skill?: string };
  if (target.skill) {
    skillCache.delete(target.skill);
  } else {
    skillCache.clear();  // bust all
  }
  return { id: req.id, ok: true, output: 'cache cleared', duration_ms: 0 };
}
```

```rust
// skill_pool.rs — expose on SkillPool
pub async fn bust_cache(&self, skill_name: Option<&str>) -> Result<(), SkillPoolError> {
    // Send __cache_bust__ to all pool slots
    for slot in self.slots.iter() {
        slot.lock().await.channel.send(
            "__cache_bust__",
            serde_json::json!({ "skill": skill_name }),
            1000,
        ).await?;
    }
    Ok(())
}
```

Wire to `PUT /api/v1/skills/:name` — any skill update automatically invalidates that skill's cache entry across all pool slots. Wire to `POST /api/v1/skills/reload` for full cache flush.

**What stays the same**: Skill caching between calls (performance benefit preserved). Pool processes stay alive.  
**Effort**: Small — one IPC message type, one pool method, one API hook.

---

### B3 — Document and enforce the Workflow tier model 🟠

**Problem**: `GET/POST /api/v1/workflows` exists and `Workflows.tsx` is in the UI, but no tier model is documented for workflows. A workflow that chains `code.review` (T0) followed by `shell.execute` (T3) could potentially inherit the T0 confirmation of the first step and execute the T3 operation without TOTP verification, depending on how workflow execution is implemented.

**Fix**: Workflows must be tier-stamped at creation time as the **maximum tier of any contained step**. A workflow containing any T3 skill is itself a T3 workflow. Execution of a T3 workflow requires TOTP at the workflow level before any step executes.

```rust
// In workflow creation handler
pub async fn create_workflow(
    State(state): State<AppState>,
    Json(payload): Json<CreateWorkflow>,
) -> Result<Json<Workflow>, ApiError> {
    // Compute effective tier = max tier of all steps
    let effective_tier = payload.steps
        .iter()
        .map(|s| state.skill_registry.tier_of(&s.skill))
        .max()
        .unwrap_or(TaskTier::T0);

    // Store effective_tier on the workflow record
    // Execution requires confirmation at effective_tier level
}
```

Add `effective_tier` and `requires_confirmation` columns to the workflows table. Make this visible in `Workflows.tsx` so the user can see before running that a workflow will require TOTP.

**What stays the same**: All workflow functionality. Workflows that contain only T0/T1 skills are unaffected.  
**Effort**: Medium — schema change, creation logic, execution gate, UI display.

---

### B4 — Remove HMAC auth from UI → Router on loopback 🟢 (security simplification)

**Problem**: The UI uses HMAC-signed fetch for every request to the Router. For a local single-user installation where the UI and Router are on the same machine, this adds cryptographic overhead to every API call and increases the attack surface (the shared secret must be accessible to the UI, meaning it lives in the browser environment or a config file readable by the browser).

The real threat HMAC defends against is a forged request from the external network. On loopback, the only process that can reach `localhost:3000` is already running on the user's machine — if an attacker has code execution on the user's machine, HMAC provides no additional protection.

**Fix**: Make UI→Router auth **session-token based** for loopback deployments, and reserve HMAC for Gateway→Router (cross-process, potentially cross-machine).

```rust
// auth.rs — add auth mode
pub enum AuthMode {
    Hmac,           // Gateway → Router, cross-machine
    SessionToken,   // UI → Router, loopback
    Disabled,       // development only
}

impl AuthMode {
    pub fn from_config(cfg: &AppConfig) -> Self {
        if cfg.auth_disabled { return AuthMode::Disabled; }
        if cfg.is_loopback_only() { return AuthMode::SessionToken; }
        AuthMode::Hmac
    }
}
```

Session token: on first UI load, Router generates a random 256-bit token, stores it in memory, and returns it in the initial handshake response (over loopback — no network exposure). The UI stores it in memory (not localStorage). All subsequent requests include it as a Bearer token. No secret needs to be pre-configured; no cryptographic operation per request.

**What stays the same**: HMAC remains for Gateway→Router. External deployments behind a reverse proxy can still use HMAC for the UI. `APEX_AUTH_DISABLED` remains for development.  
**Effort**: Medium — new auth mode, session token generation, UI auth layer update.

---

### B5 — Specify and harden the TaskClassifier 🟠

**Problem**: The classifier that assigns T0–T3 tiers to incoming tasks is undocumented. For a system where T3 gates destructive filesystem operations, "classifier determines tier" with no specified logic is a security architecture gap.

**Fix**: The classifier must be rule-based with an LLM assist, never LLM-primary. Rules are authoritative; LLM can suggest but cannot override.

```rust
// classifier.rs — explicit tier assignment rules
pub struct TaskClassifier {
    rules: Vec<TierRule>,     // hard rules — match → assign tier, no override
    llm:   Option<LlamaClient>, // optional assist for ambiguous cases
}

pub struct TierRule {
    pub pattern:    Regex,
    pub tier:       TaskTier,
    pub reason:     &'static str,
}

const HARD_RULES: &[(&str, TaskTier)] = &[
    // T3 — always, regardless of framing
    (r"(?i)(rm\s+-rf|drop\s+table|force.push|kubectl\s+delete)", TaskTier::T3),
    (r"(?i)(shell\.execute|file\.delete|db\.drop)", TaskTier::T3),

    // T2
    (r"(?i)(git\s+commit|docker\s+build|db\s+migrat)", TaskTier::T2),

    // T0 — read-only signals
    (r"(?i)^(review|read|check|list|show|what is|explain)", TaskTier::T0),
];
```

Rules are evaluated top-down; first match wins. The LLM is only consulted when no rule matches, and its output is **capped at T2** — the LLM cannot assign T3 under any circumstances. T3 is pattern-match only.

Add an audit log entry for every classification decision including which rule or LLM call produced it. This makes tier assignment observable and debuggable.

**What stays the same**: All classification outcomes. Tasks get the same tiers as before; the logic is now explicit.  
**Effort**: Medium — replace opaque classifier with rule table + LLM fallback, add classification audit.

---

## Group C: Complexity Reduction

---

### C1 — Make the Gateway an optional sidecar, not a required process 🟡

**Problem**: The Gateway is a mandatory TypeScript/Node.js process that every task passes through — adding a full HTTP round-trip plus HMAC overhead — even when the user is only using the UI and the REST adapter. For the primary use case (single user, local UI), the Gateway provides zero value and measurable overhead.

**Fix**: Integrate the REST adapter and HMAC signing directly into the Router. The Gateway becomes an optional process you start only if you need Slack, Discord, or Telegram adapters.

```
Before: UI → Gateway (Node.js, port 3001) → Router (Rust, port 3000)
After:  UI → Router (Rust, port 3000) directly
        [Optional] Slack → Gateway sidecar → Router
```

The Router gains a thin REST ingress handler (it already has one — the existing `/api/v1/tasks` endpoint is the target). The Gateway's HMAC signing moves to a per-adapter config rather than a global requirement.

```rust
// Router directly handles REST ingress — no gateway needed for UI/REST
// Gateway is now: npm start -- --adapters slack,telegram
```

**What stays the same**: Full Slack/Discord/Telegram support via the Gateway sidecar. NATS integration unchanged. All existing API endpoints unchanged.  
**Impact**: Removes one runtime process, one HTTP hop, and HMAC overhead from every UI-originated task. Simplifies local development setup (one process instead of two).  
**Effort**: Small — the Router already has the REST endpoint. The change is making the Gateway not required to start.

---

### C2 — Consolidate to four runtimes 🟡

**Current runtimes**: Rust (router), Node.js/TypeScript (gateway), Bun (skill pool), Python (execution engine), llama-server (LLM).

That is five runtimes. For a single-user system this is the primary source of operational complexity: five processes to start, five to monitor, five to configure, five potential startup failures.

**Fix**: With C1 (Gateway becomes optional sidecar), the mandatory runtimes reduce to three: Rust, Bun, and llama-server. Python becomes the fourth only when a deep task runs.

Further: if L5 Python's role is clarified per A4 (ToolExecutor, not planner), evaluate whether the Python agent loop can be replaced with Rust for the planning steps, using Python only for sandboxed tool execution inside Firecracker. This eliminates Python as a persistent process — it becomes a subprocess invoked per-VM.

The practical minimum for full APEX functionality:
- **Rust** (router + memory + agent planning)
- **Bun** (skill pool — already a pool of persistent processes)
- **llama-server** (LLM — external, unavoidable)
- **Python** (inside VM only, per deep task)

**What stays the same**: All language-specific skill code remains TypeScript. Python tools remain Python. No rewrites required for this to work — just the startup dependency graph changes.  
**Effort**: Medium-high. Low-hanging fruit: make Python not a required startup process (only invoked when a deep task fires).

---

### C3 — Unify Heartbeat, Soul, Governance, Moltbook under a lifecycle contract 🟡

**Problem**: Four active components with no documented integration. They are active in the sense that their modules compile and initialise — but their interactions with the task loop, memory system, and audit log are undefined.

**Fix**: Establish a `SystemComponent` trait that all four must implement. This forces integration to be explicit and makes it impossible to add a new "active" component without defining its lifecycle.

```rust
// system_component.rs
#[async_trait]
pub trait SystemComponent: Send + Sync {
    fn name(&self) -> &'static str;

    /// Called once at Router startup, after database is ready.
    async fn on_start(&self, state: &AppState) -> Result<(), ComponentError>;

    /// Called by Heartbeat on each interval fire.
    async fn on_heartbeat(&self, state: &AppState) -> Result<(), ComponentError>;

    /// Called when a task completes (success or failure).
    async fn on_task_complete(&self, task_id: &str, state: &AppState) -> Result<(), ComponentError>;

    /// Called at shutdown — flush any pending state.
    async fn on_shutdown(&self, state: &AppState) -> Result<(), ComponentError>;

    /// Health check — returns false if the component is degraded.
    async fn health(&self) -> bool;
}
```

`Heartbeat`, `Soul`, `Governance`, and `Moltbook` each implement this trait. The Router's startup sequence iterates `Vec<Box<dyn SystemComponent>>` and calls `on_start` on each. This makes the integration surface explicit, testable, and extensible.

**Minimal implementations to make them genuinely active:**
- **Heartbeat**: `on_heartbeat` flushes working memory orphans, runs background indexer sweep, writes a heartbeat audit log entry.
- **Soul**: `on_start` loads `SOUL.md` from `APEX_SOUL_DIR` and injects the persona into the agent system prompt. `on_heartbeat` reloads it if modified.
- **Governance**: `on_task_complete` checks the completed task against constitution rules, logs violations to the audit trail. Read-only enforcement — no task blocking in Phase 1.
- **Moltbook**: `on_task_complete` extracts entities from the task output and upserts them into `memory_entities`.

**What stays the same**: All four components remain. Their functionality expands rather than contracts.  
**Effort**: Medium — define trait, implement four minimal versions, wire into startup sequence.

---

### C4 — Replace AppConfig global with injected config 🟡

**Problem**: `AppConfig::global()` is a global singleton accessed from anywhere in the codebase. Global mutable state makes testing difficult (you cannot run tests in parallel if they share a global config), and it makes it impossible to run multiple Router instances in the same process (relevant for integration tests).

**Fix**: Pass `Arc<AppConfig>` as part of `AppState`. Remove `global()`.

```rust
// Before — called from anywhere
let timeout = AppConfig::global().skill_timeout_ms;

// After — config flows through AppState
let timeout = state.config.skill_timeout_ms;
```

This is a mechanical refactor with no behavioural change. The payoff is that integration tests can construct an `AppState` with a test-specific config without touching a global.

**What stays the same**: All configuration behaviour. All env var loading. YAML config loading.  
**Effort**: Medium — grep-and-replace `AppConfig::global()` with `state.config`, update all handler signatures. No logic changes.

---

## Group D: Performance Improvements

---

### D1 — Add SQLite WAL mode and a dedicated indexer connection 🟢

**Problem**: The background indexer performs thousands of writes to `memory_chunks` and `memory_vec` during the initial index scan. These writes contend with foreground writes (task creation, audit logging, working memory persistence) on the single SQLite write lock. During a large index scan, every foreground write queues behind the indexer.

**Fix**: Two changes, both low-risk.

**First**, enable WAL mode explicitly at startup (sqlx may default to it, but it should be explicit and verified):

```rust
// db.rs — connection setup
sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
sqlx::query("PRAGMA synchronous=NORMAL").execute(&pool).await?; // safe with WAL
sqlx::query("PRAGMA cache_size=-64000").execute(&pool).await?; // 64MB page cache
sqlx::query("PRAGMA temp_store=MEMORY").execute(&pool).await?;
```

WAL mode allows concurrent reads and one writer. Foreground reads are never blocked by the indexer writing.

**Second**, give the background indexer a separate, low-priority SQLite connection. SQLite WAL mode allows multiple concurrent readers and exactly one writer — but two connections can interleave their write transactions. The indexer connection uses `PRAGMA busy_timeout=100` to yield quickly when the foreground holds the write lock:

```rust
// background_indexer.rs
let indexer_pool = SqlitePoolOptions::new()
    .max_connections(1)  // single writer
    .after_connect(|conn, _| Box::pin(async move {
        sqlx::query("PRAGMA busy_timeout=100").execute(conn).await?; // yield fast
        Ok(())
    }))
    .connect(&db_url).await?;
```

The indexer never holds the write lock for more than one chunk insertion at a time. Foreground writes contend for at most ~1ms.

**What stays the same**: All data. All queries. No schema changes.  
**Effort**: Small — pragma setup, separate pool for indexer.

---

### D2 — Parallelise the ApexLoader boot phases 🟢

**Problem**: The ApexLoader boot sequence runs phases serially: Router → Database → Skills → Execution → TOTP → Soul → Heartbeat → Governance → Moltbook → Ready. Each phase waits for the previous to complete. The actual dependency graph is much flatter — most phases are independent.

**Fix**: Run independent phases concurrently using `tokio::join!`:

```rust
// boot sequence — actual dependency graph
let (router_result, db_result) = tokio::join!(
    check_router(),       // independent
    check_database(),     // independent
);

// skills, execution, TOTP can all check concurrently after router+db confirm
let (skills_result, exec_result, totp_result, soul_result) = tokio::join!(
    check_skills(),       // needs router
    check_vm_pool(),      // needs router
    check_totp(),         // needs router
    check_soul(),         // needs filesystem only
);

// Heartbeat, Governance, Moltbook depend on soul being loaded
let (hb_result, gov_result, molt_result) = tokio::join!(
    check_heartbeat(),
    check_governance(),
    check_moltbook(),
);
```

The boot display still shows phases in sequence for UX — phases complete asynchronously but the display advances sequentially. Total boot time drops from ~6.4 seconds (serial) to approximately the longest single phase (~2 seconds).

**What stays the same**: All boot checks. All displayed phases. User sees the same progress display.  
**Effort**: Small — restructure the boot phase array into dependency groups, run groups with `tokio::join!`.

---

### D3 — Add an L1 cache for hot memory search results 🟢

**Problem**: The agent loop calls `memory_service.search()` at the start of every Plan step. On a multi-step deep task with 20 steps, many Plan-step queries are near-identical (the task context barely changes). Each call generates an embedding (200ms), runs two SQLite queries, runs RRF, applies temporal decay and MMR. Most of that work is redundant for repeated queries.

**Fix**: Add a short-TTL in-memory cache keyed on the query text hash:

```rust
// hybrid_search.rs
pub struct HybridSearchEngine {
    // ... existing fields
    cache: Arc<Mutex<lru::LruCache<u64, (Vec<SearchResult>, Instant)>>>,
    cache_ttl: Duration,  // default: 30 seconds
}

impl HybridSearchEngine {
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        // Hash the query text + filters for cache key
        let cache_key = {
            let mut h = DefaultHasher::new();
            query.text.hash(&mut h);
            query.memory_types.hash(&mut h);
            h.finish()
        };

        // Check cache
        {
            let cache = self.cache.lock().await;
            if let Some((results, ts)) = cache.peek(&cache_key) {
                if ts.elapsed() < self.cache_ttl {
                    return Ok(results.clone());
                }
            }
        }

        // Cache miss — run full search
        let results = self.search_uncached(query).await?;

        // Store in cache
        self.cache.lock().await.put(cache_key, (results.clone(), Instant::now()));

        Ok(results)
    }
}
```

Cache TTL of 30 seconds is short enough that memory written mid-task (via `flush_to_longterm`) eventually surfaces. LRU capacity of 64 entries covers any realistic agent session.

**What stays the same**: All search logic. Cache is bypassed on TTL expiry and on explicit `memory_service.invalidate()` call after a new memory entry is written.  
**Effort**: Small — add `lru` crate, wrap search in cache check.

---

### D4 — Stream skill execution results rather than buffering 🟢

**Problem**: The current skill execution path waits for the entire skill output before returning. For skills that produce large outputs (code generation, documentation, data analysis), the user sees nothing until the skill completes. A skill that takes 5 seconds to generate 500 lines of code shows a spinner for 5 seconds then dumps everything at once.

**Fix**: Extend the IPC protocol with streaming chunks, and pipe them through the WebSocket to the UI.

```typescript
// pool_worker.ts — emit chunks as they arrive
async function* executeStreaming(skill: SkillModule, input: unknown): AsyncGenerator<string> {
    if (skill.executeStream) {
        yield* skill.executeStream(input);  // skill opts into streaming
    } else {
        const result = await skill.execute(input);  // fallback: single chunk
        yield result.output ?? '';
    }
}

// Emit each chunk with the request ID
for await (const chunk of executeStreaming(skill, req.input)) {
    process.stdout.write(JSON.stringify({
        id: req.id,
        type: 'chunk',
        data: chunk,
    }) + '\n');
}
// Final message signals completion
process.stdout.write(JSON.stringify({ id: req.id, type: 'done', ok: true }) + '\n');
```

The Rust IPC layer handles `type: 'chunk'` messages by forwarding to the WebSocket manager. The UI appends chunks to the message as they arrive, giving a streaming code-generation experience identical to a chat LLM response.

Skills that do not implement `executeStream` fall back transparently — no change to existing skills required.

**What stays the same**: All existing skills work unchanged. The IPC protocol is backward-compatible (existing `ok/output/error` response shape still valid for non-streaming).  
**Effort**: Medium — IPC protocol extension, Rust chunk forwarding, UI streaming append.

---

## Summary Table

| ID | Category | Issue | Impact | Effort |
|---|---|---|---|---|
| A1 | Correctness | Drop dual cost columns | 🔴 Silent budget errors | Small |
| A2 | Correctness | mpsc channels replace broadcast | 🔴 Silent message loss | Medium |
| A3 | Correctness | Coordinate NarrativeService + Indexer | 🔴 Corrupt index entries | Small |
| A4 | Correctness | One authoritative agent loop | 🔴 Inconsistent memory injection | Medium |
| B1 | Security | Capability enforcement in Bun pool | 🟠 T0 slot bypasses tier system | Medium |
| B2 | Security | Skill cache invalidation | 🟠 Patched skills don't reload | Small |
| B3 | Security | Workflow tier model | 🟠 T3 bypass via workflow chain | Medium |
| B4 | Security | Remove HMAC on loopback UI | 🟢 Simplification + perf | Medium |
| B5 | Security | Specify TaskClassifier rules | 🟠 Tier assignment undefined | Medium |
| C1 | Complexity | Gateway becomes optional sidecar | 🟡 One fewer required process | Small |
| C2 | Complexity | Consolidate to 4 runtimes | 🟡 Simpler ops | Medium-high |
| C3 | Complexity | SystemComponent trait | 🟡 Explicit integration contracts | Medium |
| C4 | Complexity | Inject config, remove global | 🟡 Testability | Medium |
| D1 | Performance | WAL mode + dedicated indexer conn | 🟢 No write contention | Small |
| D2 | Performance | Parallelise boot phases | 🟢 -4s cold boot | Small |
| D3 | Performance | LRU cache for search results | 🟢 Eliminate redundant embeddings | Small |
| D4 | Performance | Streaming skill output | 🟢 Perceived latency | Medium |

### Recommended implementation order

**Do immediately** (small effort, high correctness/security impact): A1, A3, B2, D1, D2  
**Do in the next sprint** (medium effort, significant impact): A2, B1, B5, C1, C3, D3  
**Plan for next milestone** (medium-high effort, architectural): A4, B3, B4, C2, C4, D4

---

*APEX Architecture Recommendations · v1.0 · Based on APEX v1.3.0 critique*
