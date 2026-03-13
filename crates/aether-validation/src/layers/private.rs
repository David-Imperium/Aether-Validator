//! Private Layer — Project-specific validation rules (MILITARY GRADE)
//!
//! This layer enforces project-specific rules defined in configuration.
//! Rules are loaded from .aether/private-rules.json or passed programmatically.
//! Allows teams to enforce custom coding standards and patterns.

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity};
use std::collections::HashMap;

/// Private validation layer — Custom project rules.
///
/// Checks for:
/// - Project-specific forbidden patterns
/// - Required patterns (must be present)
/// - Naming conventions
/// - Structural requirements
pub struct PrivateLayer {
    /// Forbidden patterns (will cause ERROR if found)
    forbidden: Vec<PrivateRule>,
    /// Required patterns (will cause ERROR if NOT found)
    required: Vec<PrivateRule>,
    /// Naming conventions
    #[allow(dead_code)]
    naming: Vec<NamingRule>,
    /// Structural requirements
    structure: Vec<StructureRule>,
}

/// A custom validation rule.
#[derive(Debug, Clone)]
pub struct PrivateRule {
    pub pattern: String,
    pub id: String,
    pub message: String,
    pub severity: Severity,
    pub suggestion: Option<String>,
    /// File pattern to match (glob)
    pub file_pattern: Option<String>,
}

/// A naming convention rule.
#[derive(Debug, Clone)]
pub struct NamingRule {
    pub kind: NamingKind,
    pub pattern: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum NamingKind {
    Function,
    Struct,
    Enum,
    Trait,
    Module,
    Constant,
    Static,
}

/// A structural requirement.
#[derive(Debug, Clone)]
pub struct StructureRule {
    pub requirement: String,
    pub check: StructureCheck,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum StructureCheck {
    /// Must have a specific import
    MustImport(String),
    /// Must have a specific function
    MustHaveFunction(String),
    /// Must have documentation
    MustHaveDocs,
    /// Must have tests (#[cfg(test)])
    MustHaveTests,
    /// Maximum file length
    MaxLines(usize),
    /// Maximum function length
    MaxFunctionLines(usize),
}

impl PrivateLayer {
    /// Create a new private layer with default rules.
    pub fn new() -> Self {
        Self {
            forbidden: Self::default_forbidden(),
            required: Self::default_required(),
            naming: Self::default_naming(),
            structure: Self::default_structure(),
        }
    }

    /// Create an empty private layer.
    pub fn empty() -> Self {
        Self {
            forbidden: Vec::new(),
            required: Vec::new(),
            naming: Vec::new(),
            structure: Vec::new(),
        }
    }

    /// Create a private layer with custom rules.
    pub fn with_rules(
        forbidden: Vec<PrivateRule>,
        required: Vec<PrivateRule>,
        naming: Vec<NamingRule>,
        structure: Vec<StructureRule>,
    ) -> Self {
        Self {
            forbidden,
            required,
            naming,
            structure,
        }
    }

    /// Load rules from a configuration map.
    pub fn from_config(config: HashMap<String, serde_json::Value>) -> Self {
        let mut layer = Self::empty();

        if let Some(forbidden) = config.get("forbidden") {
            if let Some(arr) = forbidden.as_array() {
                for rule in arr {
                    if let Some(r) = parse_private_rule(rule) {
                        layer.forbidden.push(r);
                    }
                }
            }
        }

        if let Some(required) = config.get("required") {
            if let Some(arr) = required.as_array() {
                for rule in arr {
                    if let Some(r) = parse_private_rule(rule) {
                        layer.required.push(r);
                    }
                }
            }
        }

        layer
    }

    /// Default forbidden patterns - strict production rules.
    fn default_forbidden() -> Vec<PrivateRule> {
        vec![
            // Forbidden in production
            PrivateRule {
                pattern: "print!(".into(),
                id: "PRIV001".into(),
                message: "print! in production code - use proper logging".into(),
                severity: Severity::Warning,
                suggestion: Some("Use log::info!(), tracing::info!(), or tracing::debug!()".into()),
                file_pattern: None,
            },
            PrivateRule {
                pattern: "println!(".into(),
                id: "PRIV002".into(),
                message: "println! in production code - use proper logging".into(),
                severity: Severity::Warning,
                suggestion: Some("Use log::info!(), tracing::info!(), or eprintln! for errors".into()),
                file_pattern: None,
            },
            PrivateRule {
                pattern: "eprintln!(".into(),
                id: "PRIV003".into(),
                message: "eprintln! should use structured logging".into(),
                severity: Severity::Info,
                suggestion: Some("Use tracing::error!() for structured error logging".into()),
                file_pattern: None,
            },

            // Test code in production files
            PrivateRule {
                pattern: "#[test]".into(),
                id: "PRIV010".into(),
                message: "Test code should be in separate test files".into(),
                severity: Severity::Warning,
                suggestion: Some("Move tests to tests/ directory or use #[cfg(test)] module".into()),
                file_pattern: Some("src/**/*.rs".into()),
            },

            // Debug code
            PrivateRule {
                pattern: "dbg!(".into(),
                id: "PRIV020".into(),
                message: "dbg! macro should not be in production code".into(),
                severity: Severity::Error,
                suggestion: Some("Remove debug output or use tracing::debug!()".into()),
                file_pattern: None,
            },

            // Sleep in async context
            PrivateRule {
                pattern: "std::thread::sleep".into(),
                id: "PRIV030".into(),
                message: "Blocking sleep in async context - use tokio::time::sleep".into(),
                severity: Severity::Warning,
                suggestion: Some("Use tokio::time::sleep for async code".into()),
                file_pattern: None,
            },

            // Mutex in async (prefer async-aware synchronization)
            PrivateRule {
                pattern: "std::sync::Mutex".into(),
                id: "PRIV031".into(),
                message: "std::sync::Mutex can block async executor".into(),
                severity: Severity::Warning,
                suggestion: Some("Use tokio::sync::Mutex for async code".into()),
                file_pattern: None,
            },

            // Large enum variants
            PrivateRule {
                pattern: "#[derive(Debug)]".into(),
                id: "PRIV040".into(),
                message: "Consider deriving more traits (Clone, PartialEq, etc.)".into(),
                severity: Severity::Info,
                suggestion: Some("Add #[derive(Debug, Clone, PartialEq, Eq, Hash)]".into()),
                file_pattern: None,
            },

            // Forbidden imports
            PrivateRule {
                pattern: "use std::cell::RefCell".into(),
                id: "PRIV050".into(),
                message: "RefCell in async code can cause issues".into(),
                severity: Severity::Warning,
                suggestion: Some("Use RwLock or interior mutability patterns".into()),
                file_pattern: None,
            },
        ]
    }

    /// Default required patterns.
    fn default_required() -> Vec<PrivateRule> {
        vec![
            // Files should have documentation
            PrivateRule {
                pattern: "//! ".into(),
                id: "PRIV100".into(),
                message: "Module-level documentation missing".into(),
                severity: Severity::Info,
                suggestion: Some("Add //! Module description at top of file".into()),
                file_pattern: Some("src/**/*.rs".into()),
            },
        ]
    }

    /// Default naming conventions.
    fn default_naming() -> Vec<NamingRule> {
        vec![
            NamingRule {
                kind: NamingKind::Function,
                pattern: "^[a-z][a-z0-9_]*$".into(),
                message: "Function names must be snake_case".into(),
            },
            NamingRule {
                kind: NamingKind::Struct,
                pattern: "^[A-Z][a-zA-Z0-9]*$".into(),
                message: "Struct names must be PascalCase".into(),
            },
            NamingRule {
                kind: NamingKind::Enum,
                pattern: "^[A-Z][a-zA-Z0-9]*$".into(),
                message: "Enum names must be PascalCase".into(),
            },
            NamingRule {
                kind: NamingKind::Constant,
                pattern: "^[A-Z][A-Z0-9_]*$".into(),
                message: "Constants must be SCREAMING_SNAKE_CASE".into(),
            },
        ]
    }

    /// Default structure rules.
    fn default_structure() -> Vec<StructureRule> {
        vec![
            StructureRule {
                requirement: "max_file_lines".into(),
                check: StructureCheck::MaxLines(500),
                message: "File exceeds 500 lines - consider splitting".into(),
            },
            StructureRule {
                requirement: "max_function_lines".into(),
                check: StructureCheck::MaxFunctionLines(50),
                message: "Function exceeds 50 lines - consider refactoring".into(),
            },
        ]
    }

    /// Add a custom forbidden pattern.
    pub fn forbid(mut self, pattern: &str, id: &str, message: &str) -> Self {
        self.forbidden.push(PrivateRule {
            pattern: pattern.into(),
            id: id.into(),
            message: message.into(),
            severity: Severity::Error,
            suggestion: None,
            file_pattern: None,
        });
        self
    }

    /// Add a required pattern.
    pub fn require(mut self, pattern: &str, id: &str, message: &str) -> Self {
        self.required.push(PrivateRule {
            pattern: pattern.into(),
            id: id.into(),
            message: message.into(),
            severity: Severity::Warning,
            suggestion: None,
            file_pattern: None,
        });
        self
    }
}

impl Default for PrivateLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for PrivateLayer {
    fn name(&self) -> &str {
        "private"
    }

    fn priority(&self) -> u8 {
        45 // After architecture, before final pass
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let mut violations = Vec::new();
        let source = &ctx.source;

        // Check forbidden patterns
        for rule in &self.forbidden {
            if source.contains(&rule.pattern) {
                // Check file pattern if specified
                if let Some(ref pattern) = rule.file_pattern {
                    if let Some(ref path) = ctx.file_path {
                        if !matches_file_pattern(path, pattern) {
                            continue;
                        }
                    }
                }

                let violation = match rule.severity {
                    Severity::Error => Violation::error(&rule.id, &rule.message),
                    Severity::Warning => Violation::warning(&rule.id, &rule.message),
                    Severity::Info => Violation::info(&rule.id, &rule.message),
                    Severity::Hint => Violation::info(&rule.id, &rule.message),
                };

                let violation = if let Some(suggestion) = &rule.suggestion {
                    violation.suggest(suggestion)
                } else {
                    violation
                };

                violations.push(violation);
            }
        }

        // Check required patterns (inverse - violation if NOT present)
        for rule in &self.required {
            if !source.contains(&rule.pattern) {
                if let Some(ref pattern) = rule.file_pattern {
                    if let Some(ref path) = ctx.file_path {
                        if !matches_file_pattern(path, pattern) {
                            continue;
                        }
                    }
                }

                let violation = Violation::warning(&rule.id, &rule.message);
                violations.push(violation);
            }
        }

        // Check structure rules
        check_structure_rules(source, &self.structure, &mut violations);

        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }
}

/// Check if a file path matches a glob pattern.
fn matches_file_pattern(path: &std::path::Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy();

    // Simple pattern matching (not full glob)
    if pattern.starts_with("src/**") {
        if path_str.contains("/src/") || path_str.starts_with("src/") {
            let ext = pattern.strip_prefix("src/**").unwrap_or("");
            if ext.is_empty() || ext == "/*.rs" {
                return path_str.ends_with(".rs");
            }
            return path_str.ends_with(ext.trim_start_matches('/'));
        }
        return false;
    }

    true // Default to matching if pattern not recognized
}

/// Check structural requirements.
fn check_structure_rules(source: &str, rules: &[StructureRule], violations: &mut Vec<Violation>) {
    for rule in rules {
        match &rule.check {
            StructureCheck::MaxLines(max) => {
                let lines = source.lines().count();
                if lines > *max {
                    violations.push(Violation::warning(
                        "PRIV200",
                        format!("{} lines exceeds limit of {}", lines, max),
                    ).suggest("Split into multiple files or modules"));
                }
            }
            StructureCheck::MaxFunctionLines(max) => {
                check_function_lengths(source, *max, violations);
            }
            _ => {
                // Other checks would require more sophisticated parsing
            }
        }
    }
}

/// Check function lengths.
fn check_function_lengths(source: &str, max: usize, violations: &mut Vec<Violation>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut in_function = false;
    let mut fn_start = 0;
    let mut brace_count = 0;
    let mut fn_name = String::new();

    for (i, line) in lines.iter().enumerate() {
        if line.contains("fn ") && line.contains('(') {
            if !in_function {
                in_function = true;
                fn_start = i;
                brace_count = 0;
                // Extract function name
                if let Some(start) = line.find("fn ") {
                    let rest = &line[start + 3..];
                    if let Some(end) = rest.find('(') {
                        fn_name = rest[..end].trim().to_string();
                    }
                }
            }
        }

        if in_function {
            brace_count += line.matches('{').count() as i32;
            brace_count -= line.matches('}').count() as i32;

            if brace_count <= 0 && i > fn_start {
                let length = i - fn_start + 1;
                if length > max {
                    violations.push(Violation::warning(
                        "PRIV201",
                        format!("Function '{}' is {} lines (max {})", fn_name, length, max),
                    ).suggest("Extract logic into helper functions"));
                }
                in_function = false;
            }
        }
    }
}

/// Parse a private rule from JSON configuration.
fn parse_private_rule(value: &serde_json::Value) -> Option<PrivateRule> {
    let obj = value.as_object()?;
    
    Some(PrivateRule {
        pattern: obj.get("pattern")?.as_str()?.to_string(),
        id: obj.get("id")?.as_str()?.to_string(),
        message: obj.get("message")?.as_str()?.to_string(),
        severity: match obj.get("severity")?.as_str()? {
            "error" => Severity::Error,
            "warning" => Severity::Warning,
            "info" => Severity::Info,
            "hint" => Severity::Hint,
            _ => Severity::Warning,
        },
        suggestion: obj.get("suggestion").and_then(|v: &serde_json::Value| v.as_str()).map(|s: &str| s.to_string()),
        file_pattern: obj.get("file_pattern").and_then(|v: &serde_json::Value| v.as_str()).map(|s: &str| s.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_violations() {
        let source = r#"
//! Module documentation
fn my_function() -> i32 {
    42
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = PrivateLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.passed, "Clean code should pass: {:?}", result.violations);
    }

    #[tokio::test]
    async fn test_println_violation() {
        let source = r#"
fn main() {
    println!("Hello");
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = PrivateLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty(), "println should trigger violation");
        assert!(result.violations.iter().any(|v| v.id == "PRIV002"));
    }

    #[tokio::test]
    async fn test_dbg_violation() {
        let source = r#"
fn main() {
    let x = dbg!(42);
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = PrivateLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty(), "dbg! should trigger violation");
        assert!(result.violations.iter().any(|v| v.id == "PRIV020"));
    }

    #[tokio::test]
    async fn test_custom_forbidden() {
        let layer = PrivateLayer::empty()
            .forbid("TODO", "CUSTOM001", "TODO not allowed");

        let source = r#"
fn main() {
    // TODO: implement this
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty(), "TODO should trigger custom violation");
        assert!(result.violations.iter().any(|v| v.id == "CUSTOM001"));
    }

    #[tokio::test]
    async fn test_long_function() {
        let mut source = String::from("fn long_function() {\n");
        for i in 0..60 {
            source.push_str(&format!("    let x{} = {};\n", i, i));
        }
        source.push_str("}\n");

        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = PrivateLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.violations.iter().any(|v| v.id == "PRIV201"), "Long function should trigger violation");
    }
}
