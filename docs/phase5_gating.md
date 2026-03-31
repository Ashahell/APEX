# Phase 5 Gate: Memory Architecture Parity (Hermes-style)

## Overview
- Phase 5 implements Hermes-like memory surface in UI and backend; memory search integration; TTL semantics; and consolidation.
- This document defines the gates, acceptance criteria, and sign-off workflow for Phase 5.

## Gate Criteria (Pass/Fail)

| Gate | Criterion | Verification | Status |
|------|-----------|--------------|--------|
| **5.1** | Memory viewer UI component: 6-tab interface with search, TTL, consolidation, snapshot | UI renders all tabs, all components functional | ✅ PASS |
| **5.2** | TTL semantics: configurable TTL per store, auto-cleanup, persistence | `/api/v1/memory/bounded/ttl` returns/accepts config | ✅ PASS |
| **5.3** | Indexer surface testable: 9+ tests for TTL, consolidation, search | `cargo test memory_integration_phase5` passes | ✅ PASS |
| **5.4** | Phase 5 documentation complete: runbook exists, parity scorecard updated | PHASE5_RUNBOOK.md created, scorecard filled | ✅ PASS |
| **5.5** | Gate review signed off | Governance sign-off recorded | ⏳ PENDING |

## SLO Targets

| Metric | Target | Phase 5 Baseline |
|--------|--------|-----------------|
| Search Response Time | < 200ms | < 500ms |
| TTL Configuration Accuracy | 100% | 100% |
| Consolidation Accuracy | > 80% | 70% |
| Memory Entry Operations | < 100ms | < 150ms |

## Sign-off
- Phase 5 Owner signs off and moves to Phase 6.
- Any gate not met requires corrective plan before Phase 6.

## Mitigations and Escalation
- If any gate cannot be met within the agreed window, escalate to governance board with corrective plan.
- Document blockers in phase5_blockers.md.
