//! Semantic Analysis - Intent Understanding for Dubbioso Mode
//!
//! Uses tree-sitter queries to understand code intent and detect patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of semantic analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticContext {
    /// Detected intent (e.g., "error_handling", "data_extraction", "initialization")
    pub intent: String,

    /// Detected patterns (e.g., "try_catch", "early_return", "guard_clause")
    pub patterns: Vec<String>,

    /// Anti-patterns detected (e.g., "deeply_nested", "god_function")
    pub anti_patterns: Vec<String>,

    /// Error handling style (if applicable)
    pub error_handling: Option<ErrorHandlingStyle>,

    /// Confidence of the analysis (0-1)
    pub confidence: f64,
}

/// Error handling style detected in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandlingStyle {
    /// Uses try/catch or Result/Option
    Explicit,
    /// Uses early returns or guard clauses
    GuardClause,
    /// No visible error handling
    None,
    /// Partial handling (some paths covered)
    Partial,
}

/// Pattern detector for semantic analysis
pub struct SemanticAnalyzer {
    /// Language-specific pattern rules
    patterns: HashMap<String, Vec<PatternRule>>,
}

/// A pattern rule to detect
#[derive(Debug, Clone)]
struct PatternRule {
    name: String,
    intent: String,
    keywords: Vec<&'static str>,
    anti_pattern: bool,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        Self {
            patterns: Self::build_patterns(),
        }
    }

    /// Analyze code snippet for semantic context
    pub fn analyze(&self, code: &str, language: &str) -> SemanticContext {
        let rules = self.patterns.get(language).unwrap_or_else(|| {
            self.patterns.get("generic").unwrap()
        });

        let mut detected_patterns = Vec::new();
        let mut detected_anti_patterns = Vec::new();
        let mut intents = HashMap::new();

        for rule in rules {
            if self.matches_rule(code, rule) {
                if rule.anti_pattern {
                    detected_anti_patterns.push(rule.name.clone());
                } else {
                    detected_patterns.push(rule.name.clone());
                }
                *intents.entry(rule.intent.clone()).or_insert(0) += 1;
            }
        }

        // Determine primary intent
        let intent = intents
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(intent, _)| intent.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Determine error handling style
        let error_handling = self.detect_error_handling(code, language);

        // Calculate confidence based on pattern matches
        let total_matches = detected_patterns.len() + detected_anti_patterns.len();
        let confidence = if total_matches == 0 {
            0.3
        } else {
            (0.5 + (total_matches as f64 * 0.1)).min(0.95)
        };

        SemanticContext {
            intent,
            patterns: detected_patterns,
            anti_patterns: detected_anti_patterns,
            error_handling,
            confidence,
        }
    }

    /// Analyze a function for context-aware validation
    pub fn analyze_function(&self, code: &str, function_name: &str, language: &str) -> FunctionSemanticContext {
        let base = self.analyze(code, language);

        // Additional function-specific analysis
        let has_return = self.has_explicit_return(code, language);
        let is_test = function_name.starts_with("test_")
            || function_name.starts_with("it_")
            || function_name.ends_with("_test");
        let is_handler = function_name.contains("handle")
            || function_name.contains("process")
            || function_name.contains("dispatch");
        let is_constructor = function_name == "new"
            || function_name == "init"
            || function_name.starts_with("create_");

        FunctionSemanticContext {
            base,
            function_name: function_name.to_string(),
            has_explicit_return: has_return,
            is_test_function: is_test,
            is_handler_function: is_handler,
            is_constructor,
        }
    }

    fn matches_rule(&self, code: &str, rule: &PatternRule) -> bool {
        // Simple keyword-based matching (can be enhanced with tree-sitter queries)
        let code_lower = code.to_lowercase();
        rule.keywords.iter().all(|kw| code_lower.contains(&kw.to_lowercase()))
    }

    fn detect_error_handling(&self, code: &str, language: &str) -> Option<ErrorHandlingStyle> {
        let code_lower = code.to_lowercase();

        match language {
            "rust" => {
                if code.contains("match") && (code.contains("Ok(") || code.contains("Err(")) {
                    Some(ErrorHandlingStyle::Explicit)
                } else if code.contains("?") || code.contains(".map_err(") {
                    Some(ErrorHandlingStyle::GuardClause)
                } else if code.contains("unwrap()") || code.contains("expect(") {
                    Some(ErrorHandlingStyle::Partial)
                } else {
                    Some(ErrorHandlingStyle::None)
                }
            }
            "python" => {
                if code.contains("try:") && code.contains("except") {
                    Some(ErrorHandlingStyle::Explicit)
                } else if code.contains("raise") || code.contains("return None") {
                    Some(ErrorHandlingStyle::GuardClause)
                } else {
                    Some(ErrorHandlingStyle::None)
                }
            }
            "javascript" | "typescript" => {
                if code.contains("try") && code.contains("catch") {
                    Some(ErrorHandlingStyle::Explicit)
                } else if code.contains("throw") || code.contains("return null") {
                    Some(ErrorHandlingStyle::GuardClause)
                } else {
                    Some(ErrorHandlingStyle::None)
                }
            }
            _ => {
                if code_lower.contains("try") || code_lower.contains("catch") || code_lower.contains("error") {
                    Some(ErrorHandlingStyle::Partial)
                } else {
                    Some(ErrorHandlingStyle::None)
                }
            }
        }
    }

    fn has_explicit_return(&self, code: &str, language: &str) -> bool {
        match language {
            "rust" => code.contains("-> ") || code.trim().ends_with("}"),
            "python" => code.contains("return "),
            "javascript" | "typescript" => code.contains("return "),
            _ => true,
        }
    }

    fn build_patterns() -> HashMap<String, Vec<PatternRule>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert("rust".to_string(), vec![
            // Patterns
            PatternRule {
                name: "early_return".to_string(),
                intent: "validation".to_string(),
                keywords: vec!["if", "return"],
                anti_pattern: false,
            },
            PatternRule {
                name: "guard_clause".to_string(),
                intent: "validation".to_string(),
                keywords: vec!["if", "return", "Err"],
                anti_pattern: false,
            },
            PatternRule {
                name: "try_catch".to_string(),
                intent: "error_handling".to_string(),
                keywords: vec!["match", "Ok", "Err"],
                anti_pattern: false,
            },
            PatternRule {
                name: "option_unwrap".to_string(),
                intent: "data_extraction".to_string(),
                keywords: vec!["unwrap()"],
                anti_pattern: false,
            },
            PatternRule {
                name: "propagation".to_string(),
                intent: "error_handling".to_string(),
                keywords: vec!["?"],
                anti_pattern: false,
            },
            // Anti-patterns
            PatternRule {
                name: "unwrap_in_production".to_string(),
                intent: "error_handling".to_string(),
                keywords: vec!["unwrap()"],
                anti_pattern: true,
            },
            PatternRule {
                name: "expect_without_context".to_string(),
                intent: "error_handling".to_string(),
                keywords: vec!["expect(\""],
                anti_pattern: true,
            },
            PatternRule {
                name: "deeply_nested".to_string(),
                intent: "code_quality".to_string(),
                keywords: vec!["if", "if", "if"],  // Simplified detection
                anti_pattern: true,
            },
        ]);

        // Python patterns
        patterns.insert("python".to_string(), vec![
            PatternRule {
                name: "early_return".to_string(),
                intent: "validation".to_string(),
                keywords: vec!["if", "return"],
                anti_pattern: false,
            },
            PatternRule {
                name: "try_catch".to_string(),
                intent: "error_handling".to_string(),
                keywords: vec!["try:", "except"],
                anti_pattern: false,
            },
            PatternRule {
                name: "context_manager".to_string(),
                intent: "resource_management".to_string(),
                keywords: vec!["with", "as"],
                anti_pattern: false,
            },
            PatternRule {
                name: "list_comprehension".to_string(),
                intent: "data_transformation".to_string(),
                keywords: vec!["[", "for", "in", "]"],
                anti_pattern: false,
            },
            // Anti-patterns
            PatternRule {
                name: "bare_except".to_string(),
                intent: "error_handling".to_string(),
                keywords: vec!["except:"],
                anti_pattern: true,
            },
            PatternRule {
                name: "mutable_default".to_string(),
                intent: "bug_risk".to_string(),
                keywords: vec!["def", "=[]"],
                anti_pattern: true,
            },
        ]);

        // JavaScript/TypeScript patterns
        patterns.insert("javascript".to_string(), vec![
            PatternRule {
                name: "early_return".to_string(),
                intent: "validation".to_string(),
                keywords: vec!["if", "return"],
                anti_pattern: false,
            },
            PatternRule {
                name: "try_catch".to_string(),
                intent: "error_handling".to_string(),
                keywords: vec!["try", "catch"],
                anti_pattern: false,
            },
            PatternRule {
                name: "async_await".to_string(),
                intent: "async_operation".to_string(),
                keywords: vec!["async", "await"],
                anti_pattern: false,
            },
            PatternRule {
                name: "promise_chain".to_string(),
                intent: "async_operation".to_string(),
                keywords: vec![".then(", ".catch("],
                anti_pattern: false,
            },
            // Anti-patterns
            PatternRule {
                name: "empty_catch".to_string(),
                intent: "error_handling".to_string(),
                keywords: vec!["catch", "{}"],
                anti_pattern: true,
            },
            PatternRule {
                name: "var_usage".to_string(),
                intent: "code_quality".to_string(),
                keywords: vec!["var "],
                anti_pattern: true,
            },
        ]);

        // Generic patterns (fallback)
        patterns.insert("generic".to_string(), vec![
            PatternRule {
                name: "early_return".to_string(),
                intent: "validation".to_string(),
                keywords: vec!["if", "return"],
                anti_pattern: false,
            },
            PatternRule {
                name: "try_catch".to_string(),
                intent: "error_handling".to_string(),
                keywords: vec!["try", "catch"],
                anti_pattern: false,
            },
        ]);

        patterns
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Extended context for function-level analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSemanticContext {
    /// Base semantic context
    pub base: SemanticContext,

    /// Function name
    pub function_name: String,

    /// Has explicit return type/value
    pub has_explicit_return: bool,

    /// Is a test function
    pub is_test_function: bool,

    /// Is a handler/dispatcher function
    pub is_handler_function: bool,

    /// Is a constructor
    pub is_constructor: bool,
}

impl FunctionSemanticContext {
    /// Calculate additional context score based on function characteristics
    pub fn function_context_score(&self) -> f64 {
        let mut score = self.base.confidence;

        // Test functions have more context (they document behavior)
        if self.is_test_function {
            score += 0.1;
        }

        // Handler functions are important entry points
        if self.is_handler_function {
            score += 0.15;
        }

        // Constructors should be well-understood
        if self.is_constructor {
            score += 0.1;
        }

        // Explicit returns provide more clarity
        if self.has_explicit_return {
            score += 0.05;
        }

        score.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_error_handling() {
        let analyzer = SemanticAnalyzer::new();

        let code = r#"
fn process(data: Option<String>) -> Result<(), Error> {
    let value = data.ok_or(Error::NotFound)?;
    Ok(())
}
"#;

        let ctx = analyzer.analyze(code, "rust");
        assert!(matches!(ctx.error_handling, Some(ErrorHandlingStyle::GuardClause)));
    }

    #[test]
    fn test_rust_unwrap_detection() {
        let analyzer = SemanticAnalyzer::new();

        let code = r#"
fn main() {
    let x = Some(5).unwrap();
}
"#;

        let ctx = analyzer.analyze(code, "rust");
        assert!(ctx.anti_patterns.contains(&"unwrap_in_production".to_string()));
    }

    #[test]
    fn test_function_context() {
        let analyzer = SemanticAnalyzer::new();

        let code = r#"
fn handle_request(req: Request) -> Response {
    let data = req.body.unwrap();
    process(data)
}
"#;

        let ctx = analyzer.analyze_function(code, "handle_request", "rust");
        assert!(ctx.is_handler_function);
        assert!(ctx.has_explicit_return);
    }
}

