# Phase 3 Rollback and Recovery Plan

## Overview
This document outlines the rollback and recovery procedures for the Phase 3 production hardening changes.

## Rollback Triggers
Consider rollback when:
1. Health checks consistently fail after deployment
2. Services crash due to resource constraints (OOM, CPU exhaustion)
3. Secret mounting fails (missing secret file, permission errors)
4. Network connectivity issues between services
5. Performance degradation beyond acceptable thresholds

## Rollback Options

### Option 1: Full Rollback to Previous Image
If the issue is with the hardened container images:
```bash
# Stop current stack
docker-compose -f docker-compose.prod.yml down

# If you have a previous image version, update docker-compose.prod.yml
# Change image tags to previous version (e.g., apex-router-prod:previous)

# Redeploy
docker-compose -f docker-compose.prod.yml up -d
```

### Option 2: Disable Hardening Features
If specific hardening features cause issues:
```bash
# Edit docker-compose.prod.yml and comment out:
# - read_only: true (set to false temporarily)
# - cap_drop: ALL (comment out)
# - tmpfs mounts (comment out)
# - mem_limit/cpus (remove or increase)

# Restart affected services
docker-compose -f docker-compose.prod.yml restart <service>
```

### Option 3: Adjust Resource Limits
If OOM or CPU issues:
```bash
# Increase memory limits in docker-compose.prod.yml
# router: mem_limit: 512m -> 1g
# ui: mem_limit: 256m -> 512m  
# db: mem_limit: 1g -> 2g

# Restart services
docker-compose -f docker-compose.prod.yml up -d
```

### Option 4: Disable Secrets (Emergency)
If secret mounting fails:
```bash
# Temporarily use env var instead of secret
# In docker-compose.prod.yml, comment out secrets: and use:
environment:
  - POSTGRES_PASSWORD=your_password_here

# Restart db
docker-compose -f docker-compose.prod.yml restart db
```

## Recovery Steps

### 1. Verify Service Health
```bash
# Check all services
docker-compose -f docker-compose.prod.yml ps

# Check health status
curl http://localhost:3000/health
curl http://localhost:3001/health

# Check database
docker-compose -f docker-compose.prod.yml exec db pg_isready -U apex
```

### 2. Check Logs
```bash
# All services
docker-compose -f docker-compose.prod.yml logs

# Specific service
docker-compose -f docker-compose.prod.yml logs router
docker-compose -f docker-compose.prod.yml logs ui
docker-compose -f docker-compose.prod.yml logs db
```

### 3. Resource Diagnostics
```bash
# Container stats
docker stats

# Resource limits
docker inspect apex_router_prod | grep -A 10 Memory
docker inspect apex_ui_prod | grep -A 10 Memory
```

### 4. Network Diagnostics
```bash
# Check connectivity between services
docker-compose -f docker-compose.prod.yml exec router curl -f http://db:5432
docker-compose -f docker-compose.prod.yml exec ui curl -f http://router:3000/health
```

## Emergency Contacts
- DevOps Lead: [INSERT NAME]
- Security Engineer: [INSERT NAME]
- On-Call: [INSERT CONTACT]

## Post-Incident
After rollback:
1. Document incident in incident tracker
2. Analyze root cause
3. Update this runbook with lessons learned
4. Plan fix before next deployment attempt