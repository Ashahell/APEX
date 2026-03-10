# APEX Security Threat Model

> **Version**: 1.0
> **Date**: 2026-03-09
> **Status**: Pre-Audit Preparation

---

## 1. Executive Summary

APEX is a single-user autonomous agent platform with security-first design. This document outlines the threat model for the platform, identifying assets, threats, and mitigations.

### Scope
- **In Scope**: All APEX components (Router, Skills, Memory, Execution)
- **Out of Scope**: External MCP servers, user's local environment

---

## 2. System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         APEX Platform                            │
├─────────────────────────────────────────────────────────────────┤
│  L1 Gateway    │  REST API + HMAC Auth                        │
│  L2 Router     │  Task routing, Classification                 │
│  L3 Memory     │  SQLite + Vector Store                        │
│  L4 Skills     │  33 Skills (T0-T3 tiers)                      │
│  L5 Execution  │  Docker/Firecracker Isolation                │
│  L6 UI         │  React Frontend                              │
└─────────────────────────────────────────────────────────────────┘
```

### Trust Boundaries

| Boundary | Description | Trust Level |
|----------|-------------|-------------|
| User → Gateway | HTTP API with HMAC | High |
| Gateway → Router | Internal (localhost) | High |
| Router → Skills | IPC via Skill Pool | Medium |
| Router → Execution | VM Pool (Docker/Firecracker) | Low (isolated) |

---

## 3. Assets

### 3.1 Data Assets

| Asset | Classification | Description |
|-------|---------------|-------------|
| User Tasks | Confidential | Task input/output, user queries |
| Memory Data | Confidential | Journal, reflections, entities |
| Credentials | Secret | HMAC secret, TOTP seeds |
| Capability Tokens | Confidential | Permission grants |

### 3.2 System Assets

| Asset | Description |
|-------|-------------|
| Task Router | Central orchestration |
| Skill Pool | Skill execution |
| VM Pool | Code execution isolation |
| Database | Persistence layer |

---

## 4. Threat Actors

| Actor | Motivation | Capability |
|-------|------------|------------|
| User (Owner) | Productivity | Full access |
| External Attacker | Data theft, Code execution | Network access |
| Malicious Input | Inject malicious prompts | API access |

---

## 5. Identified Threats

### T1: Prompt Injection

**Description**: Attacker crafts input to manipulate agent behavior

**Attack Vectors**:
- Direct input in chat
- Task content
- Skill parameters

**Current Mitigations**:
- Input sanitization for MCP tools
- T3 verification for destructive skills

**Risk Level**: Medium

**Recommendations**:
- [ ] Implement prompt injection detection
- [ ] Add input/output filtering
- [ ] Log all prompt modifications

---

### T2: Unauthorized Skill Execution

**Description**: Attacker executes skills without proper authorization

**Attack Vectors**:
- Direct API calls
- Capability token reuse
- Tier escalation

**Current Mitigations**:
- HMAC authentication on all APIs
- T0-T3 permission tiers
- Capability token expiration

**Risk Level**: Low

---

### T3: Code Execution Escape

**Description**: Attacker escapes Docker/Firecracker isolation

**Attack Vectors**:
- Malicious skill code
- Container breakout
- Kernel exploits

**Current Mitigations**:
- Docker: `--network none`, `--read-only`, `--cap-drop ALL`
- Firecracker: MicroVM isolation (when available)

**Risk Level**: High (if escape achieved)

**Recommendations**:
- [ ] Custom seccomp profile
- [ ] AppArmor/SELinux profiles
- [ ] Network namespace isolation
- [ ] Regular CVE patching

---

### T4: Credential Theft

**Description**: Attacker obtains HMAC secret or TOTP seeds

**Attack Vectors**:
- Environment variable access
- Memory dump
- Log exposure

**Current Mitigations**:
- Secrets in environment variables (not ideal)
- No logging of secrets

**Risk Level**: High

**Recommendations**:
- [ ] Encrypted secret storage (vault, keyring)
- [ ] Secret rotation support
- [ ] Memory-safe secret handling

---

### T5: Data Exfiltration

**Description**: Attacker extracts memory data

**Attack Vectors**:
- Vector store queries
- Journal access
- File system access

**Current Mitigations**:
- Single-user architecture
- No external network from execution

**Risk Level**: Medium

---

### T6: Denial of Service

**Description**: Attacker overwhelms system resources

**Attack Vectors**:
- Rapid task creation
- Large inputs
- Infinite loops in skills

**Current Mitigations**:
- Rate limiting (basic)
- Task budgets

**Risk Level**: Medium

**Recommendations**:
- [ ] Enhanced rate limiting per endpoint
- [ ] Input size limits
- [ ] Timeout enforcement

---

## 6. Security Controls

### 6.1 Authentication

| Control | Status | Implementation |
|---------|--------|----------------|
| HMAC Request Signing | ✅ Implemented | `auth.rs` |
| Timestamp validation | ✅ Implemented | 5-minute window |
| Capability tokens | ✅ Implemented | Expirable, tiered |

### 6.2 Authorization

| Control | Status | Implementation |
|---------|--------|----------------|
| Permission tiers | ✅ Implemented | T0-T3 |
| Skill restrictions | ✅ Implemented | Via capability tokens |
| MCP tool validation | ✅ Implemented | `validation.rs` |

### 6.3 Execution Isolation

| Control | Status | Implementation |
|---------|--------|----------------|
| Docker | ✅ Default | `--network none`, etc. |
| Firecracker | 🔧 WSL2 | Ready for testing |
| gVisor | 🔧 Not configured | Future |

### 6.4 Input Validation

| Control | Status | Implementation |
|---------|--------|----------------|
| MCP sanitization | ✅ Implemented | 10-level nesting, 100KB limit |
| Skill input schema | ✅ Implemented | Zod validation |
| Task classification | ✅ Implemented | `classifier.rs` |

---

## 7. Recommended Security Improvements

### Priority 1 (Before Production)

| Improvement | Effort | Impact |
|-------------|--------|--------|
| Encrypted secret storage | Medium | High |
| Enhanced rate limiting | Low | Medium |
| Custom seccomp profile | Medium | High |

### Priority 2 (Post-Audit)

| Improvement | Effort | Impact |
|-------------|--------|--------|
| Audit log hash chain | Medium | High |
| Session management | Medium | Medium |
| AppArmor profiles | High | Medium |

---

## 8. Audit Checklist

- [ ] Review HMAC implementation
- [ ] Verify TOTP flow
- [ ] Test Docker isolation
- [ ] Pen test authentication
- [ ] Review input validation
- [ ] Check logging for secrets
- [ ] Verify rate limiting
- [ ] Test skill execution boundaries

---

## 9. Incident Response

### Critical (Response < 1 hour)
- Escape from isolation
- Credential compromise
- Data breach

### High (Response < 4 hours)
- DoS attack
- Unauthorized skill execution
- Prompt injection detected

### Medium (Response < 24 hours)
- Rate limit violations
- Invalid input attempts
- Anomalous behavior

---

## 10. Security Contacts

| Role | Responsibility |
|------|----------------|
| Security Lead | Overall security posture |
| Platform Engineer | Isolation, execution security |
| Backend Engineer | Authentication, authorization |

---

*This document will be updated after security audit.*
