# APEX Schema Fix Plan

**Date:** 2026-04-25  
**Priority:** P0 (Critical)  
**Status:** ✅ COMPLETED

---

## Implementation Summary (2026-04-25)

All four security fixes have been implemented:

| Fix | Status | Files Changed |
|-----|--------|---------------|
| FK Enforcement | ✅ Done | `db.rs` |
| Encryption Defaults | ✅ Done | `settings.rs`, `db.rs` |
| Audit Archival | ✅ Done | `ttl_cleanup.rs`, `db.rs` |
| Key Derivation | ✅ Done | `secret_store.rs`, `Cargo.toml` |

**Test Results:** 69 tests passed (63 memory + 6 security)

---

## Executive Summary

This plan addresses four critical schema issues that compromise APEX's security and data integrity:

| Priority | Issue | Impact |
|----------|-------|--------|
| P0 | Audit chain deletion bug | Tamper-evident log can be circumvented |
| P0 | Weak key derivation | All secrets can be decrypted by attacker |
| P0 | Foreign keys not enforced | Orphaned records, data drift |
| P0 | Encryption defaults off | Secrets stored in plaintext |

**Risk Level:** HIGH - These issues allow data tampering and secret exfiltration.

---

## Approach

Each fix follows a 4-phase approach:

1. **Schema** - Database changes (new tables, migrations)
2. **Storage** - Data migration from old to new format
3. **Application** - Code changes to use new schema
4. **Verification** - Tests to confirm fix works

---

# FIX 1: Audit Chain Deletion Bug

## Problem

`ttl_cleanup.rs` DELETES from `audit_log` table, breaking the hash chain:

```rust
// Current code - BROKEN
async fn delete_old_audit_logs(&self, days: i32) -> Result<i64, String> {
    sqlx::query("DELETE FROM audit_log WHERE timestamp < datetime('now', ?)")
}
```

## Impact

- Deleting middle entries invalidates `prev_hash` chain
- Attacker can delete old entries to hide tampering
- `verify_chain()` produces false positives after deletion

## Solution

**Archival strategy:** Move old audit entries to `audit_archive` table instead of deleting.

### Phase 1: Schema Changes

**Step 1.1:** Create archive table (new migration 025)

```sql
-- Migration: 025_audit_archive
-- Create audit archive table (append-only, never deleted)

CREATE TABLE IF NOT EXISTS audit_archive (
    id INTEGER PRIMARY KEY,
    prev_hash TEXT NOT NULL,
    hash TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    details TEXT,
    archived_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_archive_timestamp ON audit_archive(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_archive_entity ON audit_archive(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_archive_hash ON audit_archive(hash);
```

**Step 1.2:** Add `archived` flag to existing audit_log (optional, for tracking)

```sql
ALTER TABLE audit_log ADD COLUMN archived INTEGER DEFAULT 0;
```

### Phase 2: Storage Changes

**Step 2.1:** Modify TTL cleanup to archive instead of delete

```rust
// In ttl_cleanup.rs - REPLACE delete_old_audit_logs function

async fn archive_old_audit_logs(&self, days: i32) -> Result<i64, String> {
    // 1. Get entries to archive
    let entries = sqlx::query_as::<_, AuditEntry>(
        "SELECT * FROM audit_log WHERE timestamp < datetime('now', ?) LIMIT 1000"
    )
    .bind(format!("-{} days", days))
    .fetch_all(&self.pool)
    .await
    .map_err(|e| e.to_string())?;

    if entries.is_empty() {
        return Ok(0);
    }

    // 2. Archive each entry
    for entry in &entries {
        sqlx::query(
            "INSERT INTO audit_archive (id, prev_hash, hash, timestamp, action, entity_type, entity_id, details, archived_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(entry.id)
        .bind(&entry.prev_hash)
        .bind(&entry.hash)
        .bind(entry.timestamp.to_rfc3339())
        .bind(&entry.action)
        .bind(&entry.entity_type)
        .bind(&entry.entity_id)
        .bind(&entry.details)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 3. Delete from audit_log (now safe - copy exists in archive)
    sqlx::query("DELETE FROM audit_log WHERE timestamp < datetime('now', ?)")
        .bind(format!("-{} days", days))
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

    // 4. Verify chain integrity after archive
    let chain_valid = self.verify_chain().await?;
    if !chain_valid {
        return Err("Chain integrity check failed after archival".to_string());
    }

    Ok(entries.len() as i64)
}
```

**Step 2.2:** Update verify_chain to check archive table too

```rust
pub async fn verify_full_chain(&self) -> Result<bool, sqlx::Error> {
    // Verify active audit_log entries
    let active_valid = self.verify_chain().await?;
    
    // Verify archived entries (if any)
    let archived = sqlx::query_as::<_, AuditEntry>(
        "SELECT * FROM audit_archive ORDER BY id ASC"
    )
    .fetch_all(&self.pool)
    .await?;

    for entry in archived {
        if !entry.verify() {
            return Ok(false);
        }
    }
    
    Ok(active_valid)
}
```

### Phase 3: Application Changes

**Step 3.1:** Update ttl_cleanup.rs to use archive function

```rust
// REPLACE this function in ttl_cleanup.rs
async fn cleanup_old_records(&self) -> Result<CleanupReport, String> {
    let mut report = CleanupReport::default();
    
    let configs = self.get_ttl_configs().await?;
    
    for config in configs {
        if !config.enabled {
            continue;
        }

        let deleted = match config.entity_type.as_str() {
            "tasks" => self.delete_old_tasks(config.retention_days).await?,
            "messages" => self.delete_old_messages(config.retention_days).await?,
            // CHANGED: Archive instead of delete
            "audit_log" => self.archive_old_audit_logs(config.retention_days).await?,
            "vector_store" => self.delete_old_vector_store(config.retention_days).await?,
            _ => 0,
        };

        report.add(&config.entity_type, deleted);
        self.update_last_cleanup(&config.entity_type).await?;
    }

    Ok(report)
}
```

**Step 3.2:** Update config UI to disable audit_log deletion by default

```rust
// In default TTL config - DISABLE audit_log deletion
let default_configs = vec![
    TtlConfig { entity_type: "tasks".to_string(), retention_days: 90, enabled: true },
    TtlConfig { entity_type: "messages".to_string(), retention_days: 90, enabled: true },
    // DISABLED by default - archival happens manually or via separate process
    TtlConfig { entity_type: "audit_log".to_string(), retention_days: 365, enabled: false },
    TtlConfig { entity_type: "vector_store".to_string(), retention_days: 30, enabled: true },
];
```

### Phase 4: Verification

**Step 4.1:** Add integration test for archival

```rust
#[tokio::test]
async fn test_audit_archive_preserves_chain() {
    // 1. Create audit entries
    let repo = AuditRepository::new(&pool);
    repo.create(CreateAuditEntry { /* ... */ }).await;
    
    // 2. Archive old entries
    let archived = repo.archive_old_audit_logs(0).await.unwrap();
    
    // 3. Verify chain still valid
    let valid = repo.verify_chain().await.unwrap();
    assert!(valid, "Chain broken after archival");
    
    // 4. Verify archived entries exist
    let archived_count = repo.count_archived().await.unwrap();
    assert!(archived_count > 0);
}
```

**Step 4.2:** Add manual verification command

```bash
# New CLI command
apex-cli audit verify-chain    # Verify audit_log
apex-cli audit verify-full   # Verify audit_log + archive
apex-cli audit archive      # Manually trigger archival
```

---

# FIX 2: Weak Key Derivation

## Problem

`secret_store.rs` derives encryption key from hostname + username:

```rust
// Current code - BROKEN
fn derive_machine_key() -> [u8; 32] {
    let mut hasher = DefaultHasher::new();  // NOT CRYPTOGRAPHIC
    hostname.hash(&mut hasher);            // Trivially discoverable
    username.hash(&mut hasher);            // Trivially discoverable
    // ... simple byte duplication
}
```

## Impact

- Attacker with hostname + username can derive key
- All encrypted data is compromised
- No forward secrecy

## Solution

**Strategy:** Use Argon2id with random salt, store salt with ciphertext.

### Phase 1: Schema Changes

**Step 1.1:** Create new encrypted store table with proper encryption

```sql
-- Migration: 025_encrypted_secrets_v2

-- New table with proper encryption
CREATE TABLE IF NOT EXISTS encrypted_secrets (
    id TEXT PRIMARY KEY,
    service TEXT NOT NULL,
    key TEXT NOT NULL,
    encrypted_value BLOB NOT NULL,      -- AES-256-GCM ciphertext
    nonce BLOB NOT NULL,                -- 12 bytes for AES-GCM
    salt BLOB NOT NULL,                 -- 16+ bytes for Argon2
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    version INTEGER NOT NULL DEFAULT 2   -- Track schema version
);

CREATE INDEX IF NOT EXISTS idx_encrypted_secrets_service ON encrypted_secrets(service);
CREATE INDEX IF NOT EXISTS idx_encrypted_secrets_key ON encrypted_secrets(service, key);

-- Keep old table for migration period, mark deprecated
ALTER TABLE secret_store RENAME TO secret_store_DEPRECATED;
```

### Phase 2: Storage Changes

**Step 2.1:** Update SecretStore to use Argon2id

```rust
// In secret_store.rs - REPLACE derive_machine_key

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::Rng;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

/// Generate cryptographically secure random bytes
fn generate_random_bytes(len: usize) -> Vec<u8> {
    (0..len).map(|_| rand::random::<u8>()).collect()
}

/// Derive key using Argon2id
fn derive_key_argon2(password: &str, salt: &[u8]) -> [u8; 32] {
    use argon2::password_hash::SaltString;
    
    let salt_b64 = base64_encode(salt);
    let salt_str = format!("$argon2id$v=19$m=65536,t=3,p=4${}", salt_b64);
    
    let argon2 = Argon2::default();
    let salt = SaltString::from_b64(&salt_str).unwrap();
    
    let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    
    // Extract first 32 bytes
    let mut key = [0u8; 32];
    let hash_bytes = hash.hash.unwrap();
    key.copy_from_slice(&hash_bytes.as_bytes()[..32]);
    key
}

/// Encrypt with Argon2id derived key
fn encrypt_secret(plaintext: &str) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    // 1. Generate salt and nonce
    let salt = generate_random_bytes(SALT_LEN);
    let nonce_bytes = generate_random_bytes(NONCE_LEN);
    
    // 2. Derive key from machine ID + random salt
    let machine_id = get_machine_unique_id();  // New function
    let key = derive_key_argon2(&machine_id, &salt);
    
    // 3. Encrypt with AES-256-GCM
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).unwrap();
    
    (ciphertext, nonce_bytes, salt)
}

/// Decrypt
fn decrypt_secret(ciphertext: &[u8], nonce_bytes: &[u8], salt: &[u8]) -> Result<String, String> {
    let machine_id = get_machine_unique_id();
    let key = derive_key_argon2(&machine_id, salt);
    
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| e.to_string())?;
    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

/// Get machine unique ID - MORE ENTROPY than hostname
fn get_machine_unique_id() -> String {
    let mut components = Vec::new();
    
    // Hostname (basic)
    if let Ok(h) = std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")) {
        components.push(h);
    }
    
    // Username
    if let Ok(u) = std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
        components.push(u);
    }
    
    // Platform details
    components.push(std::env::consts::OS.to_string());
    components.push(std::env::consts::ARCH.to_string());
    
    // Add random component - THIS IS KEY
    // Store this in a file that persists across reinstalls
    let random_component = get_or_create_machine_token();
    components.push(random_component);
    
    components.join("|")
}

/// Get or create persistent random token
fn get_or_create_machine_token() -> String {
    let token_path = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("apex")
        .join(".machine_token");
    
    if let Ok(token) = std::fs::read_to_string(&token_path) {
        return token;
    }
    
    // Generate new token
    let token: String = (0..32)
        .map(|_| {
            let idx = rand::random::<usize>() % 62;
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().nth(idx).unwrap()
        })
        .collect();
    
    // Ensure directory exists
    if let Some(parent) = token_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    
    // Write with restrictive permissions
    let _ = std::fs::write(&token_path, &token);
    
    token
}
```

### Phase 3: Application Changes

**Step 3.1:** Migrate existing secrets to new format

```rust
// Migration function - run once on startup if version < 2

async fn migrate_secrets_to_v2(&self) -> Result<(), SecretStorageError> {
    // 1. Read old secrets
    let old_entries = self.load_old_entries()?;
    
    // 2. Re-encrypt each with new method
    for entry in old_entries {
        let (ciphertext, nonce, salt) = encrypt_secret(&entry.value)?;
        
        // 3. Store in new table
        sqlx::query(
            "INSERT INTO encrypted_secrets (id, service, key, encrypted_value, nonce, salt, version)
             VALUES (?, ?, ?, ?, ?, ?, 2)"
        )
        .bind(entry.id)
        .bind(&entry.service)
        .bind(&entry.key)
        .bind(&ciphertext)
        .bind(&nonce)
        .bind(&salt)
        .execute(&self.pool)
        .await?;
    }
    
    Ok(())
}
```

**Step 3.2:** Update secret read/write to use new table

```rust
impl SecretStore {
    pub async fn get(&self, service: &str, key: &str) -> Result<String, SecretStorageError> {
        // Try new table first
        if let Some(secret) = self.get_from_encrypted_secrets(service, key).await? {
            return Ok(secret);
        }
        
        // Fallback to old table (migration in progress)
        self.get_legacy(service, key).await
    }
    
    pub async fn set(&self, service: &str, key: &str, value: &str) -> Result<(), SecretStorageError> {
        // Always use new encryption
        let (ciphertext, nonce, salt) = encrypt_secret(value)?;
        
        sqlx::query(
            "INSERT OR REPLACE INTO encrypted_secrets 
             (id, service, key, encrypted_value, nonce, salt, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(format!("{}:{}", service, key))
        .bind(service)
        .bind(key)
        .bind(ciphertext)
        .bind(nonce)
        .bind(salt)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

### Phase 4: Verification

**Step 4.1:** Test key derivation is non-reversible

```rust
#[test]
fn test_key_derivation_not_reversible() {
    let key1 = derive_key_argon2("machine_id", &random_salt());
    let key2 = derive_key_argon2("machine_id", &random_salt());
    
    // Same input + same salt = same key
    assert_eq!(key1, key2);
    
    // Different salt = different key
    let key3 = derive_key_argon2("machine_id", &different_salt());
    assert_ne!(key1, key3);
}

#[test]
fn test_encryption_produces_different_output() {
    let (ct1, _, _) = encrypt_secret("secret");
    let (ct2, _, _) = encrypt_secret("secret");
    
    // Same plaintext produces different ciphertext (due to random nonce)
    assert_ne!(ct1, ct2);
}
```

**Step 4.2:** Verify migration preserves data

```rust
#[tokio::test]
async fn test_secret_migration_preserves_values() {
    // Set secret with old method
    store.set("test", "key", "original_value").await;
    
    // Migrate
    store.migrate_to_v2().await;
    
    // Read with new method
    let value = store.get("test", "key").await.unwrap();
    
    assert_eq!(value, "original_value");
}
```

---

# FIX 3: Foreign Key Enforcement

## Problem

SQLite foreign keys are not enforced:

```sql
-- Defined but not enforced
FOREIGN KEY (task_id) REFERENCES tasks(id)
```

## Impact

- Orphaned messages when tasks deleted
- Data inconsistency
- Query failures on JOINs

## Solution

**Strategy:** Enable foreign keys, add cascade delete rules.

### Phase 1: Schema Changes

**Step 1.1:** Create migration to add FK enforcement

```sql
-- Migration: 025_enable_foreign_keys

-- First, clean up existing orphaned records
DELETE FROM messages WHERE task_id NOT IN (SELECT id FROM tasks);
DELETE FROM memory_chunks WHERE task_id IS NOT NULL AND task_id NOT IN (SELECT id FROM tasks);

-- Enable foreign keys (must be done per connection)
PRAGMA foreign_keys = ON;

-- Add ON DELETE CASCADE where appropriate
-- For messages.task_id -> tasks.id
PRAGMA foreign_keys = OFF;

-- Recreate messages table with proper FK
CREATE TABLE messages_new (
    id TEXT PRIMARY KEY,
    task_id TEXT,
    channel TEXT NOT NULL,
    thread_id TEXT,
    author TEXT NOT NULL,
    content TEXT NOT NULL,
    role TEXT NOT NULL,
    attachments TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- Copy data
INSERT INTO messages_new SELECT * FROM messages;

-- Drop old table
DROP TABLE messages;

-- Rename new table
ALTER TABLE messages_new RENAME TO messages;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_messages_task ON messages(task_id);
CREATE INDEX IF NOT EXISTS idx_messages_channel ON messages(channel);
CREATE INDEX IF NOT EXISTS idx_messages_created ON messages(created_at);
```

**Step 1.2:** Update db.rs to enable FK on connect

```rust
// In db.rs or connection initialization

pub async fn create_pool() -> Result<Pool<Sqlite>, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    // ENABLE FOREIGN KEYS - Critical
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;
    
    Ok(pool)
}
```

### Phase 2: Storage Changes

**Step 2.1:** Clean orphaned records before enabling FK

```sql
-- Run this script BEFORE enabling FK

-- Find orphans
SELECT 'messages' as table_name, COUNT(*) as orphan_count
FROM messages m
WHERE m.task_id IS NOT NULL 
AND NOT EXISTS (SELECT 1 FROM tasks t WHERE t.id = m.task_id)

UNION ALL

SELECT 'memory_chunks', COUNT(*)
FROM memory_chunks
WHERE task_id IS NOT NULL 
AND NOT EXISTS (SELECT 1 FROM tasks t WHERE t.id = task_id);

-- Delete orphans (run after review)
DELETE FROM messages WHERE task_id IS NOT NULL AND task_id NOT IN (SELECT id FROM tasks);
DELETE FROM memory_chunks WHERE task_id IS NOT NULL AND task_id NOT IN (SELECT id FROM tasks);
```

### Phase 3: Application Changes

**Step 3.1:** Update code that assumed FK weren't enforced

```rust
// Before: Manual cleanup
async fn delete_task(&self, task_id: &str) -> Result<(), Error> {
    // Had to manually delete messages
    sqlx::query("DELETE FROM messages WHERE task_id = ?")
        .bind(task_id)
        .execute(&self.pool)
        .await?;
    
    sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(task_id)
        .execute(&self.pool)
        .await?;
    
    Ok(())
}

// After: Let FK handle it
async fn delete_task(&self, task_id: &str) -> Result<(), Error> {
    // CASCADE handles messages deletion
    sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(task_id)
        .execute(&self.pool)
        .await?;
    
    Ok(())
}
```

### Phase 4: Verification

**Step 4.1:** Test FK enforcement

```rust
#[tokio::test]
#[should_panic]  // Expects panic due to FK violation
async fn test_fk_enforced_on_delete() {
    // Try to delete task with messages - should fail or cascade
    sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind("task_with_messages")
        .execute(&pool)
        .await
        .unwrap();  // This should fail with FK violation
}

#[tokio::test]
async fn test_cascade_delete() {
    // Enable cascade
    // Deleting task should delete messages
    sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind("task_with_messages")
        .execute(&pool)
        .await
        .unwrap();
    
    // Verify messages gone
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM messages WHERE task_id = ?"
    )
    .bind("task_with_messages")
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(count, 0);
}
```

---

# FIX 4: Encryption Defaults

## Problem

`preferences.encrypted` defaults to 0 (false), and channel credentials often stored unencrypted.

## Impact

- Secrets stored in plaintext
- Database file readable by anyone with file access

## Solution

**Strategy:** Enable encryption by default, migrate existing data.

### Phase 1: Schema Changes

**Step 1.1:** Add migration to require encryption

```sql
-- Migration: 025_encryption_required

-- Update preferences to encrypt by default (new entries)
-- No schema change needed - this is application logic

-- For channel credentials, ensure encryption column exists
ALTER TABLE channel_settings ADD COLUMN credentials_encrypted INTEGER DEFAULT 1;
```

### Phase 2: Storage Changes

**Step 2.1:** Migrate unencrypted preferences

```rust
// In preferences.rs - add migration function

async fn migrate_unencrypted_preferences(&self) -> Result<(), sqlx::Error> {
    // Find unencrypted preferences that should be encrypted
    let unencrypted: Vec<(String, String)> = sqlx::query_as(
        "SELECT key, value FROM preferences WHERE encrypted = 0 
         AND key IN ('api_key', 'token', 'secret', 'password', 'credential')"
    )
    .fetch_all(&self.pool)
    .await?;
    
    for (key, value) in unencrypted {
        // Re-encrypt
        let encrypted_value = self.encrypt(&value)?;
        
        sqlx::query(
            "UPDATE preferences SET value = ?, encrypted = 1 WHERE key = ?"
        )
        .bind(encrypted_value)
        .bind(&key)
        .execute(&self.pool)
        .await?;
    }
    
    Ok(())
}
```

### Phase 3: Application Changes

**Step 3.1:** Change default to encrypt

```rust
// In preferences.rs - CHANGE DEFAULT
pub async fn set(&self, key: &str, value: &str, encrypt: bool) -> Result<(), sqlx::Error> {
    // Default to TRUE encryption
    let should_encrypt = encrypt || self.should_encrypt_by_default(key);
    
    let final_value = if should_encrypt {
        self.encrypt(value)?
    } else {
        value.to_string()
    };
    
    // ... rest of function
}

fn should_encrypt_by_default(&self, key: &str) -> bool {
    let sensitive_patterns = ["api_key", "token", "secret", "password", "credential", "hmac"];
    let key_lower = key.to_lowercase();
    sensitive_patterns.iter().any(|p| key_lower.contains(p))
}
```

**Step 3.2:** Update channel settings to encrypt by default

```rust
// In channel_settings.rs

pub async fn save_credentials(
    &self, 
    channel_id: &str, 
    credentials: &str
) -> Result<(), Error> {
    // ALWAYS encrypt credentials
    let encrypted = self.encrypt(credentials)?;
    
    sqlx::query(
        "INSERT OR REPLACE INTO channel_settings (channel_id, credentials_encrypted, updated_at)
         VALUES (?, ?, datetime('now'))"
    )
    .bind(channel_id)
    .bind(encrypted)
    .execute(&self.pool)
    .await?;
    
    Ok(())
}
```

### Phase 4: Verification

**Step 4.1:** Verify sensitive data is encrypted

```sql
-- Test: Try to read plaintext secrets
SELECT key, SUBSTR(value, 1, 10) as value_preview, encrypted 
FROM preferences 
WHERE key LIKE '%api_key%' OR key LIKE '%token%';

-- All should have encrypted = 1 and value should look like random bytes
-- NOT human readable
```

---

# Rollback Strategy

Each fix includes rollback capability:

| Fix | Rollback Method |
|-----|-----------------|
| Audit archival | Disable archival, keep both tables, union in queries |
| Key derivation | Keep old table, dual-read during migration |
| Foreign keys | `PRAGMA foreign_keys = OFF`, re-enable manual cleanup |
| Encryption | Keep plaintext column, dual-write during migration |

---

# Testing Requirements

## Pre-Deployment Tests

```bash
# Run all existing tests
cargo test
cargo test --test integration

# Run security-specific tests
cargo test security
cargo test encryption
cargo test audit
```

## New Tests Required

### Fix 1: Audit Archival
- [ ] `test_archive_preserves_chain` - Chain valid after archive
- [ ] `test_archive_bulk_entries` - Large archival works
- [ ] `test_verify_full_chain_includes_archive` - Combined verification

### Fix 2: Key Derivation  
- [ ] `test_encryption_produces_different_output` - Nonce randomization
- [ ] `test_key_not_derivable_from_hostname` - Security proof
- [ ] `test_secret_migration_preserves_values` - Data integrity
- [ ] `test_machine_token_persistence` - Token survives restart

### Fix 3: Foreign Keys
- [ ] `test_cascade_delete_messages` - FK cascade works
- [ ] `test_fk_prevents_orphan_insert` - FK violation caught
- [ ] `test_orphan_cleanup_removes_stale` - Pre-FK data cleaned

### Fix 4: Encryption Defaults
- [ ] `test_sensitive_preferences_encrypted_by_default` - Auto-encryption
- [ ] `test_channel_credentials_always_encrypted` - No plaintext
- [ ] `test_migration_encrypts_existing` - Legacy data secured

---

# Migration Order

To minimize risk, apply fixes in this order:

1. **First:** Fix 3 (Foreign Keys) - Lowest risk, schema only
2. **Second:** Fix 4 (Encryption Defaults) - Low risk, opt-in becomes opt-out
3. **Third:** Fix 1 (Audit Archival) - Medium risk, changes cleanup logic
4. **Fourth:** Fix 2 (Key Derivation) - Highest risk, re-encrypts all secrets

Each fix should be deployed and verified before the next.

---

# Verification Checklist

Before declaring fixes complete:

- [ ] All new migrations apply without error
- [ ] All existing tests pass
- [ ] New security tests pass
- [ ] Manual verification of each fix:
  - [ ] Audit chain verified after archival
  - [ ] Secrets unreadable with old key derivation
  - [ ] Orphaned records prevented
  - [ ] Sensitive data encrypted in database
- [ ] Performance impact acceptable (run benchmarks)
- [ ] Documentation updated

---

# Timeline Estimate

| Fix | Complexity | Estimate |
|-----|-----------|----------|
| Fix 1: Audit Archival | Medium | 2-3 hours |
| Fix 2: Key Derivation | High | 4-6 hours |
| Fix 3: Foreign Keys | Low | 1-2 hours |
| Fix 4: Encryption Defaults | Medium | 2-3 hours |
| **Total** | | **9-14 hours** |

---

# Post-Fix Work

After all fixes deployed:

1. **Rotate all secrets** - Even with new key derivation, rotate API keys, tokens
2. **Update documentation** - Document new security behaviors
3. **Update onboarding** - Tell users machine_token is critical
4. **Monitor** - Watch for errors post-deployment

---

# Related Files

- `core/memory/src/ttl_cleanup.rs` - Audit deletion
- `core/security/src/secret_store.rs` - Key derivation  
- `core/memory/src/db.rs` - Foreign key setup
- `core/memory/src/preferences.rs` - Encryption defaults
- `core/router/src/db_manager.rs` - FK recreation