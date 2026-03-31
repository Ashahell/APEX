# APEX Production Readiness Plan

> **Version**: 1.0
> **Date**: 2026-03-31
> **Goal**: Transform APEX from pre-alpha (9.45/10 parity) to production-ready (v3.0)
> **Timeline**: 12-16 weeks across 4 phases

---

## Executive Summary

APEX has achieved 9.45/10 parity across four reference platforms with 583 passing tests and zero clippy warnings. However, it remains pre-alpha. This plan addresses every gap between current state and production readiness, with **security auditing as the primary focus**.

### Current State vs Target

| Dimension | Current (v2.0.0) | Target (v3.0) | Gap |
|-----------|-------------------|---------------|-----|
| Security | Unaudited | Externally audited + hardened | **Critical** |
| Resilience | No chaos testing | Chaos engineering verified | High |
| Performance | Benchmarks exist | Regression gates in CI | High |
| Accessibility | High-contrast theme | axe-core verified | Medium |
| UI Stability | No error boundaries | Graceful degradation | Medium |
| Container Security | Basic Docker | Seccomp + AppArmor | High |
| Observability | Metrics endpoint | Full SIEM integration | Medium |
| API Stability | Breaking changes expected | Semantic versioning enforced | Medium |
| Documentation | Runbooks exist | Production runbooks + troubleshooting | Medium |

---

## Phase 1: Security Audit & Hardening (Weeks 1-4)

**Objective**: External security audit + implement all findings + harden execution environment.

### 1.1 External Penetration Testing (Week 1-2)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Engage security firm | Project Owner | Signed SOW | Firm engaged, scope defined |
| Define audit scope | Security Team | Audit scope document | Covers: API, auth, sandbox, network, data at rest |
| Provide codebase access | Engineering | Read-only repo access + staging environment | Auditor has everything needed |
| Penetration test execution | External Firm | Draft findings report | All attack vectors tested |
| Review findings | Engineering + Security | Prioritized remediation list | Critical/High/Medium/Low categorized |

**Scope Definition**:
- **Authentication**: HMAC signing, TOTP, session management
- **Authorization**: T0-T3 permission tier enforcement
- **Input Validation**: MCP sanitization, injection detection, replay protection
- **Execution Sandbox**: Python sandbox escape, Docker container breakout
- **Data at Rest**: SQLite encryption, secret store (AES-256-GCM)
- **Network**: API surface, WebSocket, SSE endpoints
- **Supply Chain**: Dependency vulnerabilities (cargo audit, npm audit)

### 1.2 Remediate Critical/High Findings (Week 2-3)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Fix critical vulnerabilities | Engineering | Patched code + tests | All critical findings resolved |
| Fix high vulnerabilities | Engineering | Patched code + tests | All high findings resolved |
| Add regression tests | Engineering | New test cases | Each finding has a test preventing recurrence |
| Re-scan dependencies | Engineering | Clean cargo audit + npm audit | Zero known vulnerabilities |

### 1.3 Container Hardening (Week 3-4)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Seccomp profiles | Security Team | `seccomp-router.json`, `seccomp-ui.json` | Syscall whitelist applied |
| AppArmor profiles | Security Team | `apparmor-router`, `apparmor-ui` | Mandatory access control enforced |
| Read-only root filesystem | Engineering | Docker Compose updated | Containers run with `read_only: true` |
| Drop all capabilities | Engineering | Docker Compose updated | `cap_drop: [ALL]` on all services |
| Non-root user | Engineering | Dockerfiles updated | All containers run as non-root |
| Image scanning | Engineering | Trivy in CI pipeline | Zero critical/high CVEs in images |

### 1.4 Secret Management Hardening (Week 4)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Key rotation mechanism | Engineering | Automated key rotation | Keys rotate every 90 days |
| Failed auth lockout | Engineering | Account lockout after 5 failures | Lockout duration: 15 minutes |
| Secret encryption at rest | Engineering | Encrypted SQLite DB | Database file encrypted with user key |
| Audit secret access | Engineering | Access logging | All secret accesses logged with timestamp |

**Phase 1 Acceptance Criteria**:
- [ ] External audit completed with findings report
- [ ] All critical/high findings remediated
- [ ] Seccomp + AppArmor profiles applied
- [ ] Zero dependency vulnerabilities
- [ ] Secret rotation implemented
- [ ] Failed auth lockout implemented

---

## Phase 2: Resilience & Performance (Weeks 5-8)

**Objective**: Chaos engineering, performance regression testing, load testing, and UI stability.

### 2.1 Chaos Engineering (Week 5-6)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Chaos testing framework | Engineering | Toxiproxy or similar integrated | Network failures injectable |
| Database failure tests | Engineering | Chaos test suite | System recovers from DB disconnect |
| Network partition tests | Engineering | Chaos test suite | System handles network splits |
| Memory pressure tests | Engineering | Chaos test suite | System handles OOM gracefully |
| Disk full tests | Engineering | Chaos test suite | System handles disk exhaustion |
| Process crash recovery | Engineering | Chaos test suite | Auto-restart on crash |

**Chaos Test Scenarios**:
1. **Database disconnect**: SQLite connection drops → system queues writes, reconnects, replays
2. **Network partition**: Router ↔ UI disconnect → UI shows degraded state, auto-reconnects
3. **Memory pressure**: Container hits memory limit → graceful degradation, not crash
4. **Disk full**: No disk space → system stops accepting writes, alerts operator
5. **Process crash**: Router process killed → systemd/docker restarts, state recovered

### 2.2 Performance Regression Testing (Week 6-7)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| CI benchmark gates | Engineering | GitHub Actions step | Benchmarks run on every PR, fail if >10% regression |
| Load testing suite | Engineering | k6 or Locust scripts | Simulates 100 concurrent users |
| API latency SLOs | Engineering | SLO document | p95 < 500ms, p99 < 1000ms |
| Streaming latency SLOs | Engineering | SLO document | Event delivery < 200ms p95 |
| Memory usage baselines | Engineering | Baseline document | Router < 500MB, UI < 200MB |

**Performance SLOs**:
| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| API p95 latency | < 500ms | > 750ms |
| API p99 latency | < 1000ms | > 1500ms |
| Streaming event delivery | < 200ms | > 500ms |
| Concurrent connections | 1000 | > 800 |
| Memory usage (router) | < 500MB | > 750MB |
| Error rate | < 0.1% | > 1% |

### 2.3 UI Stability (Week 7-8)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Error boundaries | Frontend Team | React ErrorBoundary wrappers | All major components wrapped |
| Loading states | Frontend Team | Skeleton screens | All async ops show loading state |
| Graceful degradation | Frontend Team | Offline/degraded UI | UI works with partial data |
| Accessibility audit | Frontend Team | axe-core report | Zero critical violations |
| Keyboard navigation | Frontend Team | Keyboard-only test | All features accessible via keyboard |

### 2.4 Large Module Refactoring (Week 8)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Split `agent_loop.rs` (1137 lines) | Backend Team | 3-4 focused modules | Each < 400 lines, tests pass |
| Split `vm_pool.rs` (1251 lines) | Backend Team | 3-4 focused modules | Each < 400 lines, tests pass |
| Split `unified_config.rs` (1560 lines) | Backend Team | Config submodules | Each < 400 lines, tests pass |
| Split `skill_manager.rs` (726 lines) | Backend Team | 2-3 focused modules | Each < 400 lines, tests pass |

**Refactoring Rules**:
- No behavior changes — only structural
- All existing tests must pass
- No new clippy warnings
- Public API unchanged

**Phase 2 Acceptance Criteria**:
- [ ] Chaos test suite passes all 5 scenarios
- [ ] Performance regression gates in CI
- [ ] Load test passes 100 concurrent users
- [ ] UI error boundaries on all major components
- [ ] axe-core accessibility audit passes
- [ ] All large modules split (< 400 lines each)

---

## Phase 3: Observability & Operations (Weeks 9-12)

**Objective**: SIEM integration, production monitoring, backup/restore verification, disaster recovery.

### 3.1 SIEM Integration (Week 9-10)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Structured logging | Backend Team | JSON log format | All logs parseable by SIEM |
| Audit log forwarding | Backend Team | Syslog/HTTP endpoint | Audit logs sent to external SIEM |
| Security event alerts | Security Team | Alert rules | Critical events trigger alerts |
| Log retention policy | Operations | Retention config | Logs retained 90 days minimum |
| Dashboard creation | Operations | Grafana dashboards | Real-time security + performance views |

**Security Events to Monitor**:
- Failed authentication attempts (> 5 in 5 minutes)
- Injection detection triggers
- Replay protection rejections
- Permission tier escalations
- Secret access failures
- Anomaly detection alerts
- Container restarts

### 3.2 Backup & Disaster Recovery (Week 10-11)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Automated backups | Operations | Cron-based backup script | Daily backups, 30-day retention |
| Backup encryption | Operations | Encrypted backup files | Backups encrypted with separate key |
| Restore verification | Operations | Monthly restore test | Restore completes in < 15 minutes |
| Disaster recovery plan | Operations | DR runbook | RTO < 1 hour, RPO < 15 minutes |
| Multi-region strategy | Operations | DR architecture | Secondary region ready for failover |

**Recovery Objectives**:
| Metric | Target | Current |
|--------|--------|---------|
| RTO (Recovery Time Objective) | < 1 hour | Unknown |
| RPO (Recovery Point Objective) | < 15 minutes | Unknown |
| Backup frequency | Every 6 hours | Manual |
| Backup retention | 30 days | None |

### 3.3 API Stability Guarantees (Week 11-12)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| API versioning strategy | Backend Team | Versioning document | `/api/v1/` → `/api/v2/` migration path |
| Breaking change policy | Engineering | Policy document | 6-month deprecation notice |
| OpenAPI/Swagger spec | Backend Team | `openapi.yaml` | Full API documentation |
| API compatibility tests | Engineering | Contract tests | Backward compatibility verified |
| Changelog automation | Engineering | Auto-generated changelog | Every release has changelog |

### 3.4 Production Runbooks (Week 12)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Incident response runbook | Operations | IR runbook | Step-by-step incident handling |
| Troubleshooting guide | Engineering | Troubleshooting doc | Common issues + solutions |
| Deployment runbook | Operations | Deploy runbook | Zero-downtime deployment steps |
| Rollback runbook | Operations | Rollback runbook | Rollback in < 10 minutes |
| On-call rotation | Operations | On-call schedule | 24/7 coverage defined |

**Phase 3 Acceptance Criteria**:
- [ ] SIEM integration complete with alerting
- [ ] Automated backups with encryption
- [ ] Restore test passes in < 15 minutes
- [ ] API versioning strategy documented
- [ ] OpenAPI spec published
- [ ] Production runbooks complete

---

## Phase 4: Final Verification & Launch (Weeks 13-16)

**Objective**: End-to-end verification, pilot rollout, production launch.

### 4.1 End-to-End Verification (Week 13-14)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Full test suite | Engineering | All tests pass | 600+ tests, 0 failures |
| Security re-audit | External Firm | Clean report | No critical/high findings |
| Load test at scale | Engineering | Load test report | 1000 concurrent users, < 1% error rate |
| Chaos test verification | Engineering | Chaos test report | All 5 scenarios pass |
| Accessibility verification | Frontend Team | axe-core report | Zero violations |
| Performance verification | Engineering | Benchmark report | All SLOs met |

### 4.2 Pilot Rollout (Week 14-15)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Pilot environment | Operations | Staging environment | Mirrors production |
| Pilot users | Operations | 5-10 pilot users | Real-world usage |
| Monitoring | Operations | Dashboards + alerts | Real-time visibility |
| Feedback collection | Operations | Feedback report | User satisfaction > 80% |
| Issue resolution | Engineering | Bug fixes | All pilot issues resolved |

### 4.3 Production Launch (Week 15-16)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| Production deployment | Operations | Live production system | All services running |
| DNS + SSL | Operations | HTTPS enabled | Valid TLS certificate |
| Monitoring live | Operations | Production dashboards | Real-time metrics visible |
| Incident response ready | Operations | On-call active | 24/7 coverage |
| Launch announcement | Project Owner | Release notes | v3.0.0 published |

### 4.4 Post-Launch (Week 16+)

| Task | Owner | Deliverable | Acceptance Criteria |
|------|-------|-------------|---------------------|
| 30-day monitoring | Operations | Stability report | > 99.9% uptime |
| User feedback review | Product | Feedback summary | Action items identified |
| Performance tuning | Engineering | Optimized system | SLOs maintained |
| Security review | Security Team | Quarterly audit | No new findings |

**Phase 4 Acceptance Criteria**:
- [ ] Full test suite passes (600+ tests)
- [ ] Security re-audit clean
- [ ] Load test passes 1000 concurrent users
- [ ] Pilot users satisfied (> 80%)
- [ ] Production system live with HTTPS
- [ ] 30-day stability > 99.9% uptime

---

## Resource Requirements

| Role | Weeks Required | Responsibilities |
|------|---------------|------------------|
| Security Engineer | 8 | Audit coordination, hardening, SIEM |
| Backend Engineer | 12 | Remediation, refactoring, performance |
| Frontend Engineer | 6 | Error boundaries, accessibility, loading states |
| DevOps Engineer | 8 | CI/CD, Docker hardening, backups, monitoring |
| External Security Firm | 4 | Penetration testing, re-audit |
| Project Manager | 16 | Coordination, timeline, risk management |

---

## Risk Register

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Security audit finds critical issues | High | High | Buffer 2 weeks in Phase 1 for remediation |
| Performance SLOs not met | Medium | High | Early benchmarking in Phase 2, optimize iteratively |
| Large module refactoring introduces bugs | Medium | Medium | Comprehensive test suite, code review, canary deploy |
| External firm availability | Low | High | Engage firm early, have backup firm identified |
| Scope creep | Medium | Medium | Strict change control, defer non-critical items |

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Security audit findings (critical) | 0 | External audit report |
| Security audit findings (high) | 0 | External audit report |
| Test coverage | > 85% | `cargo tarpaulin` |
| CI pass rate | 100% | GitHub Actions |
| API p95 latency | < 500ms | Benchmark suite |
| Error rate | < 0.1% | Monitoring |
| Uptime | > 99.9% | Uptime monitoring |
| Accessibility violations | 0 | axe-core |
| Backup restore time | < 15 minutes | Monthly test |
| RTO | < 1 hour | DR test |
| RPO | < 15 minutes | DR test |

---

## Appendix: Current Known Issues

See `docs/CODEBASE_AUDIT_REPORT.md` and `docs/HANDOVER.md` for full details.

### Critical
- Security unaudited
- No chaos engineering tests
- No external penetration testing

### High
- 12 large modules (>500 lines)
- UI error boundaries missing
- No accessibility testing
- Consolidation AI is rule-based
- Event correlation IDs sparse

### Medium
- No performance regression testing in CI
- No SIEM integration
- No seccomp/AppArmor profiles
- No load testing
- UI animations minimal

### Low
- API instability expected
- Limited API examples
- No troubleshooting guide

---

*This is a living document. Update as phases progress.*
*Created: 2026-03-31 | Owner: Engineering Team*
