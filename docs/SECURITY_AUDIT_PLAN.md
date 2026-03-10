# APEX Security Audit & Improvement Plan

> **Document Version**: 1.0  
> **Date**: 2026-03-09  
> **Status**: Planning  
> **Scope**: Full Security Assessment & Hardening Plan

---

## Executive Summary

This document outlines a comprehensive security audit and improvement plan for APEX v1.3.0. It assesses current security implementations, identifies gaps, and provides a prioritized roadmap for achieving production-ready security.

### Current Security Posture

| Layer | Implementation | Coverage | Quality |
|-------|---------------|----------|---------|
| Authentication | HMAC-SHA256 | ✅ Complete | Good |
| Authorization | T0-T3 Tiers | ✅ Complete | Good |
| TOTP | Time-based OTP | ✅ Complete | Good |
| Rate Limiting | Token Bucket | ✅ Basic | Needs Enhancement |
| Input Validation | MCP Sanitization | ✅ Complete | Good |
| Audit Logging | Hash Chain | ✅ Complete | Good |
| Execution Isolation | Docker/Firecracker | ✅ Complete | Good |

**Overall Assessment**: Strong foundation with identified gaps requiring attention before production.

---

## Part 1: Current Security Implementation Analysis

### 1.1 Authentication (HMAC)

**File**: `core/router/src/auth.rs`

**Implementation**:
- HMAC-SHA256 signature verification
- Timing-safe string comparison (`timing_safe_eq`)
- 5-minute timestamp window for replay protection
- Middleware-based enforcement

**Key Functions**:
```rust
pub fn sign_request(secret: &str, method: &str, path: &str, body: &[u8], timestamp: i64) -> String
pub fn verify_request(secret: &str, method: &str, path: &str, body: &[u8], signature: &str, timestamp: i64) -> bool
pub async fn auth_middleware(State(state): State<AuthConfig>, request: Request<Body>, next: Next) -> Response
```

**Tests** (9 tests):
- ✅ `test_sign_request` - Signature generation
- ✅ `test_verify_request_valid` - Valid signature verification
- ✅ `test_verify_request_invalid_signature` - Invalid signature rejection
- ✅ `test_verify_request_wrong_secret` - Wrong secret rejection
- ✅ `test_timing_safe_eq_equal` - Timing-safe comparison
- ✅ `test_timing_safe_eq_different_length` - Length mismatch handling
- ✅ `test_timing_safe_eq_different_content` - Content mismatch
- ✅ `test_auth_config_from_env` - Config loading
- ✅ `test_sign_request_different_methods_different_signatures` - Method diversity
- ✅ `test_sign_request_different_paths_different_signatures` - Path diversity

**Strengths**:
- Constant-time comparison prevents timing attacks
- 5-minute replay protection window
- Comprehensive test coverage

**Gaps**:
- No key rotation mechanism
- Secrets stored in environment variables (not encrypted)
- No failed authentication lockout

---

### 1.2 TOTP Verification

**File**: `core/router/src/totp.rs`

**Implementation**:
- TOTP using `totp-rs` crate
- SHA1 algorithm, 6 digits, 30-second period
- In-memory secret storage (not persisted)

**Key Functions**:
```rust
pub async fn generate_secret(&self, user_id: &str) -> Result<String, String>
pub async fn verify(&self, user_id: &str, token: &str) -> Result<bool, String>
pub fn generate_otpauth_uri(secret: &str, account_name: &str, issuer: &str) -> String
```

**Tests** (5 tests):
- ✅ `test_totp_manager_new` - Manager initialization
- ✅ `test_generate_secret` - Secret generation
- ✅ `test_remove_secret` - Secret removal
- ✅ `test_verify_no_secret` - Missing secret handling
- ✅ `test_generate_otpauth_uri` - URI generation

**Strengths**:
- Standard TOTP implementation
- Proper RFC4648 encoding
- User-scoped secrets

**Gaps**:
- Secrets not persisted (lost on restart)
- No backup codes for account recovery
- No brute-force protection on verification
- Missing: Device fingerprinting/location tracking

---

### 1.3 Rate Limiting

**File**: `core/router/src/rate_limiter.rs`

**Implementation**:
- Token bucket algorithm
- Per-key rate limiting
- Configurable requests per minute
- Stats reporting

**Key Functions**:
```rust
pub async fn check_limit(&self, key: &str) -> RateLimitResult
pub async fn set_config(&self, requests_per_minute: u32)
pub async fn stats(&self) -> RateLimitStats
```

**Tests** (3 tests):
- ✅ `test_rate_limiter_allows` - Basic rate limiting
- ✅ `test_rate_limiter_different_keys` - Per-key isolation
- ✅ `test_rate_limiter_stats` - Statistics

**Current Limits** (from SECURITY.md):
| Endpoint | Limit | Window |
|----------|-------|--------|
| General API | 60 requests | 1 minute |
| Task Creation | 10 requests | 1 minute |
| Skill Execution | 30 requests | 1 minute |
| Deep Tasks | 5 requests | 1 minute |

**Strengths**:
- Per-client limiting
- Configurable
- Statistics available

**Gaps**:
- No per-endpoint specialized limits (using global limiter)
- No progressive throttling
- No distributed rate limiting (NATS mode)
- No failed auth lockout

---

### 1.4 Input Validation & Sanitization

**File**: `core/router/src/mcp/validation.rs`

**Implementation**:
- Dangerous pattern blocking
- Nesting depth limits
- String length limits
- Object key limits
- Array length limits

**Blocked Patterns**:
```rust
const DANGEROUS_PATTERNS: &[&str] = &[
    "eval(", "exec(", "compile(", "__import__", "subprocess",
    "spawn(", "Popen", "system(", "shell=True", "bash -c",
    "sh -c", "; rm -", "| rm -", "&& rm -", "&& curl",
    "| curl", "wget ", "curl -O", "--upload-file", "--output",
    "/etc/passwd", "/etc/shadow", "~/.ssh", "/.ssh/",
    "id_rsa", "id_ed25519", "..%2F", "%2E%2E", "..\\",
    "\\\\..", "{{{{", "{{", "}}", "${", "$((",
    "`", "chr(",
];
```

**Limits**:
- Max nesting depth: 10
- Max string length: 100KB
- Max object keys: 1000
- Max array length: 10000

**Tests**: None found for validation module

**Strengths**:
- Comprehensive pattern blocking
- Depth/length limits prevent DoS
- Applied to MCP tool arguments

**Gaps**:
- No tests for validation module
- No regex-based pattern matching (simple contains)
- Not applied to all input sources (chat, tasks)
- Missing: SQL injection testing

---

### 1.5 Audit Logging

**Files**: 
- `core/router/src/api/audit.rs` (API)
- `core/memory/src/audit.rs` (Repository)

**Implementation**:
- Hash chain for tamper detection
- SHA-256 hashing with previous hash
- Chain verification endpoint

**Key Functions**:
```rust
pub fn compute_hash(&self) -> String
pub async fn verify(&self, pool: &PgPool) -> Result<bool, Error>
pub async fn verify_chain(&self) -> Result<bool, Error>
```

**API Endpoints**:
- `GET /api/v1/audit` - List audit entries
- `POST /api/v1/audit` - Create audit entry
- `GET /api/v1/audit/entity/:entity_type/:entity_id` - Get by entity
- `GET /api/v1/audit/chain` - Verify chain integrity

**Tests**: None found for audit repository

**Strengths**:
- Cryptographic hash chain
- Verification endpoint available
- Entity-scoped queries

**Gaps**:
- No tests for hash chain verification
- No automatic cleanup/rotation
- Limited retention policy
- Not integrated with SIEM

---

### 1.6 Execution Isolation

**File**: `core/router/src/vm_pool.rs`

**Docker Security Flags**:
```rust
let docker_args = vec![
    "run", "-d", "--name", &container_name,
    "--memory", "2048m",           // Memory limit
    "--cpus", "2",                  // CPU limit
    "--pids-limit", "256",          // Process limit
    "--network", "none",             // Network isolation
    "--read-only",                  // Read-only filesystem
    "--tmpfs", "/tmp:rw,exec,size=64m",  // Writable tmpfs
    "--tmpfs", "/run:rw,exec,size=16m",
    "--cap-drop", "ALL",            // Drop capabilities
    "--privileged", "false",        // Not privileged
    "--restart", "no",              // No auto-restart
    "--rm",                         // Auto-remove
    "--stop-timeout", "10",         // Graceful shutdown
];
```

**Tests** (3 tests):
- ✅ `test_vm_pool_initialization`
- ✅ `test_vm_acquire_release`
- ✅ `test_vm_pool_exhaustion`

**Strengths**:
- Comprehensive Docker hardening
- Network isolation
- Capability dropping
- Read-only filesystem

**Gaps**:
- No custom seccomp profile
- No AppArmor/SELinux profiles
- No user namespace remapping
- Limited resource monitoring during execution

---

### 1.7 Permission Tiers

**Implementation**: T0-T3 system

| Tier | Name | Skills |
|------|------|--------|
| T0 | Read-only | code.review, repo.search, deps.check, file.search |
| T1 | Tap confirm | file.delete, git.commit, code.generate, etc. |
| T2 | Type confirm | db.drop, deploy.kubectl, docker.build, etc. |
| T3 | TOTP verify | shell.execute |

**Key Files**:
- `core/router/src/t3_confirm_worker.rs` - T3 confirmation handler
- `core/router/src/governance.rs` - Governance engine

**Tests**: None found for permission tier enforcement

**Strengths**:
- Clear tier separation
- T3 requires TOTP
- 5-second delay for T3 actions

**Gaps**:
- No tests for tier enforcement
- No audit of tier escalation attempts
- Skill tier assignment not programmatically verified

---

## Part 2: Security Test Coverage Analysis

### 2.1 Test Coverage Matrix

| Security Feature | Unit Tests | Integration Tests | E2E Tests | Status |
|-----------------|------------|-------------------|-----------|--------|
| HMAC Auth | 9 | 0 | 0 | ✅ Good |
| TOTP | 5 | 0 | 0 | ✅ Good |
| Rate Limiting | 3 | 0 | 0 | ⚠️ Basic |
| Input Validation | 0 | 0 | 0 | ❌ Missing |
| Audit Logging | 0 | 0 | 0 | ❌ Missing |
| Execution Isolation | 3 | 0 | 0 | ⚠️ Basic |
| Permission Tiers | 0 | 0 | 0 | ❌ Missing |
| MCP Security | 0 | 2 | 2 | ⚠️ Partial |

### 2.2 Critical Gaps

1. **No input validation tests** - Critical for security
2. **No audit chain tests** - Cannot verify integrity
3. **No permission tier tests** - Cannot verify enforcement
4. **No integration tests for auth** - Middleware not tested
5. **No penetration testing** - External review needed

---

## Part 3: Security Improvement Roadmap

### Phase 1: Immediate Security Fixes (Week 1-2)

#### 1.1 Add Input Validation Tests

**Priority**: Critical  
**Effort**: Low

```rust
// Tests to add in validation.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sanitize_dangerous_shell_patterns() {
        let result = sanitize_tool_arguments(&json!({
            "script": "echo hello; rm -rf /"
        }));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_sanitize_path_traversal() {
        let result = sanitize_tool_arguments(&json!({
            "path": "../../../etc/passwd"
        }));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_max_nesting_depth() {
        // Create deeply nested JSON
        let deep = create_nested_json(15);
        let result = sanitize_tool_arguments(&deep);
        assert!(result.is_err());
    }
}
```

#### 1.2 Add Audit Chain Tests

**Priority**: Critical  
**Effort**: Low

```rust
#[cfg(test)]
mod tests {
    use apex_memory::AuditEntry;
    
    #[test]
    fn test_compute_hash() {
        let entry = AuditEntry::new("test", "task", "1", None);
        let hash = entry.compute_hash();
        assert_eq!(hash.len(), 64); // SHA-256 hex
    }
    
    #[test]
    fn test_verify_chain() {
        // Create chain: entry1 -> entry2 -> entry3
        // Verify all hashes link correctly
    }
    
    #[test]
    fn test_tamper_detection() {
        // Modify middle entry, verify chain breaks
    }
}
```

#### 1.3 Add Permission Tier Tests

**Priority**: High  
**Effort**: Medium

```rust
#[test]
fn test_t3_requires_totp() {
    // Attempt T3 action without TOTP - should fail
}

#[test]
fn test_tier_escalation_blocked() {
    // Attempt to call T3 skill with T2 token - should fail
}
```

---

### Phase 2: Security Hardening (Week 3-4)

#### 2.1 Secret Storage Enhancement

**Priority**: High  
**Effort**: Medium

**Current Issue**: Secrets in environment variables

**Improvement Options**:

| Option | Effort | Security | Recommendation |
|--------|--------|----------|----------------|
| OS Keyring | Medium | High | ✅ Recommended |
| HashiCorp Vault | High | High | For production |
| Encrypted file | Low | Medium | Quick win |

**Implementation**:
```rust
// Use keyring crate for OS keychain
use keyring::Entry;

pub fn get_secret(service: &str) -> Result<String, Error> {
    let entry = Entry::new(service, "apex")?;
    entry.get_password()
}
```

#### 2.2 Enhanced Rate Limiting

**Priority**: High  
**Effort**: Medium

**Current**: Single global limiter

**Improvements**:
1. Per-endpoint limiters
2. Progressive throttling (2nd, 3rd attempt = longer wait)
3. IP-based vs user-based limiting
4. Distributed rate limiting for NATS mode

```rust
pub struct EnhancedRateLimiter {
    global: RateLimiter,      // Per-IP global
    endpoint: HashMap<String, RateLimiter>,  // Per-endpoint
    progressive: HashMap<String, Vec<Instant>>,  // Attempt history
}
```

#### 2.3 TOTP Persistence & Backup Codes

**Priority**: Medium  
**Effort**: Medium

**Current Issue**: TOTP secrets lost on restart

**Improvement**:
```rust
pub async fn persist_totp_secret(&self, user_id: &str, secret: &str) -> Result<(), Error> {
    // Encrypt and store in database
}

pub async fn generate_backup_codes(&self, user_id: &str) -> Result<Vec<String>, Error> {
    // Generate 10 single-use backup codes
}
```

---

### Phase 3: Production Security (Week 5-8)

#### 3.1 Custom Seccomp Profile

**Priority**: High  
**Effort**: High

**Current**: No seccomp (Docker default)

**Implementation**:
```json
// apex-seccomp.json
{
    "defaultAction": "SCMP_ACT_ERRNO",
    "syscalls": [
        {"names": ["read", "write"], "action": "SCMP_ACT_ALLOW"},
        {"names": ["exit_group"], "action": "SCMP_ACT_ALLOW"},
        // Block dangerous syscalls
        {"names": ["kexec_load"], "action": "SCMP_ACT_ERRNO"}
    ]
}
```

#### 3.2 AppArmor/SELinux Profiles

**Priority**: Medium  
**Effort**: High

**Purpose**: Mandatory Access Control

```bash
# apex-docker/apparmor profile
profile apex-docker {
    # Allow read to /app
    /app/** r,
    
    # Allow tmpfs
    /tmp/** rw,
    
    # Deny all else
    deny /**,
}
```

#### 3.3 SIEM Integration

**Priority**: Medium  
**Effort**: High

**Current**: Local audit log only

**Improvement**:
```rust
pub async fn send_to_siem(&self, entry: &AuditEntry) -> Result<(), Error> {
    // Send to Splunk, ELK, or custom SIEM
}
```

---

### Phase 4: Security Audit (Week 9-12)

#### 4.1 External Penetration Testing

**Priority**: Critical  
**Effort**: High (outsourced)

**Scope**:
1. Authentication bypass attempts
2. Authorization escalation
3. Code execution escape
4. Data exfiltration
5. DoS attacks

**Deliverables**:
- Penetration test report
- Remediation plan
- Re-testing

#### 4.2 Code Review

**Priority**: Critical  
**Effort**: High

**Focus Areas**:
1. HMAC implementation
2. TOTP flow
3. Permission checks
4. Input validation
5. Database queries

#### 4.3 Formal Verification

**Priority**: Low  
**Effort**: Very High

**Optional**: Use model checkers (CBMC, KLEE) for critical functions

---

## Part 4: Security Audit Checklist

### Pre-Audit Preparation

- [ ] Document all security implementations
- [ ] Create asset inventory
- [ ] Identify trust boundaries
- [ ] Map data flows
- [ ] Document threat model

### Technical Review

- [ ] Review HMAC implementation for timing attacks
- [ ] Verify TOTP flow for vulnerabilities
- [ ] Test rate limiting effectiveness
- [ ] Verify input sanitization coverage
- [ ] Test execution isolation
- [ ] Review audit log integrity

### Penetration Testing

- [ ] Authentication bypass attempts
- [ ] Authorization escalation
- [ ] SQL injection testing
- [ ] Prompt injection testing
- [ ] Container escape attempts
- [ ] DoS attack simulation

### Documentation

- [ ] Update SECURITY.md with findings
- [ ] Create incident response plan
- [ ] Document security architecture
- [ ] Update threat model

---

## Part 5: Risk Assessment

### Current Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Prompt Injection | Medium | High | Input sanitization |
| Credential Theft | Medium | High | Keyring integration |
| Container Escape | Low | Critical | Seccomp, AppArmor |
| DoS Attack | Medium | Medium | Enhanced rate limiting |
| Data Exfiltration | Low | High | Network isolation |

### Risk Acceptance

| Risk | Accepted? | Rationale |
|------|-----------|-----------|
| Pre-alpha status | Yes | Documented warning |
| No formal audit | No | Schedule audit |
| Environment secrets | No | Implement keyring |

---

## Part 6: Implementation Timeline

```
Week 1-2:  Security Tests
├── Add input validation tests
├── Add audit chain tests  
├── Add permission tier tests
└── Target: 90% security test coverage

Week 3-4:  Security Hardening
├── Secret storage (keyring)
├── Enhanced rate limiting
├── TOTP persistence
└── Target: Pass basic pen test

Week 5-8:  Production Security
├── Custom seccomp profile
├── AppArmor profiles
├── SIEM integration
└── Target: Production-ready

Week 9-12: Security Audit
├── External penetration testing
├── Code review
├── Remediation
└── Target: Security certification
```

---

## Part 7: Success Metrics

### Security Test Coverage
- **Current**: 17 security tests
- **Target**: 100+ security tests
- **Coverage**: 90%+ of security code

### Security Hardening
- [ ] Secrets in keyring (not env vars)
- [ ] Per-endpoint rate limiting
- [ ] TOTP backup codes
- [ ] Custom seccomp profile
- [ ] AppArmor profile

### Audit Readiness
- [ ] External penetration test passed
- [ ] No critical findings
- [ ] SIEM integration working
- [ ] Incident response plan tested

---

## Appendix A: Security Test Inventory

### Existing Tests (17 total)

| File | Test Count | Coverage |
|------|------------|----------|
| auth.rs | 9 | HMAC |
| totp.rs | 5 | TOTP |
| rate_limiter.rs | 3 | Rate limiting |
| vm_pool.rs | 3 | Execution isolation |

### Missing Tests (Critical)

| Module | Tests Needed | Priority |
|--------|--------------|----------|
| mcp/validation.rs | 10+ | Critical |
| audit.rs | 5 | Critical |
| governance.rs | 8 | High |
| skill tier enforcement | 5 | High |

---

## Appendix B: Security Configuration

### Current Limits

```yaml
rate_limiting:
  global:
    requests_per_minute: 60
    burst_size: 10
  task_creation:
    requests_per_minute: 10
  skill_execution:
    requests_per_minute: 30
  deep_tasks:
    requests_per_minute: 5

execution:
  docker:
    memory: 2048m
    cpus: 2
    pids_limit: 256
    network: none
    read_only: true
    cap_drop: ALL
```

---

## Appendix C: Incident Response

### Contact List

| Role | Responsibility | Escalation |
|------|----------------|------------|
| Security Lead | Overall posture | - |
| Platform Engineer | Isolation issues | Security Lead |
| Backend Engineer | Auth/Authz issues | Security Lead |

### Response Times

| Severity | Response Time | Examples |
|----------|---------------|----------|
| Critical | < 1 hour | Escape, breach |
| High | < 4 hours | DoS, unauthorized |
| Medium | < 24 hours | Rate limiting |

---

*Document Status: Draft - Ready for Review*
*Next Review: After Phase 1 Completion*
