# APEX Migration Plan and Pilot Rollout

## Version: 1.0
## Date: 2026-03-31
## Status: Planning

---

## 1. Executive Summary

This document outlines the migration plan for enabling all parity features in APEX. The migration follows a phased, incremental approach with feature toggles, pilot testing, and rollback capabilities.

**Target**: Enable all 8 phases of parity features with zero downtime and safe rollback paths.

---

## 2. Current State

| Component | Status | Notes |
|-----------|--------|-------|
| Streaming | ✅ Complete | Phase 1 - Rich event types, performance metrics |
| Telemetry | ✅ Complete | Phase 2 - Per-endpoint latency, error rates |
| Security | ✅ Complete | Phase 3 - 40 security tests, clippy clean |
| MCP/Tools | ✅ Complete | Phase 4 - Tool discovery, marketplace |
| Memory | ✅ Complete | Phase 5 - TTL, consolidation, search |
| UI/Theming | ✅ Complete | Phase 6 - 4 themes, accessibility |
| Ecosystem | ✅ Complete | Phase 7 - Plugin signing, governance |
| Governance | ✅ Complete | Phase 8 - Crosswalks, charter, cadence |

**Overall Parity Score**: 9.45/10

---

## 3. Migration Strategy

### 3.1 Feature Toggle System

All parity features are controlled via feature toggles in `AppConfig`:

```rust
pub struct FeatureToggles {
    pub streaming_enabled: bool,
    pub telemetry_enabled: bool,
    pub security_enabled: bool,
    pub mcp_enabled: bool,
    pub memory_enabled: bool,
    pub ui_themes_enabled: bool,
    pub ecosystem_enabled: bool,
    pub governance_enabled: bool,
}
```

**Default State**: All features enabled (parity complete)
**Rollback State**: Individual features can be disabled

### 3.2 Migration Phases

| Phase | Features | Duration | Risk |
|-------|----------|----------|------|
| **Pilot 1** | Streaming, Telemetry | 1 week | Low |
| **Pilot 2** | Security, MCP | 1 week | Low |
| **Pilot 3** | Memory, UI | 1 week | Medium |
| **Pilot 4** | Ecosystem, Governance | 1 week | Low |
| **Full Rollout** | All features | 1 week | Low |

---

## 4. Prerequisites

### 4.1 System Requirements
- Rust 1.93+
- Node.js 20+
- SQLite database
- (Optional) Embedding server for memory search

### 4.2 Pre-Migration Checks
- [ ] All tests pass (`cargo test` - 583+ tests)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] UI builds successfully (`npm run build`)
- [ ] Database migrations applied
- [ ] Backup created

### 4.3 Backup Procedure
```bash
# Backup database
cp apex.db apex.db.backup.$(date +%Y%m%d)

# Backup configuration
cp ~/.apex/config.toml ~/.apex/config.toml.backup.$(date +%Y%m%d)

# Backup soul identity
cp -r ~/.apex/soul ~/.apex/soul.backup.$(date +%Y%m%d)
```

---

## 5. Pilot Execution

### 5.1 Pilot 1: Streaming + Telemetry

**Enable**:
- Streaming surface (Hands, MCP, Task)
- Per-endpoint telemetry
- Monitoring dashboard

**Monitor**:
- Streaming connection health
- Telemetry metrics accuracy
- UI dashboard performance

**Rollback Trigger**:
- Streaming connection failures > 5%
- Telemetry data loss > 1%
- UI rendering issues

**Rollback Procedure**:
```bash
# Disable streaming
# Set APEX_STREAMING_ENABLED=false

# Disable telemetry middleware
# Remove TelemetryLayer from router
```

### 5.2 Pilot 2: Security + MCP

**Enable**:
- Injection detection
- Replay protection
- MCP tool discovery
- Marketplace scaffolding

**Monitor**:
- Security test pass rate
- MCP server health
- Plugin signing verification

**Rollback Trigger**:
- Security false positives > 10%
- MCP connection failures > 5%
- Plugin signing failures

**Rollback Procedure**:
```bash
# Disable injection classifier
# Set APEX_INJECTION_DETECTION=false

# Disable MCP features
# Remove MCP enriched endpoints
```

### 5.3 Pilot 3: Memory + UI

**Enable**:
- Bounded memory with TTL
- Memory consolidation
- 4-theme system
- High-contrast accessibility

**Monitor**:
- Memory search accuracy
- TTL cleanup execution
- Theme switching performance
- Accessibility compliance

**Rollback Trigger**:
- Memory search failures > 5%
- TTL cleanup errors
- Theme application failures
- Accessibility violations

**Rollback Procedure**:
```bash
# Reset to default theme
# localStorage.setItem('apex-theme-id', 'modern-2026')

# Disable TTL
# Set APEX_MEMORY_TTL_ENABLED=false
```

### 5.4 Pilot 4: Ecosystem + Governance

**Enable**:
- Plugin signing
- Skills Hub marketplace
- Governance charter
- Crosswalk documentation

**Monitor**:
- Plugin verification success
- Hub connectivity
- Governance policy compliance

**Rollback Trigger**:
- Plugin signing failures > 1%
- Hub connection failures
- Governance policy violations

**Rollback Procedure**:
```bash
# Disable plugin signing
# Set APEX_PLUGIN_SIGNING_ENABLED=false

# Disable hub
# Set APEX_HUB_ENABLED=false
```

---

## 6. Full Rollout

### 6.1 Enablement Sequence
1. Pilot 1 features (Streaming, Telemetry)
2. Pilot 2 features (Security, MCP)
3. Pilot 3 features (Memory, UI)
4. Pilot 4 features (Ecosystem, Governance)
5. Verify all features operational
6. Remove feature toggles (optional)

### 6.2 Verification Checklist
- [ ] All streaming endpoints functional
- [ ] Telemetry metrics accurate
- [ ] Security tests passing
- [ ] MCP tools discoverable
- [ ] Memory search working
- [ ] All 4 themes apply correctly
- [ ] Plugin signing operational
- [ ] Governance policies enforced
- [ ] All 583+ tests passing
- [ ] Clippy clean
- [ ] UI builds successfully

### 6.3 Post-Migration
- Update documentation
- Notify users of new features
- Monitor for 30 days
- Schedule first parity review

---

## 7. Rollback Procedures

### 7.1 Emergency Rollback
```bash
# Stop all services
./apex.bat stop

# Restore database backup
cp apex.db.backup.* apex.db

# Restore configuration backup
cp ~/.apex/config.toml.backup.* ~/.apex/config.toml

# Restart with default configuration
./apex.bat start
```

### 7.2 Feature-Specific Rollback
| Feature | Rollback Command |
|---------|-----------------|
| Streaming | `APEX_STREAMING_ENABLED=false` |
| Telemetry | Remove `TelemetryLayer` from router |
| Security | `APEX_INJECTION_DETECTION=false` |
| MCP | Remove enriched MCP endpoints |
| Memory | `APEX_MEMORY_TTL_ENABLED=false` |
| UI Themes | Reset to `modern-2026` theme |
| Ecosystem | `APEX_PLUGIN_SIGNING_ENABLED=false` |
| Governance | Revert charter amendments |

---

## 8. Success Criteria

### 8.1 Technical
- All 583+ tests passing
- Zero clippy warnings
- UI builds successfully
- All endpoints respond < 500ms p95
- Error rate < 0.1%

### 8.2 Parity
- Overall parity score ≥ 9.0/10
- All crosswalks complete
- All runbooks created
- Governance charter active

### 8.3 Operational
- Zero downtime during migration
- Rollback tested and verified
- Documentation complete
- User notification sent

---

## 9. Contacts

- **Migration Lead**: @engineering-ops
- **Security Review**: @security-team
- **UI/UX Review**: @frontend-team
- **Governance Review**: @governance-board

---

*This is a living document. Update as migration progresses.*
