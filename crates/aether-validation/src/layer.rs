//! Validation Layer trait — Abstraction for validation stages

use async_trait::async_trait;
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity};

/// Result from a validation layer.
#[derive(Debug, Clone)]
pub struct LayerResult {
    /// Whether the layer passed validation.
    pub passed: bool,
    /// Violations found during validation.
    pub violations: Vec<Violation>,
    /// Informational messages.
    pub infos: Vec<String>,
}

impl LayerResult {
    /// Create a passing result.
    pub fn pass() -> Self {
        Self {
            passed: true,
            violations: Vec::new(),
            infos: Vec::new(),
        }
    }

    /// Create a failing result with violations.
    pub fn fail(violations: Vec<Violation>) -> Self {
        Self {
            passed: false,
            violations,
            infos: Vec::new(),
        }
    }

    /// Add an informational message.
    pub fn with_info(mut self, info: String) -> Self {
        self.infos.push(info);
        self
    }

    /// Check if any violations are errors.
    pub fn has_errors(&self) -> bool {
        self.violations.iter().any(|v| v.severity == Severity::Error)
    }
}

/// Validation layer trait.
///
/// Each layer represents a stage in the validation pipeline:
/// 1. Syntax — Parsing errors, malformed code
/// 2. Semantic — Type checking, scope resolution
/// 3. Logic — Contract evaluation, business rules
/// 4. Architecture — Layer compliance, dependency checks
/// 5. Style — Formatting, idioms, conventions
#[async_trait]
pub trait ValidationLayer: Send + Sync {
    /// Get the layer name.
    fn name(&self) -> &str;

    /// Get the layer priority (lower runs first).
    fn priority(&self) -> u8 {
        50
    }

    /// Validate the AST.
    async fn validate(&self, ctx: &ValidationContext) -> LayerResult;

    /// Check if the pipeline should continue after this layer.
    fn can_continue(&self, result: &LayerResult) -> bool {
        !result.has_errors()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_result_pass() {
        let result = LayerResult::pass();
        assert!(result.passed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_layer_result_fail() {
        let result = LayerResult::fail(vec![Violation::error("test", "test error")]);
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
    }
}
