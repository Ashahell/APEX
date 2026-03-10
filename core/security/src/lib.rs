#![allow(clippy::all)]
#![allow(dead_code)]

pub mod capability;
pub mod encrypted_narrative;
pub mod secret_store;

pub use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Token error: {0}")]
    Token(String),

    #[error("Capability error: {0}")]
    Capability(String),

    #[error("Encryption error: {0}")]
    Encryption(String),
}

pub type SecurityResult<T> = Result<T, SecurityError>;

// Re-export main types
pub use encrypted_narrative::{
    is_sensitive_field, EncryptedNarrativeEntry, NarrativeEncryptionConfig, NarrativeKeyManager,
};
pub use secret_store::{SecretEntry, SecretStorageError, SecretStore};
