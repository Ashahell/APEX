# API Layer

## Overview

APEX exposes a comprehensive REST API with modular architecture and HMAC authentication.

## Structure

### API Modules (in core/router/src/api/)
| Module | Endpoints | Purpose |
|--------|-----------|---------|
| tasks.rs | /api/v1/tasks | Task CRUD |
| skills.rs | /api/v1/skills | Skill management |
| memory.rs | /api/v1/memory | Memory operations |
| system.rs | /api/v1/system | Health, metrics |
| bounded_memory.rs | /api/v1/memory/bounded | Hermes-style |
| skill_manager_api.rs | /api/v1/skills/auto-created | Auto-skills |
| session_search_api.rs | /api/v1/search/sessions | FTS5 search |
| user_profile_api.rs | /api/v1/user/profile | User settings |

### Modular Architecture
- 20+ API modules
- Router composition in api/mod.rs
- Shared error handling
- Consistent response format

## Authentication

### HMAC Request Signing
All API requests require:
- `X-APEX-Signature`: HMAC-SHA256 signature
- `X-APEX-Timestamp`: Unix timestamp (5min window)
- Signature = HMAC-SHA256(timestamp + method + path + body)

### Dev Mode
- Set `APEX_AUTH_DISABLED=1` to bypass

## Configuration

### Unified Config
All settings via `AppConfig::global()`:
- Database URL
- LLM settings
- Memory configuration
- Security settings

### API Endpoints
- GET `/api/v1/config` - All variables
- GET `/api/v1/config/summary` - Validated summary

## Response Format

### Success
```json
{
  "data": {...},
  "meta": {"timestamp": "...", "version": "1.0"}
}
```

### Error
```json
{
  "error": {"code": "...", "message": "..."}
}
```