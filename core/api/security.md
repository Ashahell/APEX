# API Security (MVP)

- Objective: provide basic security hardening for the MVP API surface used by OpenFang adoption into APEX.
- Scope: HMAC-based auth for API requests, rate limiting, and endpoint-level access checks.
- MVP: a small API-security wrapper that validates HMAC signature and timestamp on requests; logs access in a simple audit log.
