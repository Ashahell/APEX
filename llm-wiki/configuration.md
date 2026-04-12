# Configuration

## Overview

APEX uses a unified configuration system via `AppConfig::global()` with environment variable overrides.

## Configuration Sources (Priority Order)

1. Environment variables
2. Config file (apex.yaml)
3. Default values

## Core Configuration

### Server
| Variable | Default | Description |
|----------|---------|-------------|
| APEX_PORT | 3000 | Router HTTP port |
| APEX_HOST | 0.0.0.0 | Router host |

### Authentication
| Variable | Default | Description |
|----------|---------|-------------|
| APEX_SHARED_SECRET | dev-secret-change-in-production | HMAC signing secret |
| APEX_AUTH_DISABLED | false | Disable auth (dev only) |

### LLM/Agent
| Variable | Default | Description |
|----------|---------|-------------|
| APEX_USE_LLM | false | Enable LLM |
| LLAMA_SERVER_URL | http://localhost:8080 | llama-server URL |
| LLAMA_MODEL | qwen3-4b | Model name |

### Database
| Variable | Default | Description |
|----------|---------|-------------|
| APEX_DATABASE_URL | sqlite:apex.db | Database connection |
| DATABASE_URL | sqlite:apex.db | Fallback DB URL |
| APEX_DB_MAX_CONNECTIONS | 10 | Max pool connections (increased from 5) |
| APEX_DB_MIN_CONNECTIONS | 2 | Min pool connections (increased from 1) |

### Execution
| Variable | Default | Description |
|----------|---------|-------------|
| APEX_EXECUTION_ISOLATION | docker | docker/firecracker/gvisor/mock |
| APEX_SANDBOX_MEMORY_MB | 512 | Memory limit |
| APEX_SANDBOX_TIMEOUT_SECS | 30 | Timeout |

### NATS
| Variable | Default | Description |
|----------|---------|-------------|
| APEX_NATS_ENABLED | false | Enable NATS |
| APEX_NATS_URL | 127.0.0.1:4222 | Server URL |
| APEX_NATS_SUBJECT_PREFIX | apex | Subject prefix |

### Memory
| Variable | Default | Description |
|----------|---------|-------------|
| APEX_MEMORY_EMBEDDING_URL | http://localhost:8081 | Embedding server |
| APEX_MEMORY_RRF_K | 60 | RRF constant |
| APEX_MEMORY_MAX_RESULTS | 8 | Max search results |
| APEX_INDEXER_BATCH_SIZE | 16 | Indexing batch size |
| APEX_INDEXER_EMBED_RATE_MS | 50 | Embedding rate limit (ms) |
| APEX_INDEXER_MAX_CONCURRENT | 4 | Max concurrent embeddings |
| APEX_INDEXER_CACHE_ENABLED | true | Enable recent file caching |

### Skill Pool
| Variable | Default | Description |
|----------|---------|-------------|
| APEX_SKILL_POOL_ENABLED | true | Enable skill pool |
| APEX_SKILL_POOL_SIZE | 4 | Pool size |

## API Access

### Configuration Endpoints
- `GET /api/v1/config` - All variables
- `GET /api/v1/config/summary` - Validated summary

### UI Access
- Settings → Config tab shows all runtime configuration

## Validation

Startup validates:
- Required variables present
- Database connectivity
- LLM server reachable (if enabled)
- Port availability