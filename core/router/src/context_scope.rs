//! Context Scope - Per-conversation data isolation
//!
//! Feature 3: Context Scope Isolation
//! Provides scoped data isolation: global, session, or channel

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_config::scope_constants::*;

/// Scope type for data isolation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    /// Global - shared across all sessions
    Global,
    /// Session-specific - isolated to one session
    Session(String),
    /// Channel-specific - isolated to one channel
    Channel(String),
}

impl Default for Scope {
    fn default() -> Self {
        Scope::Global
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Global => write!(f, "global"),
            Scope::Session(id) => write!(f, "session:{}", id),
            Scope::Channel(id) => write!(f, "channel:{}", id),
        }
    }
}

impl Scope {
    /// Parse from string (e.g., "global", "session:abc123", "channel:xyz")
    pub fn from_str(s: &str) -> Result<Self, String> {
        if s == SCOPE_GLOBAL {
            Ok(Scope::Global)
        } else if let Some(id) = s.strip_prefix("session:") {
            if id.is_empty() {
                Err("Empty session ID".to_string())
            } else {
                Ok(Scope::Session(id.to_string()))
            }
        } else if let Some(id) = s.strip_prefix("channel:") {
            if id.is_empty() {
                Err("Empty channel ID".to_string())
            } else {
                Ok(Scope::Channel(id.to_string()))
            }
        } else {
            Err(format!(
                "Invalid scope format: {}. Use 'global', 'session:ID', or 'channel:ID'",
                s
            ))
        }
    }

    /// Get the scope type name
    pub fn scope_type(&self) -> &'static str {
        match self {
            Scope::Global => SCOPE_GLOBAL,
            Scope::Session(_) => SCOPE_SESSION,
            Scope::Channel(_) => SCOPE_CHANNEL,
        }
    }

    /// Get the scope ID (for session/channel types)
    pub fn scope_id(&self) -> Option<&str> {
        match self {
            Scope::Global => None,
            Scope::Session(id) | Scope::Channel(id) => Some(id),
        }
    }

    /// Check if this scope matches another (global matches everything in its type)
    pub fn matches(&self, other: &Scope) -> bool {
        match (self, other) {
            (Scope::Global, Scope::Global) => true,
            (Scope::Session(a), Scope::Session(b)) => a == b,
            (Scope::Channel(a), Scope::Channel(b)) => a == b,
            (Scope::Session(_), Scope::Channel(_)) => false,
            (Scope::Channel(_), Scope::Session(_)) => false,
            // Global doesn't match specific scopes
            (Scope::Global, _) | (_, Scope::Global) => false,
        }
    }
}

/// Scoped data wrapper - attaches scope to any data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopedData<T> {
    pub data: T,
    pub scope: Scope,
    pub created_at: i64,
    pub updated_at: i64,
}

impl<T> ScopedData<T> {
    pub fn new(data: T, scope: Scope) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            data,
            scope,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, data: T) {
        self.data = data;
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

/// Scope context - carries current scope through async operations
#[derive(Debug, Clone, Default)]
pub struct ScopeContext {
    current_scope: Option<Scope>,
    metadata: HashMap<String, String>,
}

impl ScopeContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_scope(scope: Scope) -> Self {
        Self {
            current_scope: Some(scope),
            metadata: HashMap::new(),
        }
    }

    pub fn set_scope(&mut self, scope: Scope) {
        self.current_scope = Some(scope);
    }

    pub fn get_scope(&self) -> Option<&Scope> {
        self.current_scope.as_ref()
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn clear(&mut self) {
        self.current_scope = None;
        self.metadata.clear();
    }
}

/// Scope filter - for querying scoped data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeFilter {
    /// Include global data
    pub include_global: bool,
    /// Include specific scope (session or channel)
    pub scope: Option<Scope>,
}

impl Default for ScopeFilter {
    fn default() -> Self {
        Self {
            include_global: true,
            scope: None,
        }
    }
}

impl ScopeFilter {
    /// Create a filter for global data only
    pub fn global_only() -> Self {
        Self {
            include_global: true,
            scope: None,
        }
    }

    /// Create a filter for a specific scope
    pub fn for_scope(scope: Scope) -> Self {
        Self {
            include_global: scope == Scope::Global,
            scope: Some(scope),
        }
    }

    /// Check if a scope matches this filter
    pub fn matches(&self, data_scope: &Scope) -> bool {
        // If filter wants global and data is global
        if self.include_global && *data_scope == Scope::Global {
            return true;
        }
        // If filter has specific scope and it matches
        if let Some(ref filter_scope) = self.scope {
            return data_scope == filter_scope;
        }
        false
    }
}

/// Validate scope string is valid
pub fn is_valid_scope(s: &str) -> bool {
    VALID_SCOPE_TYPES
        .iter()
        .any(|&t| s == t || s.starts_with(&format!("{}:", t)))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_global() {
        let scope = Scope::Global;
        assert_eq!(scope.scope_type(), SCOPE_GLOBAL);
        assert_eq!(scope.scope_id(), None);
    }

    #[test]
    fn test_scope_session() {
        let scope = Scope::Session("abc123".to_string());
        assert_eq!(scope.scope_type(), SCOPE_SESSION);
        assert_eq!(scope.scope_id(), Some("abc123"));
    }

    #[test]
    fn test_scope_channel() {
        let scope = Scope::Channel("channel1".to_string());
        assert_eq!(scope.scope_type(), SCOPE_CHANNEL);
        assert_eq!(scope.scope_id(), Some("channel1"));
    }

    #[test]
    fn test_scope_from_str_global() {
        let scope = Scope::from_str("global").unwrap();
        assert_eq!(scope, Scope::Global);
    }

    #[test]
    fn test_scope_from_str_session() {
        let scope = Scope::from_str("session:abc123").unwrap();
        assert_eq!(scope, Scope::Session("abc123".to_string()));
    }

    #[test]
    fn test_scope_from_str_invalid() {
        assert!(Scope::from_str("invalid").is_err());
        assert!(Scope::from_str("session:").is_err());
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(Scope::Global.to_string(), "global");
        assert_eq!(Scope::Session("id".to_string()).to_string(), "session:id");
        assert_eq!(Scope::Channel("ch".to_string()).to_string(), "channel:ch");
    }

    #[test]
    fn test_scope_matches() {
        assert!(Scope::Global.matches(&Scope::Global));
        assert!(Scope::Session("a".to_string()).matches(&Scope::Session("a".to_string())));
        assert!(!Scope::Session("a".to_string()).matches(&Scope::Session("b".to_string())));
        assert!(!Scope::Global.matches(&Scope::Session("a".to_string())));
    }

    #[test]
    fn test_scope_filter_global() {
        let filter = ScopeFilter::global_only();
        assert!(filter.matches(&Scope::Global));
        assert!(!filter.matches(&Scope::Session("id".to_string())));
    }

    #[test]
    fn test_scope_filter_specific() {
        let filter = ScopeFilter::for_scope(Scope::Session("abc".to_string()));
        assert!(filter.matches(&Scope::Session("abc".to_string())));
        assert!(!filter.matches(&Scope::Global));
    }

    #[test]
    fn test_is_valid_scope() {
        assert!(is_valid_scope("global"));
        assert!(is_valid_scope("session:123"));
        assert!(is_valid_scope("channel:abc"));
        assert!(!is_valid_scope("invalid"));
    }

    #[test]
    fn test_scoped_data() {
        let scoped = ScopedData::new("test data".to_string(), Scope::Global);
        assert_eq!(scoped.data, "test data");
        assert_eq!(scoped.scope, Scope::Global);
    }

    #[test]
    fn test_scope_context() {
        let mut ctx = ScopeContext::new();
        ctx.set_scope(Scope::Session("abc".to_string()));
        assert_eq!(ctx.get_scope(), Some(&Scope::Session("abc".to_string())));

        ctx.set_metadata("key".to_string(), "value".to_string());
        assert_eq!(ctx.get_metadata("key"), Some(&"value".to_string()));
    }
}
