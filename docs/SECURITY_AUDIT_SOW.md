# Statement of Work: APEX Security Audit

## 1. Project Overview

**Client**: APEX Project (Ashahell/APEX on GitHub)
**Project**: Comprehensive Security Audit & Penetration Testing
**Duration**: 4 weeks (2 weeks testing + 2 weeks remediation support)
**Start Date**: TBD
**System Type**: Single-user autonomous agent platform (Rust backend, React frontend, Python execution sandbox)

---

## 2. Scope of Work

### 2.1 In-Scope Components

| Component | Technology | Description |
|-----------|-----------|-------------|
| **API Surface** | Rust (Axum) | REST API with HMAC authentication, 50+ endpoints |
| **Authentication** | HMAC-SHA256 + TOTP | Request signing, time-based OTP verification |
| **Authorization** | T0-T3 Permission Tiers | Read-only → Tap confirm → Type confirm → TOTP verify |
| **Input Validation** | Rust | MCP sanitization, injection detection, replay protection |
| **Execution Sandbox** | Python + Docker | Dynamic code execution with import allowlisting |
| **Data Storage** | SQLite | Task data, memory, audit chain, encrypted secrets |
| **Secret Management** | AES-256-GCM | Encrypted key-value store for credentials |
| **Real-time Channels** | WebSocket + SSE | Streaming events, live updates |
| **Container Deployment** | Docker Compose | Multi-service deployment (router, UI, embedding, LLM) |

### 2.2 Out-of-Scope Components

- Third-party LLM providers (llama.cpp, external APIs)
- External messaging adapters (Slack, Discord, Telegram)
- Moltbook federated network (separate project)
- User's local environment configuration

---

## 3. Testing Methodology

### 3.1 Attack Vectors

| Category | Tests | Priority |
|----------|-------|----------|
| **Authentication Bypass** | HMAC forgery, TOTP bypass, replay attacks, timestamp manipulation | Critical |
| **Authorization Escalation** | T0→T3 escalation, permission tier bypass, role confusion | Critical |
| **Injection Attacks** | LLM prompt injection, SQL injection, command injection, XSS, path traversal | Critical |
| **Sandbox Escape** | Python import bypass, Docker breakout, filesystem access, network access | Critical |
| **Data Exposure** | Secret store extraction, audit chain tampering, SQLite file access | High |
| **Denial of Service** | Rate limit bypass, resource exhaustion, memory/CPU abuse | High |
| **Supply Chain** | Dependency vulnerabilities, Docker image CVEs, npm/Rust crate issues | Medium |
| **Configuration** | Environment variable leakage, default credentials, debug endpoints | Medium |

### 3.2 Testing Approach

1. **Automated Scanning**: cargo audit, npm audit, Trivy image scanning, dependency-check
2. **Manual Testing**: Code review, logic analysis, attack simulation
3. **Dynamic Testing**: API fuzzing, WebSocket/SSE testing, sandbox escape attempts
4. **Social Engineering**: N/A (single-user system, no multi-tenancy)

---

## 4. Deliverables

### 4.1 Required Deliverables

| Deliverable | Format | Due Date |
|-------------|--------|----------|
| **Audit Plan** | PDF document | Week 1, Day 3 |
| **Draft Findings Report** | PDF + JSON | Week 2, Day 5 |
| **Final Findings Report** | PDF + JSON | Week 3, Day 3 |
| **Remediation Verification** | PDF document | Week 4, Day 5 |
| **Executive Summary** | 1-page PDF | Week 4, Day 5 |

### 4.2 Report Contents

Each findings report must include:
- **Vulnerability description** with CVSS v3.1 score
- **Attack scenario** with step-by-step reproduction
- **Impact assessment** (confidentiality, integrity, availability)
- **Remediation recommendation** with code examples where applicable
- **Evidence** (screenshots, logs, proof-of-concept code)
- **Risk rating**: Critical / High / Medium / Low / Informational

---

## 5. Client Responsibilities

### 5.1 Pre-Audit Preparation

- [ ] Provide read-only access to GitHub repository
- [ ] Deploy staging environment matching production configuration
- [ ] Provide API documentation and architecture diagrams
- [ ] Share threat model and security assumptions
- [ ] Provide test credentials and TOTP secrets
- [ ] Ensure staging environment is isolated from production

### 5.2 During Audit

- [ ] Designate technical point of contact (available during business hours)
- [ ] Respond to clarification requests within 24 hours
- [ ] Provide additional code/context as requested
- [ ] Review draft findings within 3 business days

### 5.3 Post-Audit

- [ ] Implement remediation for Critical/High findings within 2 weeks
- [ ] Implement remediation for Medium findings within 4 weeks
- [ ] Provide remediation evidence for re-verification
- [ ] Schedule follow-up call to discuss findings

---

## 6. Timeline

| Week | Activities | Milestones |
|------|-----------|------------|
| **Week 1** | Kickoff, environment setup, automated scanning, code review | Audit Plan delivered |
| **Week 2** | Manual testing, attack simulation, sandbox escape attempts | Draft Findings Report delivered |
| **Week 3** | Client remediation, re-testing of critical findings | Final Findings Report delivered |
| **Week 4** | Final verification, executive summary, knowledge transfer | Remediation Verification delivered |

---

## 7. Success Criteria

- [ ] All Critical and High findings identified and documented
- [ ] Proof-of-concept provided for each finding
- [ ] Remediation recommendations are actionable and tested
- [ ] Final report includes executive summary for non-technical stakeholders
- [ ] Knowledge transfer session completed with engineering team
- [ ] Zero Critical/High findings remaining after remediation verification

---

## 8. Exclusions & Limitations

- **No production testing**: All testing must occur in staging environment
- **No destructive testing**: Do not delete data, corrupt databases, or cause service outages
- **No social engineering**: Single-user system has no multi-tenancy attack surface
- **No physical security**: Out of scope for this engagement
- **Rate limiting**: Do not exceed 100 requests/second during testing
- **Data handling**: Do not exfiltrate or store any test data beyond the engagement

---

## 9. Acceptance Criteria

The engagement is complete when:
1. All deliverables are submitted and accepted
2. Critical/High findings are remediated and verified
3. Executive summary is approved by project owner
4. Knowledge transfer session is completed
5. Final invoice is submitted and processed

---

## 10. Signatures

**Client**: ________________________ Date: __________
**Auditor**: _______________________ Date: __________

---

*This SOW is based on the APEX v2.0.0 codebase as of 2026-03-31.*
