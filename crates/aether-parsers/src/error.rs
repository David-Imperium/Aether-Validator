//! Parse errors

use thiserror::Error;

/// Result type for parsing operations.
pub type ParseResult<T> = std::result::Result<T, ParseError>;

/// Errors that can occur during parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    /// Syntax error in source code.
    #[error("Syntax error at {line}:{column}: {message}")]
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },

    /// Parse failed with generic error.
    #[error("Parse failed: {0}")]
    ParseFailed(String),

    /// Unexpected token.
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
    },

    /// Parser not found for language.
    #[error("Parser not found for language: {0}")]
    ParserNotFound(String),

    /// Unknown file extension.
    #[error("Unknown file extension: {0}")]
    UnknownExtension(String),

    /// IO error during parsing.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_syntax() {
        let err = ParseError::Syntax {
            line: 10,
            column: 5,
            message: "unexpected closing brace".to_string(),
        };
        assert!(err.to_string().contains("10:5"));
    }
}
