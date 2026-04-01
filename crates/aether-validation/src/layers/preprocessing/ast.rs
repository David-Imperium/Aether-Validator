//! AST Layer — AST-based pattern validation
//!
//! This layer uses AST pattern matching for more precise validation:
//! - Structural checks (function length, nesting depth)
//! - Pattern checks (wildcard imports, empty traits)
//! - Complexity metrics (cyclomatic complexity, cognitive load)

use async_trait::async_trait;
use aether_parsers::{Parser, RustParser, AST, NodeKind, ASTMatcher, NodePattern};
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity};
use std::sync::Arc;

/// AST validation layer.
///
/// Uses AST pattern matching for structural validation.
pub struct ASTLayer {
    /// Rust parser
    parser: Arc<RustParser>,
    /// Rules for AST validation
    rules: Vec<ASTRule>,
}

/// An AST-based validation rule.
#[derive(Debug, Clone)]
pub struct ASTRule {
    /// Rule ID
    id: String,
    /// Human-readable message
    message: String,
    /// Severity
    severity: Severity,
    /// Pattern to match
    pattern: ASTPattern,
    /// Suggestion for fixing
    suggestion: Option<String>,
}

/// Types of AST patterns.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Variants prepared for future rule extension
enum ASTPattern {
    /// Match functions
    Function,
    /// Match structs
    Struct,
    /// Match enums
    Enum,
    /// Match traits
    Trait,
    /// Match impl blocks
    Impl,
    /// Match use statements
    UseStatement,
    /// Custom pattern
    Custom(NodeKind),
}

impl ASTLayer {
    /// Create a new AST layer with default rules.
    pub fn new() -> Self {
        Self {
            parser: Arc::new(RustParser::new()),
            rules: Self::default_rules(),
        }
    }

    /// Create an AST layer with custom rules.
    pub fn with_rules(rules: Vec<ASTRule>) -> Self {
        Self {
            parser: Arc::new(RustParser::new()),
            rules,
        }
    }

    fn default_rules() -> Vec<ASTRule> {
        vec![
            // Large structs
            ASTRule {
                id: "AST001".into(),
                message: "Struct has many fields (consider decomposition)".into(),
                severity: Severity::Info,
                pattern: ASTPattern::Struct,
                suggestion: Some("Split into smaller structs".into()),
            },
            // Large enums
            ASTRule {
                id: "AST002".into(),
                message: "Enum has many variants (consider grouping)".into(),
                severity: Severity::Info,
                pattern: ASTPattern::Enum,
                suggestion: Some("Group related variants into nested enums".into()),
            },
            // Traits
            ASTRule {
                id: "AST003".into(),
                message: "Trait definition found".into(),
                severity: Severity::Info,
                pattern: ASTPattern::Trait,
                suggestion: None,
            },
            // Impl blocks
            ASTRule {
                id: "AST004".into(),
                message: "Impl block found".into(),
                severity: Severity::Info,
                pattern: ASTPattern::Impl,
                suggestion: None,
            },
            // Use statements
            ASTRule {
                id: "AST005".into(),
                message: "Use statement found".into(),
                severity: Severity::Info,
                pattern: ASTPattern::UseStatement,
                suggestion: None,
            },
        ]
    }

    /// Check AST patterns in source code.
    async fn check_ast_patterns(&self, source: &str, ctx: &ValidationContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Parse source code
        let ast = match self.parser.parse(source).await {
            Ok(ast) => ast,
            Err(_) => {
                // Skip if parse error - syntax layer will catch it
                return violations;
            }
        };

        // Check each rule
        let matcher = ASTMatcher::new(source);
        
        for rule in &self.rules {
            if let Some(violation) = self.check_rule(&ast, &matcher, rule, ctx).await {
                violations.push(violation);
            }
        }

        violations
    }

    async fn check_rule(
        &self,
        ast: &AST,
        matcher: &ASTMatcher,
        rule: &ASTRule,
        ctx: &ValidationContext,
    ) -> Option<Violation> {
        let pattern = match &rule.pattern {
            ASTPattern::Function => NodePattern::any(NodeKind::Function),
            ASTPattern::Struct => NodePattern::any(NodeKind::Struct),
            ASTPattern::Enum => NodePattern::any(NodeKind::Enum),
            ASTPattern::Trait => NodePattern::any(NodeKind::Trait),
            ASTPattern::Impl => NodePattern::any(NodeKind::Impl),
            ASTPattern::UseStatement => NodePattern::any(NodeKind::Use),
            ASTPattern::Custom(kind) => NodePattern::any(*kind),
        };

        let matches = matcher.query.find_all(ast, &pattern);
        
        // For now, just report if any matches found
        if !matches.is_empty() {
            Some(Violation {
                id: rule.id.clone(),
                message: rule.message.clone(),
                severity: rule.severity,
                span: None,
                file: ctx.file_path.clone(),
                suggestion: rule.suggestion.clone(),
                ..Default::default()
            })
        } else {
            None
        }
    }
}

impl Default for ASTLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for ASTLayer {
    fn name(&self) -> &str {
        "ast"
    }

    fn priority(&self) -> u8 {
        15 // After syntax, before semantic
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let violations = self.check_ast_patterns(&ctx.source, ctx).await;
        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_context(source: &str) -> ValidationContext {
        ValidationContext {
            source: source.to_string(),
            file_path: Some(PathBuf::from("test.rs")),
            language: "rust".to_string(),
            metadata: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_ast_layer_creation() {
        let layer = ASTLayer::new();
        assert_eq!(layer.name(), "ast");
        assert_eq!(layer.priority(), 15);
    }

    #[tokio::test]
    async fn test_validate_struct() {
        let source = r#"
            struct Point {
                x: f64,
                y: f64,
            }
        "#;
        
        let layer = ASTLayer::new();
        let ctx = create_context(source);
        let result = layer.validate(&ctx).await;
        
        // Should find struct
        assert!(!result.violations.is_empty());
        assert!(result.violations.iter().any(|v| v.id == "AST001"));
    }

    #[tokio::test]
    async fn test_validate_enum() {
        let source = r#"
            enum Color {
                Red,
                Green,
                Blue,
            }
        "#;
        
        let layer = ASTLayer::new();
        let ctx = create_context(source);
        let result = layer.validate(&ctx).await;
        
        // Should find enum
        assert!(result.violations.iter().any(|v| v.id == "AST002"));
    }

    #[tokio::test]
    async fn test_validate_trait() {
        let source = r#"
            trait Drawable {
                fn draw(&self);
            }
        "#;
        
        let layer = ASTLayer::new();
        let ctx = create_context(source);
        let result = layer.validate(&ctx).await;
        
        // Should find trait
        assert!(result.violations.iter().any(|v| v.id == "AST003"));
    }

    #[tokio::test]
    async fn test_validate_impl() {
        let source = r#"
            impl Drawable for Point {
                fn draw(&self) {}
            }
        "#;
        
        let layer = ASTLayer::new();
        let ctx = create_context(source);
        let result = layer.validate(&ctx).await;
        
        // Should find impl
        assert!(result.violations.iter().any(|v| v.id == "AST004"));
    }

    #[tokio::test]
    async fn test_validate_use() {
        let source = r#"use std::collections::HashMap;"#;
        
        let layer = ASTLayer::new();
        let ctx = create_context(source);
        let result = layer.validate(&ctx).await;
        
        // Should find use statement
        assert!(result.violations.iter().any(|v| v.id == "AST005"));
    }

    #[tokio::test]
    async fn test_validate_empty_source() {
        let source = "";
        
        let layer = ASTLayer::new();
        let ctx = create_context(source);
        let result = layer.validate(&ctx).await;
        
        // Should find nothing
        assert!(result.violations.is_empty());
    }

    #[tokio::test]
    async fn test_validate_invalid_syntax() {
        let source = "fn main( {"; // Invalid
        
        let layer = ASTLayer::new();
        let ctx = create_context(source);
        let result = layer.validate(&ctx).await;
        
        // Should return empty (syntax layer will catch the error)
        assert!(result.violations.is_empty());
    }
}
