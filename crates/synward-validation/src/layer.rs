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
    /// Number of violations filtered by whitelist (for learning feedback)
    pub whitelisted_count: usize,
}

impl LayerResult {
    /// Create a passing result.
    pub fn pass() -> Self {
        Self {
            passed: true,
            violations: Vec::new(),
            infos: Vec::new(),
            whitelisted_count: 0,
        }
    }

    /// Create a failing result with violations.
    pub fn fail(violations: Vec<Violation>) -> Self {
        Self {
            passed: false,
            violations,
            infos: Vec::new(),
            whitelisted_count: 0,
        }
    }

    /// Add an informational message.
    pub fn with_info(mut self, info: String) -> Self {
        self.infos.push(info);
        self
    }

    /// Filter violations using a whitelist predicate.
    /// Returns a new result with only non-whitelisted violations.
    pub fn filter_whitelisted<F>(self, is_whitelisted: F) -> Self
    where
        F: Fn(&Violation) -> bool,
    {
        let mut whitelisted_count = 0;
        let violations: Vec<Violation> = self
            .violations
            .into_iter()
            .filter(|v| {
                if is_whitelisted(v) {
                    whitelisted_count += 1;
                    false
                } else {
                    true
                }
            })
            .collect();

        let passed = violations.is_empty();
        Self {
            passed,
            violations,
            infos: self.infos,
            whitelisted_count,
        }
    }

    /// Check if any violations are errors.
    pub fn has_errors(&self) -> bool {
        self.violations.iter().any(|v| v.severity == Severity::Error)
    }
}

/// LearnedConfig reference for Memory-Driven validation
///
/// This is a simple type alias to avoid circular dependencies.
/// The actual LearnedConfig struct is in synward-intelligence.
/// Layers that support dynamic configuration should downcast this
/// to their expected config type.
pub type LayerConfig = serde_json::Value;

/// Validation layer trait.
///
/// Each layer represents a stage in the validation pipeline:
/// 1. Syntax — Parsing errors, malformed code
/// 2. Semantic — Type checking, scope resolution
/// 3. Logic — Contract evaluation, business rules
/// 4. Architecture — Layer compliance, dependency checks
/// 5. Style — Formatting, idioms, conventions
///
/// ## Memory-Driven Configuration
///
/// Layers can receive dynamic configuration via the `config` parameter
/// in `validate_with_config()`. This enables:
/// - Dynamic thresholds (e.g., complexity limits from project history)
/// - Custom rules discovered from codebase patterns
/// - Whitelisted patterns from user acceptance
/// - Style conventions learned from the project
#[async_trait]
pub trait ValidationLayer: Send + Sync {
    /// Get the layer name.
    fn name(&self) -> &str;

    /// Get the layer priority (lower runs first).
    fn priority(&self) -> u8 {
        50
    }

    /// Validate with optional learned configuration.
    ///
    /// This is the main entry point for Memory-Driven validation.
    /// Layers should override this to use config when available.
    async fn validate_with_config(
        &self,
        ctx: &ValidationContext,
        config: Option<&LayerConfig>,
    ) -> LayerResult {
        // Default: ignore config, use legacy behavior
        let _ = config; // Suppress unused warning
        self.validate(ctx).await
    }

    /// Validate the AST (legacy, without config).
    ///
    /// Layers should implement this for backward compatibility.
    /// Memory-aware layers should override `validate_with_config()`.
    async fn validate(&self, ctx: &ValidationContext) -> LayerResult;

    /// Check if the pipeline should continue after this layer.
    /// By default, always continue (non-critical layers).
    /// Critical layers (syntax) override this to stop on errors.
    fn can_continue(&self, _result: &LayerResult) -> bool {
        true // Continue by default - only critical layers stop on errors
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
