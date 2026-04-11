# Phase 3 Production Hardening Runbook

## Overview
This runbook documents the production hardening steps implemented in Phase 3 for APEX. It serves as the authoritative reference for deploying and operating the hardened production stack.

## Hardening Summary

### Container Security
| Service | User | Read-Only | Cap Drop | Security Opt |
|---------|------|-----------|----------|--------------|
| router | 1000:1000 | true | ALL | no-new-privileges |
| ui | 1000:1000 | true | ALL | no-new-privileges |
| db | 999:999 | false (data volume) | ALL | no-new-privileges |

### Resource Limits
| Service | Memory | CPU |
|---------|--------|-----|
| router | 512m | 0.5 |
| ui | 256m | 0.25 |
| db | 1g | 1.0 |

### Secrets Management
- Database password stored in Docker secret (`db_password`)
- Secret file: `./secrets/db_password.txt`
- NEVER commit secrets to git; add to `.gitignore`

### TLS/SSL
- TLS termination via nginx (optional, see `docker-compose.prod.tls.yml`)
- Generate self-signed certs: `bash scripts/generate_certs.sh`
- For production, use certificates from trusted CA

## Deployment Commands

### Standard Deployment (No TLS)
```bash
# Validate config
docker-compose -f docker-compose.prod.yml config

# Bring up stack
docker-compose -f docker-compose.prod.yml up -d

# Verify health
docker-compose -f docker-compose.prod.yml ps
docker-compose -f docker-compose.prod.yml logs --tail=50
```

### TLS-Enabled Deployment
```bash
# Generate certificates first (dev/test only)
bash scripts/generate_certs.sh

# Deploy with TLS
docker-compose -f docker-compose.prod.tls.yml up -d
```

### Smoke Tests
```bash
# Run Phase 3 smoke tests
bash scripts/prod_smoke.sh http://localhost

# Test with specific host
bash scripts/prod_smoke.sh http://192.168.1.100
```

## Health Checks
All services have health checks configured:
- router: `curl -f http://localhost/health`
- ui: `curl -f http://localhost/health`
- db: `pg_isready -U apex -d apex`

## Monitoring
Logs are forwarded to JSON file driver with rotation:
- Max size: 10MB per file
- Max files: 3 per container

View logs:
```bash
docker-compose -f docker-compose.prod.yml logs -f router
docker-compose -f docker-compose.prod.yml logs -f ui
docker-compose -f docker-compose.prod.yml logs -f db
```

## Rollback Procedure
See `docs/PHASE3_ROLLBACK.md` for detailed rollback steps.

## Troubleshooting

### Service won't start
1. Check logs: `docker-compose -f docker-compose.prod.yml logs <service>`
2. Verify secrets file exists: `ls -la secrets/db_password.txt`
3. Check resource availability: `docker stats`

### Health check failures
1. Verify network connectivity between containers
2. Check service logs for errors
3. Verify health check endpoint is exposed

### Out of memory
- Review mem_limit settings in docker-compose.prod.yml
- Adjust based on workload

## Maintenance
- Rotate secrets periodically
- Update base images for security patches
- Review and update resource limits as needed