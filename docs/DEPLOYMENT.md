# APEX Deployment Guide

> **Status**: Ready for testing
> **Version**: 1.6.0

---

## Quick Start (Docker)

```bash
# Build the router image
docker build -t apex-router:1.6.0 -f.docker/Dockerfile.router core/

# Run the container
docker run -d \
  --name apex-router \
  -p 3000:3000 \
  -e APEX_SHARED_SECRET=your-secret-here \
  -e APEX_USE_LLM=0 \
  apex-router:1.6.0

# Verify
curl http://localhost:3000/health
```

---

## Production Deployment

### 1. Build the Image

```bash
# Build release binary
cd core && cargo build --release

# Build Docker image
docker build -t apex/apex-router:1.6.0 -f docker/Dockerfile.router .
```

### 2. Run with Docker

```bash
docker run -d \
  --name apex-router \
  -p 3000:3000 \
  -v apex-data:/data \
  -e APEX_SHARED_SECRET="${APEX_SHARED_SECRET}" \
  -e APEX_DATABASE_URL="sqlite:/data/apex.db" \
  apex/apex-router:1.6.0
```

### 3. Run with Docker Compose

```bash
# Production stack
docker compose -f docker/docker-compose.prod.yml up -d

# Scale the router
docker compose -f docker/docker-compose.prod.yml up -d --scale router=2
```

---

## Update Mechanism

### Option 1: Pull New Image

```bash
# Pull latest image
docker pull apex/apex-router:1.6.1

# Recreate container
docker compose -f docker/docker-compose.prod.yml up -d --force-recreate router
```

### Option 2: In-Place Update Script

```bash
# Run the update script
./scripts/update-apex.sh 1.6.1
```

---

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `APEX_SHARED_SECRET` | Yes | - | HMAC signing secret |
| `APEX_PORT` | No | 3000 | Router port |
| `APEX_DATABASE_URL` | No | sqlite:apex.db | Database URL |
| `APEX_USE_LLM` | No | 0 | Enable LLM (1/0) |
| `APEX_AUTH_DISABLED` | No | 0 | Disable auth (1/0) |
| `LLAMA_SERVER_URL` | No | localhost:8080 | LLM endpoint |

---

## Health Checks

```bash
# Check router health
curl http://localhost:3000/health

# Check if router is ready
curl http://localhost:3000/api/v1/system/health
```

---

## Rollback

```bash
# Rollback to previous version
docker compose -f docker/docker-compose.prod.yml up -d --force-recreate router:previous
```

---

## Monitoring

```bash
# View logs
docker logs -f apex-router

# View metrics
curl http://localhost:3000/api/v1/metrics

# Check stats
curl http://localhost:3000/api/v1/system/health
```