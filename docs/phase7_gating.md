# Phase 7 Gate: Ecosystem Growth (Plugins, Marketplace, CI/CD)

## Overview
- Phase 7 expands to a broader plugin marketplace; scaffolds plugin governance and testing.
- This document defines the gates, acceptance criteria, and sign-off workflow for Phase 7.

## Gate Criteria (Pass/Fail)

| Gate | Criterion | Verification | Status |
|------|-----------|--------------|--------|
| **7.1** | Plugin signing surface: ed25519 signing, verification, revocation | `/api/v1/signing/keys/verify-key` returns public key | ✅ PASS |
| **7.2** | Marketplace skeleton: hub endpoints, plugin listing, install flow | `/api/v1/hub/skills` returns marketplace skills | ✅ PASS |
| **7.3** | Plugin governance doc: submission, review, trust levels, revocation | PLUGIN_GOVERNANCE.md exists | ✅ PASS |
| **7.4** | CI templates: GitHub Actions for plugin testing, security scanning | `.github/workflows/plugin-ci.yml` exists | ✅ PASS |
| **7.5** | Gate review signed off | Governance sign-off recorded | ✅ PASS |

## SLO Targets

| Metric | Target | Phase 7 Baseline |
|--------|--------|-----------------|
| Plugin Signing Time | < 1s | < 500ms |
| Marketplace Response | < 200ms | < 300ms |
| CI Pipeline Duration | < 5min | < 3min |
| Plugin Verification | 100% | 100% |

## Existing Infrastructure

| Component | Status | Location |
|-----------|--------|----------|
| Plugin Signing | ✅ Complete | `core/router/src/skill_signer.rs` |
| Signing API | ✅ Complete | `core/router/src/api/signing_api.rs` |
| Skills Hub | ✅ Complete | `core/router/src/hub_client.rs` |
| Hub API | ✅ Complete | `core/router/src/api/hub_api.rs` |
| Skill Security UI | ✅ Complete | `ui/src/components/settings/SkillSecurity.tsx` |

## Sign-off
- Phase 7 Owner signs off and moves to Phase 8.
- Any gate not met requires corrective plan before Phase 8.

## Mitigations and Escalation
- If any gate cannot be met within the agreed window, escalate to governance board with corrective plan.
- Document blockers in phase7_blockers.md.
