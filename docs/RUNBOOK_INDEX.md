# APEX Runbook Index

## Version: 1.0
## Date: 2026-03-31

---

## Quick Reference

| Runbook | Location | Primary Contact |
|---------|----------|----------------|
| Phase 0: Kickoff | [PHASE0_RUNBOOK.md](PHASE0_RUNBOOK.md) | @engineering-ops |
| Phase 1: Streaming | [PHASE1_RUNBOOK.md](PHASE1_RUNBOOK.md) | @backend-team |
| Phase 2: Telemetry | [PHASE2_RUNBOOK.md](PHASE2_RUNBOOK.md) | @backend-team |
| Phase 3: Security | [PHASE3_RUNBOOK.md](PHASE3_RUNBOOK.md) | @security-team |
| Phase 4: MCP/Tools | [PHASE4_RUNBOOK.md](PHASE4_RUNBOOK.md) | @backend-team |
| Phase 5: Memory | [PHASE5_RUNBOOK.md](PHASE5_RUNBOOK.md) | @backend-team |
| Phase 6: UI/Theming | [PHASE6_RUNBOOK.md](PHASE6_RUNBOOK.md) | @frontend-team |
| Phase 7: Ecosystem | [PHASE7_RUNBOOK.md](PHASE7_RUNBOOK.md) | @backend-team |
| Phase 8: Governance | [PHASE8_RUNBOOK.md](PHASE8_RUNBOOK.md) | @governance-board |
| Phase 9: Migration | [PHASE9_RUNBOOK.md](PHASE9_RUNBOOK.md) | @engineering-ops |

---

## Common Debug Commands

### System Health
```bash
# Check system health
curl -s http://localhost:3000/api/v1/system/health | jq .

# Check metrics
curl -s http://localhost:3000/api/v1/metrics | jq .

# Check streaming stats
curl -s http://localhost:3000/api/v1/stream/stats | jq .
```

### Service Management
```bash
# Start all services
./apex.bat start

# Stop all services
./apex.bat stop

# Restart all services
./apex.bat restart

# Check service status
./apex.bat status
```

### Testing
```bash
# Run all Rust tests
cd core && cargo test

# Run specific phase tests
cd core && cargo test --test streaming_integration
cd core && cargo test --test telemetry_integration
cd core && cargo test --test security_integration
cd core && cargo test --test memory_integration_phase5

# Run UI tests
cd ui && npm test

# Run clippy
cd core && cargo clippy -- -D warnings

# Build UI
cd ui && npm run build
```

### Database
```bash
# Backup database
cp apex.db apex.db.backup.$(date +%Y%m%d)

# Restore database
cp apex.db.backup.* apex.db
```

---

## Escalation Matrix

| Severity | Response Time | Contact | Examples |
|----------|--------------|---------|----------|
| Critical | < 1 hour | @engineering-ops | System down, data loss |
| High | < 4 hours | @backend-team | Feature broken, security issue |
| Medium | < 24 hours | @frontend-team | UI issue, performance degradation |
| Low | < 1 week | @tech-writers | Documentation update needed |

---

## Incident Response Flow

1. **Identify** the issue using debug commands above
2. **Consult** the relevant phase runbook
3. **Apply** immediate fix or rollback
4. **Verify** system health
5. **Document** the incident
6. **Escalate** if needed using matrix above

---

## Artifact Inventory

### Documentation
| Document | Location | Status |
|----------|----------|--------|
| AGENTS.md | [AGENTS.md](../AGENTS.md) | ✅ Complete |
| CODEBASE_AUDIT_REPORT.md | [CODEBASE_AUDIT_REPORT.md](CODEBASE_AUDIT_REPORT.md) | ✅ Complete |
| MIGRATION_PLAN.md | [MIGRATION_PLAN.md](MIGRATION_PLAN.md) | ✅ Complete |
| GOVERNANCE_CHARTER.md | [GOVERNANCE_CHARTER.md](GOVERNANCE_CHARTER.md) | ✅ Complete |
| GOVERNANCE_CADENCE.md | [GOVERNANCE_CADENCE.md](GOVERNANCE_CADENCE.md) | ✅ Complete |
| PLUGIN_GOVERNANCE.md | [PLUGIN_GOVERNANCE.md](PLUGIN_GOVERNANCE.md) | ✅ Complete |
| parity-scorecard.md | [parity-scorecard.md](parity-scorecard.md) | ✅ Complete |

### Crosswalks
| Document | Location | Score |
|----------|----------|-------|
| OpenClaw | [crosswalk_openclaw_apex.md](crosswalk_openclaw_apex.md) | 9.2/10 |
| AgentZero | [crosswalk_agentzero_apex.md](crosswalk_agentzero_apex.md) | 9.4/10 |
| Hermes | [crosswalk_hermes_apex.md](crosswalk_hermes_apex.md) | 9.8/10 |
| OpenFang | [crosswalk_openfang_apex.md](crosswalk_openfang_apex.md) | 9.4/10 |

### Gating Documents
| Phase | Document | Status |
|-------|----------|--------|
| Phase 0 | [phase0_gating.md](phase0_gating.md) | ✅ Complete |
| Phase 1 | [phase1_gating.md](phase1_gating.md) | ✅ Complete |
| Phase 2 | [phase2_gating.md](phase2_gating.md) | ✅ Complete |
| Phase 3 | [phase3_gating.md](phase3_gating.md) | ✅ Complete |
| Phase 4 | [phase4_gating.md](phase4_gating.md) | ✅ Complete |
| Phase 5 | [phase5_gating.md](phase5_gating.md) | ✅ Complete |
| Phase 6 | [phase6_gating.md](phase6_gating.md) | ✅ Complete |
| Phase 7 | [phase7_gating.md](phase7_gating.md) | ✅ Complete |
| Phase 8 | [phase8_gating.md](phase8_gating.md) | ✅ Complete |
| Phase 9 | [phase9_gating.md](phase9_gating.md) | ✅ Complete |

---

*This index is maintained as part of the parity rollout. Update when new runbooks are added.*
