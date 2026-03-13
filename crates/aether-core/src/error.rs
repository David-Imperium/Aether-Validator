//! Error types for Aether Core

use thiserror::Error;

/// Result type alias for Aether Core operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for Aether Core.
#[derive(Debug, Error)]
pub enum Error {
    /// Parsing error.
    #[error("Parse error: {0}")]
    Parse(String),
    
    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Session not found.
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    /// Timeout exceeded.
    #[error("Timeout exceeded after {0} seconds")]
    Timeout(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Parse("unexpected token".to_string());
        assert!(err.to_string().contains("unexpected token"));
    }
}
