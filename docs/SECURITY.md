# APEX Security Audit

> ⚠️ **Status**: This document details the security posture of APEX v1.3.0

---

## Security Model Overview

APEX implements a **multi-tier permission system** (T0-T3) with increasing verification requirements:

| Tier | Name | Verification | Example Actions |
|------|------|--------------|------------------|
| T0 | Read-only | None | Query, search, read |
| T1 | Tap confirm | Click confirmation | File writes, drafts |
| T2 | Type confirm | Type action name | Git push, API calls |
| T3 | TOTP verify | Time-based OTP + 5s delay | Shell execution, destructive ops |

---

## Authentication

### HMAC Request Signing

All API requests require HMAC-SHA256 signature authentication:

```http
X-APEX-Signature: <hmac-sha256-signature>
X-APEX-Timestamp: <unix-timestamp>
```

**Signature Calculation:**
```
message = timestamp + method + path + body
signature = HMAC-SHA256(shared_secret, message)
```

- Timestamp must be within 5 minutes to prevent replay attacks
- Set `APEX_AUTH_DISABLED=1` for local development ONLY

### Capability Tokens

Task execution includes capability tokens that encode:
- Task ID
- Permission tier
- Allowed skills
- Allowed domains
- Expiration time
- Maximum cost

---

## TOTP Verification (T3)

T3 operations require TOTP verification using `totp-rs`:

```rust
// TOTP configuration
Secret key: Base32-encoded, stored securely
Digits: 6
Period: 30 seconds
Algorithm: SHA1
```

### T3 Actions Requiring Verification:
- `shell.execute` - Shell command execution
- Any skill marked as T3 tier

---

## Input Validation

### SQL Injection Prevention
- All user input is parameterized in SQL queries
- Using `sqlx` with bind variables

### Prompt Injection Defense
Agent prompts include sanitization for:
- DAN/jailbreak patterns
- Developer mode bypass attempts
- New instruction overrides
- Spanish-to-English translation tricks

### File Path Validation
- Sanitized identifiers (alphanumeric, dash, underscore, space only)
- Path traversal prevention

---

## Network Security

### Firecracker VMs (Linux)
- Network isolation: `none` (no internet by default)
- Memory limit: 2048MB
- CPU limit: 2
- Process limit: 256

### Docker Isolation
- `--memory=2048m`
- `--cpus=2`
- `--pids-limit=256`
- `--network=none`
- `--read-only` filesystem with `--tmpfs=/tmp`

### gVisor (Linux fallback)
- Runsc sandbox for container isolation
- Network namespace isolation

---

## Audit Trail

All security-relevant events are logged:

```rust
// Audit log entry
struct AuditEntry {
    timestamp: DateTime<Utc>,
    action: String,
    actor: String,
    resource: String,
    outcome: String,  // success/failure
    metadata: Json,
}
```

### Protected Values
- T3 operations require hardware token
- Constitution values immutable without T3
- SOUL.md checksum verification

---

## Environment Security

| Variable | Security Note |
|----------|---------------|
| `APEX_SHARED_SECRET` | Critical - HMAC signing key |
| `APEX_AUTH_DISABLED` | NEVER in production |
| `APEX_NATS_ENABLED` | Distributed mode security |

---

## Known Limitations

1. **Pre-alpha status** - Not production-ready
2. **No security audit** - Requires third-party review
3. **Local-only by default** - Bind to localhost
4. **Single-user model** - No multi-tenancy

---

## Recommendations for Production

1. Enable TOTP for all destructive operations
2. Use Firecracker/gVisor isolation
3. Configure network allowlists
4. Enable audit log retention
5. Rotate shared secret regularly
6. Enable rate limiting
7. Use TLS in production

---

## Security Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `totp-rs` | 5 | TOTP generation |
| `hmac` | 0.12 | Request signing |
| `sha2` | 0.10 | Hashing |
| `base32` | 0.5 | TOTP secret encoding |

---

*Last Updated: 2026-03-06*
