# Schema Audit & Fixes

**Status:** Complete
**Date:** 2026-04-25
**Source:** [raw/APEX_Memory_System_Spec_v2.md](raw/APEX_Memory_System_Spec_v2.md)

## Critical Findings

### Fix 1: Audit Chain Deletion Bug
**Problem**: Deleting audit log entries broke the tamper-evident chain.
**Solution**: Archive entries instead of deleting. Add `archived_at` column.
**Files**: `core/memory/src/ttl_cleanup.rs`, `core/memory/src/db.rs`

### Fix 2: Weak Key Derivation
**Problem**: Simple password hashing for secrets store.
**Solution**: Argon2id + machine-specific token (machine-id + CPU info).
**Files**: `core/security/src/secret_store.rs`, `core/router/Cargo.toml` (argon2 crate)

### Fix 3: Missing Foreign Key Enforcement
**Problem**: No ON DELETE CASCADE, risk of orphaned records.
**Solution**: `PRAGMA foreign_keys = ON`, CASCADE DELETE on all FK constraints.
**Files**: `core/memory/src/db.rs`

### Fix 4: Encryption Not Default
**Problem**: Sensitive settings stored in plaintext by default.
**Solution**: Auto-encrypt all secret-type settings. Added encryption check in `settings.rs`.
**Files**: `core/memory/src/settings.rs`, `core/memory/src/db.rs`

## Test Results
- 69 tests passing (63 memory + 6 security)
- Schema audit tests: complete

## Related
- [concepts/architecture.md](concepts/architecture.md) — Security model