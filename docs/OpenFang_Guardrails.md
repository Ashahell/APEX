OpenFang Adoption Guardrails

Date: 2026-03-25
Status: Draft

Purpose
- Establish guardrails to ensure security, compliance, and safe operation during adoption of OpenFang features into APEX.

Scope
- Applies to Hands, MCP, memory/embeddings, streaming, adapters, and API layers introduced by OpenFang adoption.

Core Guardrails
- Least privilege: enforce per-user/role access control for Hands and tools.
- Prompt safety: sanitize all prompts and screen captures; block dangerous input patterns.
- Auditability: maintain a verifiable audit trail for actions (Merkle-like structure) and ensure logs are tamper-evident where feasible.
- Isolation: execute Hands in sandboxed environments with strict network, file-system, and resource controls.
- Change control: major changes require governance sign-off, testing, and rollback plans.
- Rollback: have a clear rollback path for any risky adoption steps with runbooks.
- Data governance: retention, rotation, and access controls for memory embeddings and narrative memory.
- Incident response: escalation paths, runbooks, and post-incident reviews.

Enforcement
- Code-level checks, unit tests, and integration tests to enforce guardrails.
- Operational dashboards to monitor guardrail violations and audit signals.

Review cadence
- Guardrails and enforcement policies to be reviewed with governance cadence (Quarterly).

This document is a living artifact and will be refined as adoption progresses.
