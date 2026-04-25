# APEX Schema Audit Report

**Date:** 2026-04-25  
**Auditor:** OpenCode (Schema Analysis)  
**Scope:** Database migrations 001-024

---

## Executive Summary

| Category | Rating | Notes |
|----------|--------|-------|
| Schema Design | ⚠️ Mixed | Functional but inconsistent patterns |
| Security | ❌ Weak | Critical flaws in encryption and audit |
| Data Integrity | ❌ Broken | Audit chain deletable, FK gaps |
| Performance | ⚠️ Partial | Indexes exist but gaps remain |

---

## Critical Findings

### 1. Audit Chain Deletion Bug (CRITICAL)

**File:** `core/memory/src/ttl_cleanup.rs:76-86`

```rust
async fn delete_old_audit_logs(&self, days: i32) -> Result<i64, String> {
    let result = sqlx::query(
        "DELETE FROM audit_log WHERE timestamp < datetime('now', ?)"
    )
    // ...
}
```

**Problem:** The TTL cleanup feature DELETES from `audit_log` table. This breaks the hash chain - if you delete audit entries in the middle of the chain, `prev_hash` references become invalid, and `verify_chain()` will fail or produce false positives.

**Impact:** The "tamper-evident" audit log can be rendered useless by routine cleanup. An attacker who gains access could delete old entries to hide tampering.

**Recommendation:** 
- Never delete from audit_log
- Use archival (move to separate table) instead of deletion
- Or accept that audit_log is append-only and remove the TTL config for it

---

### 2. Weak Key Derivation for Secrets (CRITICAL)

**File:** `core/security/src/secret_store.rs:69-100`

```rust
fn derive_machine_key() -> [u8; 32] {
    let mut hasher = DefaultHasher::new();
    
    // hostname - easily discoverable
    if let Ok(hostname) = std::env::var("COMPUTERNAME") {
        hostname.hash(&mut hasher);
    }
    // username - easily discoverable
    if let Ok(username) = std::env::var("USERNAME") {
        username.hash(&mut hasher);
    }
    // OS/ARCH - constant for all deployments
    std::env::consts::OS.hash(&mut hasher);
    std::env::consts::ARCH.hash(&mut hasher);
    
    // Expand to 32 bytes via simple byte duplication
    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&hash.to_le_bytes());
    // ... same pattern repeated
}
```

**Problems:**
1. `DefaultHasher` is NOT a cryptographic hash function
2. Hostname + username are trivially discoverable
3. Key derivation is deterministic - same machine = same key
4. No salt, no PBKDF2, no Argon2
5. Single key encrypts ALL secrets - compromise = total compromise

**Impact:** If attacker knows your hostname and username, they can derive your encryption key and decrypt all secrets.

**Recommendation:**
- Use proper KDF (Argon2, PBKDF2, or at minimum SHA-256 with random salt stored alongside ciphertext)
- Derive unique keys per secret or use key wrapping
- Consider hardware security module integration

---

### 3. Missing Foreign Key Constraints

**Issue:** SQLite foreign keys are NOT enforced by default.

Looking at migrations, tables define `FOREIGN KEY` but there's no `PRAGMA foreign_keys = ON;`

**Example from 001_initial.sql:**
```sql
FOREIGN KEY (task_id) REFERENCES tasks(id)
```

**Problem:** If `tasks` row is deleted, orphaned `messages` records remain. This is called "weak" referential integrity in SQLite - it exists syntactically but not enforced.

**Recommendation:** Enable foreign keys in connection:
```sql
PRAGMA foreign_keys = ON;
```

---

### 4. Inconsistent ID Generation

| Table | ID Type | Generation |
|-------|---------|-------------|
| tasks | TEXT | ULID (likely) |
| messages | TEXT | ULID |
| audit_log | INTEGER | AUTOINCREMENT |
| secret_refs | TEXT | Application |
| memory_chunks | TEXT | Application |

**Issue:** Mixed approaches cause:
- audit_log uses AUTOINCREMENT which can have race conditions in concurrent writes
- No consistent strategy for distributed scenarios
- TEXT IDs with no length constraints

**Recommendation:** Standardize on ULID for all string IDs, use BIGINT with auto-increment for audit log if ordering matters.

---

### 5. No Unique Constraints on Natural Keys

**Examples:**
- `messages.author` - TEXT with no uniqueness
- `messages.channel` - TEXT with no uniqueness  
- `secret_refs.secret_name` - has UNIQUE, but most tables don't

**Impact:** Duplicate entries possible (e.g., same message from same author).

---

## Schema Issues by Migration

### 001_initial.sql

**Good:**
- Basic schema is reasonable
- Indexes on common query fields

**Issues:**
- `datetime('now')` uses SQLite local time, not UTC - potential timezone bugs
- No `NOT NULL` on some columns that should require it (e.g., `messages.content`)

### 004_integer_timestamps.sql → 009_timestamp_integer.sql

**Finding:** Migrated from string timestamps to integers but some tables still use strings.

**Impact:** Inconsistent query patterns, performance issues comparing across tables.

### 013_enhanced_memory.sql

**Good:**
- FTS5 with triggers for sync
- BM25 for ranking

**Issues:**
- `memory_chunks` has no unique constraint on `id` (it's TEXT, created manually)
- No validation that chunk_index is correct

### 021_secrets_expansion.sql

**Finding:** 64 predefined secrets inserted via INSERT OR IGNORE.

**Issues:**
- Predefined secrets assume future needs - rigid
- If a predefined secret is not used, it's dead data
- `targets` field is JSON stored as TEXT - not queryable

---

## Performance Concerns

### Missing Indexes

```sql
-- messages table - common queries that lack indexes:
-- No index on channel + created_at (time-range queries by channel)
-- No index on author (who is most active?)
-- No index on role (filter by user/assistant)

-- tasks table
-- No composite index on (status, created_at) for dashboard queries
-- No index on skill_name
-- No index on tier + status combinations

-- audit_log  
-- No index on timestamp (ordering by time is common but uses id)
```

### Vector Search

**Issue:** sqlite_vec extension is loaded but:
- No evidence of vector index creation
- Embedding dimension not validated
- No HNSW or IVM indexes visible

---

## Security Gaps

### 1. Preferences Table

```sql
CREATE TABLE preferences (
    key TEXT PRIMARY KEY,
    value TEXT,  -- Encrypted flag exists but value is plaintext by default!
    encrypted INTEGER DEFAULT 0,
    updated_at TEXT
);
```

**Problem:** `encrypted` defaults to 0 (false), but many entries will be stored unencrypted.

### 2. Channel Credentials

```sql
CREATE TABLE channel_settings (
    -- ...
    credentials_encrypted: Option<String>,  -- Often stored unencrypted!
);
```

**Evidence:** Migration 010 has this field but encryption is optional.

### 3. No Encryption at Rest

**Issue:** SQLite database file itself is unencrypted. If attacker gets file access, they read everything.

**Note:** There's a `preferences.encrypted` field, but:
1. It's opt-in (defaults to false)
2. The key derivation is broken (see Finding #2)

---

## Data Quality Issues

### 1. Nullable Text Fields Without Defaults

```sql
output_content TEXT,  -- Nullable but no default
error_message TEXT,    -- Nullable but no default
attachments TEXT,     -- Nullable but stores JSON!
```

**Issue:** Code must handle NULL, empty string, and "{}" differently.

### 2. JSON as Text

**Examples:**
- `secret_refs.targets` - JSON in TEXT column
- `working_memory.entities_json` - JSON in TEXT column
- `memory_chunks` - stores content but attributes are separate JSON

**Problem:** 
- Can't query JSON fields with SQL
- No validation that JSON is valid
- Schema doesn't reflect data structure

### 3. Denormalized Data

`messages.attachments` stores attachments inline rather than a separate table.

---

## Recommendations Priority

### P0 (Critical - Fix Now)

1. **Remove audit_log deletion** - Archive instead of delete, or disable TTL for audit
2. **Fix key derivation** - Use proper KDF, not hostname hashing
3. **Enable FK constraints** - Add `PRAGMA foreign_keys = ON;`

### P1 (High - Next Sprint)

4. Standardize ID generation (ULID everywhere)
5. Add missing indexes on common query patterns
6. Encrypt preferences by default or remove the field

### P2 (Medium - Backlog)

7. Migrate JSON columns to proper tables
8. Add unique constraints on natural keys
9. Use UTC consistently for timestamps
10. Add data validation at API layer

---

## Test Coverage Assessment

From AGENTS.md: "461 tests"

**What tests exist:**
- Unit tests for audit hash chain (audit.rs:494 lines of tests)
- Integration tests for API endpoints
- Security tests for sandbox, input validation, permission tiers

**What's NOT tested:**
- TTL cleanup with audit chain (would fail!)
- Key derivation weakness (would expose the issue)
- Concurrent writes to audit_log
- Migration rollback scenarios
- Foreign key violations

**Assessment:** Test coverage looks comprehensive for happy paths but misses critical failure modes.

---

## Conclusion

The APEX schema has good structure (FTS5, vector store, audit chain concept) but critical security and integrity flaws:

1. **Audit chain is broken** - deletable, defeating its purpose
2. **Secret encryption is weak** - effectively obfuscation, not encryption  
3. **Foreign keys not enforced** - data drift will occur
4. **No encryption at rest** - database file is plaintext

These are not minor issues - they are fundamental security architecture problems that should be addressed before any production use.

**Verdict:** Functional for development/prototyping. Not ready for production with sensitive data.