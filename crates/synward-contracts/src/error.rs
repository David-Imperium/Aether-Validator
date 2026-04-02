//! Contract errors

use thiserror::Error;

/// Result type for contract operations.
pub type ContractResult<T> = std::result::Result<T, ContractError>;

/// Errors that can occur during contract operations.
#[derive(Debug, Error)]
pub enum ContractError {
    /// Contract not found.
    #[error("Contract not found: {0}")]
    NotFound(String),

    /// Error loading contract file.
    #[error("Failed to load contract from {0}: {1}")]
    LoadError(String, String),

    /// Error parsing contract YAML.
    #[error("Failed to parse contract from {0}: {1}")]
    ParseError(String, String),

    /// Evaluation error.
    #[error("Contract evaluation failed: {0}")]
    EvaluationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_not_found() {
        let err = ContractError::NotFound("RUST001".to_string());
        assert!(err.to_string().contains("RUST001"));
    }
}
