//! Violation — Represents a validation violation
#![allow(clippy::cognitive_complexity)] // Violation formatting requires complex matching

use std::path::PathBuf;
use std::collections::HashMap;

/// Severity of a violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum Severity {
    /// Critical error - blocking and requires immediate attention.
    Critical,
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
    /// Occurrence count (for deduplication).
    pub count: usize,
    /// All locations where this violation occurs.
    pub locations: Vec<Span>,
}

impl Default for Violation {
    fn default() -> Self {
        Self {
            id: String::new(),
            message: String::new(),
            severity: Severity::Warning,
            span: None,
            file: None,
            suggestion: None,
            count: 1,
            locations: Vec::new(),
        }
    }
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
            count: 1,
            locations: Vec::new(),
        }
    }

    /// Create an error-level violation.
    pub fn error(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            ..Self::new(id, message)
        }
    }

    /// Create a critical-level violation.
    pub fn critical(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Critical,
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
        let span = Span { line, column };
        self.span = Some(span);
        if self.locations.is_empty() {
            self.locations.push(span);
        }
        self
    }

    /// Add a location where this violation occurs.
    pub fn add_location(mut self, line: usize, column: usize) -> Self {
        self.locations.push(Span { line, column });
        self.count = self.locations.len();
        self
    }

    /// Add a suggested fix.
    pub fn suggest(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Merge another violation into this one (for deduplication).
    pub fn merge(&mut self, other: &Violation) {
        self.count += 1;
        if let Some(span) = other.span {
            self.locations.push(span);
        }
    }
}

/// Deduplicate violations by ID and message.
/// Returns violations with count and all locations.
pub fn deduplicate_violations(violations: Vec<Violation>) -> Vec<Violation> {
    let mut seen: HashMap<(String, String), Violation> = HashMap::new();

    for v in violations {
        let key = (v.id.clone(), v.message.clone());
        if let Some(existing) = seen.get_mut(&key) {
            existing.merge(&v);
        } else {
            seen.insert(key, v);
        }
    }

    let mut result: Vec<Violation> = seen.into_values().collect();
    // Sort by severity (Critical > Error > Warning > Info > Hint)
    result.sort_by(|a, b| {
        let order = |s: Severity| match s {
            Severity::Critical => 0,
            Severity::Error => 1,
            Severity::Warning => 2,
            Severity::Info => 3,
            Severity::Hint => 4,
        };
        order(a.severity).cmp(&order(b.severity))
    });
    result
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
        assert_eq!(v.count, 1);
    }

    #[test]
    fn test_violation_locations() {
        let v = Violation::warning("W001", "test")
            .at(1, 0)
            .add_location(5, 10)
            .add_location(10, 0);

        assert_eq!(v.count, 3);
        assert_eq!(v.locations.len(), 3);
    }

    #[test]
    fn test_deduplication() {
        let violations = vec![
            Violation::warning("W001", "clone").at(1, 0),
            Violation::warning("W001", "clone").at(5, 10),
            Violation::warning("W001", "clone").at(10, 0),
            Violation::error("E001", "panic").at(2, 0),
        ];

        let deduped = deduplicate_violations(violations);

        assert_eq!(deduped.len(), 2);
        assert_eq!(deduped[0].id, "E001"); // Errors first
        assert_eq!(deduped[1].id, "W001");
        assert_eq!(deduped[1].count, 3);
        assert_eq!(deduped[1].locations.len(), 3);
    }
}
