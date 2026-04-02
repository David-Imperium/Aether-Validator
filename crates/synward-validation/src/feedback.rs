//! Violation Feedback — AI-friendly suggestions for violations
//!
//! This module provides contextual feedback for violations:
//! - Code fix suggestions
//! - Learning resources
//! - Pattern-based recommendations

use crate::violation::{Violation, Severity};
use std::collections::HashMap;

/// Feedback level for suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackLevel {
    /// Simple fix suggestion
    QuickFix,
    /// Explanation with context
    Detailed,
    /// Learning resource
    Educational,
}

/// Feedback for a violation
#[derive(Debug, Clone)]
pub struct ViolationFeedback {
    /// The violation ID
    pub violation_id: String,
    /// Quick fix suggestion
    pub quick_fix: Option<String>,
    /// Detailed explanation
    pub explanation: Option<String>,
    /// Learning resources
    pub resources: Vec<Resource>,
    /// Code examples
    pub examples: Vec<CodeExample>,
}

/// Learning resource
#[derive(Debug, Clone)]
pub struct Resource {
    pub title: String,
    pub url: Option<String>,
    pub description: String,
}

/// Code example for fixing the violation
#[derive(Debug, Clone)]
pub struct CodeExample {
    pub title: String,
    pub before: String,
    pub after: String,
    pub description: String,
}

/// Feedback provider for generating contextual suggestions
pub struct FeedbackProvider {
    /// Map of violation IDs to feedback templates
    templates: HashMap<String, ViolationFeedback>,
}

impl FeedbackProvider {
    /// Create a new feedback provider with default templates
    pub fn new() -> Self {
        Self {
            templates: Self::default_templates(),
        }
    }
    
    /// Get feedback for a violation
    pub fn get_feedback(&self, violation: &Violation) -> Option<ViolationFeedback> {
        self.templates.get(&violation.id).cloned()
    }
    
    /// Get feedback with custom level
    pub fn get_feedback_at_level(&self, violation: &Violation, level: FeedbackLevel) -> Option<ViolationFeedback> {
        let mut feedback = self.get_feedback(violation)?;
        
        match level {
            FeedbackLevel::QuickFix => {
                feedback.explanation = None;
                feedback.resources.clear();
                feedback.examples.truncate(1);
            }
            FeedbackLevel::Detailed => {
                feedback.resources.clear();
            }
            FeedbackLevel::Educational => {
                // Keep everything
            }
        }
        
        Some(feedback)
    }
    
    /// Generate AI-friendly description
    pub fn generate_description(&self, violation: &Violation) -> String {
        let severity_emoji = match violation.severity {
            Severity::Critical => "💥",
            Severity::Error => "🔴",
            Severity::Warning => "🟡",
            Severity::Info => "🔵",
            Severity::Hint => "⚪",
        };
        
        let mut desc = format!("{} [{}] {}", severity_emoji, violation.id, violation.message);
        
        if let Some(feedback) = self.get_feedback(violation) {
            if let Some(quick_fix) = feedback.quick_fix {
                desc.push_str(&format!("\n  💡 Quick fix: {}", quick_fix));
            }
        } else if let Some(suggestion) = &violation.suggestion {
            desc.push_str(&format!("\n  💡 Suggestion: {}", suggestion));
        }
        
        desc
    }
    
    /// Default feedback templates for common violations
    fn default_templates() -> HashMap<String, ViolationFeedback> {
        let mut templates = HashMap::new();
        
        // Logic violations
        templates.insert("LOGIC001".into(), ViolationFeedback {
            violation_id: "LOGIC001".into(),
            quick_fix: Some("Replace panic! with Result return type".into()),
            explanation: Some("panic! causes the program to crash. In library code, use Result<T, E> to allow callers to handle errors.".into()),
            resources: vec![Resource {
                title: "Error Handling in Rust".into(),
                url: Some("https://doc.rust-lang.org/book/ch09-00-error-handling.html".into()),
                description: "Learn about recoverable and unrecoverable errors".into(),
            }],
            examples: vec![CodeExample {
                title: "Replace panic with Result".into(),
                before: "fn divide(a: i32, b: i32) -> i32 {\n    if b == 0 { panic!(\"division by zero\"); }\n    a / b\n}".into(),
                after: "fn divide(a: i32, b: i32) -> Result<i32, &'static str> {\n    if b == 0 { return Err(\"division by zero\"); }\n    Ok(a / b)\n}".into(),
                description: "Return Result instead of panicking".into(),
            }],
        });
        
        templates.insert("LOGIC002".into(), ViolationFeedback {
            violation_id: "LOGIC002".into(),
            quick_fix: Some("Use .expect(\"descriptive message\") or proper error handling".into()),
            explanation: Some("unwrap() panics on None/Err without context. Always provide context for debugging.".into()),
            resources: vec![],
            examples: vec![CodeExample {
                title: "Add context to unwrap".into(),
                before: "let value = option.unwrap();".into(),
                after: "let value = option.expect(\"config value must be set\");".into(),
                description: "Use expect with descriptive message".into(),
            }],
        });
        
        // Style violations
        templates.insert("STYLE001".into(), ViolationFeedback {
            violation_id: "STYLE001".into(),
            quick_fix: Some("Convert to snake_case: bad_function_name → bad_function_name".into()),
            explanation: Some("Rust functions use snake_case by convention. This improves readability and consistency.".into()),
            resources: vec![Resource {
                title: "Rust Naming Conventions".into(),
                url: Some("https://rust-lang.github.io/api-guidelines/naming.html".into()),
                description: "Official Rust API naming guidelines".into(),
            }],
            examples: vec![],
        });
        
        templates.insert("STYLE002".into(), ViolationFeedback {
            violation_id: "STYLE002".into(),
            quick_fix: Some("Convert to PascalCase: bad_struct_name → BadStructName".into()),
            explanation: Some("Types and structs use PascalCase in Rust. This distinguishes types from variables.".into()),
            resources: vec![],
            examples: vec![],
        });
        
        // Architecture violations
        templates.insert("ARCH003".into(), ViolationFeedback {
            violation_id: "ARCH003".into(),
            quick_fix: Some("Move test code to tests/ directory".into()),
            explanation: Some("Test code in production files increases binary size and can accidentally be shipped. Use the tests/ directory.".into()),
            resources: vec![],
            examples: vec![],
        });
        
        // Rust advanced violations (Phase 2)
        templates.insert("RUST019".into(), ViolationFeedback {
            violation_id: "RUST019".into(),
            quick_fix: Some("Add // SAFETY: comment explaining invariants".into()),
            explanation: Some("Unsafe blocks require safety comments to document why the code is safe. This helps reviewers and future maintainers.".into()),
            resources: vec![Resource {
                title: "Unsafe Rust".into(),
                url: Some("https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html".into()),
                description: "Learn about unsafe Rust and safety invariants".into(),
            }],
            examples: vec![CodeExample {
                title: "Add safety comment".into(),
                before: "unsafe {\n    *ptr = value;\n}".into(),
                after: "// SAFETY: ptr is guaranteed valid by the caller\n// and points to properly initialized memory\nunsafe {\n    *ptr = value;\n}".into(),
                description: "Document safety invariants".into(),
            }],
        });
        
        templates.insert("RUST026".into(), ViolationFeedback {
            violation_id: "RUST026".into(),
            quick_fix: Some("Wrap in Mutex or RwLock, or use atomic types".into()),
            explanation: Some("Static mutable variables are not thread-safe. Use synchronization primitives from std::sync.".into()),
            resources: vec![],
            examples: vec![],
        });
        
        templates
    }
}

impl Default for FeedbackProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feedback_provider_creation() {
        let provider = FeedbackProvider::new();
        assert!(!provider.templates.is_empty());
    }
    
    #[test]
    fn test_get_feedback_logic001() {
        let provider = FeedbackProvider::new();
        let violation = Violation::error("LOGIC001", "panic! found");
        
        let feedback = provider.get_feedback(&violation).unwrap();
        assert_eq!(feedback.violation_id, "LOGIC001");
        assert!(feedback.quick_fix.is_some());
        assert!(!feedback.examples.is_empty());
    }
    
    #[test]
    fn test_generate_description() {
        let provider = FeedbackProvider::new();
        let violation = Violation::warning("STYLE001", "Function should use snake_case");
        
        let desc = provider.generate_description(&violation);
        assert!(desc.contains("🟡"));
        assert!(desc.contains("STYLE001"));
        assert!(desc.contains("snake_case"));
    }
    
    #[test]
    fn test_feedback_level_quick_fix() {
        let provider = FeedbackProvider::new();
        let violation = Violation::error("LOGIC001", "panic! found");
        
        let feedback = provider.get_feedback_at_level(&violation, FeedbackLevel::QuickFix).unwrap();
        assert!(feedback.quick_fix.is_some());
        assert!(feedback.explanation.is_none());
        assert!(feedback.resources.is_empty());
    }
    
    #[test]
    fn test_feedback_missing_suggestion() {
        let provider = FeedbackProvider::new();
        let violation = Violation::warning("UNKNOWN", "Unknown violation");
        
        let feedback = provider.get_feedback(&violation);
        assert!(feedback.is_none());
        
        let desc = provider.generate_description(&violation);
        assert!(desc.contains("UNKNOWN"));
    }
}
