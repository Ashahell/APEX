# Phase 3 Runbook: Security Review and CI Verification

## Overview
- Phase 3 strengthens security posture and ensures CI/CD gates reflect parity milestones.
- This runbook provides operational steps for incident response, debugging, and escalation.

## Phase 3 Features

| Feature | Description |
|---------|-------------|
| Injection Detection Tests | 40 tests covering prompt injection, SQL injection, command injection, path traversal, XSS |
| Replay Protection Tests | 9 tests covering duplicate rejection, distinct signatures, capacity limits |
| Config Validation Tests | 12 tests covering all config sections, defaults, and edge cases |
| Security Integration Suite | `security_integration.rs` with comprehensive attack surface coverage |

---

## Incident Response Procedures

### 1. Injection Detected in Production

**Symptoms:**
- `InjectionClassifier` flags user input as unsafe
- Request blocked or sanitized
- Audit log shows injection attempt

**Immediate Steps:**
```bash
# Check injection classifier patterns
# Review: core/router/src/security/injection_classifier.rs

# Check recent injection detections
# Review audit logs for InjectionType matches

# Verify classifier is not causing false positives
# Run: cargo test --test security_integration
```

**Debug Commands:**
```bash
# Test specific input against classifier
# Add to injection_classifier.rs tests temporarily

# Check threat level distribution
# Review ThreatLevel enum: Safe, Low, Medium, High, Critical
```

**Common Causes:**
- Legitimate code discussion flagged as injection (false positive)
- Actual injection attempt (true positive)
- New injection variant not covered by patterns

**Rollback:** Adjust classifier thresholds, add whitelist patterns for legitimate use cases.

---

### 2. Replay Attack Detected

**Symptoms:**
- `replay_protection::record_and_check()` returns true (replay detected)
- Request rejected with replay error
- Multiple identical signatures observed

**Immediate Steps:**
```bash
# Check replay protection store
# Review: core/router/src/security/replay_protection.rs

# Verify signature generation is unique
# Check HMAC timestamp + nonce combination

# Check for clock skew issues
# Verify server time is synchronized
```

**Debug Commands:**
```bash
# Reset replay protection store (for testing)
# replay_protection::reset()

# Check store capacity
# Default: 10000 entries, oldest evicted when full
```

**Common Causes:**
- Client retrying failed request without new signature
- Clock drift causing timestamp reuse
- Signature generation bug

**Rollback:** Increase store capacity, adjust timestamp window.

---

### 3. Config Validation Failure

**Symptoms:**
- Router fails to start due to invalid config
- Default values not applied correctly
- Environment variable override not working

**Immediate Steps:**
```bash
# Check config defaults
# Review: core/router/src/unified_config.rs

# Verify environment variables
echo $APEX_SHARED_SECRET
echo $APEX_AUTH_DISABLED
echo $APEX_PORT
```

**Debug Commands:**
```bash
# Run config validation tests
cargo test config_default_is_valid
cargo test config_port_valid_range
cargo test config_db_connection_pool_valid

# Check all config sections
# server, auth, channels, agent, execution, database, memory, heartbeat, skills, streaming
```

**Common Causes:**
- Missing environment variable
- Invalid port number
- Database connection string malformed

**Rollback:** Restore default config, fix environment variables.

---

## Debug Commands Quick Reference

| Command | Purpose |
|---------|---------|
| `cargo test --test security_integration` | Run all Phase 3 security tests |
| `cargo test injection_` | Run injection detection tests |
| `cargo test replay_` | Run replay protection tests |
| `cargo test config_` | Run config validation tests |
| `cargo check -p apex-router` | Verify compilation |

---

## Test Commands

```bash
# Run all Phase 3 security tests
cd core && cargo test --test security_integration

# Run specific test categories
cd core && cargo test injection_prompt
cd core && cargo test injection_sql
cd core && cargo test injection_cmd
cd core && cargo test injection_path_traversal
cd core && cargo test injection_xss
cd core && cargo test injection_safe
cd core && cargo test replay_
cd core && cargo test config_
```

---

## Escalation Paths

| Issue | First Contact | Escalation |
|-------|--------------|------------|
| Injection false positives | @backend-team | @security-team |
| Replay protection issues | @backend-team | @engineering-ops |
| Config validation failures | @backend-team | @infra-team |
| CI failures | @engineering-ops | @sre-team |

---

## Rollback Procedure

If Phase 3 changes cause critical issues:

1. **Disable injection classifier (if causing false positives):**
   ```rust
   // In injection_classifier.rs, temporarily return safe
   // This is a last resort - prefer adjusting thresholds
   ```

2. **Restart services:**
   ```bash
   cargo run --release --bin apex-router
   ```

3. **Verify recovery:**
   ```bash
   cargo test --test security_integration
   # All 40 tests should pass
   ```

---

## Verification Checklist

After any incident, verify:

- [ ] All 40 security integration tests pass
- [ ] Injection classifier detects known patterns
- [ ] Replay protection rejects duplicates
- [ ] Config validation passes for all sections
- [ ] No false positives on legitimate inputs
- [ ] CI pipeline green

---

## Contacts

- On-call: @engineering-ops
- Security: @security-team
- Backend: @backend-team
- Infrastructure: @infra-team

---

## Last Updated

- Phase 3: Security Review and CI Verification
- Version: 1.0
- Date: 2026-03-31
