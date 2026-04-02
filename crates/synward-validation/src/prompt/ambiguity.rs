//! Ambiguity Detection
//!
//! Identifies unclear or underspecified parts of a request.

use super::intent::Intent;
use super::scope::ScopeResult;

/// Type of ambiguity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmbiguityType {
    /// What to modify.
    Scope,
    /// What value to use.
    Value,
    /// Where to add.
    Location,
    /// How it should work.
    Behavior,
    /// What depends on this.
    Dependency,
    /// Multiple interpretations.
    Conflict,
}

impl std::fmt::Display for AmbiguityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scope => write!(f, "SCOPE"),
            Self::Value => write!(f, "VALUE"),
            Self::Location => write!(f, "LOCATION"),
            Self::Behavior => write!(f, "BEHAVIOR"),
            Self::Dependency => write!(f, "DEPENDENCY"),
            Self::Conflict => write!(f, "CONFLICT"),
        }
    }
}

/// An ambiguity in the prompt.
#[derive(Debug, Clone)]
pub struct Ambiguity {
    /// Ambiguity type.
    pub ambiguity_type: AmbiguityType,
    /// Human-readable description.
    pub description: String,
    /// Question to ask the user.
    pub question: String,
    /// Suggested answers.
    pub options: Vec<String>,
    /// Severity (0.0-1.0, how critical).
    pub severity: f32,
}

impl Ambiguity {
    /// Create a new ambiguity.
    pub fn new(ambiguity_type: AmbiguityType, description: impl Into<String>) -> Self {
        Self {
            ambiguity_type,
            description: description.into(),
            question: String::new(),
            options: Vec::new(),
            severity: 0.5,
        }
    }

    /// Set the question.
    pub fn with_question(mut self, question: impl Into<String>) -> Self {
        self.question = question.into();
        self
    }

    /// Add an option.
    pub fn with_option(mut self, option: impl Into<String>) -> Self {
        self.options.push(option.into());
        self
    }

    /// Set severity.
    pub fn with_severity(mut self, severity: f32) -> Self {
        self.severity = severity.clamp(0.0, 1.0);
        self
    }
}

/// Request for clarification.
#[derive(Debug, Clone)]
pub struct ClarificationRequest {
    /// Message to show the user.
    pub message: String,
    /// Ambiguities that need clarification.
    pub ambiguities: Vec<Ambiguity>,
    /// Suggested answers.
    pub suggested_answers: Vec<String>,
}

impl ClarificationRequest {
    /// Create a new clarification request.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ambiguities: Vec::new(),
            suggested_answers: Vec::new(),
        }
    }

    /// Add an ambiguity.
    pub fn add_ambiguity(&mut self, ambiguity: Ambiguity) {
        self.ambiguities.push(ambiguity);
    }
}

/// Detects ambiguities in prompts.
pub struct AmbiguityDetector {
    /// Value-related keywords that need specification.
    value_keywords: Vec<String>,
    /// Location-related keywords.
    location_keywords: Vec<String>,
    /// Vague behavior keywords.
    behavior_keywords: Vec<String>,
}

impl AmbiguityDetector {
    /// Create a new detector.
    pub fn new() -> Self {
        Self {
            value_keywords: vec![
                "speed".into(), "size".into(), "amount".into(),
                "count".into(), "value".into(), "number".into(),
                "duration".into(), "timeout".into(),
            ],
            location_keywords: vec![
                "where".into(), "location".into(), "position".into(),
                "place".into(), "add".into(),
            ],
            behavior_keywords: vec![
                "better".into(), "faster".into(), "improve".into(),
                "optimize".into(), "fix".into(),
            ],
        }
    }

    /// Detect ambiguities in a prompt.
    pub fn detect(
        &self,
        prompt: &str,
        intent: Intent,
        scope: &ScopeResult,
    ) -> Vec<Ambiguity> {
        let mut ambiguities = Vec::new();
        let prompt_lower = prompt.to_lowercase();

        // Check for scope ambiguity
        if (scope.is_ambiguous || scope.entities.is_empty())
            && (intent == Intent::Modify || intent == Intent::Fix) {
                ambiguities.push(
                    Ambiguity::new(AmbiguityType::Scope, "No specific target identified")
                        .with_question("Which file or component should I modify?")
                        .with_severity(0.8)
                );
            }

        // Check for value ambiguity
        for keyword in &self.value_keywords {
            if prompt_lower.contains(keyword) {
                // Check if a value is specified
                if !self.has_value_specification(&prompt_lower, keyword) {
                    ambiguities.push(
                        Ambiguity::new(AmbiguityType::Value, format!("{} value not specified", keyword))
                            .with_question(format!("What {} value should be used?", keyword))
                            .with_option("Use default value")
                            .with_option("Specify custom value")
                            .with_severity(0.4)
                    );
                }
            }
        }

        // Check for location ambiguity
        if intent == Intent::Create {
            for keyword in &self.location_keywords {
                if prompt_lower.contains(keyword) && scope.entities.is_empty() {
                    ambiguities.push(
                        Ambiguity::new(AmbiguityType::Location, "Location not specified")
                            .with_question("Where should this be added?")
                            .with_severity(0.5)
                    );
                    break;
                }
            }
        }

        // Check for behavior ambiguity
        for keyword in &self.behavior_keywords {
            if prompt_lower.contains(keyword) {
                ambiguities.push(
                    Ambiguity::new(AmbiguityType::Behavior, format!("Vague behavior: '{}'", keyword))
                        .with_question(format!("What does '{}' mean in this context?", keyword))
                        .with_severity(0.6)
                );
            }
        }

        // Check for dependency ambiguity (DELETE intent)
        if intent == Intent::Delete {
            ambiguities.push(
                Ambiguity::new(AmbiguityType::Dependency, "Deletion may affect other components")
                    .with_question("This component may be used elsewhere. Continue?")
                    .with_option("Yes, delete it")
                    .with_option("No, keep it")
                    .with_option("Show dependencies first")
                    .with_severity(0.7)
            );
        }

        ambiguities
    }

    /// Check if a value is specified after a keyword.
    fn has_value_specification(&self, prompt: &str, keyword: &str) -> bool {
        if let Some(pos) = prompt.find(keyword) {
            let after = &prompt[pos + keyword.len()..];
            // Look for numbers or specific values
            for word in after.split_whitespace().take(5) {
                if word.parse::<f64>().is_ok() {
                    return true;
                }
                if ["default", "custom", "auto", "manual"].contains(&word) {
                    return true;
                }
            }
        }
        false
    }

    /// Create a clarification request from ambiguities.
    pub fn create_clarification(&self, ambiguities: Vec<Ambiguity>) -> Option<ClarificationRequest> {
        if ambiguities.is_empty() {
            return None;
        }

        let mut request = ClarificationRequest::new(
            "I need some clarification before proceeding:"
        );

        for ambiguity in ambiguities {
            request.add_ambiguity(ambiguity);
        }

        Some(request)
    }
}

impl Default for AmbiguityDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_scope_ambiguity() {
        let detector = AmbiguityDetector::new();
        let scope = ScopeResult::new();
        let ambiguities = detector.detect("Fix the bug", Intent::Fix, &scope);
        
        assert!(!ambiguities.is_empty());
        assert!(ambiguities.iter().any(|a| a.ambiguity_type == AmbiguityType::Scope));
    }

    #[test]
    fn test_detect_value_ambiguity() {
        let detector = AmbiguityDetector::new();
        let scope = ScopeResult::new();
        let ambiguities = detector.detect("Set the speed", Intent::Modify, &scope);
        
        assert!(ambiguities.iter().any(|a| a.ambiguity_type == AmbiguityType::Value));
    }

    #[test]
    fn test_detect_behavior_ambiguity() {
        let detector = AmbiguityDetector::new();
        let scope = ScopeResult::new();
        let ambiguities = detector.detect("Make it faster", Intent::Modify, &scope);
        
        assert!(ambiguities.iter().any(|a| a.ambiguity_type == AmbiguityType::Behavior));
    }

    #[test]
    fn test_detect_dependency_ambiguity() {
        let detector = AmbiguityDetector::new();
        let scope = ScopeResult::new();
        let ambiguities = detector.detect("Delete the class", Intent::Delete, &scope);
        
        assert!(ambiguities.iter().any(|a| a.ambiguity_type == AmbiguityType::Dependency));
    }

    #[test]
    fn test_no_ambiguity_with_value() {
        let detector = AmbiguityDetector::new();
        let scope = ScopeResult::new();
        let ambiguities = detector.detect("Set the speed to 10", Intent::Modify, &scope);
        
        // Should not have value ambiguity since value is specified
        assert!(!ambiguities.iter().any(|a| a.ambiguity_type == AmbiguityType::Value));
    }

    #[test]
    fn test_create_clarification() {
        let detector = AmbiguityDetector::new();
        let scope = ScopeResult::new();
        let ambiguities = detector.detect("Fix the bug", Intent::Fix, &scope);
        let clarification = detector.create_clarification(ambiguities);
        
        assert!(clarification.is_some());
        let request = clarification.unwrap();
        assert!(!request.ambiguities.is_empty());
    }

    #[test]
    fn test_ambiguity_severity() {
        let ambiguity = Ambiguity::new(AmbiguityType::Value, "test")
            .with_severity(0.5);
        
        assert_eq!(ambiguity.severity, 0.5);
    }

    #[test]
    fn test_ambiguity_type_display() {
        assert_eq!(AmbiguityType::Scope.to_string(), "SCOPE");
        assert_eq!(AmbiguityType::Value.to_string(), "VALUE");
    }
}
