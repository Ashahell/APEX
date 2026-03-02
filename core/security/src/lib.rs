pub mod capability;

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
