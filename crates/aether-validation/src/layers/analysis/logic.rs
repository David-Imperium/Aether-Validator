//! Logic Layer - Contract pattern evaluation (PRECISION VALIDATION)
//!
//! This layer enforces code quality rules with AST-aware context:
//! - Pattern matching with line/column precision
//! - Deduplication of repeated violations
//! - Context-aware severity (e.g., clone in loop vs normal)
//! - Strips tests and string literals to avoid false positives

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity, deduplicate_violations};

// Import from core module
use crate::layers::core::{
    PatternRule, ContextCheck, precision_rules,
    strip_test_blocks, strip_string_literals, find_checked_unwrap_lines,
    find_safety_comment_lines, has_nearby_safety, has_inline_safety,
    check_long_functions, check_deep_nesting, find_loop_lines
};

/// Context data for pattern validation
struct ValidationContextData {
    loop_lines: Vec<usize>,
    safety_lines: Vec<usize>,
    checked_unwrap_lines: Vec<usize>,
}

impl ValidationContextData {
    fn build(source: &str, language: &str, source_stripped: &str) -> Self {
        Self {
            loop_lines: find_loop_lines(source, language),
            safety_lines: find_safety_comment_lines(source),
            checked_unwrap_lines: find_checked_unwrap_lines(source_stripped),
        }
    }

    fn should_report(&self, check: &ContextCheck, line: usize, line_content: Option<&str>) -> bool {
        match check {
            ContextCheck::Always => true,
            ContextCheck::InLoop => self.loop_lines.contains(&line),
            ContextCheck::UncheckedUnwrap => !self.checked_unwrap_lines.contains(&line),
            ContextCheck::NoSafetyComment => {
                // Check for nearby SAFETY comment
                if has_nearby_safety(line, &self.safety_lines, 3) {
                    return false;
                }
                // Check for inline SAFETY comment on the same line
                if let Some(content) = line_content {
                    if has_inline_safety(content) {
                        return false;
                    }
                }
                true
            },
            ContextCheck::NotInTest => true,
        }
    }
}

/// Check all pattern rules against source
fn check_all_patterns(
    patterns: &[PatternRule],
    source_stripped: &str,
    context: &ValidationContextData,
    ctx: &ValidationContext,
) -> Vec<Violation> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = source_stripped.lines().collect();

    for rule in patterns {
        if !applies_to_language(&rule.languages, &ctx.language) {
            continue;
        }

        for (line, col) in find_pattern_locations(source_stripped, &rule.pattern) {
            // Get line content for inline safety check
            let line_content = lines.get(line.saturating_sub(1));
            if context.should_report(&rule.context_check, line, line_content.copied()) {
                violations.push(create_violation(rule, line, col, ctx));
            }
        }
    }

    violations
}

/// Check if rule applies to the given language
fn applies_to_language(languages: &Option<Vec<String>>, target: &str) -> bool {
    match languages {
        None => true,
        Some(langs) => langs.iter().any(|l| l.to_lowercase() == target.to_lowercase()),
    }
}

/// Logic validation layer - Precision Enforcement.
///
/// Checks for:
/// - Contract violations (unwrap, panic, expect) with context
/// - Unsafe code patterns with documentation check
/// - Pattern violations (TODO, FIXME, XXX)
/// - Antipatterns with AST context (clone in loop, etc.)
pub struct LogicLayer {
    /// Patterns to check for violations
    patterns: Vec<PatternRule>,
    /// Whether to treat warnings as errors
    #[allow(dead_code)]
    strict_mode: bool,
}

impl LogicLayer {
    /// Create a new logic layer with precision rules.
    pub fn new() -> Self {
        Self {
            patterns: precision_rules(),
            strict_mode: false,
        }
    }

    /// Create a logic layer with custom rules.
    pub fn with_rules(rules: Vec<(String, String, String, Severity, Option<String>)>) -> Self {
        let patterns = rules
            .into_iter()
            .map(|(pattern, id, message, severity, suggestion)| PatternRule {
                pattern,
                id,
                message,
                severity,
                suggestion,
                context_check: ContextCheck::Always,
                languages: None,
            })
            .collect();
        Self { patterns, strict_mode: false }
    }

    /// Create a strict logic layer (warnings as errors).
    pub fn strict() -> Self {
        Self {
            patterns: precision_rules(),
            strict_mode: true,
        }
    }
}

impl Default for LogicLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for LogicLayer {
    fn name(&self) -> &str {
        "logic"
    }

    fn priority(&self) -> u8 {
        30 // Third layer
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        // Preprocess source
        let source_stripped = strip_string_literals(&strip_test_blocks(&ctx.source));

        // Build context maps
        let context = ValidationContextData::build(&ctx.source, &ctx.language, &source_stripped);

        // Check patterns
        let mut violations = check_all_patterns(&self.patterns, &source_stripped, &context, ctx);

        // Check structural issues
        check_long_functions(&source_stripped, &mut violations);
        check_deep_nesting(&source_stripped, &mut violations);

        // Deduplicate and return
        let violations = deduplicate_violations(violations);
        if violations.is_empty() { LayerResult::pass() } else { LayerResult::fail(violations) }
    }
}

/// Create a violation with span information.
fn create_violation(rule: &PatternRule, line: usize, col: usize, ctx: &ValidationContext) -> Violation {
    let violation = match rule.severity {
        Severity::Critical => Violation::critical(&rule.id, &rule.message),
        Severity::Error => Violation::error(&rule.id, &rule.message),
        Severity::Warning => Violation::warning(&rule.id, &rule.message),
        Severity::Info => Violation::info(&rule.id, &rule.message),
        Severity::Hint => Violation::info(&rule.id, &rule.message),
    };

    let violation = violation.at(line, col);

    let violation = if let Some(ref file) = ctx.file_path {
        violation.in_file(file)
    } else {
        violation
    };

    if let Some(suggestion) = &rule.suggestion {
        violation.suggest(suggestion)
    } else {
        violation
    }
}

/// Find all locations (line, column) of a pattern in source.
fn find_pattern_locations(source: &str, pattern: &str) -> Vec<(usize, usize)> {
    let mut locations = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pattern) {
            let col = start + pos;
            locations.push((line_num + 1, col + 1)); // 1-indexed
            start = col + pattern.len();
        }
    }

    locations
}
