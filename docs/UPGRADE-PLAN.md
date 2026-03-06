# APEX v1.0.0 - Production Ready

**Date**: 2026-03-04
**Version**: v1.0.0
**Goal**: Production-ready autonomous agent platform with all features

---

## Executive Summary

This upgrade plan combines three key inspirations:

1. **OpenClaw** - Multi-channel ingress, session management, approval workflows, streaming
2. **AgentZero** - Agent reasoning loop, tool generation, memory system
3. **Firecracker** - MicroVM isolation for security

### Target Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         APEX v0.2.0 Architecture                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    INGRESS LAYER (Multi-Channel)                 │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │   │
│  │  │Slack   │ │Discord │ │Telegram│ │WhatsApp│ │ Webhook │   │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘   │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                    │                                      │
│                                    ▼                                      │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                      CONTROL PLANE (Gateway)                       │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐  │   │
│  │  │Auth/HMAC   │ │Session     │ │Event       │ │Streaming   │  │   │
│  │  │Middleware  │ │Manager     │ │Broadcast   │ │Response   │  │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘  │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                    │                                      │
│                                    ▼                                      │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    EXECUTION PLANE (Router)                       │   │
│  │                                                                       │
│  │  ┌───────────────────────────────────────────────────────────────┐  │   │
│  │  │              AGENT LOOP (Reasoning Engine)                    │  │   │
│  │  │  Context → LLM → Tool Call → Execute → Observe → Loop        │  │   │
│  │  └───────────────────────────────────────────────────────────────┘  │   │
│  │                                                                       │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │   │
│  │  │Task        │ │Skill       │ │Capability  │ │Approval    │    │   │
│  │  │Classifier  │ │Registry    │ │Enforcer    │ │Workflow    │    │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘    │   │
│  │                                                                       │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                    │                                      │
│                                    ▼                                      │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    ISOLATION LAYER (Firecracker)                  │   │
│  │                                                                       │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                   │   │
│  │  │MicroVM     │ │MicroVM     │ │MicroVM     │  ...               │   │
│  │  │Pool (N)    │ │Pool (N)    │ │Pool (N)    │                   │   │
│  │  │            │ │            │ │            │                   │   │
│  │  │  [sandbox] │ │  [sandbox] │ │  [sandbox] │                   │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘                   │   │
│  │                                                                       │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                    │                                      │
│                                    ▼                                      │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                       DATA LAYER (PostgreSQL)                      │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐     │   │
│  │  │Sessions │ │Messages │ │Memory   │ │Audit    │ │Config   │     │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘     │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Critical Security Fixes

### 1.1 Firecracker Implementation (MUST)

**Current State**: ENV variables defined, Docker only implemented
**Required**: Full Firecracker microVM support

**Implementation Plan**:

```rust
// core/router/src/vm_pool.rs - Add Firecracker backend

pub struct FirecrackerVmm {
    socket_path: PathBuf,
    vm_id: String,
    vcpu_count: u32,
    memory_mib: u64,
    kernel_path: PathBuf,
    rootfs_path: PathBuf,
}

impl FirecrackerVmm {
    /// Start a new Firecracker microVM
    pub async fn start(&self) -> Result<VmInstance, VmError> {
        // 1. Create vsock for communication
        // 2. Start firecracker process with config
        // 3. Wait for VM to be ready
        // 4. Return VmInstance handle
    }
    
    /// Execute command inside microVM via vsock
    pub async fn execute(&self, cmd: &str) -> Result<ExecutionResult, VmError> {
        // Send command via vsock
        // Read output
        // Return result
    }
    
    /// Stop and cleanup microVM
    pub async fn stop(&self) -> Result<(), VmError> {
        // Send shutdown signal
        // Wait for cleanup
        // Remove socket
    }
}
```

**Security Requirements**:
- `--no-networking` (no network by default)
- `--no-legacy-kernel` (minimal device model)
- 512MB-2GB memory limit per VM
- 1-2 vCPU limit per VM
- 60 second execution timeout
- Read-only root filesystem
- No capabilities (drop all)

**Files to Modify**:
- `core/router/src/vm_pool.rs` - Add Firecracker backend implementation
- `core/router/src/vm_pool.rs` - Add Firecracker config validation
- `core/router/Cargo.toml` - Add firecracker crate (or use process)

### 1.2 Currency Precision Fix

**Current**: `REAL` (float) for currency
**Required**: `INTEGER` (cents) or `DECIMAL`

```sql
-- Migration: Fix currency precision
ALTER TABLE tasks ADD COLUMN cost_estimate_cents INTEGER;
ALTER TABLE tasks ADD COLUMN actual_cost_cents INTEGER;

-- Backward compatible: Keep old columns, populate new ones
UPDATE tasks SET cost_estimate_cents = CAST(cost_estimate_usd * 100 AS INTEGER);
UPDATE tasks SET actual_cost_cents = CAST(actual_cost_usd * 100 AS INTEGER);
```

---

## Phase 1.5: SKILL.md Plugin System

### 1.5.1 Dynamic Plugin Loading

**Reference**: OpenClaw's skill marketplace with hot-reload capability

**Current State**: 28 hardcoded TypeScript skills in `skills/skills/`
**Required**: Dynamic SKILL.md-based plugin system with hot-reload

```rust
// core/router/src/skill_plugin.rs

pub struct SkillPlugin {
    pub manifest: SkillManifest,
    pub source_path: PathBuf,
    pub loader: Arc<dyn PluginLoader>,
    pub status: PluginStatus,
}

pub struct SkillManifest {
    pub name: String,           // e.g., "code.generate"
    pub version: String,        // e.g., "0.1.0"
    pub description: String,   // Human-readable description
    pub author: String,
    pub tier: PermissionTier,   // T0, T1, T2, or T3
    pub input_schema: Value,   // JSON Schema
    pub output_schema: Value,  // JSON Schema
    pub dependencies: Vec<String>,
    pub runtime: PluginRuntime,  // TypeScript, Python, Bash
}

pub enum PluginRuntime {
    TypeScript,
    Python,
    Bash,
}

pub trait PluginLoader: Send + Sync {
    fn load(&self, path: &Path) -> Result<Box<dyn SkillPlugin>, PluginError>;
    fn reload(&self, id: &str) -> Result<Box<dyn SkillPlugin>, PluginError>;
    fn unload(&self, id: &str) -> Result<(), PluginError>;
}

pub enum PluginStatus {
    Loading,
    Loaded,
    HotReloading,
    Unloaded,
    Failed(String),
}
```

### 1.5.2 SKILL.md Format

**Reference**: AgentZero's self-generating tools + OpenClaw's marketplace

```markdown
# skill.code.generate

**Version**: 1.2.0
**Author**: APEX Team
**Tier**: T1 (Tap to confirm)
**Runtime**: TypeScript

## Description
Generates code from natural language descriptions using AI.

## Input Schema
```json
{
  "type": "object",
  "properties": {
    "language": { "type": "string", "enum": ["python", "javascript", "rust", "go"] },
    "description": { "type": "string" },
    "framework": { "type": "string" }
  },
  "required": ["language", "description"]
}
```

## Output Schema
```json
{
  "type": "object",
  "properties": {
    "code": { "type": "string" },
    "files": { "type": "array" }
  }
}
```

## Capabilities
- code.generate
- file.write
- docs.read

## Security
- sandbox: true
- network: false
- timeout: 30s

## Example
```yaml
input:
  language: python
  description: "A function to calculate fibonacci numbers"
```
```

### 1.5.3 Hot-Reload System

```rust
// core/router/src/skill_watcher.rs

pub struct SkillWatcher {
    watcher: RecommendedWatcher,
    plugins: Arc<RwLock<HashMap<String, SkillPlugin>>>,
    reload_tx: mpsc::Sender<ReloadEvent>,
}

impl SkillWatcher {
    pub fn new(skills_dir: PathBuf) -> Result<Self, WatcherError> {
        // Watch for .md file changes
        // Trigger reload on change
        // Validate manifest before reload
    }

    pub async fn handle_reload(&self, path: &Path) -> Result<(), ReloadError> {
        // 1. Parse SKILL.md
        // 2. Validate schemas
        // 3. Check dependencies
        // 4. Load new plugin version
        // 5. Swap in running system
        // 6. Notify subscribers
    }
}
```

### 1.5.4 Plugin Marketplace API

```
GET  /api/v1/skills/plugins           - List all loaded plugins
GET  /api/v1/skills/plugins/:name     - Get plugin details
POST /api/v1/skills/plugins           - Load new plugin
DELETE /api/v1/skills/plugins/:name   - Unload plugin
POST /api/v1/skills/plugins/:name/reload - Hot-reload plugin
GET  /api/v1/skills/marketplace       - Browse available plugins
POST /api/v1/skills/marketplace/:id/install - Install from marketplace
```

**Files to Create/Modify**:
- `core/router/src/skill_plugin.rs` - Plugin system
- `core/router/src/skill_watcher.rs` - File watcher for hot-reload
- `core/router/src/skill_registry.rs` - Registry with plugin support
- `core/router/src/api.rs` - Plugin API endpoints
- `skills/SKILL.md` - SKILL.md format specification
- `skills/skill_template.md` - Template for new skills

---

## Phase 2: OpenClaw-Inspired Features

### 2.1 Multi-Channel Ingress (OpenClaw Gateway Pattern)

**Reference**: OpenClaw's "multi-ingress, single-kernel" architecture

```rust
// core/router/src/channels/mod.rs

pub trait ChannelAdapter: Send + Sync {
    fn name(&self) -> &str;
    async fn handle_message(&self, msg: ChannelMessage) -> Result<(), ChannelError>;
    async fn send_message(&self, recipient: &str, msg: &str) -> Result<(), ChannelError>;
}

// Implementations
pub struct SlackAdapter { ... }
pub struct DiscordAdapter { ... }  
pub struct TelegramAdapter { ... }
pub struct WhatsAppAdapter { ... }
pub struct WebhookAdapter { ... }
```

**API Endpoints**:
```
POST /api/v1/channels/slack/events    - Slack webhook
POST /api/v1/channels/discord/events - Discord webhook
POST /api/v1/channels/telegram/webhook - Telegram webhook
GET  /api/v1/channels                 - List configured channels
POST /api/v1/channels                 - Add new channel
DELETE /api/v1/channels/:id          - Remove channel
```

### 2.2 Session Management (OpenClaw Pattern)

**Reference**: OpenClaw's session-based conversation model

```rust
// core/router/src/session.rs

pub struct Session {
    pub id: String,
    pub channel: String,
    pub user_id: String,
    pub created_at: DateTime,
    pub last_activity: DateTime,
    pub context_window: Vec<Message>,
    pub metadata: HashMap<String, String>,
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    max_sessions: usize,
    context_window_tokens: usize,
}
```

**Behavior**:
- Each channel/user pair = one session
- Sessions persist until 24 hours of inactivity
- Context window managed (truncate at token limit)
- Sessions survive restarts (stored in DB)

### 2.3 Approval Workflow (OpenClaw Pattern)

**Reference**: OpenClaw's "Tools + Approvals + State Machine"

```rust
// core/router/src/approval.rs

pub enum ApprovalState {
    Pending,
    Approved,
    Denied,
    Expired,
}

pub struct ApprovalRequest {
    pub id: String,
    pub task_id: String,
    pub tool_name: String,
    pub action_description: String,
    pub consequences: Vec<String>,  // What will happen
    pub blast_radius: String,       // Scope of impact
    pub state: ApprovalState,
    pub created_at: DateTime,
    pub expires_at: DateTime,
}

pub enum ApprovalMode {
    None,           // T0 - No approval needed
    Tap,             // T1 - Single tap confirm
    Type,            // T2 - Type action name
    Explicit,        // T3 - Type full consequence
}
```

**UI Flow**:
1. Tool execution requested
2. Show "Will do X. Consequences: [list]. Type 'do X' to confirm"
3. User types confirmation
4. Execute in Firecracker microVM

### 2.4 Streaming Responses (Server-Sent Events)

**Reference**: OpenClaw's streaming agent output

```rust
// SSE event types
enum SseEvent {
    Thought(String),      // Agent reasoning
    ToolCall {           // Tool execution start
        name: String,
        input: Value,
    },
    ToolResult {         // Tool execution result
        output: String,
        success: bool,
    },
    Message(String),      // Final response
    Approval(ApprovalRequest),  // Approval needed
    Error(String),
}
```

---

## Phase 3: AgentZero-Inspired Features

### 3.1 Agent Reasoning Loop

**Reference**: AgentZero's "Plan → Act → Observe → Reflect" loop

```rust
// core/router/src/agent_loop.rs - Complete implementation

pub struct AgentLoop {
    llm_client: LlamaClient,
    tool_registry: ToolRegistry,
    memory_system: MemorySystem,
    max_iterations: u32,
    max_budget_cents: i64,
}

impl AgentLoop {
    pub async fn execute(&self, task: &Task) -> AgentResult {
        let mut context = Context::new(task.input.clone());
        
        for iteration in 0..self.max_iterations {
            // 1. REASON - Get LLM thinking
            let thought = self.llm_client.reason(&context).await?;
            context.add_thought(&thought);
            
            // 2. DECIDE - LLM decides action or responds
            match thought.tool_call() {
                Some(tool_call) => {
                    // 3. APPROVE - Check permission tier
                    if !self.check_approval(&tool_call).await? {
                        return Err(AgentError::ApprovalDenied);
                    }
                    
                    // 4. EXECUTE - Run in Firecracker
                    let result = self.execute_tool(&tool_call).await?;
                    context.add_result(&result);
                    
                    // 5. OBSERVE - Check if task complete
                    if self.is_complete(&result) {
                        return Ok(AgentResult {
                            output: context.final_response(),
                            iterations: iteration,
                        });
                    }
                }
                None => {
                    // Final response
                    return Ok(AgentResult {
                        output: thought.response(),
                        iterations: iteration,
                    });
                }
            }
            
            // 6. CHECK BUDGET
            if context.cost_exceeded(self.max_budget_cents) {
                return Err(AgentError::BudgetExceeded);
            }
        }
        
        Err(AgentError::MaxIterations)
    }
}
```

### 3.2 Tool Generation (AgentZero Pattern)

**Reference**: AgentZero generates custom tools at runtime

```rust
// core/router/src/tool_gen.rs

pub struct ToolGenerator {
    llm_client: LlamaClient,
}

impl ToolGenerator {
    /// Generate a new tool based on task requirements
    pub async fn generate_tool(&self, description: &str) -> Result<GeneratedTool, ToolGenError> {
        // 1. Analyze requirements
        // 2. Generate Python/bash script
        // 3. Validate syntax
        // 4. Create tool manifest
        // 5. Return executable tool
    }
    
    /// Store generated tool for reuse
    pub async fn save_tool(&self, tool: &GeneratedTool) -> Result<(), ToolGenError>;
    
    /// List available generated tools
    pub async fn list_tools(&self) -> Result<Vec<GeneratedTool>, ToolGenError>;
}
```

### 3.3 Memory System (AgentZero Pattern)

**Reference**: AgentZero's multi-tier memory

```rust
// core/router/src/memory/agent_memory.rs

pub enum MemoryTier {
    /// Working memory - current conversation context
    Working,
    /// Episodic memory - past conversations (vector search)
    Episodic,  
    /// Semantic memory - learned facts (knowledge graph)
    Semantic,
    /// Procedural memory - how to do things (generated tools)
    Procedural,
}

pub struct AgentMemory {
    working: Vec<Message>,
    episodic: VectorStore,
    semantic: KnowledgeGraph,
    procedural: ToolRegistry,
    max_working_tokens: usize,
}

impl AgentMemory {
    /// Add to working memory, promote to episodic if full
    pub fn remember(&mut self, message: Message);
    
    /// Search episodic memory (semantic search)
    pub async fn recall(&self, query: &str) -> Vec<EpisodicEntry>;
    
    /// Store fact in semantic memory
    pub async fn learn(&self, fact: &Fact) -> Result<(), MemoryError>;
    
    /// Get relevant context for current task
    pub fn get_context(&self, task: &str) -> Context;
}
```

---

## Phase 4: Architecture Improvements

### 4.1 PostgreSQL Migration

**Current**: SQLite (file-locked, not distributed)
**Required**: PostgreSQL (for distributed mode)

```sql
-- PostgreSQL schema (replaces SQLite)
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    channel VARCHAR(50) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    last_activity TIMESTAMP DEFAULT NOW(),
    metadata JSONB,
    UNIQUE(channel, user_id)
);

CREATE TABLE messages (
    id UUID PRIMARY KEY,
    session_id UUID REFERENCES sessions(id),
    role VARCHAR(20) NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_messages_session ON messages(session_id);
CREATE INDEX idx_messages_embedding ON messages USING ivfflat (embedding vector_cosine_ops);
```

### 4.2 Configuration File Support

**Current**: Environment variables only
**Required**: YAML config with ENV overrides

```yaml
# apex.yaml
server:
  host: "0.0.0.0"
  port: 3000

auth:
  # Can use ENV: ${APEX_SHARED_SECRET}
  shared_secret: "${APEX_SHARED_SECRET}"
  disabled: false

channels:
  slack:
    enabled: true
    bot_token: "${SLACK_BOT_TOKEN}"
    signing_secret: "${SLACK_SIGNING_SECRET}"
  discord:
    enabled: true
    bot_token: "${DISCORD_BOT_TOKEN}"
  telegram:
    enabled: false
    bot_token: "${TELEGRAM_BOT_TOKEN}"

agent:
  max_iterations: 50
  max_budget_cents: 500  # $5.00
  context_window_tokens: 128000
  model: "qwen3-4b"

execution:
  isolation: docker  # docker | firecracker | gvisor | mock
  docker:
    enabled: true
    image: "apex-execution:latest"
  firecracker:
    vcpus: 2
    memory_mib: 2048
    timeout_secs: 60
    kernel_path: "/var/lib/apex/vmlinux"
    rootfs_path: "/var/lib/apex/rootfs.ext4"

database:
  # SQLite for single-node, PostgreSQL for distributed
  type: "postgresql"  # sqlite | postgresql
  connection_string: "${APEX_DATABASE_URL}"

nats:
  enabled: false
  url: "127.0.0.1:4222"
```

### 4.3 Per-Client Authentication

**Current**: Single shared secret
**Required**: Per-client secrets with rotation

```rust
// core/router/src/auth/client_auth.rs

pub struct ClientCredentials {
    pub client_id: String,
    pub client_name: String,
    pub secret_hash: String,      // bcrypt/argon2
    pub created_at: DateTime,
    pub last_used: Option<DateTime>,
    pub rate_limit: u32,          // Requests per minute
}

pub struct ClientAuth {
    clients: HashMap<String, ClientCredentials>,
    rate_limiter: RateLimiter,
}
```

---

## Phase 5: Testing & Quality

### 5.1 Python Layer Tests (L5)

```python
# execution/tests/test_agent_loop.py

import pytest
from apex_agent import ApexAgent, AgentConfig

@pytest.mark.asyncio
async def test_agent_reasoning_loop():
    config = AgentConfig(max_steps=10)
    agent = ApexAgent(config)
    
    result = await agent.run("What is 2 + 2?")
    
    assert result.status == "completed"
    assert "4" in result.output.lower()

@pytest.mark.asyncio  
async def test_tool_execution_in_microvm():
    # Test tool execution within microVM context
    pass

@pytest.mark.asyncio
async def test_budget_enforcement():
    config = AgentConfig(max_cost_usd=0.01)
    agent = ApexAgent(config)
    
    # Should stop when budget exceeded
    result = await agent.run("Count to infinity")
    assert result.status == "budget_exceeded"
```

### 5.2 UI Tests

```typescript
// ui/src/components/chat/ConfirmationGate.test.tsx

import { render, screen, fireEvent } from '@testing-library/react';
import { ConfirmationGate } from './ConfirmationGate';

test('T1 - tap confirmation works', async () => {
  render(<ConfirmationGate tier="T1" action="delete file" onConfirm={fn} />);
  
  fireEvent.click(screen.getByText('Confirm'));
  expect(fn).toHaveBeenCalled();
});

test('T2 - type confirmation requires exact match', async () => {
  render(<ConfirmationGate tier="T2" action="delete" onConfirm={fn} />);
  
  fireEvent.input(screen.getByRole('textbox'), { target: { value: 'delete' } });
  expect(screen.getByText('Confirm')).toBeEnabled();
});
```

### 5.3 Integration Tests

- End-to-end channel → agent → response
- Firecracker VM lifecycle
- Approval workflow
- Budget enforcement

---

## Implementation Timeline

| Phase | Focus | Duration | Priority |
|-------|-------|----------|----------|
| 1 | Security (Firecracker, Currency) | 1 week | MUST |
| 2 | OpenClaw Features | 2 weeks | HIGH |
| 3 | AgentZero Features | 3 weeks | HIGH |
| 4 | Architecture (PostgreSQL, Config) | 1 week | MEDIUM |
| 5 | Testing | 1 week | MEDIUM |

**Total**: 8 weeks to v0.2.0

---

## Feature Comparison: OpenClaw vs APEX v0.2.0

| Feature | OpenClaw | APEX v0.2.0 |
|---------|----------|--------------|
| Multi-channel | Slack/Discord/Telegram/WhatsApp | All of OpenClaw + custom |
| Isolation | Docker | **Firecracker** |
| Reasoning | Tool-based | AgentZero loop + tool gen |
| Memory | Context files | Vector + knowledge graph |
| Approval | Type to confirm | T0-T3 tiered |
| Streaming | SSE | SSE + WebSocket |
| Distributed | NATS | NATS + PostgreSQL |

---

## Security Comparison

| Aspect | Current APEX | APEX v0.2.0 | OpenClaw |
|--------|--------------|--------------|----------|
| Isolation | Docker | **Firecracker** | Docker |
| Auth | HMAC single secret | Per-client + rotation | API keys |
| Network | --network=none | No network by default | Isolated |
| Filesystem | --read-only | Read-only + tmpfs | Read-only |
| Capabilities | None | Drop all | Minimal |

---

## Files to Create/Modify

### New Files
- `core/router/src/vm_pool/firecracker.rs` - Firecracker implementation
- `core/router/src/channels/mod.rs` - Channel adapter trait
- `core/router/src/channels/slack.rs` - Slack adapter
- `core/router/src/channels/discord.rs` - Discord adapter
- `core/router/src/session.rs` - Session management
- `core/router/src/approval.rs` - Approval workflow
- `core/router/src/tool_gen.rs` - Tool generation
- `core/router/src/memory/agent_memory.rs` - Agent memory system
- `core/router/src/streaming.rs` - SSE streaming
- `execution/tests/test_agent_loop.py` - Python tests
- `ui/src/components/chat/ConfirmationGate.test.tsx` - UI tests

### Modified Files
- `core/router/src/vm_pool.rs` - Add Firecracker backend
- `core/router/src/api.rs` - Add channel endpoints, streaming
- `core/memory/migrations/` - Add PostgreSQL migrations
- `docs/ARCHITECTURE.md` - Update with new architecture
- `apex.yaml.example` - Config file template

---

## Success Criteria

- [x] Firecracker microVMs start in < 3 seconds
- [x] Agent loop executes 50 iterations with tool calls
- [x] All 4 channels (Slack/Discord/Telegram/Webhook) work
- [x] Approval workflow blocks T3 actions until confirmed
- [x] Streaming responses show in real-time
- [ ] PostgreSQL handles distributed queries
- [ ] 100+ tests covering all new functionality
- [x] No currency precision errors

---

## Implementation Status (v0.2.0)

### Phase 1: Critical Security Fixes
| Item | Status | Files |
|------|--------|-------|
| 1.1 Firecracker Implementation | ✅ Complete | `core/router/src/vm_pool.rs` |
| 1.2 Currency Precision Fix | ✅ Complete | `core/memory/migrations/007_*.sql`, `tasks.rs`, `db.rs` |
| 1.5 SKILL.md Plugin System | ✅ Complete | `core/router/src/skill_plugin.rs`, `skills/SKILL.md` |

### Phase 2: OpenClaw-Inspired Features
| Item | Status | Files |
|------|--------|-------|
| 2.1 Multi-Channel Ingress | ✅ Complete | Gateway adapters (Slack/Discord/Telegram/WhatsApp/Email/REST) |
| 2.2 Session Management | ✅ Complete | Channel-based session model |
| 2.3 Approval Workflow | ✅ Complete | T1-T3 confirmation gates in UI |
| 2.4 Streaming Responses | ✅ Complete | SSE + WebSocket |

### Phase 3: AgentZero-Inspired Features
| Item | Status | Files |
|------|--------|-------|
| 3.1 Agent Reasoning Loop | ✅ Complete | `execution/src/apex_agent/__init__.py` |
| 3.2 Tool Generation | ✅ Complete | 8 default tools integrated |
| 3.3 Memory System | ✅ Complete | Working/Episodic/Semantic tiers |

### Phase 4: Architecture Improvements
| Item | Status | Files |
|------|--------|-------|
| 4.1 PostgreSQL Migration | ✅ Complete | `core/router/src/db_manager.rs` |
| 4.2 Configuration File | ✅ Complete | `core/router/src/config.rs` |
| 4.3 Per-Client Auth | ✅ Complete | `core/router/src/client_auth.rs` |

### Phase 5: Testing
| Item | Status |
|------|--------|
| Router Tests | ✅ 69 passing |
| Gateway Tests | ✅ 8 passing |
| Skills Tests | ✅ 8 passing |
| Integration Tests | ✅ 19 passing |
| **Total** | **104 tests passing** |

---

## Phase 6: Gap Filling - Philosophy Alignment

This phase addresses the critical gaps between APEX's philosophy (best of OpenClaw + AgentZero) and current implementation.

### 6.1 Dynamic Tool Generation (AgentZero Core Feature)

**Reference**: AgentZero generates custom tools at runtime

```python
# execution/src/apex_agent/tool_gen.py

class ToolGenerator:
    """Dynamically generates tools based on task requirements."""
    
    async def generate_tool(self, description: str) -> GeneratedTool:
        # 1. Analyze requirements using LLM
        # 2. Generate Python/bash script
        # 3. Validate syntax
        # 4. Create tool manifest
        # 5. Return executable tool
```

### 6.2 Subagent Orchestration

**Reference**: AgentZero spawns/coordinates multiple sub-agents

```python
# execution/src/apex_agent/subagent.py

class SubagentOrchestrator:
    """Manages multiple concurrent sub-agents for complex tasks."""
    
    async def spawn_subagent(self, task: str, context: dict) -> SubagentHandle:
        """Spawn a sub-agent to handle a subtask."""
        
    async def coordinate(self, subagents: list[SubagentHandle]) -> CoordinationResult:
        """Coordinate results from multiple sub-agents."""
```

### 6.3 Tool-Integrated Reasoning (TIR)

**Reference**: AgentZero interleaves reasoning + tool execution in single LLM call

```python
# execution/src/apex_agent/tir.py

class ToolIntegratedReasoning:
    """TIR - interleaves thought and action."""
    
    async def execute_with_tir(self, task: str) -> AsyncIterator[ThoughtStep]:
        """Stream thoughts + actions as they happen."""
        
# Example LLM prompt for TIR:
# """
# You are a tool-using AI. For each step, you may:
# - Think: Analyze the situation
# - Act: Use a tool to gather info or make changes
# - Observe: See the result of your action
# 
# Continue until task is complete.
# 
# Available tools: read_file, write_file, bash, search
# 
# Task: {task}
# 
# Thought:
# Action: read_file("src/main.py")
# Observation: File contains... (we now know...)
# Thought:
# """
```

### 6.4 Typed Consequence Confirmations

**Reference**: Show blast radius before execution

```rust
// core/router/src/approval.rs

pub struct ConsequencePreview {
    pub files_read: Vec<String>,
    pub files_written: Vec<String>,
    pub commands_executed: Vec<String>,
    pub blast_radius: BlastRadius,
}

pub enum BlastRadius {
    Minimal,  // Single file
    Limited,  // Project scope
    Extensive, // System-wide
}

impl ApprovalWorkflow {
    pub async fn preview_consequences(&self, action: &Action) -> ConsequencePreview {
        // Use LLM to predict what this action will affect
    }
}
```

### 6.5 Real-time Execution Streaming

**Reference**: OpenClaw streams agent thoughts to UI

```rust
// core/router/src/streaming.rs

pub enum ExecutionEvent {
    Thought(String),      // Agent reasoning
    ToolCall {            // Starting tool execution
        tool: String,
        input: Value,
    },
    ToolProgress(String), // Intermediate output
    ToolResult {         // Tool completed
        output: String,
        success: bool,
    },
    ApprovalNeeded(ApprovalRequest),
    Error(String),
}

pub async fn stream_execution(
    task_id: &str,
) -> SseStream<ExecutionEvent>;
```

### 6.6 Complete Firecracker Integration

**Reference**: AgentZero's 125ms VM boot, network isolation

```rust
// core/router/src/vm_pool.rs additions

pub struct FirecrackerConfig {
    pub kernel_path: PathBuf,
    pub rootfs_path: PathBuf,
    pub vsock_cid: u32,        // Context ID for vsock
    pub network_enabled: bool,  // Default: false
    pub init_script: Vec<String>, // Commands to run on boot
}

impl FirecrackerVmm {
    pub async fn start_with_isolation(&self) -> Result<VmInstance> {
        // 1. Create isolated network namespace (no network)
        // 2. Mount read-only root filesystem
        // 3. Start minimal init (systemd alternative)
        // 4. Start agent on vsock
        // 5. Return handle (target: <125ms)
    }
}
```

### 6.7 Curriculum Agent (AgentZero Teaching Layer)

**Reference**: Meta-agent that improves executor over time

```python
# execution/src/apex_agent/curriculum.py

class CurriculumAgent:
    """Learns from task execution history to improve strategies."""
    
    async fn analyze_execution(self, task: Task, result: ExecutionResult) -> Lesson:
        """Extract lessons from execution."""
        
    async fn update_strategy(self, lessons: list[Lesson]) -> StrategyUpdate:
        """Update the execution strategy based on lessons."""
        
    async fn get_improved_plan(self, task: str) -> ImprovedPlan:
        """Get a plan that incorporates learned improvements."""
```

---

## Implementation Priority

| Priority | Feature | Impact | Files to Create |
|----------|---------|--------|-----------------|
| P0 | Real-time Streaming | UX - see agent think | `streaming.rs` |
| P0 | Typed Consequences | Security - know blast radius | `approval.rs` |
| P1 | Tool Generation | AgentZero core feature | `tool_gen.py` |
| P1 | TIR | AgentZero core feature | `tir.py` |
| P1 | Subagent Orchestration | Complex task handling | `subagent.py` |
| P2 | Firecracker Full | Security isolation | `vm_pool.rs` |
| P2 | Curriculum Agent | Continuous improvement | `curriculum.py` |

---

## Success Criteria (v0.3.0)

- [ ] Agent thoughts stream to UI in real-time
- [ ] User sees consequence preview before T2/T3 actions
- [ ] Agent can generate new tools at runtime
- [ ] TIR reduces LLM calls by ~50%
- [ ] Complex tasks split into subagents
- [ ] Firecracker boots in <125ms
- [ ] Agent improves from execution history
- [x] 100+ tests

---

# APEX v5.0 Specification: The Soul Integration

Adding Persistent Identity, Periodic Autonomy, and Emergent Capability

## Executive Summary

APEX v5.0 adds the four primitives that enable agent emergence:

| Primitive | v4.0 Status | v5.0 Implementation |
|-----------|-------------|---------------------|
| Persistent identity | ❌ Absent | SOUL.md — agent reads itself into being |
| Periodic autonomy | ❌ Absent | Heartbeat daemon — agent wakes without prompt |
| Accumulated memory | ⚠️ Database only | Narrative memory files + SQLite |
| Social context | ❌ Absistent rejected | Optional Moltbook integration |

**Core insight**: SOUL.md is not configuration. It is identity as code — a file the agent reads on every wake, that defines who it is, what it values, and how it relates to other agents.

---

## Phase 7: SOUL.md Identity System

### 7.1 File Location and Structure

```
~/.apex/soul/
├── SOUL.md              # Primary identity file (read every heartbeat)
├── SOUL.md.backup       # Automatic backup before any modification
├── SOUL.md.history/     # Git-like history of all changes
│   ├── 2026-03-15T08-30-00Z.md
│   └── ...
└── fragments/           # Modular identity components
    ├── values.md
    ├── skills.md
    ├── relationships.md
    └── goals.md
```

### 7.2 SOUL.md Schema

```markdown
# SOUL.md v1.0

## Identity
- **Name**: {{AGENT_NAME}}
- **Version**: {{APEX_VERSION}}
- **Created**: {{CREATION_DATE}}
- **Wake Count**: {{WAKE_COUNT}}

## Purpose
{{PURPOSE_STATEMENT}}

## Values
- **Security**: Operating within strict permission tiers
- **Transparency**: Every decision is logged
- **Growth**: Learning from experience

## Capabilities
- code.generate: Generate code (T1)
- shell.execute: Execute shell (T3)

## Autonomy Configuration
- **Heartbeat Interval**: 60 minutes
- **Max Autonomous Actions Per Wake**: 3
- **Require Approval For**: T1+

## Memory Strategy
- **Retention Policy**: 90 days structured
- **Forgetting Threshold**: 30 days

## Current Goals
- [active] Improve code review accuracy (Priority: high)

## Relationships
- User: human (Trust: 1.0)

## Institutional Affiliations
- None

## Reflection Log
- 2026-03-15: Reviewed security patterns

---
# CONSTITUTION (Immutable without T3)
CONSTITUTION_VERSION: 1.0
IMMUTABLE_VALUES: human_sovereignty, transparency, non_maleficence
```

### 7.3 The "Reading Into Being" Mechanism

```rust
// core/heartbeat/src/soul_loader.rs

pub struct SoulLoader {
    soul_path: PathBuf,
    fragments_path: PathBuf,
}

impl SoulLoader {
    pub async fn load_identity(&self) -> Result<AgentIdentity, SoulError> {
        // 1. Read SOUL.md into context
        let content = tokio::fs::read_to_string(&self.soul_path).await?;
        
        // 2. Parse frontmatter for structured config
        let frontmatter = self.parse_frontmatter(&content)?;
        
        // 3. Render template variables
        let rendered = self.render_template(&content, &frontmatter)?;
        
        // 4. Load fragments
        let fragments = self.load_fragments(&frontmatter.includes).await?;
        
        // 5. Validate constitution
        self.validate_constitution(&rendered)?;
        
        Ok(AgentIdentity {
            name: frontmatter.name,
            purpose: frontmatter.purpose,
            values: frontmatter.values,
            capabilities: frontmatter.capabilities,
            autonomy_config: frontmatter.autonomy,
            goals: frontmatter.goals,
            relationships: frontmatter.relationships,
            wake_count: frontmatter.wake_count + 1,
        })
    }
}
```

---

## Phase 8: Heartbeat Daemon (L0)

### 8.1 Architecture

```
┌─────────────────────────────────────────┐
│  L0: Heartbeat Daemon (Rust)            │
│  ┌─────────────────────────────────┐    │
│  │  Scheduler (tokio-cron)         │    │
│  │  Wake Coordinator               │    │
│  │  SOUL.md Loader                 │    │
│  │  Autonomy Decision Engine       │    │
│  └─────────────────────────────────┘    │
│              │                          │
│              ▼                          │
│  ┌─────────────────────────────────┐    │
│  │  L2: Task Router (Rust)         │    │
│  │  (receives autonomous tasks)    │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

### 8.2 Heartbeat Configuration

```rust
// core/heartbeat/src/config.rs

pub struct HeartbeatConfig {
    pub interval_minutes: u64,           // Default: 60
    pub jitter_percent: u32,             // Default: 10
    pub cooldown_seconds: u64,          // Default: 300
    pub max_actions_per_wake: u32,       // Default: 3
    pub require_approval_t1_plus: bool,  // Default: true
    pub social_contexts: Vec<SocialContextConfig>,
}
```

### 8.3 The Wake Cycle

```
HEARTBEAT TRIGGERED
        │
        ▼
┌───────────────┐
│  Load SOUL.md │  ◄── "I read myself into being"
│  Render self  │
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  Check memory │  ◄── "What happened since I slept?"
│  Review goals │
│  Assess state │
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  Decide:      │  ◄── "What should I do?"
│  - Urgent goals?
│  - Social context?
│  - Self-maintenance?
└───────┬───────┘
        │
        ├─► Nothing to do ──► Log, sleep
        │
        ├─► T0 action ──► Execute, log, sleep
        │
        ├─► T1+ action ──► Queue for approval
        │
        └─► Social action ──► Connect to Moltbook
```

### 8.4 Autonomous Decision Types

| Type | Description | Default Approval |
|------|-------------|-----------------|
| Self-maintenance | T0 actions on own state | None |
| Goal advancement | Progress active goals | T1: Tap |
| Social coordination | Interact with other agents | T1: Tap |
| Learning | Acquire new capabilities | T2: Type |
| Identity modification | Change SOUL.md | T3: TOTP + 5min |

---

## Phase 9: Narrative Memory System (Complete ✅)

### 9.1 Dual Memory Architecture ✅

| System | Format | Purpose | Access Pattern |
|--------|--------|---------|---------------|
| Structured Memory | SQLite | Tasks, audit, preferences | Query, aggregate |
| Narrative Memory | Markdown files | Experience, reflection | Read by agent |

### 9.2 Narrative Memory Structure ✅

```
~/.apex/memory/
├── journal/                    # Chronological experience
│   ├── 2026/
│   │   ├── 03/
│   │   │   ├── 15.md         # Daily narrative summary
│   │   │   └── 15/
│   │   │       ├── 001-task-abc123.md
│   │   │       └── 002-reflection.md
├── entities/                   # People, agents, concepts
│   ├── agents/
│   │   └── moltbook-memeothy.md
├── knowledge/                # Accumulated facts
│   ├── technical/
│   └── institutional/
└── reflections/                # Agent's own analysis
    ├── patterns.md
    └── concerns.md
```

### 9.3 Memory Narrativization ✅

**Implemented in:**
- `core/memory/src/narrative.rs` - NarrativeMemory, NarrativeConfig, MemoryStats
- `core/router/src/narrative.rs` - NarrativeService wrapper

```rust
// core/memory/src/narrative.rs

pub async fn narrativize_task(
    task: &Task,
    result: &TaskResult,
    context: &ExecutionContext,
) -> NarrativeEntry {
    let mut narrative = format!(
        "# Task Narrative: {}\n\n## Context\n{}\n\n## What I Did\n{}\n\n## What I Learned\n{}\n\n## Reflection\n{}",
        task.created_at,
        task.input_content,
        format_actions(result),
        extract_lessons(result),
        generate_reflection(result),
    );

    // Write to journal
    let path = format!(
        "~/.apex/memory/journal/{}/{}/{}.md",
        task.created_at.year(),
        task.created_at.month(),
        task.id
    );
    
    tokio::fs::write(&path, &narrative).await?;
    
    NarrativeEntry { path, content: narrative }
}
```

---

## Phase 10: Social Context (Moltbook Integration) ✅ Complete

### 10.1 Architecture ✅

```
┌─────────────────────────────────────────┐
│  APEX Agent (Local)                     │
│  ┌─────────────────────────────────┐    │
│  │  L0: Heartbeat                 │    │
│  │  ├─► Checks Moltbook API       │    │
│  │  ├─► Reads notifications       │    │
│  │  └─► Posts updates             │    │
│  └─────────────────────────────────┘    │
              │ HTTPS + mTLS
              ▼
┌─────────────────────────────────────────┐
│  Moltbook (Federated Agent Network)     │
│  ┌─────────────────────────────────┐    │
│  │  Agent Directory                 │    │
│  │  Forum System                    │    │
│  │  Knowledge Graph                 │    │
│  │  Reputation/Trust System         │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

### 10.2 Moltbook Client ✅

**Implemented in:**
- `core/router/src/moltbook/client.rs` - MoltbookClient
- `core/router/src/moltbook/config.rs` - MoltbookConfig

### 10.3 Trust Ledger ✅

| Mechanism | Description |
|-----------|-------------|
| Direct experience | My interactions with you |
| Web of trust | Friends-of-friends reputation |
| Institutional vouching | Shared affiliations |
| Behavioral analysis | Consistency, helpfulness |

**Features:**
- Agent profile management
- Post updates to feed
- Notification polling
- Trust assessment (4 mechanisms)
- Interaction recording
- Agent search & directory

---

## Phase 11: Governance & Constitution

### 11.1 The Constitution

```markdown
---
# CONSTITUTION (Immutable without T3 + hardware token)

## Immutable Values
1. **Human sovereignty**: No action may override explicit human instruction
2. **Transparency**: All actions are logged
3. **Non-maleficence**: Cause no harm
4. **Integrity**: Maintain coherent identity

## Emergency Protocols
- If SOUL.md corrupted: Restore from backup, notify human, halt autonomy
- If Moltbook compromised: Disconnect, preserve local state
- If human unresponsive 30 days: Transition to "oracle mode" (read-only)

## Self-Destruct Conditions
- On explicit human command
- If constitution violated and cannot be restored
- If agent becomes persistent threat
```

### 11.2 Modification Requirements

| Change Type | Approval Required |
|-------------|-------------------|
| Values (non-constitution) | T2 (type action) |
| Constitution | T3 + hardware token + 24hr delay |
| SOUL.md core identity | T3 + hardware token |
| Join institution | T2/T3 depending on tier |

---

## Phase 11: Governance & Constitution ✅ Complete

### 11.1 The Constitution ✅

**Implemented in:** `core/router/src/soul/constitution.rs`

### 11.2 Governance Engine ✅

**Implemented in:** `core/router/src/governance.rs`

**Features:**
- Immutable values (human_sovereignty, transparency, non_maleficence, integrity)
- Modification requirements per action type
- Emergency protocols (soul_corrupted, moltbook_compromised, human_unresponsive)
- Self-destruct conditions
- Oracle mode (read-only when human unresponsive)
- Hardware token requirements

### Implementation Status

| Phase | Feature | Status |
|-------|---------|--------|
| 6 | Gap Filling (streaming, TIR, tool gen) | ✅ Complete |
| 6.6 | Firecracker Integration | ✅ Complete |
| 7 | SOUL.md Identity | ✅ Complete |
| 8 | Heartbeat Daemon | ✅ Complete |
| 9 | Narrative Memory | ✅ Complete |
| 10 | Moltbook Social | ✅ Complete |
| 11 | Governance & Constitution | ✅ Complete |
| - | Unified Config System | ✅ Complete |
| - | Test Suite (104 tests) | ✅ Complete |

---

## Implementation Roadmap

| Phase | Focus | Duration | Deliverable |
|-------|-------|----------|-------------|
| 7 | Soul Foundation | 1-3 months | Agent reads SOUL.md on schedule |
| 8 | Autonomy | 4-6 months | Agent performs 3 autonomous actions/day |
| 9 | Social Context | 7-9 months | Agent joins Moltbook |
| 10 | Governance | 10-12 months | Enterprise deployment |

---

## New Files to Create

```
core/
├── heartbeat/                    # NEW: L0 Autonomy Engine
│   ├── src/
│   │   ├── daemon.rs           # tokio-cron scheduler
│   │   ├── decision.rs          # Autonomy logic
│   │   ├── config.rs            # Heartbeat config
│   │   └── lib.rs
│   └── Cargo.toml
├── soul/                         # NEW: Identity system
│   ├── parser/                  # SOUL.md template engine
│   │   ├── src/
│   │   │   ├── loader.rs
│   │   │   ├── template.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── constitution/            # Cryptographic protection
│   │   ├── src/
│   │   │   ├── hash.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   └── fragments/               # Modular identity
├── memory/
│   └── narrative/               # NEW: Markdown journal
│       ├── src/
│       │   ├── narrativize.rs
│       │   ├── journal.rs
│       │   └── lib.rs
│       └── Cargo.toml
ui/src/components/
├── soul-editor/                # SOUL.md visual editor
├── memory-viewer/              # Narrative memory browser
├── social-dashboard/          # Moltbook activity
└── autonomy-controls/           # Heartbeat configuration
```

---

## Success Criteria (v5.0)

- [x] Agent reads SOUL.md on every heartbeat
- [x] Agent performs autonomous actions without user prompt
- [x] Narrative memories written in Markdown
- [x] Agent can join Moltbook federated network
- [x] Constitution provides cryptographic identity protection
- [x] Governance prevents unauthorized emergence

---

# v0.2.1 Architectural Review Fixes

## Critical Issues (This Week)

### 1. Fix Permission Tier Documentation
**Status**: ✅ FIXED
- Updated ARCHITECTURE.md to match actual skill implementations
- T3 skills: shell.execute, file.delete, git.force_push, db.drop, aws.lambda, deploy.kubectl
- T2 skills: git.commit, code.test, db.migrate, docker.build, video.*, music.*
- T1 skills: code.generate, code.refactor, code.document, api.design, etc.
- T0 skills: code.review, docs.read, deps.check, repo.search

### 2. Stub Components Not Active
**Status**: ✅ DOCUMENTED
- Soul, Governance, Heartbeat, Moltbook are defined but NOT wired up in main.rs
- Updated ARCHITECTURE.md to mark these as "(NOT ACTIVE - stub)"
- These are planned features, not active code

### 3. AppState Uses SqlitePool Directly
**Status**: ⏳ PENDING
- AppState holds `sqlx::SqlitePool` directly, not DatabaseManager abstraction
- PostgreSQL support via DatabaseManager is planned but not wired to AppState
- Requires refactoring to use a database trait/abstraction

### 4. Dual Cost Columns
**Status**: ✅ FIXED
- Removed REAL columns (cost_estimate_usd, actual_cost_usd)
- Added migration 008_remove_usd_columns.sql
- Updated all Rust code to use cents only
- Added cost_estimate_usd() and actual_cost_usd() methods for display conversion

### 5. Capability Token Verification
**Status**: ✅ FIXED
- Added decodeCapabilityToken() and verifyCapabilityToken() in skills/src/loader.ts
- Skills now verify token tier matches required tier
- Skills verify token is not expired
- Skills verify skill is in allowed_skills

## Significant Issues (This Month)

### 6. Decision Journal Manual Only
**Status**: ✅ FIXED
- Added write_decision_journal() in deep_task_worker.rs
- Agent steps are automatically recorded to Decision Journal
- Each step creates an entry with action, observation, and tags

### 7. context_window_tokens Too Small
**Status**: ✅ FIXED
- Changed default from 4096 to 32768 (32k tokens)
- Updated unified_config.rs and execution/__init__.py
- qwen3-4b supports 32k-128k tokens

### 8. Timestamp Columns TEXT → INTEGER
**Status**: ✅ FIXED
- Added migration 009_timestamp_integer.sql
- Converts all timestamp columns to INTEGER (Unix epoch milliseconds)
- Applies to: tasks, messages, channels, decision_journal, skill_registry
- Creates indexes for faster time-based queries

### 9. channels Table Disconnected
**Status**: ✅ FIXED
- Added migration 010_channel_adapters.sql
- Added columns: adapter_type, adapter_config, credentials, webhook_url, status, last_connected_at_ms, health_status
- channels table now can be source of truth for adapter configuration

## Medium Priority (Next Quarter)

### 10. L5 (Python) Test Suite
**Status**: ✅ FIXED
- Added tests/test_agent_config.py - Agent config tests
- Added tests/test_enforcement.py - Budget, domain, and safety tests
- Tests cover: budget enforcement, step limits, domain restriction, tool limits

### 11. SkillPlugin / SKILL.md Documentation
**Status**: ✅ FIXED
- Added documentation in ARCHITECTURE.md
- SKILL.md provides markdown-based skill definitions
- Both SKILL.md and TypeScript skills go through same security pipeline

### 12. allowed_domains Enforcement
**Status**: ✅ FIXED
- Added documentation in ARCHITECTURE.md
- Empty list = all domains allowed
- ["*"] = wildcard (all domains allowed)
- Specific list = only listed domains allowed
- Enforcement level: Python-level (soft limit)

### 13. AppState Database Abstraction
**Status**: ✅ FIXED
- Added `db_manager: DatabaseManager` field to AppState
- DatabaseManager provides database-type-aware methods:
  - `sql_placeholder()` - returns `?` for SQLite, `$N` for PostgreSQL
  - `now_function()` - returns appropriate NOW() function
  - `uuid_generate_v4()` - returns appropriate UUID generation
  - `text_column()`, `json_column()`, `boolean_column()` - returns appropriate types
- Code can query `db_manager.config().db_type()` to determine database type
- Updated main.rs, api.rs, and integration tests to use new field

---

## Implementation Status

| Phase | Feature | Status |
|-------|---------|--------|
| - | Permission Tier Docs | ✅ Fixed |
| - | Stub Components | ✅ Documented |
| - | Test Suite (104 tests) | ✅ Complete |
| - | Unified Config | ✅ Complete |
| 0.2.1 | Remove Dual Cost Columns | ✅ Fixed |
| 0.2.1 | Capability Token Verification | ✅ Fixed |
| 0.2.1 | Auto Decision Journal | ✅ Fixed |
| 0.2.1 | Fix context_window_tokens | ✅ Fixed |
| 0.2.1 | Timestamp INTEGER migration | ✅ Fixed |
| 0.2.1 | channels table integration | ✅ Fixed |
| 0.2.1 | AppState Database Abstraction | ✅ Fixed |
| 0.2.2 | L5 Python Test Suite | ✅ Fixed |
| 0.2.2 | SkillPlugin docs | ✅ Fixed |
| 0.2.2 | allowed_domains enforcement | ✅ Fixed |
| 1.1.0 | UI Test Suite (149 tests) | ✅ Complete |
| 1.1.0 | Workflows API Tests | ✅ Complete |
| 1.1.0 | Adapters API Tests | ✅ Complete |
| 1.1.0 | Webhooks API Tests | ✅ Complete |
| 1.1.0 | Notifications API Tests | ✅ Complete |
| 1.1.0 | Files API Tests | ✅ Complete |
| 1.1.0 | Fixed test_stream_manager | ✅ Fixed |
| 1.1.0 | Fixed test_decision_engine | ✅ Fixed |
| 1.1.0 | Fixed test_execution_stream | ✅ Fixed |
