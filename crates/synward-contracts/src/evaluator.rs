//! Rule Evaluator — Evaluate contract rules against source

use crate::loader::RuleDefinition;
use crate::error::ContractResult;
use crate::pattern::PatternFactory;
use synward_validation::{Violation, deduplicate_violations};

/// Rule evaluator for pattern matching.
///
/// The evaluator supports:
/// - Simple text patterns
/// - Regex patterns (prefixed with `regex:`)
/// - Composite patterns: `and:[p1, p2]`, `or:[p1, p2]`, `not:pattern`
/// - AST patterns (prefixed with `ast:`) - simplified
pub struct RuleEvaluator {
    /// Pattern factory for creating patterns
    pattern_factory: PatternFactory,
}

impl RuleEvaluator {
    /// Create a new evaluator.
    pub fn new() -> Self {
        Self {
            pattern_factory: PatternFactory::new(),
        }
    }

    /// Evaluate a rule against source code.
    pub fn evaluate(&mut self, rule: &RuleDefinition, source: &str) -> ContractResult<Vec<Violation>> {
        let pattern = self.pattern_factory.create(&rule.pattern)?;
        let matches = pattern.matches(source)?;

        let violations: Vec<Violation> = matches
            .into_iter()
            .map(|m| self.create_violation(rule, &m.matched, m.start, source))
            .collect();

        Ok(violations)
    }

    /// Evaluate multiple rules.
    pub fn evaluate_all(&mut self, rules: &[RuleDefinition], source: &str) -> ContractResult<Vec<Violation>> {
        let mut violations = Vec::new();
        for rule in rules {
            violations.extend(self.evaluate(rule, source)?);
        }
        // Deduplicate violations by ID and message
        Ok(deduplicate_violations(violations))
    }

    fn create_violation(&self, rule: &RuleDefinition, matched: &str, position: usize, source: &str) -> Violation {
        let message = rule.message.clone()
            .unwrap_or_else(|| format!("Pattern matched: {}", matched));

        // Calculate line number from position in source
        let line = source[..position].chars().filter(|&c| c == '\n').count() + 1;
        
        let mut violation = Violation::warning(&rule.pattern, message);
        violation.span = Some(synward_validation::Span {
            line,
            column: 1,
        });
        
        if let Some(suggestion) = &rule.suggestion {
            violation = violation.suggest(suggestion);
        }

        violation
    }
}

impl Default for RuleEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_simple_pattern() {
        let mut evaluator = RuleEvaluator::new();
        let rule = RuleDefinition {
            pattern: "unwrap()".to_string(),
            message: Some("Unwrap without context".to_string()),
            suggestion: None,
        };
        
        let source = "let x = option.unwrap();";
        let violations = evaluator.evaluate(&rule, source).unwrap();
        
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_evaluator_no_match() {
        let mut evaluator = RuleEvaluator::new();
        let rule = RuleDefinition {
            pattern: "unwrap()".to_string(),
            message: Some("Unwrap without context".to_string()),
            suggestion: None,
        };
        
        let source = "let x = option?;";
        let violations = evaluator.evaluate(&rule, source).unwrap();
        
        assert!(violations.is_empty());
    }

    #[test]
    fn test_evaluator_regex_pattern() {
        let mut evaluator = RuleEvaluator::new();
        let rule = RuleDefinition {
            pattern: "regex:\\bpanic!\\(".to_string(),
            message: Some("panic! found".to_string()),
            suggestion: Some("Use Result instead".to_string()),
        };
        
        let source = "fn main() { panic!(\"error\"); }";
        let violations = evaluator.evaluate(&rule, source).unwrap();
        
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_evaluator_multiple_matches() {
        let mut evaluator = RuleEvaluator::new();
        let rule = RuleDefinition {
            pattern: "regex:\\.unwrap\\(\\)".to_string(),
            message: Some("unwrap() found".to_string()),
            suggestion: None,
        };
        
        let source = "let a = x.unwrap(); let b = y.unwrap();";
        let violations = evaluator.evaluate(&rule, source).unwrap();
        
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_evaluator_with_suggestion() {
        let mut evaluator = RuleEvaluator::new();
        let rule = RuleDefinition {
            pattern: ".clone()".to_string(),
            message: Some("Clone called".to_string()),
            suggestion: Some("Consider using a reference".to_string()),
        };
        
        let source = "let x = data.clone();";
        let violations = evaluator.evaluate(&rule, source).unwrap();
        
        assert_eq!(violations.len(), 1);
        assert!(violations[0].suggestion.is_some());
    }
    
    #[test]
    fn test_evaluator_and_pattern() {
        let mut evaluator = RuleEvaluator::new();
        let rule = RuleDefinition {
            pattern: "and:[unwrap, Result]".to_string(),
            message: Some("Both patterns found".to_string()),
            suggestion: None,
        };
        
        let source = "let x: Result<T, E> = opt.unwrap();";
        let violations = evaluator.evaluate(&rule, source).unwrap();
        
        assert_eq!(violations.len(), 1);
    }
    
    #[test]
    fn test_evaluator_or_pattern() {
        let mut evaluator = RuleEvaluator::new();
        let rule = RuleDefinition {
            pattern: "or:[unwrap, expect]".to_string(),
            message: Some("Either pattern found".to_string()),
            suggestion: None,
        };
        
        let source = "let x = opt.unwrap();";
        let violations = evaluator.evaluate(&rule, source).unwrap();
        
        assert_eq!(violations.len(), 1);
    }
    
    #[test]
    fn test_evaluator_not_pattern() {
        let mut evaluator = RuleEvaluator::new();
        let rule = RuleDefinition {
            pattern: "not:unsafe".to_string(),
            message: Some("No unsafe found".to_string()),
            suggestion: None,
        };
        
        let source = "fn safe_function() {}";
        let violations = evaluator.evaluate(&rule, source).unwrap();
        
        assert_eq!(violations.len(), 1);
        
        let source = "unsafe { }";
        let violations = evaluator.evaluate(&rule, source).unwrap();
        
        assert!(violations.is_empty());
    }
}
