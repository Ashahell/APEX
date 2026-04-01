Security Audit Statement of Work (SOW)
====================================

Overview
- This document defines the scope, deliverables, timeline, and acceptance criteria for an external security audit of the APEX project.
- Objective: verify security controls, identify vulnerabilities, and provide remediation guidance to achieve production readiness.

Scope of Work
- Review code and dependencies across Rust (core) and TypeScript (gateway, skills) layers for security weaknesses, including but not limited to:
  - Dependency vulnerability analysis and licensing review
  - Secure configuration and secret handling
  - Input validation, sanitization, and output encoding
  - Authentication, authorization, and audit capabilities
  - Containerization and runtime security (Docker/seccomp/AppArmor)
  - Logging, observability, and incident response traces
- Perform penetration testing on exposed surfaces (APIs, endpoints, and gRPC-like channels) as scoped in the engagement.
- Validate security hardening measures implemented in Phase 1 (seccomp/AppArmor placeholders, non-root containers, read-only FS, capabilities dropped).
- Provide remediation guidance and verify fixes in a follow-on re-test.

Deliverables
- Security Assessment Report with findings categorized by severity (Critical, High, Medium, Low).
- Remediation Plan with prioritized fixes, responsible owners, and estimated effort.
- Evidence pack including build logs, vulnerability reports, and test artifacts.
- Re-test Verification Report confirming remediation effectiveness.

Timeline & Milestones
- Kickoff: [Date]
- Interim Findings: [Date]
- Remediation Window: [Date] – [Date]
- Re-test Window: [Date] – [Date]
- Final Report & Sign-off: [Date]


Engagement Details
- Information provided by APEX team: repository access, build artifacts, container configurations, CI/CD pipelines, and runbooks.
- Security firm: [To be decided]
- Contact: [Primary security lead] <email>

Acceptance Criteria
- All high/severity issues resolved or mitigated to an agreed acceptable level.
- Documentation updated in docs/SECURITY_AUDIT_SOW.md with remediation evidence.
- Re-test demonstrates no unresolved critical issues.

Notes
- This SOW is designed to align with the current parity baseline (v2.0.0) and production-readiness commitments.
