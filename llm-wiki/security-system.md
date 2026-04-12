# Security System

## Overview

APEX implements a security-first architecture with multiple layers of protection.

## Components

### Authentication
- **HMAC Authentication**: Request signing with shared secret
- **TOTP Verification**: T3 tier requires 6-digit authenticator code

### Permission Tiers
| Tier | Access | Confirmation |
|------|--------|---------------|
| T0 | Read-only | None |
| T1 | File writes | Tap |
| T2 | External APIs | Type action |
| T3 | Destructive | TOTP |

### Security Features
- Input validation and injection detection
- Audit chain with SHA-256 hash chain
- Replay protection with signature store
- Rate limiting per endpoint
- Anomaly detection for runtime behavior

## Version

- Security implementation complete (Phases 0-13)
- Not formally penetration tested yet