# Phase 7 Runbook: Ecosystem Growth (Plugins, Marketplace, CI/CD)

## Overview
- Phase 7 expands to a broader plugin marketplace; scaffolds plugin governance and testing.
- This runbook provides operational steps for incident response, debugging, and escalation.

## Phase 7 Features

| Feature | Description |
|---------|-------------|
| Plugin Signing | ed25519 cryptographic signing for all plugins |
| Skills Hub | Marketplace for plugin discovery and installation |
| Trust Levels | Community → Trusted → Verified progression |
| Governance | Submission, review, revocation policies |

---

## Incident Response Procedures

### 1. Plugin Signing Failures

**Symptoms:**
- Signature verification fails
- Plugins rejected during installation
- Public key unavailable

**Immediate Steps:**
```bash
# Check signing API
curl -s http://localhost:3000/api/v1/signing/keys/verify-key | jq .

# Check signature stats
curl -s http://localhost:3000/api/v1/signing/signatures/stats | jq .

# Verify plugin signature
curl -X POST http://localhost:3000/api/v1/signing/skills/<name>/verify \
  -H "Content-Type: application/json" \
  -d '{"skill_name": "<name>", "content": "...", "signature": {...}}'
```

**Debug Commands:**
```bash
# Check signing keys
# Review: core/router/src/skill_signer.rs

# Check key directory
ls -la ~/.apex/signing/keys/
```

**Common Causes:**
- Missing signing keys
- Expired signatures
- Tampered plugin content

**Rollback:** Regenerate signing keys, re-sign affected plugins.

---

### 2. Marketplace Issues

**Symptoms:**
- Hub connection fails
- Plugin listing empty
- Installation fails

**Immediate Steps:**
```bash
# Check hub status
curl -s http://localhost:3000/api/v1/hub/status | jq .

# List marketplace skills
curl -s http://localhost:3000/api/v1/hub/skills | jq .

# Check featured skills
curl -s http://localhost:3000/api/v1/hub/skills/featured | jq .
```

**Debug Commands:**
```bash
# Check hub client
# Review: core/router/src/hub_client.rs

# Check hub API
# Review: core/router/src/api/hub_api.rs
```

**Common Causes:**
- Hub server unavailable
- Network connectivity issues
- Invalid plugin metadata

**Rollback:** Switch to local plugin registry, investigate hub connectivity.

---

### 3. CI Pipeline Failures

**Symptoms:**
- Plugin tests fail
- Security scan rejects plugin
- Deployment blocked

**Immediate Steps:**
```bash
# Run plugin tests locally
cd skills && pnpm test

# Run security scan
cargo clippy -- -D warnings

# Check CI configuration
cat .github/workflows/plugin-ci.yml
```

**Debug Commands:**
```bash
# Check plugin validation
# Review: core/router/src/api/signing_api.rs

# Check plugin governance
# Review: docs/PLUGIN_GOVERNANCE.md
```

**Common Causes:**
- Test failures in plugin code
- Security policy violations
- CI configuration errors

**Rollback:** Revert plugin changes, fix CI configuration.

---

## Debug Commands Quick Reference

| Command | Purpose |
|---------|---------|
| `curl -s http://localhost:3000/api/v1/signing/keys/verify-key` | Check signing key |
| `curl -s http://localhost:3000/api/v1/hub/status` | Hub status |
| `curl -s http://localhost:3000/api/v1/hub/skills` | List marketplace skills |
| `cd skills && pnpm test` | Run plugin tests |

---

## Test Commands

```bash
# Run all tests
cd core && cargo test

# Run plugin-specific tests
cd skills && pnpm test
```

---

## Escalation Paths

| Issue | First Contact | Escalation |
|-------|--------------|------------|
| Signing failures | @backend-team | @security-team |
| Marketplace issues | @backend-team | @engineering-ops |
| CI failures | @engineering-ops | @sre-team |

---

## Rollback Procedure

If Phase 7 changes cause critical issues:

1. **Disable plugin signing:**
   ```bash
   # Skip signature verification temporarily
   ```

2. **Restart services:**
   ```bash
   cargo run --release --bin apex-router
   ```

3. **Verify recovery:**
   ```bash
   curl -s http://localhost:3000/api/v1/hub/status
   ```

---

## Verification Checklist

After any incident, verify:

- [ ] Signing API returns public key
- [ ] Hub status is healthy
- [ ] Marketplace skills listed
- [ ] Plugin tests pass
- [ ] CI pipeline green

---

## Contacts

- On-call: @engineering-ops
- Backend Plugins: @backend-team
- Security: @security-team
- Marketplace: @marketplace-team

---

## Last Updated

- Phase 7: Ecosystem Growth
- Version: 1.0
- Date: 2026-03-31
