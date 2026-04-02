//! Rule Generation - Generate candidate validation rules from patterns

use crate::patterns::{Anomaly, AnomalyType, DiscoveredPattern};
use serde::{Deserialize, Serialize};

/// A candidate validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateRule {
    /// Rule ID
    pub id: String,

    /// Rule description
    pub description: String,

    /// Severity
    pub severity: String,

    /// Pattern to match
    pub pattern: String,

    /// Confidence (0.0 - 1.0)
    pub confidence: f32,

    /// Example violations
    pub examples: Vec<String>,

    /// Whether approved by user
    pub approved: bool,
}

/// Generate candidate rules from discovered patterns
pub struct RuleGenerator {
    /// Minimum confidence to generate a rule
    min_confidence: f32,
}

impl RuleGenerator {
    /// Create a new generator
    pub fn new() -> Self {
        Self {
            min_confidence: 0.7,
        }
    }

    /// Generate rules from patterns
    pub fn generate(&self, patterns: &[DiscoveredPattern]) -> Vec<CandidateRule> {
        patterns
            .iter()
            .filter(|p| p.confidence >= self.min_confidence && !p.approved)
            .map(|p| self.pattern_to_rule(p))
            .collect()
    }

    /// Generate rules from anomalies
    pub fn generate_from_anomalies(&self, anomalies: &[Anomaly]) -> Vec<CandidateRule> {
        anomalies
            .iter()
            .filter(|a| a.severity >= 3)
            .map(|a| self.anomaly_to_rule(a))
            .collect()
    }

    fn pattern_to_rule(&self, pattern: &DiscoveredPattern) -> CandidateRule {
        CandidateRule {
            id: format!("DISCOVERED_{}", pattern.id),
            description: pattern.description.clone(),
            severity: if pattern.confidence > 0.9 {
                "error".to_string()
            } else {
                "warning".to_string()
            },
            pattern: Self::extract_pattern(&pattern.examples),
            confidence: pattern.confidence,
            examples: pattern.examples.clone(),
            approved: false,
        }
    }

    fn anomaly_to_rule(&self, anomaly: &Anomaly) -> CandidateRule {
        let id = match anomaly.anomaly_type {
            AnomalyType::HighComplexity => "COMPLEXITY",
            AnomalyType::DeepNesting => "NESTING",
            AnomalyType::ExcessiveUnwrap => "UNWRAP",
            AnomalyType::MissingErrorHandler => "NO_ERROR_HANDLER",
            AnomalyType::UnusualPattern => "UNUSUAL",
            AnomalyType::QualityIssue => "QUALITY",
        };

        CandidateRule {
            id: format!("DISCOVERED_{}", id),
            description: anomaly.description.clone(),
            severity: if anomaly.severity >= 4 {
                "error".to_string()
            } else {
                "warning".to_string()
            },
            pattern: anomaly.anomaly_type.to_pattern(),
            confidence: 0.8,
            examples: vec![],
            approved: false,
        }
    }

    fn extract_pattern(examples: &[String]) -> String {
        // Placeholder - would extract common pattern from examples
        if examples.is_empty() {
            "".to_string()
        } else {
            examples[0].clone()
        }
    }
}

impl Default for RuleGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AnomalyType {
    #[allow(clippy::wrong_self_convention)]
    fn to_pattern(&self) -> String {
        match self {
            AnomalyType::HighComplexity => "function_length > 50",
            AnomalyType::DeepNesting => "nesting_depth > 4",
            AnomalyType::ExcessiveUnwrap => r#"\.unwrap\(\)"#,
            AnomalyType::MissingErrorHandler => "no_error_handling",
            AnomalyType::UnusualPattern => "unusual",
            AnomalyType::QualityIssue => "quality_issue",
        }
        .to_string()
    }
}
