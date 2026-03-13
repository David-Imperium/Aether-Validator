//! Violation — Represents a validation violation

use std::path::PathBuf;

/// Severity of a violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum Severity {
    /// Blocking error.
    Error,
    /// Non-blocking warning.
    #[default]
    Warning,
    /// Informational message.
    Info,
    /// Suggestion for improvement.
    Hint,
}


/// A validation violation.
#[derive(Debug, Clone)]
pub struct Violation {
    /// Contract or rule ID.
    pub id: String,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
    /// Source location.
    pub span: Option<Span>,
    /// File path.
    pub file: Option<PathBuf>,
    /// Suggested fix.
    pub suggestion: Option<String>,
}

impl Violation {
    /// Create a new violation.
    pub fn new(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
            severity: Severity::Warning,
            span: None,
            file: None,
            suggestion: None,
        }
    }

    /// Create an error-level violation.
    pub fn error(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            ..Self::new(id, message)
        }
    }

    /// Create a warning-level violation.
    pub fn warning(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            ..Self::new(id, message)
        }
    }

    /// Create an info-level violation.
    pub fn info(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Info,
            ..Self::new(id, message)
        }
    }

    /// Set the file path.
    pub fn in_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Set the source location.
    pub fn at(mut self, line: usize, column: usize) -> Self {
        self.span = Some(Span { line, column });
        self
    }

    /// Add a suggested fix.
    pub fn suggest(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Source location.
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_creation() {
        let v = Violation::error("E001", "test error")
            .in_file("test.rs")
            .at(10, 5)
            .suggest("fix this");
        
        assert_eq!(v.id, "E001");
        assert_eq!(v.severity, Severity::Error);
        assert!(v.file.is_some());
        assert!(v.suggestion.is_some());
    }
}
