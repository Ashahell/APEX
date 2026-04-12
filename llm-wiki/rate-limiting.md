# Rate Limiting

## Overview

APEX implements per-endpoint rate limiting with progressive throttling and circuit breaker patterns.

## Components

### Rate Limiter
- Per-endpoint limits
- Progressive throttling
- Token bucket algorithm

### Circuit Breaker
- Failure threshold tracking
- Half-open state for recovery
- Configurable reset timeout

### Enhanced Rate Limiter
- Endpoint-specific rules
- User-based limits
- Burst handling

## Configuration

### Environment Variables
- `APEX_RATE_LIMIT_ENABLED` - Enable rate limiting
- `APEX_RATE_LIMIT_REQUESTS` - Requests per window
- `APEX_RATE_LIMIT_WINDOW_SECS` - Window duration

### Per-Endpoint Limits
```yaml
/tasks: 100/minute
/mcp: 50/minute
/llms: 10/minute
```

## API Endpoints
- `/api/v1/system/ratelimit` - Get rate limit stats
- Rate limit headers on responses:
  - `X-RateLimit-Limit`
  - `X-RateLimit-Remaining`
  - `X-RateLimit-Reset`

## Behavior
| Status | Action |
|--------|--------|
| Under limit | Allow request |
| Near limit | Warn in headers |
| At limit | 429 Too Many Requests |
| Circuit open | 503 Service Unavailable |