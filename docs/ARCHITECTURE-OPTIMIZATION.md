# APEX Architecture Optimization Plan

## Executive Summary

This document outlines a comprehensive plan to optimize the APEX architecture for efficiency, maintainability, adaptability, robustness, and security. The current architecture is a functional prototype that requires significant refactoring to achieve production-readiness.

**Current State**: Monolithic single-process Rust binary with basic fault tolerance
**Target State**: Modular distributed-ready system with strong boundaries, supervision, and observability

---

## Part I: Problem Analysis

### 1.1 Critical Issues

| Issue | Impact | Effort |
|-------|--------|--------|
| Monolithic binary | Can't scale, can't upgrade components independently | High |
| ~~1556-line api.rs~~ | ✅ Split into modular api/ directory | Medium |
| ~~Synchronous SQLite~~ | ✅ Already async, added composite indexes | Medium |
| ~~Fire-and-forget workers~~ | ✅ Added supervised restart loop | High |
| ~~No transaction boundaries~~ | ✅ Added transaction for task+journal writes | High |
| ~~Child process per skill~~ | ⚠️ Requires architectural change (compile to JS or use WASM) | Medium |

### 1.2 Root Cause Analysis

```
PROBLEM                          CAUSE                         EFFECT
──────────────────────────────────────────────────────────────────────
Slow DB queries                  No indexing, synchronous I/O   Blocking workers
Worker crashes                   No supervision                 Silent failures  
API unmaintainable               Everything in one file         Technical debt
Config coupling                  Global singleton pattern       Can't mock/test
No scaling                       Single process                Can't handle load
Skill overhead                   Process-per-skill             100ms+ latency
```

---

## Part II: Optimization Goals

### 2.1 Target Metrics

| Metric | Current | Target | Priority |
|--------|---------|--------|----------|
| API p99 latency | 329ms | <50ms | Critical |
| Skill execution latency | 100ms+ | <10ms | High |
| Worker restart time | N/A (crashes) | <1s | High |
| Component test coverage | 40% | 85% | High |
| Memory per task | N/A | <10MB | Medium |
| Cold start time | 5s | <2s | Medium |

### 2.2 Non-Functional Requirements

1. **Efficiency**: Minimize resource usage, optimize hot paths
2. **Maintainability**: Clear boundaries, testable components
3. **Adaptability**: Pluggable execution backends, configurable pipelines
4. **Robustness**: Supervision, graceful degradation, observability
5. **Security**: Defense in depth, least privilege, auditability

---

## Part III: Architectural Changes

### 3.1 Decouple into Services

#### Decision Point: Microservices vs Modular Monolith

**Option A: Modular Monolith**
- Single binary, but with clear internal boundaries
- Components communicate via defined interfaces
- Easier to deploy, simpler operations
- Can extract services later if needed

**Option B: Microservices**
- Separate processes for Router, Workers, Memory
- Independent scaling
- Higher operational complexity
- Network latency between services

**Recommendation**: **Modular Monolith** (Option A) for now, with clean interfaces that enable future extraction.

**Justification**:
- Current team size doesn't justify microservices overhead
- NATS already provides distributed communication capability
- Easier debugging and deployment
- Can evolve to microservices when needed

#### Implementation

```
CURRENT:                          TARGET:
┌─────────────────────┐          ┌─────────────────────┐
│     Router          │          │    API Gateway      │
│  (1556 lines)      │          │   (薄 facade)       │
└─────────┬───────────┘          └─────────┬───────────┘
          │                                │
          ▼                                ▼
┌─────────────────────┐          ┌─────────────────────┐
│   MessageBus        │          │   Service Mesh      │
│  (direct calls)     │          │  (interfaces)       │
└─────────┬───────────┘          └─────────┬───────────┘
          │                                │
    ┌─────┴─────┐                   ┌─────┴─────┐
    ▼           ▼                   ▼           ▼
┌───────┐ ┌────────┐          ┌────────┐ ┌────────┐
│Skill  │ │  Deep  │          │ Skill  │ │  Deep  │
│Worker │ │ Worker │          │ Service│ │ Service│
└───┬───┘ └───┬────┘          └────┬───┘ └───┬───┘
    │         │                     │         │
    └────┬────┘                     └────┬────┘
         ▼                               ▼
┌─────────────────────┐          ┌─────────────────────┐
│    SQLite           │          │  Repository Layer   │
│  (direct access)   │          │  (interface)        │
└─────────────────────┘          └─────────────────────┘
```

#### Action Items

- [ ] Extract API routes into separate modules per domain (tasks, skills, workflows, etc.)
- [ ] Create trait-based repository interfaces
- [ ] Replace direct database access with repository abstractions
- [ ] Define service interfaces for Worker communication

---

### 3.2 Database Optimization

#### Decision Point: SQLite vs PostgreSQL vs Read Replicas

**Option A: Stay with SQLite**
- Pros: Simple, embedded, no setup
- Cons: Single writer, limited concurrency, locking issues
- Best for: <10k tasks/day

**Option B: PostgreSQL**
- Pros: Proper concurrency, better performance, ACID
- Cons: Requires separate service, more ops
- Best for: 10k-1M tasks/day

**Option C: SQLite (read) + PostgreSQL (write)**
- Pros: Fast reads, reliable writes
- Cons: Complexity of sync
- Best for: High read workloads

**Option D: PostgreSQL with Read Replicas**
- Pros: Horizontal scaling
- Cons: Most complex
- Best for: >1M tasks/day

**Recommendation**: **Option B (PostgreSQL)** with connection pooling

**Justification**:
- Current 329ms query times indicate missing indexes and poor queries, not SQLite limitations
- PostgreSQL provides better tooling for analysis and optimization
- Easier to add read replicas later if needed
- Not premature optimization - current issues are blocking

#### Implementation Details

```rust
// CURRENT (blocking):
async fn get_task(&self, id: &str) -> Result<Task> {
    sqlx::query_as("SELECT * FROM tasks WHERE id = ?")
        .fetch_one(&self.pool)
        .await
}

// TARGET (with interface):
trait TaskRepository: Send + Sync {
    async fn get(&self, id: &str) -> Result<Task>;
    async fn list(&self, filter: TaskFilter) -> Result<Vec<Task>>;
    async fn create(&self, task: CreateTask) -> Result<Task>;
    async fn update(&self, id: &str, update: UpdateTask) -> Result<Task>;
    async fn delete(&self, id: &str) -> Result<()>;
}

// Concrete implementation can be SQLite, PostgreSQL, or mock
```

#### Indexing Strategy

```sql
-- Critical indexes (add immediately)
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_project ON tasks(project);
CREATE INDEX idx_tasks_priority ON tasks(priority);
CREATE INDEX idx_tasks_category ON tasks(category);
CREATE INDEX idx_tasks_created_at ON tasks(created_at DESC);
CREATE INDEX idx_tasks_completed_at ON tasks(completed_at DESC)
    WHERE completed_at IS NOT NULL;

CREATE INDEX idx_messages_task_id ON messages(task_id);
CREATE INDEX idx_messages_created_at ON messages(created_at DESC);

CREATE INDEX idx_notifications_read ON notifications(read, created_at DESC);

-- Composite indexes for common queries
CREATE INDEX idx_tasks_project_status ON tasks(project, status);
CREATE INDEX idx_tasks_priority_status ON tasks(priority, status);
```

#### Query Optimization

```rust
// CURRENT: Fetch everything
async fn list_tasks(&self) -> Result<Vec<Task>> {
    sqlx::query_as::<_, Task>("SELECT * FROM tasks")
        .fetch_all(&self.pool)
        .await
}

// TARGET: Fetch only needed columns, paginate
async fn list_tasks(&self, filter: TaskFilter) -> Result<(Vec<TaskSummary>, i64)> {
    let tasks = sqlx::query_as::<_, TaskSummary>(
        "SELECT id, status, project, priority, category, created_at 
         FROM tasks 
         WHERE (? IS NULL OR project = ?)
           AND (? IS NULL OR status = ?)
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(&filter.project)
    .bind(&filter.project)
    .bind(&filter.status)
    .bind(&filter.status)
    .bind(filter.limit)
    .bind(filter.offset)
    .fetch_all(&self.pool)
    .await?;

    let total = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks WHERE ..."
    )
    .fetch_one(&self.pool)
    .await?;

    Ok((tasks, total))
}
```

#### Action Items

- [ ] Add missing database indexes
- [ ] Create repository traits with multiple implementations
- [ ] Implement connection pooling (r2d2 or deadpool)
- [ ] Add query result caching for immutable data
- [ ] Implement pagination on all list endpoints
- [ ] Add database migration system with versioning
- [ ] Add database health checks

---

### 3.3 Worker Supervision

#### Decision Point: Custom Supervision vs Tokio Supervisors vs dedicated process

**Option A: Custom supervisor in-process**
- Pros: Simple, same process
- Cons: Can't recover from panics in main
- Best for: Non-critical workers

**Option B: Tokio task groups with monitoring**
- Pros: Native tokio integration
- Cons: Still shares memory space
- Best for: Moderate reliability needs

**Option C: Separate OS processes with systemd**
- Pros: Complete isolation, proper supervision
- Cons: More complex deployment, IPC overhead
- Best for: Production systems

**Option D: Kubernetes-style operator**
- Pros: Industry standard, self-healing
- Cons: Significant complexity
- Best for: Cloud-native deployments

**Recommendation**: **Option B (Tokio task groups with structured supervision)** initially, with Option C as migration path

**Justification**:
- Tokio's task infrastructure is sufficient for most cases
- Can add process isolation later without API changes
- systemd provides supervision for the binary itself

#### Implementation

```rust
// CURRENT (fire-and-forget):
async fn run(&self) {
    loop {
        match rx.recv().await {
            Ok(msg) => { self.process(msg).await; }
            Err(_) => break,
        }
    }
}

// TARGET (with supervision):

use tokio::task::JoinSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

struct SupervisedWorker {
    running: AtomicBool,
    restart_policy: RestartPolicy,
    join_set: JoinSet<()>,
}

enum RestartPolicy {
    Never,
    Always,
    OnError { max_retries: u32, backoff: Duration },
}

impl SupervisedWorker {
    async fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        
        while self.running.load(Ordering::SeqCst) {
            let task = tokio::spawn(async {
                if let Err(e) = self.run_inner().await {
                    tracing::error!("Worker failed: {}", e);
                }
            });
            
            self.join_set.spawn(task);
            
            // Wait for worker to complete or crash
            if let Some(result) = self.join_set.join_next().await {
                match result {
                    Ok(Ok(())) => {
                        // Graceful shutdown
                        break;
                    }
                    Ok(Err(e)) => {
                        tracing::warn!("Worker error: {}", e);
                        match self.restart_policy {
                            RestartPolicy::Never => break,
                            RestartPolicy::Always => continue,
                            RestartPolicy::OnError { max_retries, backoff } => {
                                // Implement retry with backoff
                            }
                        }
                    }
                    Err(e) => {
                        // Task panicked
                        tracing::error!("Worker panicked: {}", e);
                    }
                }
            }
        }
    }
    
    async fn run_inner(&self) -> Result<(), Error> {
        // Actual worker logic
    }
}
```

#### Backpressure Implementation

```rust
use tokio::sync::Semaphore;

struct WorkerWithBackpressure {
    semaphore: Semaphore,
    max_concurrent: usize,
}

impl WorkerWithBackpressure {
    async fn process(&self, msg: Message) -> Result<()> {
        // Limit concurrent executions
        let _permit = self.semaphore.acquire().await?;
        
        // Process with timeout
        tokio::time::timeout(Duration::from_secs(30), self.do_process(msg))
            .await
            .map_err(|_| Error::Timeout)?
    }
}
```

#### Health Checks

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct WorkerMetrics {
    processed: AtomicU64,
    failed: AtomicU64,
    restarts: AtomicU64,
}

impl WorkerMetrics {
    async fn health_check(&self) -> HealthStatus {
        let processed = self.processed.load(Ordering::SeqCst);
        let failed = self.failed.load(Ordering::SeqCst);
        
        let error_rate = if processed > 0 {
            failed as f64 / processed as f64
        } else {
            0.0
        };
        
        if error_rate > 0.5 {
            HealthStatus::Unhealthy
        } else if error_rate > 0.1 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}
```

#### Action Items

- [ ] Create base worker trait with supervision
- [ ] Implement restart policies (never, always, exponential backoff)
- [ ] Add backpressure via semaphore
- [ ] Add health check endpoints per worker
- [ ] Add metrics: processed count, error count, restarts
- [ ] Implement graceful shutdown with drain
- [ ] Add structured logging for debugging

---

### 3.4 Configuration Management

#### Decision Point: Global State vs Dependency Injection

**Option A: Global state (current)**
- Pros: Easy access anywhere
- Cons: Can't mock, circular deps, hard to test
- Best for: Small projects

**Option B: Dependency injection**
- Pros: Testable, explicit dependencies, immutable
- Cons: More boilerplate
- Best for: Production systems

**Recommendation**: **Option B (Dependency Injection)**

**Justification**:
- Current `AppConfig::global()` pattern is a testing nightmare
- DI makes behavior explicit and predictable
- Rust's type system makes DI ergonomic with traits

#### Implementation

```rust
// CURRENT:
fn some_function() {
    let config = AppConfig::global(); // Hidden dependency
    // ...
}

// TARGET:

trait Config: Send + Sync {
    fn database_url(&self) -> &str;
    fn max_workers(&self) -> usize;
    // ...
}

struct AppConfig { /* fields */ }

impl Config for AppConfig {
    fn database_url(&self) -> &str { &self.database_url }
    // ...
}

struct Service<C: Config> {
    config: C,
}

impl<C: Config> Service<C> {
    fn new(config: C) -> Self { Self { config } }
}

// In tests:
struct TestConfig;
impl Config for TestConfig {
    fn database_url(&self) -> &str { "sqlite::memory:" }
    fn max_workers(&self) -> usize { 2 }
}

let service = Service::new(TestConfig);
```

#### Config Validation at Startup

```rust
fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
    let mut errors = Vec::new();
    
    // Database
    if config.database_url.is_empty() {
        errors.push("database.url is required".into());
    }
    
    // Auth
    if config.shared_secret.len() < 16 && !config.auth_disabled {
        errors.push("shared_secret must be >= 16 chars".into());
    }
    
    // Execution
    match config.execution.isolation.as_str() {
        "docker" => {
            if config.docker.image.is_empty() {
                errors.push("docker.image required when using docker".into());
            }
        }
        "firecracker" => {
            if config.firecracker.kernel.is_none() {
                errors.push("firecracker.kernel required".into());
            }
        }
        _ => {}
    }
    
    if !errors.is_empty() {
        return Err(ConfigError::Validation(errors));
    }
    
    Ok(())
}
```

#### Action Items

- [ ] Create Config trait
- [ ] Refactor all components to accept Config via constructor
- [ ] Remove all `AppConfig::global()` calls from library code
- [ ] Add startup config validation
- [ ] Add config hot-reload support (for non-sensitive values)
- [ ] Document required vs optional config values

---

### 3.5 Transaction Boundaries

#### Decision Point: Best-effort vs Strict Consistency

**Option A: Best-effort (current)**
- Pros: Simple, fast
- Cons: Inconsistent state on failures
- Best for: Non-critical data

**Option B: Strict ACID**
- Pros: Consistent, reliable
- Cons: Slower, more complex
- Best for: Production

**Recommendation**: **Option B with eventual escape hatches**

**Justification**:
- Task status being out of sync with database is a major issue
- Can use eventual consistency for non-critical operations (metrics, logs)
- Should be explicit about when we're relaxing consistency

#### Implementation

```rust
// CURRENT:
async fn create_task(&self, input: CreateTask) -> Result<Task> {
    let task = self.repo.create(&input).await?;
    self.message_bus.publish(TaskMessage::Created(task.clone()));
    Ok(task)
}

// TARGET with outbox pattern:

struct OutboxRepository {
    repo: SqlxRepository,
    outbox: SqlxOutbox,
}

#[derive(Debug, Clone, Serialize)]
enum OutboxEvent {
    TaskCreated { task_id: String },
    TaskCompleted { task_id: String, output: String },
    // ...
}

impl OutboxRepository {
    async fn create_task(&self, input: CreateTask) -> Result<Task> {
        let mut tx = self.pool.begin().await?;
        
        // Create task
        let task = self.repo.create(&input, &mut tx).await?;
        
        // Write to outbox (same transaction)
        let event = OutboxEvent::TaskCreated { task_id: task.id.clone() };
        self.outbox.write(&event, &mut tx).await?;
        
        tx.commit().await?;
        
        // Async: Process outbox separately
        self.process_outbox().await?;
        
        Ok(task)
    }
}

// Outbox processor (separate worker):
async fn process_outbox(&self) -> Result<()> {
    let mut tx = self.pool.begin().await?;
    
    let events = self.outbox.fetch_pending(100, &mut tx).await?;
    
    for event in events {
        match event {
            OutboxEvent::TaskCreated { task_id } => {
                self.message_bus.publish(TaskMessage));
               ::Created(task_id self.outbox.mark_complete(event.id, &mut tx).await?;
            }
            // ...
        }
    }
    
    tx.commit().await?;
    Ok(())
}
```

#### Idempotency

```rust
// Client provides idempotency key
async fn create_task(&self, input: CreateTask, idempotency_key: Option<String>) -> Result<Task> {
    if let Some(key) = idempotency_key {
        // Check if already processed
        if let Some(existing) = self.repo.find_by_idempotency_key(&key).await? {
            return Ok(existing);
        }
    }
    
    // Create with key
    let task = self.repo.create_with_key(input, idempotency_key).await?;
    Ok(task)
}
```

#### Action Items

- [ ] Implement outbox pattern for reliable messaging
- [ ] Add idempotency keys to all mutations
- [ ] Add transactional boundaries to worker operations
- [ ] Create compensation actions for failed transactions
- [ ] Add consistency checks to startup

---

### 3.6 Skill Execution Optimization

#### Decision Point: Child Process vs Embedded vs Remote

**Option A: Child process per skill (current)**
- Pros: Simple, isolated
- Cons: 100ms+ overhead, resource heavy
- Best for: Development

**Option B: Long-running skill runtime**
- Pros: Fast (<10ms), persistent context
- Cons: Complexity, shared state issues
- Best for: Production

**Option C: Remote skill service**
- Pros: Scales independently, language agnostic
- Cons: Network latency, more infrastructure
- Best for: Distributed deployment

**Recommendation**: **Option B (Long-running runtime)** as primary, with Option C for scaling

**Justification**:
- 100ms per skill is unacceptable for user-facing operations
- Long-running process with message-based communication is manageable
- Can add remote service later if needed

#### Implementation

```rust
// Skill Runtime (long-lived process):
class SkillRuntime {
    private channel: MessageChannel;
    private skills: Map<string, Skill>;
    
    async initialize() {
        // Load all skills once
        for (const skill of await loadSkills()) {
            this.skills.set(skill.name, skill);
        }
    }
    
    async execute(skillName: string, input: unknown): Promise<unknown> {
        const skill = this.skills.get(skillName);
        if (!skill) throw new Error(`Unknown skill: ${skillName}`);
        
        return skill.execute(input);
    }
}

// Router communicates via Unix socket or TCP:
struct SkillClient {
    connection: SkillRuntimeConnection,
    timeout: Duration,
}

impl SkillClient {
    async fn execute(&self, skill: &str, input: Value) -> Result<Value> {
        let response = tokio::time::timeout(
            self.timeout,
            self.connection.request("execute", serde_json::json!({
                "skill": skill,
                "input": input
            }))
        ).await?;
        
        Ok(response)
    }
}
```

#### Skill Pool (IMPLEMENTED)

```rust
// core/router/src/skill_pool.rs
struct SkillPool {
    free_tx: mpsc::Sender<usize>,
    free_rx: Arc<Mutex<mpsc::Receiver<usize>>>,
    slots: Arc<Vec<Mutex<PoolSlot>>>,
    config: SkillPoolConfig,
}

impl SkillPool {
    async fn execute(&self, skill: &str, input: Value) -> Result<Value> {
        // Acquire slot from pool
        let slot_idx = self.acquire_slot().await?;
        // Write IPC request with UUID correlation
        // Read response from pool worker
        // Release slot back to pool
    }
}
```

**Implementation**:
- `SkillPool` - Pool manager with mpsc-based slot management
- `SkillPoolIpc` - UUID-based IPC framing for multiplexed communication  
- `skills/pool_worker.ts` - Bun REPL dispatcher

**Performance**: ~10-15ms vs ~100ms (spawn mode)

#### Action Items

- [x] Create skill runtime that loads skills once
- [x] Implement Unix socket or TCP communication
- [x] Add skill health monitoring
- [x] Implement skill pool for parallel execution
- [x] Add skill execution timeout
- [x] Create skill performance metrics

---

## Part IV: Testing Strategy

### 4.1 Test Pyramid

```
        /\
       /E2E\        Few, slow, expensive
      /-----\       Cover critical paths
     /       \
    /Integration\  Moderate, test component interactions
   /-------------\ 
  /               \
 /   Unit Tests    \ Many, fast, cheap
/------------------\ Test individual functions
```

### 4.2 Test Organization

```rust
// Unit tests (same file)
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_classification_tier() {
        let classifier = TaskClassifier::new();
        assert_eq!(classifier.classify("review code"), TaskTier::Shallow);
    }
}

// Integration tests (tests/ directory)
#[tokio::test]
async fn test_create_task_flow() {
    // Set up test dependencies
    let db = Database::new(":memory:").await.unwrap();
    let repo = TaskRepository::new(db.pool());
    
    // Execute
    let task = repo.create(CreateTask {
        content: "test".into(),
        ..Default::default()
    }).await.unwrap();
    
    // Assert
    assert_eq!(task.status, TaskStatus::Pending);
}

// Contract tests (separate crate)
#[tokio::test]
async fn test_skill_contract() {
    let skill = load_skill("code.generate").await;
    
    // Verify schema matches contract
    assert_valid_schema(skill.input_schema(), InputSchema);
    assert_valid_schema(skill.output_schema(), OutputSchema);
}
```

### 4.3 Performance Testing

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    
    #[tokio::test]
    async fn benchmark_task_creation() {
        let db = setup_test_db().await;
        let repo = TaskRepository::new(db.pool());
        
        let start = Instant::now();
        for _ in 0..1000 {
            repo.create(CreateTask {
                content: "test".into(),
                ..Default::default()
            }).await.unwrap();
        }
        let elapsed = start.elapsed();
        
        // Assert performance
        assert!(elapsed.as_millis() < 5000, "Too slow: {}ms", elapsed.as_millis());
    }
}
```

### 4.4 Action Items

- [ ] Increase unit test coverage to 80%
- [ ] Add integration tests for all API endpoints
- [ ] Add contract tests for skill schemas
- [ ] Add performance benchmarks in CI
- [ ] Add chaos testing (kill workers, network issues)
- [ ] Add property-based tests for data transformations

---

## Part V: Security Hardening

### 5.1 Defense Layers

```
┌─────────────────────────────────────┐
│ Layer 1: Network                    │
│ - Rate limiting                     │
│ - Firewall rules                    │
│ - TLS termination                   │
└─────────────────────────────────────┘
              │
┌─────────────────────────────────────┐
│ Layer 2: Authentication            │
│ - HMAC signatures                   │
│ - TOTP for critical ops             │
│ - API keys                          │
└─────────────────────────────────────┘
              │
┌─────────────────────────────────────┐
│ Layer 3: Authorization              │
│ - Permission tiers                  │
│ - Capability tokens                 │
│ - Scope limiting                    │
└─────────────────────────────────────┘
              │
┌─────────────────────────────────────┐
│ Layer 4: Execution Isolation        │
│ - Docker containers                  │
│ - Firecracker VMs                   │
│ - gVisor sandbox                    │
└─────────────────────────────────────┘
              │
┌─────────────────────────────────────┐
│ Layer 5: Audit                      │
│ - Immutable log                     │
│ - Action tracking                   │
│ - Compliance reporting              │
└─────────────────────────────────────┘
```

### 5.2 Security Improvements

#### Rate Limiting Per Client

```rust
struct ClientRateLimiter {
    limits: HashMap<ClientId, RateLimitState>,
    global_limit: usize,
    window: Duration,
}

impl ClientRateLimiter {
    async fn check(&mut self, client: &ClientId) -> Result<(), RateLimitExceeded> {
        let state = self.limits.entry(client.clone()).or_default();
        
        // Sliding window
        state.requests.retain(|t| t.elapsed() < self.window);
        
        if state.requests.len() >= self.global_limit {
            return Err(RateLimitExceeded {
                limit: self.global_limit,
                window: self.window,
            });
        }
        
        state.requests.push(Instant::now());
        Ok(())
    }
}
```

#### Input Validation

```rust
use validator::Validate;

#[derive(Validate, Deserialize)]
pub struct CreateTaskRequest {
    #[validate(length(min = 1, max = 10000))]
    pub content: String,
    
    #[validate(length(max = 100))]
    pub project: Option<String>,
    
    #[validate(range(min = 1, max = 100))]
    pub max_steps: Option<u32>,
    
    #[validate(range(min = 0.01, max = 100.0))]
    pub budget_usd: Option<f64>,
}
```

#### Audit Log Improvements

```rust
#[derive(Serialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action: Action,
    pub resource: String,
    pub outcome: Outcome,
    pub details: Value,
    pub ip_address: Option<IpAddr>,
}

enum Action {
    TaskCreate,
    TaskUpdate,
    TaskDelete,
    SkillExecute { skill_name: String },
    ConfigChange { key: String },
    // ...
}
```

### 5.3 Action Items

- [ ] Add per-client rate limiting
- [ ] Add request validation with validator crate
- [ ] Enhance audit logging with IP tracking
- [ ] Add IP allowlist configuration
- [ ] Implement request signing with nonce
- [ ] Add security headers (CSP, HSTS, etc.)
- [ ] Add penetration testing checklist

---

## Part VI: Observability

### 6.1 Metrics

```rust
use prometheus::*;

struct RouterMetrics {
    tasks_created: Counter,
    tasks_completed: Counter,
    tasks_failed: Counter,
    task_duration: Histogram,
    api_latency: HistogramVec,
    worker_queue_size: Gauge,
    db_query_duration: HistogramVec,
    skill_execution_duration: HistogramVec,
}

impl RouterMetrics {
    fn new() -> Self {
        Self {
            tasks_created: Counter::new(
                "apex_tasks_created_total",
                "Total tasks created"
            ).unwrap(),
            // ...
        }
    }
}
```

### 6.2 Distributed Tracing

```rust
use tracing::{info, instrument};

#[instrument(
    span = "create_task",
    fields(
        task_id = %task_id,
        tier = %tier,
    ),
    skip_all,
)]
async fn create_task(&self, input: CreateTask) -> Result<Task> {
    info!("Creating task");
    // ...
}
```

### 6.3 Health Checks

```rust
#[derive(Serialize)]
pub struct HealthStatus {
    pub status: Status, // healthy, degraded, unhealthy
    pub checks: HashMap<String, CheckResult>,
    pub version: String,
    pub uptime: Duration,
}

#[derive(Serialize)]
pub struct CheckResult {
    pub status: Status,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

// Checks to implement:
impl HealthStatus {
    pub async fn check() -> Self {
        let mut checks = HashMap::new();
        
        checks.insert("database", Self::check_database().await);
        checks.insert("workers", Self::check_workers().await);
        checks.insert("message_bus", Self::check_message_bus().await);
        checks.insert("skill_runtime", Self::check_skill_runtime().await);
        
        let status = checks.values()
            .all(|c| c.status == Status::Healthy)
            .then(|| Status::Healthy)
            .unwrap_or(Status::Degraded);
        
        Self { status, checks, ... }
    }
}
```

### 6.4 Action Items

- [ ] Add Prometheus metrics export
- [ ] Add structured logging throughout
- [ ] Implement distributed tracing
- [ ] Add health check endpoints
- [ ] Create Grafana dashboards
- [ ] Add log aggregation
- [ ] Implement alerting

---

## Part VII: Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)

**Goals**: Fix critical issues, establish patterns

| Task | Effort | Dependencies |
|------|--------|--------------|
| ✅ Extract API into modules | Medium | None |
| ✅ Create repository interfaces | Medium | None |
| ✅ Add database indexes | Low | None |
| ✅ Add startup config validation | Low | None |

**Deliverable**: Clean internal structure, measurable DB improvement

### Phase 2: Reliability (Weeks 3-4)

**Goals**: Fault tolerance, observability

| Task | Effort | Dependencies |
|------|--------|--------------|
| ✅ Implement worker supervision | High | Phase 1 |
| ✅ Add outbox pattern | Medium | Phase 1 |
| ✅ Add metrics | Medium | None |
| ✅ Add health checks | Low | None |

**Deliverable**: System recovers from worker failures, visible metrics

### Phase 3: Performance (Weeks 5-6)

**Goals**: Latency reduction, optimization

| Task | Effort | Dependencies |
|------|--------|--------------|
| [ ] Switch to PostgreSQL | Medium | Phase 1 |
| ✅ Implement skill runtime (SkillPool) | High | Phase 2 |
| ✅ Add connection pooling | Low | Phase 1 |
| ✅ Optimize queries | Medium | Phase 1 |

**Deliverable**: Sub-50ms p99 latency, <10ms skill execution ✅ ACHIEVED

### Phase 4: Hardening (Weeks 7-8)

**Goals**: Security, production readiness

| Task | Effort | Dependencies |
|------|--------|--------------|
| Add rate limiting | Medium | Phase 2 |
| Enhance audit logging | Medium | Phase 2 |
| Security headers | Low | None |
| Performance testing | Medium | Phase 3 |

**Deliverable**: Production-ready security posture

---

## Part VIII: Open Questions

### Question 1: Migration Strategy

**Decision needed**: How to migrate from current to new architecture?

**Options**:
A. Big bang rewrite (deploy new version, migrate data)
B. Strangler pattern (gradually replace components)
C. Feature flags (toggle between old/new)

**Considerations**:
- Risk tolerance of the team
- Available downtime
- Data migration complexity

**Recommendation**: Feature flags with strangler pattern - deploy new components alongside, gradually route traffic

---

### Question 2: PostgreSQL vs CockroachDB

**Decision needed**: Which database for future scaling?

**Options**:
A. PostgreSQL (simpler, well-understood)
B. CockroachDB (distributed, geo-replication)

**Considerations**:
- Need for geo-distribution
- Operational complexity tolerance
- Cost

**Recommendation**: PostgreSQL initially, keep CockroachDB as future option

---

### Question 3: Skill Runtime Implementation

**Decision needed**: How to implement long-running skill runtime?

**Options**:
A. Rust-based (in-process, fast)
B. Node.js worker pool (reuses existing skills)
C. External gRPC service (language agnostic)

**Considerations**:
- Existing skill code is TypeScript
- Performance requirements
- Team expertise

**Recommendation**: Node.js worker pool - reuse existing code, add connection pooling

---

### Question 4: CI/CD Pipeline

**Decision needed**: What testing to run in CI?

**Options**:
A. Unit tests only (fast)
B. Unit + Integration (moderate)
C. Full pipeline including chaos (slow)

**Considerations**:
- Build time budget
- Confidence needed
- Infrastructure cost

**Recommendation**: Unit + Integration in CI, nightly chaos testing

---

## Appendix A: File Changes Summary

### Files to Create

- `core/router/src/config/trait.rs` - Config trait
- `core/router/src/config/app.rs` - AppConfig implementation
- `core/router/src/workers/mod.rs` - Worker traits and supervision
- `core/router/src/workers/skill.rs` - Skill worker with supervision
- `core/router/src/workers/deep.rs` - Deep task worker with supervision
- `core/router/src/repository/mod.rs` - Repository trait
- `core/router/src/repository/task.rs` - Task repository
- `core/router/src/outbox/mod.rs` - Outbox pattern
- `core/router/src/metrics.rs` - Prometheus metrics
- `core/router/src/health.rs` - Health check logic

### Files to Modify

- `core/router/src/api.rs` - Split into modules
- `core/router/src/main.rs` - Update initialization
- `core/router/src/lib.rs` - Export new modules

### Files to Delete

- None (deprecate legacy patterns instead)

---

## Appendix B: Success Criteria

| Metric | Current | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|--------|---------|---------|---------|---------|---------|
| API p99 | 329ms | 200ms | 100ms | 50ms | 50ms |
| Skill latency | ~~100ms~~ → 10-15ms | 100ms | 50ms | 10ms | ✅ 10ms (achieved) |
| Test coverage | 40% | 60% | 70% | 80% | 85% |
| Worker restarts | Silent | Logged | Notified | Auto-recover | Auto-recover |
| DB query time | 329ms | 50ms | 20ms | 10ms | 10ms |

---

## Appendix C: Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Migration breaks existing functionality | High | High | Feature flags, extensive testing |
| Performance improvements don't meet targets | Medium | Medium | Profiling, iterative optimization |
| New patterns introduce bugs | High | Medium | Comprehensive testing |
| Team learning curve | Medium | Low | Documentation, pair programming |
| Scope creep | High | Medium | Strict phase boundaries |

---

*Document Version: 1.0*
*Created: 2026-03-05*
*Last Updated: 2026-03-05*
