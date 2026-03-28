# APEX Streaming Phase 2-3 Implementation Plan

## Overview
This document outlines a concrete 4-6 week plan to complete Phase 2 (UI wiring + UI tests) and Phase 3 (telemetry/SLOs/runbooks) for the APEX streaming MVP.

---

## Phase 2: UI Wiring & Testing (Weeks 1-3)

### Week 1: Connect Backend to UI

| Task | Owner | Deliverable |
|------|-------|-------------|
| **2.1** Wire StreamingDashboard to routing (add route in App.tsx) | Frontend | `/streaming` route registered |
| **2.2** Add signed URL generation endpoint (backend) | Backend | `/api/v1/streams/sign` endpoint |
| **2.3** Update useStreaming hook to call signed endpoint | Frontend | Hook uses signed URLs |
| **2.4** Add tab navigation from sidebar | Frontend | "Streaming" in sidebar nav |

**Milestone 1 (End of Week 1):** UI can connect to `/stream/stats` with proper auth

### Week 2: Full Stream Integration

| Task | Owner | Deliverable |
|------|-------|-------------|
| **2.5** Connect Hands panel to real backend | Frontend | `/stream/hands/:task_id` renders |
| **2.6** Connect MCP panel to real backend | Frontend | `/stream/mcp/:task_id` renders |
| **2.7** Connect Task panel to real backend | Frontend | `/stream/task/:task_id` renders |
| **2.8** Add task selector dropdown | Frontend | User can select task to monitor |

**Milestone 2 (End of Week 2):** All 4 panels render real data from backend

### Week 3: UI Testing & Polish

| Task | Owner | Deliverable |
|------|-------|-------------|
| **2.9** Add Playwright/Cypress E2E test for streaming | QA/Frontend | Test verifies stream renders |
| **2.10** Add reconnection test | QA/Frontend | Test verifies auto-reconnect |
| **2.11** Add error handling UI | Frontend | Error states display properly |
| **2.12** Accessibility audit | Frontend | Keyboard nav, ARIA labels |

**Milestone 3 (End of Week 3):** UI tests pass, accessibility verified

---

## Phase 3: Telemetry, SLOs & Runbooks (Weeks 4-6)

### Week 4: Telemetry Foundation

| Task | Owner | Deliverable |
|------|-------|-------------|
| **3.1** Add Prometheus metrics endpoint | Backend | `/metrics` exposes streaming metrics |
| **3.2** Add latency histogram | Backend | Track request latency |
| **3.3** Add Grafana dashboard JSON | Infra | Dashboard for streaming |
| **3.4** Add connection duration metric | Backend | Track avg connection time |

**Milestone 4 (End of Week 4):** Metrics exposed and dashboard available

### Week 5: SLOs & Alerting

| Task | Owner | Deliverable |
|------|-------|-------------|
| **3.5** Define SLIs for streaming | Platform | Document: latency p95, error rate |
| **3.6** Add alerting rules | Infra | Alerts for SLO breaches |
| **3.7** Add uptime monitoring | Infra | Monitor endpoint availability |
| **3.8** Document SLOs in STREAMING_ROLLOUT.md | Platform | Official SLO document |

**Milestone 5 (End of Week 5):** SLOs defined, alerts configured

### Week 6: Runbooks & Handoff

| Task | Owner | Deliverable |
|------|-------|-------------|
| **3.9** Expand STREAMING_RUNBOOKS.md | Platform | Full incident response |
| **3.10** Add rollback procedures | Platform | Rollback steps documented |
| **3.11** Create onboarding doc | Platform | How to use streaming UI |
| **3.12** Final review & signoff | All | Phase 2-3 complete |

**Milestone 6 (End of Week 6):** Runbooks complete, handoff ready

---

## Ownership Matrix

| Area | Owner | Backup |
|------|-------|--------|
| Frontend (UI) | @frontend-team | @platform-team |
| Backend (Auth/Streams) | @backend-team | @platform-team |
| Telemetry/Infra | @infra-team | @platform-team |
| SLOs/Runbooks | @platform-team | @all |

---

## Success Criteria

- [ ] All 4 streaming panels render real data
- [ ] E2E tests pass for streaming flows
- [ ] Metrics endpoint exposes streaming data
- [ ] SLOs defined and monitored
- [ ] Runbooks complete and tested
- [ ] No critical security issues

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Auth token expiry during stream | Medium | Auto-refresh tokens |
| High memory with many streams | Medium | Implement backpressure |
| Metrics overload | Low | Rate-limit exports |
| SLO breach false positives | Low | Tune alerting thresholds |

---

## Dependencies

- PR A (backend SSE auth) - ✅ Complete
- PR B (UI skeleton) - ✅ Complete  
- PR C (parity docs) - ⏳ In Review

---

## Timeline Summary

```
Week 1: [2.1] → [2.2] → [2.3] → [2.4] → Milestone 1
Week 2: [2.5] → [2.6] → [2.7] → [2.8] → Milestone 2
Week 3: [2.9] → [2.10] → [2.11] → [2.12] → Milestone 3
Week 4: [3.1] → [3.2] → [3.3] → [3.4] → Milestone 4
Week 5: [3.5] → [3.6] → [3.7] → [3.8] → Milestone 5
Week 6: [3.9] → [3.10] → [3.11] → [3.12] → Milestone 6 ✅
```

---

**Document Status:** Ready for Review  
**Last Updated:** 2026-03-28
