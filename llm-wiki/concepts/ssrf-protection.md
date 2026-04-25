# SSRF Protection

**Status:** Implemented
**Date:** 2026-04-25
**CVE:** CVE-2026-4308 (Agent Zero)

## Overview

SSRF (Server-Side Request Forgery) protection for the `web.fetch` tool. Inspired by Agent Zero's v1.9 security fix (CVE-2026-4308).

APEX was already protected against path traversal (CVE-2026-4307) via `vm_pool.rs`, `content_hash.rs`, and `injection_classifier.rs`. This implementation adds comprehensive SSRF protection.

## What Is Blocked

| Category | Examples | CIDR/Range |
|----------|----------|------------|
| Localhost | `localhost`, `127.0.0.1`, `::1` | Loopback |
| Private Networks | `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16` | RFC 1918 |
| Cloud Metadata | `169.254.169.254`, `metadata.google.internal` | AWS/GCP/Azure |
| IPv6 Private | `fc00::/7`, `fe80::/10` | ULA, link-local |
| IPv4-Mapped | `::ffff:10.x.x.x` | IPv4-in-IPv6 |

## Implementation

**File:** `execution/src/apex_agent/__init__.py`

### URL Validation
1. Parse URL into components (scheme, host, port)
2. Resolve hostname to IP via DNS
3. Check resolved IP against blocklist
4. Block CNAME redirection attacks (DNS resolution)

### Key Functions
- `is_blocked_ip(ip: str) -> bool` — Check against blocklist
- `validate_url(url: str) -> Result[str, SSRFError]` — Full validation
- `resolve_dns(hostname: str) -> list[str]` — Prevent CNAME bypass

## Tests

12 SSRF protection tests added to execution test suite:
- localhost variants
- private IP ranges
- cloud metadata endpoints
- IPv6 addresses
- DNS CNAME redirection

**Total execution tests:** 64 passing

## Related CVEs

| CVE | Description | APEX Status |
|-----|-------------|-------------|
| CVE-2026-4307 | Path traversal | ✅ Already protected |
| CVE-2026-4308 | SSRF | ✅ Implemented 2026-04-25 |

## Related

- [concepts/security.md](concepts/security.md)
- [project-overview.md](project-overview.md)