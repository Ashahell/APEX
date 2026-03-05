# APEX REST API Documentation

## Base URL
```
http://localhost:3000
```

## Endpoints

### Tasks

#### Create Task
```
POST /api/v1/tasks
```

Create a new task. Auto-tiers based on content classification.

**Request Body:**
```json
{
  "content": "string (required)",
  "channel": "string (optional)",
  "thread_id": "string (optional)",
  "author": "string (optional)",
  "max_steps": "number (optional, default: 3)",
  "budget_usd": "number (optional, default: 1.0)",
  "time_limit_secs": "number (optional)",
  "project": "string (optional)",
  "priority": "string (optional, default: medium)",
  "category": "string (optional)"
}
```

**Response:**
```json
{
  "task_id": "string",
  "status": "pending|running|completed|failed",
  "tier": "instant|shallow|deep",
  "capability_token": "string",
  "instant_response": "string (optional - only for instant tier)"
}
```

#### List Tasks
```
GET /api/v1/tasks?project=&status=&priority=&category=&limit=100&offset=0
```

List all tasks (most recent first). Supports filtering by project, status, priority, and category.

**Query Parameters:**
- `project` - Filter by project name
- `status` - Filter by status (pending, running, completed, failed, cancelled)
- `priority` - Filter by priority (low, medium, high, urgent)
- `category` - Filter by category
- `limit` - Number of tasks to return (default: 100)
- `offset` - Number of tasks to skip

**Response:**
```json
[
  {
    "task_id": "string",
    "status": "string",
    "output": "string (optional)",
    "error": "string (optional)",
    "project": "string (optional)",
    "priority": "string (optional)",
    "category": "string (optional)",
    "created_at": "string (optional)"
  }
]
```

#### Get Filter Options
```
GET /api/v1/tasks/filter-options
```

Get available filter values for tasks.

**Response:**
```json
{
  "projects": ["project1", "project2"],
  "categories": ["bug", "feature", "research"],
  "priorities": ["low", "medium", "high", "urgent"],
  "statuses": ["pending", "running", "completed", "failed", "cancelled"]
}
```

#### Get Task
```
GET /api/v1/tasks/:task_id
```

Get task status and result.

**Response:**
```json
{
  "task_id": "string",
  "status": "pending|running|completed|failed",
  "output": "string (optional)",
  "error": "string (optional)",
  "project": "string (optional)",
  "priority": "string (optional)",
  "category": "string (optional)",
  "created_at": "string (optional)"
}
```

#### Update Task
```
PUT /api/v1/tasks/:task_id
```

Update task fields (project, priority, category).

**Request Body:**
```json
{
  "project": "string (optional)",
  "priority": "string (optional)",
  "category": "string (optional)"
}
```

**Response:**
```json
{
  "task_id": "string",
  "status": "string",
  "output": "string (optional)",
  "error": "string (optional)",
  "project": "string (optional)",
  "priority": "string (optional)",
  "category": "string (optional)",
  "created_at": "string (optional)"
}
```

#### Cancel Task
```
POST /api/v1/tasks/:task_id/cancel
```

Cancel a running task.

**Response:**
```json
{
  "success": true
}
```

### Messages

#### List Messages
```
GET /api/v1/messages?limit=100&offset=0&channel=slack
```

List recent messages. Optional query params: `limit`, `offset`, `channel`.

**Response:**
```json
[
  {
    "id": "string",
    "task_id": "string | null",
    "channel": "string",
    "thread_id": "string | null",
    "author": "string",
    "content": "string",
    "role": "string",
    "created_at": "string"
  }
]
```

#### Get Task Messages
```
GET /api/v1/messages/task/:task_id
```

Get all messages for a specific task.

**Response:**
```json
[
  {
    "id": "string",
    "task_id": "string",
    "channel": "string",
    "thread_id": "string | null",
    "author": "string",
    "content": "string",
    "role": "string",
    "created_at": "string"
  }
]
```

### Skills

#### List Skills
```
GET /api/v1/skills
```

List all registered skills.

**Response:**
```json
[
  {
    "name": "string",
    "version": "string",
    "tier": "t0|t1|t2|t3",
    "description": "string"
  }
]
```

#### Get Skill
```
GET /api/v1/skills/:name
```

Get skill details.

**Response:**
```json
{
  "name": "string",
  "version": "string",
  "tier": "t0|t1|t2|t3",
  "description": "string",
  "input_schema": {},
  "output_schema": {}
}
```

#### Register Skill
```
POST /api/v1/skills
```

Register a new skill.

**Request Body:**
```json
{
  "name": "string",
  "version": "string",
  "tier": "t0|t1|t2|t3",
  "description": "string",
  "command": "string",
  "input_schema": {},
  "output_schema": {}
}
```

#### Delete Skill
```
DELETE /api/v1/skills/:name
```

Delete a skill.

#### Execute Skill
```
POST /api/v1/skills/execute
```

Execute a skill directly.

**Request Body:**
```json
{
  "skill_name": "string",
  "input": {}
}
```

#### Update Skill Health
```
PUT /api/v1/skills/:name/health
```

Update skill health status.

**Request Body:**
```json
{
  "healthy": true
}
```

### Deep Tasks

#### Execute Deep Task
```
POST /api/v1/deep
```

Execute a task using the LLM agent.

**Request Body:**
```json
{
  "content": "string (required)",
  "max_steps": "number (optional)",
  "budget_usd": "number (optional)",
  "time_limit_secs": "number (optional)"
}
```

**Response:**
```json
{
  "task_id": "string",
  "status": "running"
}
```

### Metrics

#### Get Metrics
```
GET /api/v1/metrics
```

Get router metrics.

**Response:**
```json
{
  "tasks": {
    "total": 10
  },
  "by_tier": {
    "instant": 5,
    "deep": 5
  },
  "by_status": {
    "completed": 8,
    "failed": 2
  },
  "total_cost_usd": 2.50,
  "tasks_completed": 8,
  "tasks_failed": 2
}
```

### VM Pool

#### Get VM Stats
```
GET /api/v1/vm/stats
```

Get VM pool statistics.

**Response:**
```json
{
  "enabled": true,
  "backend": "docker",
  "total": 3,
  "ready": 2,
  "busy": 1,
  "starting": 0,
  "stopped": 0,
  "available": 2
}
```

### Health

#### Health Check
```
GET /health
```

**Response:**
```json
{
  "status": "ok"
}
```

#### Root
```
GET /
```

**Response:**
```
APEX Router v0.1.0 - See /api/v1/tasks for task endpoints
```

### Configuration

#### Get All Config
```
GET /api/v1/config
```

**Response:**
```json
{
  "config": {
    "APEX_PORT": "3000",
    "APEX_HOST": "0.0.0.0",
    "APEX_USE_LLM": "0",
    "LLAMA_SERVER_URL": "http://localhost:8080",
    ...
  },
  "source": "environment"
}
```

#### Get Config Summary
```
GET /api/v1/config/summary
```

**Response:**
```json
{
  "server": { "host": "0.0.0.0", "port": 3000 },
  "auth_enabled": true,
  "database_type": "sqlite",
  "nats_enabled": false,
  "use_llm": false,
  "execution_backend": "none",
  "heartbeat_enabled": false,
  "config_source": "environment",
  "validation_errors": []
}
```

## Task Tiers

| Tier | Description | Use Case |
|------|-------------|----------|
| `instant` | Immediate response | Greetings, simple queries |
| `shallow` | Skill execution | File operations, shell commands |
| `deep` | LLM agent | Complex reasoning, multi-step tasks |

## Cost Estimation

Costs are estimated based on:
- LLM tokens used (for deep tasks)
- Execution time (for skill tasks)

Actual costs are calculated after task completion and stored in the database.

## Error Responses

All endpoints may return:

```json
{
  "error": "string"
}
```

Common HTTP status codes:
- `200` - Success
- `400` - Bad Request
- `404` - Not Found
- `500` - Internal Server Error

---

## Gateway Adapters

The APEX Gateway (`gateway/`) provides adapters for external integrations.

### REST API Adapter

Runs on port 3001, proxies requests to the router on port 3000.

```
http://localhost:3001
```

**Endpoints:**
- `GET /health` - Health check
- `POST /api/tasks` - Create task
- `GET /api/tasks` - List tasks
- `GET /api/tasks/:task_id` - Get task
- `GET /api/metrics` - Get metrics
- `GET /api/skills` - List skills
- `GET /api/skills/:name` - Get skill
- `POST /api/skills/execute` - Execute skill

### Slack Adapter

Location: `gateway/src/adapters/slack/`

Requires Slack Bot Token and Signing Secret.

### Discord Adapter

Location: `gateway/src/adapters/discord/`

Requires Discord Bot Token and Gateway Intents.

### Telegram Adapter

Location: `gateway/src/adapters/telegram/`

Requires Telegram Bot Token from BotFather.

---

## v0.2.0 New Endpoints

### VM Pool Stats

```
GET /api/v1/vm/stats
```

Get VM pool statistics.

**Response:**
```json
{
  "total": 5,
  "ready": 3,
  "busy": 1,
  "starting": 1,
  "stopped": 0,
  "available": 3,
  "backend": "Firecracker",
  "enabled": true
}
```

### Configuration

APEX v0.2.0 supports YAML configuration files. Configuration can also be provided via environment variables.

**Environment Variables:**
| Variable | Default | Description |
|----------|---------|-------------|
| `APEX_PORT` | 3000 | Router port |
| `APEX_SHARED_SECRET` | dev-secret | HMAC signing secret |
| `APEX_AUTH_DISABLED` | false | Disable authentication |
| `APEX_NATS_ENABLED` | false | Enable NATS distributed mode |
| `APEX_NATS_URL` | 127.0.0.1:4222 | NATS server URL |
| `APEX_DATABASE_URL` | sqlite:apex.db | Database connection |
| `APEX_USE_FIRECRACKER` | false | Enable |
| `AP Firecracker VMsEX_USE_GVISOR` | false | Enable gVisor |
| `APEX_USE_DOCKER` | false | Enable Docker execution |
| `APEX_VM_VCPU` | 2 | CPUs per VM |
| `APEX_VM_MEMORY_MIB` | 2048 | Memory per VM in MiB |
| `LLAMA_SERVER_URL` | localhost:8080 | LLM endpoint |

**YAML Configuration:**
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

agent:
  max_iterations: 50
  max_budget_cents: 500
  context_window_tokens: 4096
  model: "qwen3-4b"

execution:
  isolation: "firecracker"  # firecracker | gvisor | docker
  firecracker:
    vcpus: 2
    memory_mib: 2048
    timeout_secs: 60
```

---

## Authentication

All API requests (except `/health` and `/`) require HMAC-SHA256 authentication.

### Headers Required:
- `X-APEX-Signature`: HMAC-SHA256 signature
- `X-APEX-Timestamp`: Unix timestamp (within 5 minutes)

### Signature Calculation:
```
signature = HMAC-SHA256(timestamp + method + path + body, shared_secret)
```

Example:
```javascript
const timestamp = Math.floor(Date.now() / 1000);
const message = timestamp + 'POST' + '/api/v1/tasks' + JSON.stringify(body);
const signature = crypto.createHmac('sha256', secret).update(message).digest('hex');
```

---

## SOUL Identity

### Get SOUL Identity
```
GET /api/v1/soul
```

Returns the agent's SOUL.md identity.

**Response:**
```json
{
  "success": true,
  "identity": {
    "name": "APEX",
    "version": "1.0",
    "created": "2026-03-01T00:00:00Z",
    "wake_count": 42,
    "purpose": "Your AI assistant",
    "values": [...],
    "capabilities": [...],
    "autonomy_config": {...},
    "memory_strategy": {...},
    "relationships": [...],
    "affiliations": [...],
    "current_goals": [...],
    "reflections": [...],
    "constitution": {...}
  }
}
```

### Update SOUL Identity
```
PUT /api/v1/soul
```

**Request Body:**
```json
{
  "content": "# SOUL.md content..."
}
```

**Response:**
```json
{
  "success": true,
  "message": "SOUL.md updated successfully"
}
```

### Get SOUL Fragments
```
GET /api/v1/soul/fragments
```

Returns modular identity fragments (values.md, skills.md, relationships.md, goals.md).

---

## Heartbeat / Autonomy

### Get Heartbeat Config
```
GET /api/v1/heartbeat/config
```

**Response:**
```json
{
  "enabled": true,
  "interval_minutes": 60,
  "jitter_percent": 10,
  "cooldown_seconds": 300,
  "max_actions_per_wake": 3,
  "require_approval_t1_plus": true
}
```

### Update Heartbeat Config
```
POST /api/v1/heartbeat/config
```

**Request Body:**
```json
{
  "enabled": true,
  "interval_minutes": 60,
  "jitter_percent": 10,
  "cooldown_seconds": 300,
  "max_actions_per_wake": 3,
  "require_approval_t1_plus": true
}
```

### Get Heartbeat Stats
```
GET /api/v1/heartbeat/stats
```

**Response:**
```json
{
  "wake_count": 42,
  "last_wake": "2026-03-04T10:30:00Z",
  "actions_performed": 126,
  "autonomous_actions": 15
}
```

### Trigger Manual Wake
```
POST /api/v1/heartbeat/trigger
```

### Toggle Heartbeat
```
POST /api/v1/heartbeat/toggle
```

**Request Body:**
```json
{
  "enabled": false
}
```

---

## Narrative Memory

### Get Memory Stats
```
GET /api/v1/memory/stats
```

**Response:**
```json
{
  "journal_entries": 150,
  "entities": 25,
  "knowledge_items": 80,
  "reflections": 12,
  "total_files": 267
}
```

### Get Journal Entries
```
GET /api/v1/memory/journal
```

### Get Entities
```
GET /api/v1/memory/entities
```

### Get Knowledge
```
GET /api/v1/memory/knowledge
```

### Get Reflections
```
GET /api/v1/memory/reflections
```

### Add Reflection
```
POST /api/v1/memory/reflections
```

**Request Body:**
```json
{
  "title": "Lesson learned",
  "content": "What I learned from this task..."
}
```

---

## System

### Health & Monitoring

#### Get System Health
```
GET /api/v1/system/health
```

**Response:**
```json
{
  "uptime_secs": 3600,
  "requests_total": 150,
  "errors_total": 2,
  "error_rate": 1.33,
  "avg_response_time_ms": 25.5,
  "requests_by_endpoint": {
    "/api/v1/tasks": 50,
    "/api/v1/skills": 30
  },
  "last_error": null
}
```

#### Get Cache Statistics
```
GET /api/v1/system/cache
```

**Response:**
```json
{
  "total_entries": 10,
  "expired_entries": 2,
  "active_entries": 8
}
```

#### Clear Cache
```
DELETE /api/v1/system/cache
```

**Response:**
```json
{
  "success": true,
  "message": "Cache cleared"
}
```

#### Configure Cache
```
POST /api/v1/system/cache/config
```

**Request Body:**
```json
{
  "default_ttl_secs": 120,
  "endpoint_ttl": {
    "/api/v1/skills": 300,
    "/api/v1/tasks": 60
  }
}
```

#### Get Rate Limit Stats
```
GET /api/v1/system/ratelimit
```

**Response:**
```json
{
  "active_keys": 5,
  "total_requests": 150,
  "requests_per_minute": 60,
  "burst_size": 10
}
```

---

## Moltbook Social

### Get Moltbook Status
```
GET /api/v1/moltbook/status
```

### Get Agent Directory
```
GET /api/v1/moltbook/agents
```

### Connect to Moltbook
```
POST /api/v1/moltbook/connect
```

### Disconnect from Moltbook
```
POST /api/v1/moltbook/disconnect
```

### Get Social Profile
```
GET /api/v1/social/profile
```

### Create Social Post
```
POST /api/v1/social/post
```

**Request Body:**
```json
{
  "content": "Hello from APEX!"
}
```

### Get Notifications
```
GET /api/v1/social/notifications
```

### Search Agents
```
GET /api/v1/social/agents/search?q=query
```

### Get Agent Directory
```
GET /api/v1/social/agents/directory
```

### Assess Trust
```
GET /api/v1/social/trust?agent_id=id
```

---

## Governance

### Get Governance Policy
```
GET /api/v1/governance/policy
```

**Response:**
```json
{
  "policy": {
    "constitution_hash": "...",
    "immutable_values": [...],
    "emergency_protocols": [...]
  },
  "oracle_mode": false
}
```

### Check Action Allowed
```
POST /api/v1/governance/check
```

**Request Body:**
```json
{
  "action_type": "values",
  "approval_tier": "T1"
}
```

### Get Immutable Values
```
GET /api/v1/governance/immutable
```

### Get Emergency Protocols
```
GET /api/v1/governance/emergency
```

### Toggle Oracle Mode
```
POST /api/v1/governance/oracle
```

**Request Body:**
```json
{
  "enable": true
}
```
