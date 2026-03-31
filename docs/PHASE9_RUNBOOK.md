# Phase 9 Runbook: Migration Plan and Pilot Rollout

## Overview
- Phase 9 defines safe incremental migration; pilot with parity-enabled features.
- This runbook provides operational steps for incident response, debugging, and escalation.

## Phase 9 Features

| Feature | Description |
|---------|-------------|
| Migration Plan | Step-by-step migration with 4 pilot phases |
| Feature Toggles | Per-feature enablement control |
| Rollback Paths | Tested rollback procedures for each feature |
| Pilot Runbooks | Enablement, monitoring, rollback procedures |

---

## Incident Response Procedures

### 1. Migration Failure

**Symptoms:**
- Feature enablement fails
- Database migration errors
- Configuration conflicts

**Immediate Steps:**
```bash
# Stop all services
./apex.bat stop

# Check migration status
# Review: docs/MIGRATION_PLAN.md

# Restore from backup
cp apex.db.backup.* apex.db
```

**Debug Commands:**
```bash
# Check feature toggle state
# Review: core/router/src/unified_config.rs

# Check database migrations
ls core/memory/migrations/
```

**Common Causes:**
- Incomplete backup
- Configuration syntax errors
- Database schema mismatch

**Rollback:** Execute emergency rollback procedure from MIGRATION_PLAN.md §7.1

---

### 2. Pilot Feature Issues

**Symptoms:**
- Enabled feature causes errors
- Performance degradation
- User-facing issues

**Immediate Steps:**
```bash
# Disable specific feature
# Example: APEX_STREAMING_ENABLED=false

# Restart services
./apex.bat restart

# Verify feature disabled
curl -s http://localhost:3000/api/v1/stream/stats
```

**Debug Commands:**
```bash
# Check feature-specific logs
# Review: logs/apex-router.log

# Check feature health endpoint
curl -s http://localhost:3000/api/v1/system/health
```

**Common Causes:**
- Feature configuration error
- Resource exhaustion
- Dependency failure

**Rollback:** Disable feature, investigate root cause, re-enable when fixed.

---

### 3. Rollback Execution

**Symptoms:**
- Rollback triggered by monitoring
- Manual rollback requested
- Emergency rollback needed

**Immediate Steps:**
```bash
# Execute emergency rollback
./apex.bat stop
cp apex.db.backup.* apex.db
cp ~/.apex/config.toml.backup.* ~/.apex/config.toml
./apex.bat start
```

**Debug Commands:**
```bash
# Verify rollback success
curl -s http://localhost:3000/api/v1/system/health
curl -s http://localhost:3000/api/v1/metrics
```

**Common Causes:**
- Critical bug discovered
- Performance regression
- Security vulnerability

**Rollback:** Rollback is the recovery procedure. Monitor for stability.

---

## Debug Commands Quick Reference

| Command | Purpose |
|---------|---------|
| `./apex.bat stop` | Stop all services |
| `./apex.bat start` | Start all services |
| `./apex.bat restart` | Restart all services |
| `curl -s http://localhost:3000/api/v1/system/health` | Check system health |
| `cat docs/MIGRATION_PLAN.md` | Review migration plan |

---

## Verification Checklist

After any incident, verify:

- [ ] All services running
- [ ] System health endpoint returns OK
- [ ] Enabled features operational
- [ ] Disabled features properly disabled
- [ ] Database integrity maintained
- [ ] Configuration valid

---

## Contacts

- On-call: @engineering-ops
- Migration Lead: @engineering-ops
- Security: @security-team
- Governance: @governance-board

---

## Last Updated

- Phase 9: Migration Plan and Pilot Rollout
- Version: 1.0
- Date: 2026-03-31
