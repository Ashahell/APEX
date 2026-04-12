# Deployment

## Overview

APEX supports multiple deployment modes: development, production, and distributed.

## Deployment Modes

### Development
- Local services via `apex.bat`
- LLM disabled by default
- SQLite database
- No isolation (mock execution)

### Production
- Docker/Docker Compose
- PostgreSQL database
- Non-root containers
- TLS termination (optional)

### Distributed (NATS)
- Multi-instance scaling
- NATS message bus
- Connection pooling

## Docker Deployment

### Production Stack
```bash
# Standard (no TLS)
docker-compose -f docker-compose.prod.yml up -d

# With TLS
docker-compose -f docker-compose.prod.tls.yml up -d
```

### Hardening Features
- Non-root users (1000:1000, 999:999)
- Read-only filesystem
- cap_drop: ALL
- no-new-privileges
- Resource limits (mem/cpu)
- Health checks
- JSON logging with rotation

### Services
| Service | Port | Purpose |
|---------|------|---------|
| router | 3000 | API/agent loop |
| ui | 3001 | Web UI |
| db | 5432 | PostgreSQL |
| nginx | 80/443 | TLS termination |

## Deployment Scripts

### Core Scripts
- `scripts/backup_db.sh` - Database backup
- `scripts/restore_db.sh` - Database restore
- `scripts/prod_smoke.sh` - Health validation
- `scripts/generate_certs.sh` - TLS certs (dev)

## Environment Variables

### Required
| Variable | Description |
|----------|-------------|
| APEX_SHARED_SECRET | HMAC signing secret |
| APEX_DATABASE_URL | Database connection |

### Optional
| Variable | Default | Description |
|----------|---------|-------------|
| APEX_PORT | 3000 | Router port |
| APEX_USE_LLM | false | Enable LLM |
| APEX_NATS_ENABLED | false | Enable NATS |

## Kubernetes (Optional)

See docs/KUBERNETES.md for K8s deployment manifest.

## Health Checks

All services expose `/health` endpoint:
- Router: `curl http://localhost:3000/health`
- UI: `curl http://localhost:3001/health`
- DB: `pg_isready -U apex -d apex`