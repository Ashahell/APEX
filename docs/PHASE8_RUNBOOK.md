# Phase 8 Runbook: Crosswalks and Governance Cadence

## Overview
- Phase 8 produces formal crosswalks and governance cadence to sustain parity adoption.
- This runbook provides operational steps for incident response, debugging, and escalation.

## Phase 8 Features

| Feature | Description |
|---------|-------------|
| Crosswalk Documents | 4 completed crosswalks (OpenClaw, AgentZero, Hermes, OpenFang) |
| Governance Charter | Formal governance structure, roles, amendment process |
| Governance Cadence | Review cycles, meeting schedules, reporting requirements |
| Parity Scorecard | Updated with all axis completions |

---

## Incident Response Procedures

### 1. Crosswalk Update Needed

**Symptoms:**
- New feature implemented but not reflected in crosswalk
- Parity score outdated
- Evidence links broken

**Immediate Steps:**
```bash
# Check crosswalk documents
cat docs/crosswalk_openclaw_apex.md
cat docs/crosswalk_agentzero_apex.md
cat docs/crosswalk_hermes_apex.md
cat docs/crosswalk_openfang_apex.md

# Update parity scorecard
cat docs/parity-scorecard.md
```

**Debug Commands:**
```bash
# Verify evidence links
# Check that all referenced files exist
ls docs/*.md
ls core/router/src/*.rs
ls ui/src/components/**/*.tsx
```

**Common Causes:**
- Feature implemented without documentation update
- Evidence file moved or renamed
- Parity score not recalculated

**Rollback:** Revert crosswalk to previous version, recalculate scores.

---

### 2. Governance Charter Amendment

**Symptoms:**
- Policy change needed
- Role responsibilities unclear
- Decision-making process disputed

**Immediate Steps:**
```bash
# Check current charter
cat docs/GOVERNANCE_CHARTER.md

# Check governance cadence
cat docs/GOVERNANCE_CADENCE.md

# Review audit trail
curl -s http://localhost:3000/api/v1/governance/immutable | jq .
```

**Debug Commands:**
```bash
# Check governance API
curl -s http://localhost:3000/api/v1/governance/policy | jq .
curl -s http://localhost:3000/api/v1/governance/emergency | jq .
```

**Common Causes:**
- Charter outdated
- Role confusion
- Emergency protocol triggered

**Rollback:** Revert charter amendment, restore previous version.

---

### 3. Parity Score Discrepancy

**Symptoms:**
- Parity score doesn't match implementation status
- Crosswalk scores inconsistent
- Evidence missing for claimed features

**Immediate Steps:**
```bash
# Check all crosswalk scores
grep "Parity Score" docs/crosswalk_*.md

# Verify evidence files exist
# Check referenced files in crosswalks
```

**Debug Commands:**
```bash
# Recalculate parity scores
# Review each primitive's implementation status
# Update scores based on actual implementation
```

**Common Causes:**
- Score calculated before feature complete
- Evidence not linked properly
- Implementation regressed

**Rollback:** Restore previous scores, investigate discrepancy.

---

## Debug Commands Quick Reference

| Command | Purpose |
|---------|---------|
| `cat docs/crosswalk_*.md` | Check all crosswalks |
| `cat docs/GOVERNANCE_CHARTER.md` | Check governance charter |
| `cat docs/parity-scorecard.md` | Check parity scores |
| `curl -s http://localhost:3000/api/v1/governance/policy` | Check governance policy |

---

## Verification Checklist

After any incident, verify:

- [ ] All 4 crosswalks updated with implementation status
- [ ] Governance charter current and accurate
- [ ] Governance cadence documented
- [ ] Parity scorecard reflects actual status
- [ ] All evidence links valid

---

## Contacts

- On-call: @engineering-ops
- Governance: @governance-board
- Documentation: @tech-writers

---

## Last Updated

- Phase 8: Crosswalks and Governance Cadence
- Version: 1.0
- Date: 2026-03-31
