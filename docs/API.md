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
