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

## Rate Limiting

APEX implements token bucket rate limiting to prevent DoS:

### Current Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| General API | 60 requests | 1 minute |
| Task Creation | 10 requests | 1 minute |
| Skill Execution | 30 requests | 1 minute |
| Deep Tasks | 5 requests | 1 minute |

---

## Execution Isolation

### Docker Security (Current Implementation)

APEX uses Docker with hardened security settings:

```bash
# Resource limits
--memory 2048m              # 2GB memory limit
--cpus 2                    # 2 CPU cores
--pids-limit 256            # Max 256 processes

# Isolation
--network none              # Network isolation (no internet access)
--read-only                 # Read-only filesystem
--tmpfs /tmp:rw,exec       # Writable tmpfs for temp files
--cap-drop ALL              # Drop all capabilities
--privileged=false          # Not privileged
--restart no                # No auto-restart
--rm                        # Auto-remove on exit
--stop-timeout 10           # Graceful shutdown
```

### Firecracker (When Available)

Firecracker provides stronger isolation via microVMs:
- No shared kernel with host
- Near-native performance
- Minimal attack surface

### Security Comparison

| Isolation | Network | Filesystem | Capabilities |
|-----------|---------|------------|-------------|
| Docker (hardened) | None | Read-only + tmpfs | Dropped ALL |
| Firecracker | Isolated | Separate | Minimal |

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

## MCP Security (Added 2026-03-09)

### Connection Pooling
MCP servers are managed via `McpServerManager` with connection pooling:
- Configurable min/max connections per server
- Connection timeout and idle timeout
- Health check on all pooled connections

### Input Sanitization
All MCP tool arguments are validated:

| Check | Limit |
|-------|-------|
| Nesting depth | 10 levels |
| String length | 100KB |
| Object keys | 1000 |
| Array length | 10000 |

### Blocked Patterns
- Shell injection (`;`, `|`, `&&`, etc.)
- Path traversal (`../`, absolute paths)
- Code execution (`eval`, `exec`, `__import__`)

---

## Recommendations for Production

1. Enable TOTP for all destructive operations
2. Use Firecracker/gVisor isolation (or Docker with hardened settings)
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
