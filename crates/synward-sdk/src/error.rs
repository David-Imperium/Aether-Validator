//! SDK Error types

use thiserror::Error;

/// SDK result type
pub type SdkResult<T> = Result<T, SdkError>;

/// SDK error types
#[derive(Error, Debug)]
pub enum SdkError {
    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Certification failed: {0}")]
    Certification(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for SdkError {
    fn from(err: serde_json::Error) -> Self {
        SdkError::Internal(format!("JSON error: {}", err))
    }
}

impl From<std::io::Error> for SdkError {
    fn from(err: std::io::Error) -> Self {
        SdkError::Connection(format!("IO error: {}", err))
    }
}
