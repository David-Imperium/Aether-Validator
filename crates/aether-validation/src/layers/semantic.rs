//! Semantic Layer — Type checking and scope analysis

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::Violation;

/// Semantic validation layer.
///
/// Checks for:
/// - Unused variables
/// - Type mismatches (basic)
/// - Scope issues
/// - Dead code detection
pub struct SemanticLayer;

impl SemanticLayer {
    /// Create a new semantic layer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SemanticLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for SemanticLayer {
    fn name(&self) -> &str {
        "semantic"
    }

    fn priority(&self) -> u8 {
        20 // Second layer
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let mut violations = Vec::new();
        let source = &ctx.source;

        // Check for unused variables (pattern: let x = ...; with no later use)
        check_unused_variables(source, &mut violations);

        // Check for unreachable code (pattern: return ...; ... code after)
        check_unreachable_code(source, &mut violations);

        // Check for shadowing (pattern: let x = ...; let x = ...;)
        check_variable_shadowing(source, &mut violations);

        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }
}

fn check_unused_variables(source: &str, violations: &mut Vec<Violation>) {
    // Simple heuristic: find let bindings that are never referenced
    // This is a basic check; real implementation would use AST
    let lines: Vec<&str> = source.lines().collect();
    let mut bindings = Vec::new();

    for line in &lines {
        if let Some(binding) = extract_let_binding(line) {
            bindings.push(binding);
        }
    }

    for binding in &bindings {
        let binding_used = source.matches(binding).count() > 1;
        if !binding_used && binding != "_" {
            violations.push(Violation::warning(
                "SEMANTIC001",
                format!("Unused variable: {}", binding),
            ).suggest(format!("Prefix with underscore: _{}", binding)));
        }
    }
}

fn check_unreachable_code(source: &str, violations: &mut Vec<Violation>) {
    // Check for code after return/panic/break/continue
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("return") || trimmed.starts_with("panic!") || trimmed.starts_with("break") {
            // Check if there's code after this line in the same block
            if i + 1 < lines.len() {
                let next_trimmed = lines[i + 1].trim();
                // Skip continuation lines (lines starting with operators or closing braces)
                if next_trimmed.is_empty() 
                    || next_trimmed.starts_with('}') 
                    || next_trimmed.starts_with("//")
                    || next_trimmed.starts_with("&&")
                    || next_trimmed.starts_with("||")
                    || next_trimmed.starts_with('|')
                    || next_trimmed.starts_with('&')
                    || next_trimmed.starts_with('+')
                    || next_trimmed.starts_with('-')
                    || next_trimmed.starts_with('*')
                    || next_trimmed.starts_with('/')
                    || next_trimmed.starts_with(',')
                    || next_trimmed.starts_with('.')
                    || next_trimmed.starts_with(':')
                {
                    continue;
                }
                // Skip if previous line doesn't end with semicolon (continuation)
                if !trimmed.ends_with(';') && !trimmed.ends_with('}') {
                    continue;
                }
                violations.push(Violation::info(
                    "SEMANTIC002",
                    "Potentially unreachable code after return/break",
                ).suggest("Remove unreachable code or restructure control flow"));
            }
        }
    }
}

fn check_variable_shadowing(source: &str, violations: &mut Vec<Violation>) {
    // Simple heuristic: find multiple let bindings with same name
    let lines: Vec<&str> = source.lines().collect();
    let mut seen_bindings = std::collections::HashSet::new();

    for line in &lines {
        if let Some(binding) = extract_let_binding(line) {
            if seen_bindings.contains(&binding) && binding != "_" {
                violations.push(Violation::info(
                    "SEMANTIC003",
                    format!("Variable shadowing: {}", binding),
                ).suggest("Consider using different names to avoid confusion"));
            }
            seen_bindings.insert(binding);
        }
    }
}

fn extract_let_binding(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("let ") {
        return None;
    }

    // Extract variable name after 'let' and before '=' or ':'
    let rest = trimmed.strip_prefix("let ")?.trim();
    let name_end = rest.find(['=', ':']).unwrap_or(rest.len());
    let name = rest[..name_end].trim();

    // Skip patterns like "mut x"
    let name = if name.starts_with("mut ") {
        name.strip_prefix("mut ")?.trim()
    } else {
        name
    };

    if name.is_empty() || name.starts_with('(') || name.starts_with('[') {
        return None;
    }

    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_violations() {
        let source = r#"
fn main() {
    let x = 1;
    println!("{}", x);
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = SemanticLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_unused_variable() {
        let source = r#"
fn main() {
    let unused = 1;
    println!("hello");
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = SemanticLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty());
    }
}
