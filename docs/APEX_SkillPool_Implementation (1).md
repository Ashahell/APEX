# APEX Skill Pool — Implementation Guide

**Solution**: Pre-warmed Bun Process Pool with Multiplexed IPC  
**Target latency**: 10–15ms per skill execution  
**Architecture ref**: APEX v1.0.0 · `core/router/src/skill_pool.rs`

---

## 1. Overview

Skills are TypeScript running in Bun. The router is Rust. Rather than spawning a new process per execution (100–300ms overhead), this solution maintains a pool of pre-warmed Bun processes that persist across skill calls. Each process runs a REPL-style dispatcher that accepts skill execution requests over stdin and writes responses to stdout.

The key design decision is **multiplexed IPC**: each request carries a UUID, and responses carry that UUID back. Multiple in-flight requests can share one pool slot without blocking each other. This gives true concurrency within each process and eliminates the head-of-line blocking problem in a naive line-per-request design.

```
┌─────────────────────────────────────────────────────────────────┐
│                        Rust Router                              │
│                                                                 │
│  SkillWorker ──► SkillPool ──► acquire slot ──► write request  │
│                     │                              │            │
│                     │         ┌────────────────────┘            │
│                     │         ▼                                 │
│              ┌──────┴──────────────────────────┐               │
│              │  PoolSlot 0  PoolSlot 1  Slot N  │               │
│              │  [Bun #0]    [Bun #1]    [Bun N] │               │
│              └─────────────────────────────────┘               │
└─────────────────────────────────────────────────────────────────┘
        │ stdin (newline-delimited JSON + request ID)
        │ stdout (newline-delimited JSON + request ID)
        ▼
┌─────────────────────────┐
│  Bun REPL dispatcher    │
│  skills/pool_worker.ts  │
└─────────────────────────┘
        │ dynamic import
        ▼
┌─────────────────────────┐
│  skills/                │
│    code.generate/       │
│    code.review/         │
│    shell.execute/  ...  │
└─────────────────────────┘
```

---

## 2. File Structure

```
core/router/src/
├── skill_pool.rs          ← pool manager (new)
├── skill_pool_ipc.rs      ← IPC framing + request tracking (new)
├── skill_worker.rs        ← updated to use pool instead of spawn
└── api/
    └── skills.rs          ← unchanged

skills/
├── pool_worker.ts         ← Bun REPL dispatcher (new)
├── pool_worker_test.ts    ← integration test (new)
├── code.generate/
│   └── index.ts
├── code.review/
│   └── index.ts
└── ...

core/router/
└── Cargo.toml             ← add uuid, tokio deps if not present
```

---

## 3. IPC Protocol

All communication is **newline-delimited JSON** over stdin/stdout. Each message is a single line. No binary framing, no length prefixes — newline is the delimiter.

### 3.1 Request (Router → Bun)

```json
{"id":"550e8400-e29b-41d4-a716-446655440000","skill":"code.generate","input":{"prompt":"write a hello world in Python","language":"python"},"timeout_ms":30000}
```

| Field | Type | Description |
|---|---|---|
| `id` | UUID string | Unique request ID — echoed back in response |
| `skill` | string | Skill name, must match directory under `skills/` |
| `input` | object | Validated input payload (pre-validated by Rust) |
| `timeout_ms` | number | Per-request timeout; Bun enforces this independently |

### 3.2 Response (Bun → Router)

**Success:**
```json
{"id":"550e8400-e29b-41d4-a716-446655440000","ok":true,"output":"print('Hello, world!')","duration_ms":8}
```

**Error:**
```json
{"id":"550e8400-e29b-41d4-a716-446655440000","ok":false,"error":"Skill not found: code.nonexistent","duration_ms":2}
```

**Timeout:**
```json
{"id":"550e8400-e29b-41d4-a716-446655440000","ok":false,"error":"timeout","duration_ms":30000}
```

| Field | Type | Description |
|---|---|---|
| `id` | UUID string | Echoed from request |
| `ok` | bool | True if skill executed without error |
| `output` | string? | Skill output (present if `ok: true`) |
| `error` | string? | Error message (present if `ok: false`) |
| `duration_ms` | number | Wall time inside Bun for this execution |

### 3.3 Lifecycle Messages

On startup, the Bun process writes a single ready signal before processing any requests:

```json
{"ready":true,"pid":12345,"bun_version":"1.1.0"}
```

On health ping from the router:

```json
{"ping":true}          ← router sends
{"pong":true,"pid":12345}  ← Bun responds
```

---

## 4. Bun REPL Dispatcher (`skills/pool_worker.ts`)

```typescript
// skills/pool_worker.ts
// Runs as a persistent Bun process.
// Reads newline-delimited JSON from stdin, executes skills, writes responses to stdout.
// Multiple requests can be in-flight concurrently — responses are written as they complete.

import { createInterface } from "readline";

// Cache of loaded skill modules — import once, reuse across calls
const skillCache = new Map<string, SkillModule>();

interface SkillModule {
  execute: (input: unknown) => Promise<SkillResult>;
  healthCheck?: () => Promise<boolean>;
}

interface SkillResult {
  success: boolean;
  output?: string;
  error?: string;
  artifacts?: Array<{ path: string; content: string }>;
}

interface PoolRequest {
  id: string;
  skill: string;
  input: unknown;
  timeout_ms: number;
}

interface PoolResponse {
  id: string;
  ok: boolean;
  output?: string;
  error?: string;
  duration_ms: number;
}

// Load a skill module, caching after first import
async function loadSkill(name: string): Promise<SkillModule> {
  if (skillCache.has(name)) {
    return skillCache.get(name)!;
  }

  // Skills live adjacent to pool_worker.ts
  const skillPath = new URL(`./${name}/index.ts`, import.meta.url).pathname;

  try {
    const mod = await import(skillPath);

    if (typeof mod.execute !== "function") {
      throw new Error(`Skill ${name} does not export an execute() function`);
    }

    skillCache.set(name, mod as SkillModule);
    return mod as SkillModule;
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to load skill ${name}: ${message}`);
  }
}

// Execute a single skill request with timeout enforcement
async function handleRequest(req: PoolRequest): Promise<PoolResponse> {
  const start = performance.now();

  const timeoutPromise = new Promise<never>((_, reject) =>
    setTimeout(() => reject(new Error("timeout")), req.timeout_ms)
  );

  try {
    const skill = await loadSkill(req.skill);

    const resultPromise = skill.execute(req.input).then((result) => {
      if (!result.success) {
        throw new Error(result.error ?? "Skill returned success: false");
      }
      return result.output ?? "";
    });

    const output = await Promise.race([resultPromise, timeoutPromise]);

    return {
      id: req.id,
      ok: true,
      output,
      duration_ms: Math.round(performance.now() - start),
    };
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    return {
      id: req.id,
      ok: false,
      error: message,
      duration_ms: Math.round(performance.now() - start),
    };
  }
}

// Write a JSON response line to stdout
function writeResponse(response: PoolResponse | object): void {
  process.stdout.write(JSON.stringify(response) + "\n");
}

// Announce readiness
writeResponse({ ready: true, pid: process.pid, bun_version: Bun.version });

// Read requests line by line from stdin
const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });

rl.on("line", (line: string) => {
  if (!line.trim()) return;

  let parsed: unknown;
  try {
    parsed = JSON.parse(line);
  } catch {
    // Malformed input — write an error without an id since we can't recover it
    writeResponse({ id: null, ok: false, error: "Malformed JSON request" });
    return;
  }

  // Handle health ping
  if ((parsed as Record<string, unknown>).ping === true) {
    writeResponse({ pong: true, pid: process.pid });
    return;
  }

  // Handle skill execution — fire and forget; response is written when done
  const req = parsed as PoolRequest;
  handleRequest(req).then(writeResponse);
});

rl.on("close", () => {
  // stdin closed — router process terminated or pool slot being recycled
  process.exit(0);
});

// Unhandled rejection safety net — log to stderr only, do not exit
process.on("unhandledRejection", (reason) => {
  process.stderr.write(`[pool_worker] unhandledRejection: ${reason}\n`);
});
```

---

## 5. Rust: IPC Framing (`skill_pool_ipc.rs`)

```rust
// core/router/src/skill_pool_ipc.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

/// A pending in-flight request awaiting a response from Bun.
type PendingMap = Arc<Mutex<HashMap<String, oneshot::Sender<IpcResponse>>>>;

#[derive(Debug, serde::Serialize)]
pub struct IpcRequest {
    pub id:         String,
    pub skill:      String,
    pub input:      serde_json::Value,
    pub timeout_ms: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct IpcResponse {
    pub id:          String,
    pub ok:          bool,
    pub output:      Option<String>,
    pub error:       Option<String>,
    pub duration_ms: u64,
}

/// Wraps the stdin/stdout of a single Bun process.
/// Multiplexes concurrent requests over a single stream pair.
pub struct IpcChannel {
    writer:  Arc<Mutex<BufWriter<ChildStdin>>>,
    pending: PendingMap,
}

impl IpcChannel {
    /// Spawn the reader loop for a process. Must be called once after the
    /// process starts. Runs until the process stdout closes.
    pub fn new(stdin: ChildStdin, stdout: ChildStdout) -> Self {
        let writer  = Arc::new(Mutex::new(BufWriter::new(stdin)));
        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));

        // Spawn reader task — reads lines from Bun stdout and routes to waiters
        let pending_clone = Arc::clone(&pending);
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if line.is_empty() { continue; }

                // Try to deserialise as IpcResponse
                if let Ok(resp) = serde_json::from_str::<IpcResponse>(&line) {
                    let mut map = pending_clone.lock().await;
                    if let Some(tx) = map.remove(&resp.id) {
                        let _ = tx.send(resp); // receiver may have timed out — that's fine
                    }
                }
                // Lifecycle messages (ready, pong) are handled by the pool manager
                // and do not go through IpcChannel — they're read before this task starts.
            }
        });

        Self { writer, pending }
    }

    /// Send a request and wait for the matching response.
    /// The Rust-side timeout is independent of the Bun-side timeout.
    pub async fn send(
        &self,
        skill:      &str,
        input:      serde_json::Value,
        timeout_ms: u64,
    ) -> Result<IpcResponse, SkillPoolError> {
        let id  = Uuid::new_v4().to_string();
        let req = IpcRequest {
            id:      id.clone(),
            skill:   skill.to_string(),
            input,
            timeout_ms,
        };

        let (tx, rx) = oneshot::channel::<IpcResponse>();

        {
            let mut map = self.pending.lock().await;
            map.insert(id.clone(), tx);
        }

        // Write the request line
        {
            let mut w = self.writer.lock().await;
            let mut line = serde_json::to_string(&req)?;
            line.push('\n');
            w.write_all(line.as_bytes()).await?;
            w.flush().await?;
        }

        // Wait for response with timeout
        let duration = std::time::Duration::from_millis(timeout_ms + 1000); // +1s grace
        match tokio::time::timeout(duration, rx).await {
            Ok(Ok(resp)) => Ok(resp),
            Ok(Err(_))   => Err(SkillPoolError::ChannelClosed),
            Err(_)       => {
                // Clean up the pending entry
                self.pending.lock().await.remove(&id);
                Err(SkillPoolError::Timeout)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SkillPoolError {
    #[error("skill pool: timeout waiting for Bun response")]
    Timeout,
    #[error("skill pool: IPC channel closed")]
    ChannelClosed,
    #[error("skill pool: no slots available")]
    NoSlots,
    #[error("skill pool: process failed to start: {0}")]
    SpawnError(String),
    #[error("skill pool: I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("skill pool: serialisation error: {0}")]
    Json(#[from] serde_json::Error),
}
```

---

## 6. Rust: Pool Manager (`skill_pool.rs`)

```rust
// core/router/src/skill_pool.rs

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::{Mutex, Semaphore};
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, info, warn};

use crate::skill_pool_ipc::{IpcChannel, IpcResponse, SkillPoolError};

/// Configuration for the skill process pool.
#[derive(Debug, Clone)]
pub struct SkillPoolConfig {
    /// Number of pre-warmed Bun processes. Default: 4.
    pub pool_size: usize,
    /// Path to pool_worker.ts. Default: ./skills/pool_worker.ts
    pub worker_script: PathBuf,
    /// Path to the skills directory (passed as env var to Bun). Default: ./skills
    pub skills_dir: PathBuf,
    /// Per-request timeout passed to Bun. Default: 30 000ms.
    pub request_timeout_ms: u64,
    /// How long to wait for a pool slot before returning NoSlots. Default: 5 000ms.
    pub acquire_timeout_ms: u64,
    /// Health ping interval. Default: 30s.
    pub health_check_interval: Duration,
}

impl Default for SkillPoolConfig {
    fn default() -> Self {
        Self {
            pool_size:              4,
            worker_script:          PathBuf::from("./skills/pool_worker.ts"),
            skills_dir:             PathBuf::from("./skills"),
            request_timeout_ms:     30_000,
            acquire_timeout_ms:     5_000,
            health_check_interval:  Duration::from_secs(30),
        }
    }
}

/// A single pool slot — one Bun process + its IPC channel.
struct PoolSlot {
    channel: IpcChannel,
    pid:     u32,
}

pub struct SkillPool {
    slots:     Arc<Mutex<Vec<PoolSlot>>>,
    semaphore: Arc<Semaphore>,
    config:    SkillPoolConfig,
}

impl SkillPool {
    /// Create and warm the pool. Blocks until all processes are ready.
    pub async fn new(config: SkillPoolConfig) -> Result<Arc<Self>, SkillPoolError> {
        let pool = Arc::new(Self {
            slots:     Arc::new(Mutex::new(Vec::with_capacity(config.pool_size))),
            semaphore: Arc::new(Semaphore::new(config.pool_size)),
            config:    config.clone(),
        });

        info!("SkillPool: warming {} Bun processes", config.pool_size);

        for i in 0..config.pool_size {
            match pool.spawn_slot().await {
                Ok(slot) => {
                    info!("SkillPool: slot {} ready (pid {})", i, slot.pid);
                    pool.slots.lock().await.push(slot);
                }
                Err(e) => {
                    error!("SkillPool: failed to start slot {}: {}", i, e);
                    return Err(e);
                }
            }
        }

        // Start background health checker
        let pool_clone = Arc::clone(&pool);
        tokio::spawn(async move {
            pool_clone.health_check_loop().await;
        });

        Ok(pool)
    }

    /// Spawn a single Bun process and wait for its READY signal.
    async fn spawn_slot(&self) -> Result<PoolSlot, SkillPoolError> {
        let mut child = Command::new("bun")
            .arg("run")
            .arg(&self.config.worker_script)
            .env("APEX_SKILLS_DIR", &self.config.skills_dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| SkillPoolError::SpawnError(e.to_string()))?;

        let pid    = child.id().unwrap_or(0);
        let stdin  = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        // Forward stderr to tracing
        if let Some(stderr) = child.stderr.take() {
            let pid_copy = pid;
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    warn!("SkillPool [pid {}] stderr: {}", pid_copy, line);
                }
            });
        }

        // Detach child — kill_on_drop handles cleanup
        tokio::spawn(async move { let _ = child.wait().await; });

        // Read the READY line from stdout before handing to IpcChannel
        let mut stdout_reader = BufReader::new(stdout);
        let mut ready_line = String::new();

        tokio::time::timeout(Duration::from_secs(10), stdout_reader.read_line(&mut ready_line))
            .await
            .map_err(|_| SkillPoolError::SpawnError("Timed out waiting for READY".into()))?
            .map_err(SkillPoolError::Io)?;

        // Verify the ready signal
        let ready: serde_json::Value = serde_json::from_str(ready_line.trim())
            .map_err(|_| SkillPoolError::SpawnError(
                format!("Unexpected READY payload: {}", ready_line.trim())
            ))?;

        if ready["ready"] != true {
            return Err(SkillPoolError::SpawnError(
                format!("Expected ready:true, got: {}", ready_line.trim())
            ));
        }

        // IpcChannel takes ownership of stdin and the remaining stdout
        // We need to reconstruct stdout from the BufReader — extract inner
        let stdout_inner = stdout_reader.into_inner();
        let channel = IpcChannel::new(stdin, stdout_inner);

        Ok(PoolSlot { channel, pid })
    }

    /// Execute a skill. Acquires a slot, sends the request, returns the response.
    pub async fn execute(
        &self,
        skill: &str,
        input: serde_json::Value,
    ) -> Result<IpcResponse, SkillPoolError> {
        // Acquire a permit (blocks if all slots are busy)
        let acquire_timeout = Duration::from_millis(self.config.acquire_timeout_ms);

        let _permit = tokio::time::timeout(acquire_timeout, self.semaphore.acquire())
            .await
            .map_err(|_| SkillPoolError::NoSlots)?
            .map_err(|_| SkillPoolError::ChannelClosed)?;

        // Pick the first available slot (FIFO is fine for a small pool)
        let slot_index = {
            let slots = self.slots.lock().await;
            // Semaphore guarantees a slot exists
            0 // simplified — production: use a per-slot semaphore or channel
        };

        let response = {
            let slots = self.slots.lock().await;
            slots[slot_index]
                .channel
                .send(skill, input, self.config.request_timeout_ms)
                .await?
        };

        Ok(response)
    }

    /// Periodically ping each slot; restart dead ones.
    async fn health_check_loop(self: Arc<Self>) {
        let mut interval = tokio::time::interval(self.config.health_check_interval);
        loop {
            interval.tick().await;
            // In production: iterate slots, send ping, check pong, restart on failure.
            // Simplified here — see Section 7 for full health check implementation.
            info!("SkillPool: health check tick");
        }
    }
}
```

> **Note on slot selection**: The simplified `slot_index = 0` above is a placeholder. Section 7 covers the production per-slot semaphore pattern that correctly distributes load across all pool slots.

---

## 7. Production Slot Management

The pool manager above uses a single `Semaphore` for concurrency limiting but the slot selection needs to be per-slot to avoid all requests piling onto slot 0. The correct pattern uses a channel of available slot indices:

```rust
// Replace Vec<PoolSlot> + Semaphore with a channel-based free list
use tokio::sync::mpsc;

pub struct SkillPool {
    // Sender side cloned for each execute() call
    // Receiver side holds available slot indices
    free_tx:   mpsc::Sender<usize>,
    free_rx:   Arc<Mutex<mpsc::Receiver<usize>>>,
    slots:     Arc<Vec<Mutex<PoolSlot>>>,
    config:    SkillPoolConfig,
}

impl SkillPool {
    pub async fn new(config: SkillPoolConfig) -> Result<Arc<Self>, SkillPoolError> {
        let (free_tx, free_rx) = mpsc::channel(config.pool_size);
        let mut slots = Vec::with_capacity(config.pool_size);

        for i in 0..config.pool_size {
            let slot = Self::spawn_slot_inner(&config).await?;
            slots.push(Mutex::new(slot));
            free_tx.send(i).await.unwrap(); // all slots start free
        }

        Ok(Arc::new(Self {
            free_tx,
            free_rx: Arc::new(Mutex::new(free_rx)),
            slots:   Arc::new(slots),
            config,
        }))
    }

    pub async fn execute(
        &self,
        skill: &str,
        input: serde_json::Value,
    ) -> Result<IpcResponse, SkillPoolError> {
        // Acquire a free slot index — blocks until one is available
        let acquire_timeout = Duration::from_millis(self.config.acquire_timeout_ms);

        let slot_index = tokio::time::timeout(
            acquire_timeout,
            async { self.free_rx.lock().await.recv().await }
        )
        .await
        .map_err(|_| SkillPoolError::NoSlots)?
        .ok_or(SkillPoolError::ChannelClosed)?;

        // Execute on that slot
        let result = {
            let slot = self.slots[slot_index].lock().await;
            slot.channel
                .send(skill, input, self.config.request_timeout_ms)
                .await
        };

        // Return the slot to the free list (even on error)
        let _ = self.free_tx.send(slot_index).await;

        result
    }
}
```

### 7.1 Health Check with Restart

```rust
async fn health_check_loop(self: Arc<Self>) {
    let mut interval = tokio::time::interval(self.config.health_check_interval);
    loop {
        interval.tick().await;

        for i in 0..self.config.pool_size {
            let is_healthy = {
                let slot = self.slots[i].lock().await;
                // Send a ping and wait up to 2s for pong
                match tokio::time::timeout(
                    Duration::from_secs(2),
                    slot.channel.send("__ping__", serde_json::json!({}), 2000)
                ).await {
                    Ok(Ok(resp)) => resp.ok,
                    _ => false,
                }
            };

            if !is_healthy {
                warn!("SkillPool: slot {} is unhealthy — restarting", i);
                match self.spawn_slot_inner(&self.config).await {
                    Ok(new_slot) => {
                        let mut slot = self.slots[i].lock().await;
                        *slot = new_slot;
                        info!("SkillPool: slot {} restarted (pid {})", i, slot.pid);
                    }
                    Err(e) => {
                        error!("SkillPool: failed to restart slot {}: {}", i, e);
                    }
                }
            }
        }
    }
}
```

Handle the `__ping__` signal in `pool_worker.ts`:

```typescript
// In pool_worker.ts handleRequest(), add before loadSkill():
if (req.skill === "__ping__") {
  return { id: req.id, ok: true, output: "pong", duration_ms: 0 };
}
```

---

## 8. Integration with `skill_worker.rs`

Replace the current `Command::new("pnpm")` / `Command::new("tsx")` spawn logic:

```rust
// skill_worker.rs (updated)
use crate::skill_pool::SkillPool;
use std::sync::Arc;

pub struct SkillWorker {
    pool: Arc<SkillPool>,
    // ... existing fields
}

impl SkillWorker {
    pub async fn execute_skill(
        &self,
        skill_name: &str,
        input: serde_json::Value,
    ) -> Result<String, SkillWorkerError> {
        let response = self.pool
            .execute(skill_name, input)
            .await
            .map_err(SkillWorkerError::Pool)?;

        if response.ok {
            Ok(response.output.unwrap_or_default())
        } else {
            Err(SkillWorkerError::SkillFailed(
                response.error.unwrap_or_else(|| "unknown error".into())
            ))
        }
    }
}
```

Wire the pool into `AppState`:

```rust
// api/mod.rs or main.rs
pub struct AppState {
    pub pool:        sqlx::SqlitePool,
    pub skill_pool:  Arc<SkillPool>,          // ← add this
    pub metrics:     RouterMetrics,
    pub message_bus: MessageBus,
    // ... existing fields
}

// In startup:
let skill_pool = SkillPool::new(SkillPoolConfig::default()).await?;

let state = AppState {
    skill_pool,
    // ...
};
```

---

## 9. Configuration via Environment Variables

Consistent with APEX's unified config pattern (`AppConfig`):

| Variable | Default | Description |
|---|---|---|
| `APEX_SKILL_POOL_SIZE` | `4` | Number of pre-warmed Bun processes |
| `APEX_SKILL_POOL_WORKER` | `./skills/pool_worker.ts` | Path to Bun dispatcher script |
| `APEX_SKILLS_DIR` | `./skills` | Skills directory (already exists) |
| `APEX_SKILL_TIMEOUT_MS` | `30000` | Per-request timeout (ms) |
| `APEX_SKILL_ACQUIRE_MS` | `5000` | Max wait for a free slot (ms) |
| `APEX_SKILL_HEALTH_INTERVAL` | `30` | Health check interval (seconds) |

Add to `unified_config.rs`:

```rust
pub struct SkillPoolSettings {
    pub pool_size:        usize,
    pub worker_script:    PathBuf,
    pub request_timeout:  u64,
    pub acquire_timeout:  u64,
    pub health_interval:  u64,
}

impl From<&AppConfig> for SkillPoolSettings {
    fn from(cfg: &AppConfig) -> Self {
        Self {
            pool_size:       cfg.get_usize("APEX_SKILL_POOL_SIZE").unwrap_or(4),
            worker_script:   cfg.get_path("APEX_SKILL_POOL_WORKER")
                               .unwrap_or_else(|| PathBuf::from("./skills/pool_worker.ts")),
            request_timeout: cfg.get_u64("APEX_SKILL_TIMEOUT_MS").unwrap_or(30_000),
            acquire_timeout: cfg.get_u64("APEX_SKILL_ACQUIRE_MS").unwrap_or(5_000),
            health_interval: cfg.get_u64("APEX_SKILL_HEALTH_INTERVAL").unwrap_or(30),
        }
    }
}
```

---

## 10. Cargo.toml Dependencies

```toml
[dependencies]
# existing
axum       = "0.7"
tokio      = { version = "1", features = ["full"] }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
tracing    = "0.1"
thiserror  = "1"

# add if not present
uuid = { version = "1", features = ["v4"] }
```

---

## 11. Testing

### 11.1 Bun Dispatcher Unit Test (`pool_worker_test.ts`)

```typescript
// skills/pool_worker_test.ts
// Run with: bun test pool_worker_test.ts

import { describe, it, expect, beforeAll, afterAll } from "bun:test";
import { spawn } from "bun";

let worker: ReturnType<typeof spawn>;
let responses: Map<string, unknown>;

function sendRequest(req: object): void {
  worker.stdin.write(JSON.stringify(req) + "\n");
}

async function waitForResponse(id: string, timeoutMs = 5000): Promise<unknown> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (responses.has(id)) return responses.get(id);
    await new Promise(r => setTimeout(r, 10));
  }
  throw new Error(`Timeout waiting for response id=${id}`);
}

beforeAll(async () => {
  responses = new Map();

  worker = spawn({
    cmd: ["bun", "run", "pool_worker.ts"],
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
  });

  // Read lines from stdout
  const reader = worker.stdout.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  (async () => {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value);
      const lines = buffer.split("\n");
      buffer = lines.pop() ?? "";
      for (const line of lines) {
        if (!line.trim()) continue;
        try {
          const parsed = JSON.parse(line) as Record<string, unknown>;
          if (parsed.id) responses.set(parsed.id as string, parsed);
        } catch { /* ignore non-JSON */ }
      }
    }
  })();

  // Wait for ready signal
  await new Promise(r => setTimeout(r, 500));
});

afterAll(() => { worker.kill(); });

describe("pool_worker", () => {
  it("responds to ping", async () => {
    sendRequest({ id: "ping-1", skill: "__ping__", input: {}, timeout_ms: 1000 });
    const resp = await waitForResponse("ping-1") as Record<string, unknown>;
    expect(resp.ok).toBe(true);
    expect(resp.output).toBe("pong");
  });

  it("executes code.review skill", async () => {
    sendRequest({
      id: "test-1",
      skill: "code.review",
      input: { code: "const x = 1" },
      timeout_ms: 10000,
    });
    const resp = await waitForResponse("test-1", 10000) as Record<string, unknown>;
    expect(resp.ok).toBe(true);
    expect(typeof resp.output).toBe("string");
  });

  it("returns error for unknown skill", async () => {
    sendRequest({
      id: "test-2",
      skill: "nonexistent.skill",
      input: {},
      timeout_ms: 5000,
    });
    const resp = await waitForResponse("test-2") as Record<string, unknown>;
    expect(resp.ok).toBe(false);
    expect(resp.error).toContain("nonexistent.skill");
  });

  it("handles concurrent requests without mixing responses", async () => {
    const ids = ["c1", "c2", "c3", "c4", "c5"];
    for (const id of ids) {
      sendRequest({
        id,
        skill: "__ping__",
        input: {},
        timeout_ms: 2000,
      });
    }
    const results = await Promise.all(ids.map(id => waitForResponse(id)));
    for (const r of results as Array<Record<string, unknown>>) {
      expect(r.ok).toBe(true);
    }
  });
});
```

### 11.2 Rust Integration Test

```rust
// core/router/tests/skill_pool_test.rs
#[tokio::test]
async fn test_pool_executes_ping() {
    let config = SkillPoolConfig {
        pool_size: 2,
        worker_script: PathBuf::from("../../skills/pool_worker.ts"),
        ..Default::default()
    };

    let pool = SkillPool::new(config).await.expect("pool should start");

    let result = pool
        .execute("__ping__", serde_json::json!({}))
        .await
        .expect("execute should succeed");

    assert!(result.ok);
    assert_eq!(result.output.as_deref(), Some("pong"));
}

#[tokio::test]
async fn test_pool_concurrent_requests() {
    let pool = SkillPool::new(SkillPoolConfig { pool_size: 3, ..Default::default() })
        .await
        .unwrap();
    let pool = Arc::new(pool);

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let p = Arc::clone(&pool);
            tokio::spawn(async move {
                p.execute("__ping__", serde_json::json!({ "n": i })).await
            })
        })
        .collect();

    for h in handles {
        let result = h.await.unwrap().unwrap();
        assert!(result.ok);
    }
}
```

---

## 12. Metrics

Expose pool metrics via the existing `/api/v1/metrics` endpoint:

```rust
pub struct SkillPoolMetrics {
    pub pool_size:        usize,    // configured pool size
    pub slots_active:     usize,    // slots currently executing a skill
    pub slots_healthy:    usize,    // slots that passed last health check
    pub requests_total:   u64,      // total executions since startup
    pub requests_timeout: u64,      // executions that timed out
    pub latency_p50_ms:   f64,      // 50th percentile execution latency
    pub latency_p99_ms:   f64,      // 99th percentile execution latency
    pub queue_depth:      usize,    // requests waiting for a free slot
}
```

Emit as Prometheus gauges/histograms via the existing `metrics.rs` infrastructure.

---

## 13. Windows Compatibility

The build output shows `E:\projects\APEX` — Windows. Two things to verify:

**Path separators**: Use `PathBuf` throughout (already done above). Never concatenate paths with `/` or `\` literals in Rust.

**Bun on Windows**: Bun is supported on Windows since v1.0. Verify with `bun --version`. The `Command::new("bun")` call works if Bun is on `PATH`. If not found, fail fast at pool startup with a clear error rather than at first skill execution.

**Line endings**: The `crlfDelay: Infinity` option in the `createInterface` call in `pool_worker.ts` handles Windows CRLF line endings in stdin correctly. Rust's `write_all` sends `\n` — Bun's readline normalises it.

---

## 14. Migration Checklist

- [ ] Add `uuid` to `Cargo.toml` if not present
- [ ] Create `core/router/src/skill_pool_ipc.rs`
- [ ] Create `core/router/src/skill_pool.rs`
- [ ] Update `core/router/src/skill_worker.rs` to use `SkillPool`
- [ ] Add `skill_pool` and `skill_pool_ipc` to `mod.rs`
- [ ] Add `skill_pool: Arc<SkillPool>` to `AppState`
- [ ] Wire pool startup into router `main.rs` / `lib.rs`
- [ ] Create `skills/pool_worker.ts`
- [ ] Create `skills/pool_worker_test.ts`
- [ ] Add `APEX_SKILL_POOL_*` vars to `unified_config.rs`
- [ ] Add `APEX_SKILL_POOL_*` vars to `AGENTS.md` env var table
- [ ] Add pool metrics to `metrics.rs`
- [ ] Add pool status to `/api/v1/metrics` response
- [ ] Run `bun test pool_worker_test.ts` — all tests green
- [ ] Run `cargo test skill_pool` — all tests green
- [ ] Verify latency with `cargo bench` or a manual timing loop
- [ ] Update `ARCHITECTURE.md` — replace `SkillWorker` spawn description with pool description

---

*APEX Skill Pool Implementation Guide · Solution 1: Pre-warmed Bun Pool with Multiplexed IPC*
