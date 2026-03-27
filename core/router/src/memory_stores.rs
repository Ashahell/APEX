//! Bounded Memory System (Hermes-style)
//!
//! Provides Hermes-compatible bounded memory with:
//! - MEMORY.md: Agent's personal notes (default 2,200 chars)
//! - USER.md: User profile (default 1,375 chars)
//! - Frozen snapshot pattern for system prompt injection
//! - Character limits with automatic consolidation
//! - O(1) exact content lookups via HashMap index

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use ulid::Ulid;

use crate::unified_config::memory_constants::*;

/// Memory store types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreType {
    /// Agent's personal notes about environment, workflows, lessons
    Memory,
    /// User profile: preferences, communication style, pet peeves
    User,
}

impl StoreType {
    #[inline]
    pub fn default_limit(&self) -> usize {
        match self {
            StoreType::Memory => DEFAULT_MEMORY_CHAR_LIMIT,
            StoreType::User => DEFAULT_USER_CHAR_LIMIT,
        }
    }

    #[inline]
    pub fn file_name(&self) -> &'static str {
        match self {
            StoreType::Memory => MEMORY_FILE,
            StoreType::User => USER_FILE,
        }
    }
}

/// Single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique identifier (ULID)
    pub id: String,
    /// Entry content
    pub content: String,
    /// Creation timestamp (Unix epoch)
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Entry type (Memory or User)
    pub entry_type: StoreType,
}

impl MemoryEntry {
    /// Create a new memory entry
    pub fn new(content: String, entry_type: StoreType) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Ulid::new().to_string(),
            content,
            created_at: now,
            updated_at: now,
            entry_type,
        }
    }
}

/// Bounded memory store with character limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStore {
    /// All entries in the store
    entries: Vec<MemoryEntry>,
    /// HashSet for O(1) exact content lookups (for duplicate detection)
    #[serde(skip)]
    content_index: HashSet<String>,
    /// Maximum character limit
    char_limit: usize,
    /// Current used characters
    pub used_chars: usize, // Made public for API access
    /// Store type
    store_type: StoreType,
}

impl MemoryStore {
    /// Create a new memory store with default limits
    pub fn new(store_type: StoreType) -> Self {
        Self {
            entries: Vec::new(),
            content_index: HashSet::new(),
            char_limit: store_type.default_limit(),
            used_chars: 0,
            store_type,
        }
    }

    /// Create with custom limit
    pub fn with_limit(store_type: StoreType, char_limit: usize) -> Self {
        Self {
            entries: Vec::new(),
            content_index: HashSet::new(),
            char_limit,
            used_chars: 0,
            store_type,
        }
    }

    /// Get current usage as percentage (0.0 to 1.0)
    #[inline]
    pub fn usage_percent(&self) -> f32 {
        if self.char_limit == 0 {
            return 0.0;
        }
        self.used_chars as f32 / self.char_limit as f32
    }

    /// Get current usage as percentage string
    #[inline]
    pub fn usage_display(&self) -> String {
        format!("{}%", (self.usage_percent() * 100.0) as usize)
    }

    /// Check if usage is at warning threshold
    #[inline]
    pub fn is_warning(&self) -> bool {
        self.usage_percent() >= MEMORY_WARNING_THRESHOLD
    }

    /// Check if usage is at critical threshold
    #[inline]
    pub fn is_critical(&self) -> bool {
        self.usage_percent() >= MEMORY_CRITICAL_THRESHOLD
    }

    /// Get available characters
    #[inline]
    pub fn available_chars(&self) -> usize {
        self.char_limit.saturating_sub(self.used_chars)
    }

    /// Get current character limit
    #[inline]
    pub fn char_limit(&self) -> usize {
        self.char_limit
    }

    /// Get number of entries
    #[inline]
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Get all entries (immutable)
    #[inline]
    pub fn entries(&self) -> &[MemoryEntry] {
        &self.entries
    }

    /// Check if entry can be added without exceeding limit
    #[inline]
    pub fn can_add(&self, content: &str) -> bool {
        self.used_chars + content.len() <= self.char_limit
    }

    /// Check if content exceeds maximum entry length
    #[inline]
    pub fn exceeds_max_length(&self, content: &str) -> bool {
        content.len() > MAX_ENTRY_LENGTH
    }

    /// Add a new entry
    ///
    /// Returns the entry ID on success
    ///
    /// # Errors
    /// - `EntryTooShort` if content is below minimum length
    /// - `DuplicateEntry` if exact content already exists
    /// - `CapacityExceeded` if adding would exceed limit
    pub fn add_entry(&mut self, content: String) -> Result<String, MemoryError> {
        // Validate length
        if content.len() < MIN_ENTRY_LENGTH {
            return Err(MemoryError::EntryTooShort {
                min_length: MIN_ENTRY_LENGTH,
                actual_length: content.len(),
            });
        }

        if content.len() > MAX_ENTRY_LENGTH {
            return Err(MemoryError::EntryTooLong {
                max_length: MAX_ENTRY_LENGTH,
                actual_length: content.len(),
            });
        }

        // O(1) duplicate check using HashSet index
        if self.content_index.contains(&content) {
            return Err(MemoryError::DuplicateEntry);
        }

        // Check capacity
        if !self.can_add(&content) {
            return Err(MemoryError::CapacityExceeded {
                current: self.used_chars,
                limit: self.char_limit,
                needed: content.len(),
            });
        }

        let entry = MemoryEntry::new(content.clone(), self.store_type);
        let id = entry.id.clone();

        self.used_chars += content.len();
        self.content_index.insert(content); // Add to O(1) lookup index
        self.entries.push(entry);

        Ok(id)
    }

    /// Remove an entry by unique substring match
    ///
    /// # Errors
    /// - `EntryNotFound` if no entry contains the substring
    /// - `MultipleMatches` if substring matches multiple entries
    pub fn remove_entry(&mut self, old_text: &str) -> Result<(), MemoryError> {
        // Find matching entries
        let matches: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.content.contains(old_text))
            .map(|(i, _)| i)
            .collect();

        match matches.len() {
            0 => Err(MemoryError::EntryNotFound),
            1 => {
                let entry = self.entries.remove(matches[0]);
                self.used_chars -= entry.content.len();
                self.content_index.remove(&entry.content); // Remove from O(1) index
                Ok(())
            }
            _ => Err(MemoryError::MultipleMatches {
                count: matches.len(),
                suggestion: "Use a more specific substring".to_string(),
            }),
        }
    }

    /// Replace an entry's content by unique substring match
    ///
    /// # Errors
    /// - `EntryNotFound` if no entry contains the substring
    /// - `MultipleMatches` if substring matches multiple entries
    /// - `CapacityExceeded` if new content would exceed limit
    pub fn replace_entry(
        &mut self,
        old_text: &str,
        new_content: String,
    ) -> Result<(), MemoryError> {
        // Validate new content length
        if new_content.len() < MIN_ENTRY_LENGTH {
            return Err(MemoryError::EntryTooShort {
                min_length: MIN_ENTRY_LENGTH,
                actual_length: new_content.len(),
            });
        }

        if new_content.len() > MAX_ENTRY_LENGTH {
            return Err(MemoryError::EntryTooLong {
                max_length: MAX_ENTRY_LENGTH,
                actual_length: new_content.len(),
            });
        }

        // Find matching entries
        let matches: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.content.contains(old_text))
            .map(|(i, _)| i)
            .collect();

        let index = match matches.len() {
            0 => return Err(MemoryError::EntryNotFound),
            1 => matches[0],
            _ => {
                return Err(MemoryError::MultipleMatches {
                    count: matches.len(),
                    suggestion: "Use a more specific substring".to_string(),
                })
            }
        };

        let old_entry = &self.entries[index];
        let old_content = old_entry.content.clone();
        let char_diff = new_content.len() as i64 - old_entry.content.len() as i64;

        // Check if new content would exceed limit
        if self.used_chars as i64 + char_diff > self.char_limit as i64 {
            return Err(MemoryError::CapacityExceeded {
                current: self.used_chars,
                limit: self.char_limit,
                needed: new_content.len(),
            });
        }

        // Update index: remove old, add new (only if different)
        self.content_index.remove(&old_content);
        self.content_index.insert(new_content.clone());

        self.used_chars = (self.used_chars as i64 + char_diff) as usize;
        self.entries[index].content = new_content;
        self.entries[index].updated_at = chrono::Utc::now().timestamp();

        Ok(())
    }

    /// Get a frozen snapshot for system prompt injection
    ///
    /// Format:
    /// ```text
    /// ═══ MEMORY (67% — 1,474/2,200 chars) ═══
    /// Entry 1 content§Entry 2 content§Entry 3 content
    /// ```
    pub fn to_snapshot(&self) -> String {
        let entries: Vec<String> = self.entries.iter().map(|e| e.content.clone()).collect();

        let header = if self.store_type == StoreType::Memory {
            "MEMORY"
        } else {
            "USER PROFILE"
        };

        format!(
            "═══ {} ({}% — {}/{} chars) ═══\n{}\n",
            header,
            (self.usage_percent() * 100.0) as usize,
            self.used_chars,
            self.char_limit,
            entries.join(ENTRY_DELIMITER)
        )
    }

    /// Get statistics for display
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            store_type: self.store_type,
            used_chars: self.used_chars,
            char_limit: self.char_limit,
            usage_percent: self.usage_percent(),
            entry_count: self.entries.len(),
            is_warning: self.is_warning(),
            is_critical: self.is_critical(),
        }
    }

    /// Load from file
    pub async fn load_from_file(path: &PathBuf) -> Result<Self, MemoryError> {
        if !path.exists() {
            // Return empty store if file doesn't exist
            let store_type = if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n == USER_FILE)
                .unwrap_or(false)
            {
                StoreType::User
            } else {
                StoreType::Memory
            };
            return Ok(Self::new(store_type));
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| MemoryError::IoError(e.to_string()))?;

        Self::parse(&content)
    }

    /// Save to file
    pub async fn save_to_file(&self, path: &PathBuf) -> Result<(), MemoryError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| MemoryError::IoError(e.to_string()))?;
        }

        let snapshot = self.to_snapshot();
        tokio::fs::write(path, snapshot)
            .await
            .map_err(|e| MemoryError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Parse content from snapshot format
    fn parse(content: &str) -> Result<Self, MemoryError> {
        // Extract header for store type
        let store_type = if content.contains("═══ MEMORY") {
            StoreType::Memory
        } else {
            StoreType::User
        };

        // Extract entries (after header line, split by delimiter)
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() < 2 {
            return Ok(Self::new(store_type));
        }

        // Entries are on lines after header, joined by delimiter
        let entries_text = lines[1..].join(ENTRY_DELIMITER);
        let entry_contents: Vec<&str> = entries_text
            .split(ENTRY_DELIMITER)
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut store = Self::new(store_type);

        for entry_content in entry_contents {
            let content = entry_content.trim().to_string();
            if content.len() >= MIN_ENTRY_LENGTH {
                if let Err(e) = store.add_entry(content) {
                    // Skip entries that can't be added (duplicates, etc.)
                    tracing::debug!(error = %e, "Skipping entry during parse");
                }
            }
        }

        // Rebuild index after parsing all entries
        store.content_index = store.entries.iter().map(|e| e.content.clone()).collect();

        Ok(store)
    }
}

/// Memory statistics for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub store_type: StoreType,
    pub used_chars: usize,
    pub char_limit: usize,
    pub usage_percent: f32,
    pub entry_count: usize,
    pub is_warning: bool,
    pub is_critical: bool,
}

/// Memory errors
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error(
        "Entry too short (minimum {} chars, got {})",
        min_length,
        actual_length
    )]
    EntryTooShort {
        min_length: usize,
        actual_length: usize,
    },

    #[error("Entry too long (maximum {} chars, got {})", max_length, actual_length)]
    EntryTooLong {
        max_length: usize,
        actual_length: usize,
    },

    #[error("Duplicate entry")]
    DuplicateEntry,

    #[error("Capacity exceeded: {current}/{limit} chars, need {} more", needed)]
    CapacityExceeded {
        current: usize,
        limit: usize,
        needed: usize,
    },

    #[error("Entry not found")]
    EntryNotFound,

    #[error("Multiple entries matched ({count}). {suggestion}")]
    MultipleMatches { count: usize, suggestion: String },

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_creation() {
        let store = MemoryStore::new(StoreType::Memory);
        assert_eq!(store.char_limit(), DEFAULT_MEMORY_CHAR_LIMIT);
        assert_eq!(store.entry_count(), 0);
        assert_eq!(store.usage_percent(), 0.0);
    }

    #[test]
    fn test_add_entry() {
        let mut store = MemoryStore::new(StoreType::Memory);

        let id = store.add_entry("This is a test entry".to_string());
        assert!(id.is_ok());
        assert_eq!(store.entry_count(), 1);
        assert_eq!(store.used_chars, 20); // "This is a test entry"
    }

    #[test]
    fn test_add_entry_too_short() {
        let mut store = MemoryStore::new(StoreType::Memory);

        let result = store.add_entry("short".to_string());
        assert!(matches!(result, Err(MemoryError::EntryTooShort { .. })));
    }

    #[test]
    fn test_add_entry_duplicate() {
        let mut store = MemoryStore::new(StoreType::Memory);

        let content = "This is a unique entry".to_string();
        assert!(store.add_entry(content.clone()).is_ok());
        assert!(matches!(
            store.add_entry(content),
            Err(MemoryError::DuplicateEntry)
        ));
    }

    #[test]
    fn test_capacity_exceeded() {
        let mut store = MemoryStore::with_limit(StoreType::Memory, 50);

        // Add entries until near limit
        store.add_entry("Entry one here".to_string()).unwrap();
        store.add_entry("Entry two here".to_string()).unwrap();

        // Next entry should fail as it would exceed limit
        let result = store
            .add_entry("This is a very long entry that exceeds remaining capacity".to_string());
        assert!(matches!(result, Err(MemoryError::CapacityExceeded { .. })));
    }

    #[test]
    fn test_remove_entry() {
        let mut store = MemoryStore::new(StoreType::Memory);

        store.add_entry("First entry".to_string()).unwrap();
        store.add_entry("Second entry".to_string()).unwrap();
        store.add_entry("Third entry".to_string()).unwrap();

        // Remove using substring
        assert!(store.remove_entry("Second").is_ok());
        assert_eq!(store.entry_count(), 2);
    }

    #[test]
    fn test_replace_entry() {
        let mut store = MemoryStore::new(StoreType::Memory);

        store.add_entry("Old content here".to_string()).unwrap();

        assert!(store
            .replace_entry("Old", "New content here".to_string())
            .is_ok());
        assert_eq!(store.entries()[0].content, "New content here");
    }

    #[test]
    fn test_snapshot_format() {
        let mut store = MemoryStore::new(StoreType::Memory);
        store.add_entry("First entry".to_string()).unwrap();
        store.add_entry("Second entry".to_string()).unwrap();

        let snapshot = store.to_snapshot();

        assert!(snapshot.contains("═══ MEMORY"));
        assert!(snapshot.contains("chars")); // Format: "used_chars/char_limit chars"
        assert!(snapshot.contains("First entry"));
        assert!(snapshot.contains("Second entry"));
    }

    #[test]
    fn test_warning_threshold() {
        let mut store = MemoryStore::with_limit(StoreType::Memory, 100);

        assert!(!store.is_warning());

        // Add content to reach ~85%
        store.add_entry("A".repeat(85).to_string()).unwrap();

        assert!(store.is_warning());
        assert!(!store.is_critical());
    }

    #[test]
    fn test_multiple_matches_error() {
        let mut store = MemoryStore::new(StoreType::Memory);

        store.add_entry("word apple here".to_string()).unwrap();
        store.add_entry("word banana here".to_string()).unwrap();

        let result = store.remove_entry("word");
        assert!(matches!(
            result,
            Err(MemoryError::MultipleMatches { count: 2, .. })
        ));
    }
}
