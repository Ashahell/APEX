# APEX Security Model

**Status:** Implemented (v1.6.0)
**Date:** 2026-04-25
**Sources:** [raw/SECURITY.md](raw/SECURITY.md), [raw/SECURITY_AUDIT_PLAN.md](raw/SECURITY_AUDIT_PLAN.md)

## Defense Layers

### 1. HMAC Request Signing

All API requests between Gateway ↔ Router ↔ UI are signed:
- Header: `X-APEX-Signature` = HMAC-SHA256(timestamp + method + path + body)
- Header: `X-APEX-Timestamp` = Unix timestamp
- Timestamp must be within 5 minutes (replay attack prevention)
- Configurable via `APEX_AUTH_DISABLED=1` for local dev

### 2. T0-T3 Permission Tiers

| Tier | Actions | Gate |
|------|---------|------|
| **T0** | Read-only queries, search | None |
| **T1** | File writes, drafts | Tap to confirm |
| **T2** | External API calls, git push | Type action text to confirm |
| **T3** | Destructive ops, cost >$10, shell.execute | TOTP + 5-min delay |

> Note: `shell.execute` moved from T2 → T3 per security audit.

### 3. TOTP Verification

- T3 tasks require 6-digit TOTP code from authenticator app
- Setup via `POST /api/v1/totp/setup`
- Verify via `POST /api/v1/totp/verify`
- Status check via `GET /api/v1/totp/status`

### 4. VM Isolation (L5 Execution)

- **Firecracker** (Linux): Dedicated kernel, no host sharing, ephemeral storage
- **gVisor** (fallback): User-space kernel, less overhead
- **Docker** (Windows fallback): Container isolation
- Network blocked by default (allowlist required)
- Resource limits enforced (memory, CPU, time)

### 5. SSRF Protection (CVE-2026-4308)

Web fetch tool blocks requests to:
- `localhost`, `127.0.0.1`, `::1`
- Private IP ranges: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`
- Cloud metadata: `169.254.169.254`, `metadata.google.internal`
- IPv6 equivalents and IPv4-mapped addresses
- DNS resolution to prevent CNAME redirection attacks

### 6. Schema Security (Post-Audit Fixes)

| Fix | Implementation |
|-----|----------------|
| Audit chain deletion | Archive instead of delete; `archived_at` column |
| Weak key derivation | Argon2id + machine-specific token (CPU info) |
| Missing FK enforcement | `PRAGMA foreign_keys = ON`, CASCADE DELETE |
| Encryption not default | Auto-encrypt all secret-type settings |

### 7. Input Validation

- Prompt injection defense: user input sanitized before LLM
- Command injection mitigation in skill execution
- Content hashing for file integrity
- Injection classifier for dangerous patterns

### 8. Death Spiral Detection

Execution pattern anomalies detected:
- `FileCreationBurst`: Too many files created in short time
- `ToolCallLoop`: Same tool called repeatedly without progress
- `NoSideEffects`: Task produces no observable changes

## Security Tests

- 57 security tests (input validation, audit chain, permission tiers)
- 12 SSRF protection tests
- 33 sandbox security tests

## Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `APEX_SHARED_SECRET` | HMAC signing secret | `dev-secret-change-in-production` |
| `APEX_AUTH_DISABLED` | Disable auth | false |
| `APEX_USE_FIRECRACKER` | Enable Firecracker VMs | false |
| `APEX_USE_GVISOR` | Enable gVisor | false |

## Related

- [architecture.md](concepts/architecture.md)
- [schema-audit.md](concepts/schema-audit.md)
- [schema-fix-plan.md](concepts/schema-fix-plan.md)
- [raw/SECURITY_THREAT_MODEL.md](raw/SECURITY_THREAT_MODEL.md)