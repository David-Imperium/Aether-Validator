//! Neural Orchestrator — ensemble coordination and confidence routing.
//!
//! Coordinates the three neural networks and decides:
//! - When to trust neural results (high confidence)
//! - When to combine neural + symbolic (moderate confidence)
//! - When to defer to symbolic validation (low confidence)
//!
//! Implements the confidence routing defined in AETHER_NEURAL.md:
//!
//! - Confidence > 0.9  -> Neural result, high trust
//! - Confidence 0.7-0.9 -> Neural result + symbolic verification
//! - Confidence 0.5-0.7 -> Symbolic result + neural suggestion
//! - Confidence < 0.5  -> Symbolic only, neural uncertain

use crate::models::code_reasoner::Classification;
use crate::models::pattern_memory::PatternMatch;
use crate::models::drift_predictor::DriftPrediction;
use serde::{Deserialize, Serialize};

/// Confidence threshold for routing decisions.
pub struct ConfidenceThresholds {
    /// Above this: full neural trust.
    pub high: f32,
    /// Above this: neural + symbolic verification.
    pub moderate: f32,
    /// Above this: symbolic with neural suggestions.
    pub low: f32,
}

impl Default for ConfidenceThresholds {
    fn default() -> Self {
        Self {
            high: 0.9,
            moderate: 0.7,
            low: 0.5,
        }
    }
}

/// Confidence level for a neural prediction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// > 0.9: high confidence, trust neural result.
    High,
    /// 0.7 — 0.9: moderate, verify with symbolic.
    Moderate,
    /// 0.5 — 0.7: low, defer to symbolic.
    Low,
    /// < 0.5: very low, ignore neural output.
    Uncertain,
}

impl ConfidenceLevel {
    /// Classify a confidence score into a level.
    pub fn from_score(score: f32, thresholds: &ConfidenceThresholds) -> Self {
        if score >= thresholds.high {
            Self::High
        } else if score >= thresholds.moderate {
            Self::Moderate
        } else if score >= thresholds.low {
            Self::Low
        } else {
            Self::Uncertain
        }
    }
}

/// Routing decision made by the orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// The confidence level of this decision.
    level: ConfidenceLevel,
    /// Overall confidence score (0.0 — 1.0).
    confidence: f32,
    /// Whether to defer to symbolic validation.
    defer_to_symbolic: bool,
    /// Brief explanation for the routing decision.
    explanation: Option<String>,
}

impl RoutingDecision {
    /// Get the confidence score.
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    /// Whether the orchestrator recommends deferring to symbolic validation.
    pub fn should_defer(&self) -> bool {
        self.defer_to_symbolic
    }

    /// Get the explanation for this routing decision.
    pub fn explanation(&self) -> Option<&str> {
        self.explanation.as_deref()
    }
}

/// Neural Orchestrator — coordinates network outputs and makes routing decisions.
pub struct NeuralOrchestrator {
    thresholds: ConfidenceThresholds,
}

impl NeuralOrchestrator {
    /// Create an orchestrator with default confidence thresholds.
    pub fn default() -> Self {
        Self {
            thresholds: ConfidenceThresholds::default(),
        }
    }

    /// Create an orchestrator with custom thresholds.
    pub fn with_thresholds(thresholds: ConfidenceThresholds) -> Self {
        Self { thresholds }
    }

    /// Make a routing decision based on all network outputs.
    ///
    /// The decision is based on:
    /// 1. Code Reasoner confidence (primary signal)
    /// 2. Pattern Memory match quality (supporting signal)
    /// 3. Drift Predictor warning (contextual signal)
    pub fn route(
        &self,
        classifications: &[Classification],
        _similar_patterns: &[PatternMatch],
        _drift_warning: Option<&DriftPrediction>,
    ) -> RoutingDecision {
        if classifications.is_empty() {
            // No classifications — either model not loaded or no issues found.
            // Trust symbolic validation.
            return RoutingDecision {
                level: ConfidenceLevel::Uncertain,
                confidence: 0.0,
                defer_to_symbolic: true,
                explanation: Some("No neural classifications available".into()),
            };
        }

        // Primary signal: max confidence from Code Reasoner
        let max_confidence = classifications
            .iter()
            .map(|c| c.confidence)
            .fold(0.0f32, f32::max);

        // Supporting signal: boost if Pattern Memory found strong matches
        // (not yet implemented — patterns are empty placeholders)
        let adjusted_confidence = max_confidence;

        let level = ConfidenceLevel::from_score(adjusted_confidence, &self.thresholds);
        let defer = matches!(level, ConfidenceLevel::Low | ConfidenceLevel::Uncertain);

        let explanation = match level {
            ConfidenceLevel::High => {
                Some(format!(
                    "Neural classification confident ({:.1}%) — {} issues detected",
                    adjusted_confidence * 100.0,
                    classifications.len()
                ))
            }
            ConfidenceLevel::Moderate => {
                Some(format!(
                    "Neural result with moderate confidence ({:.1}%) — symbolic verification recommended",
                    adjusted_confidence * 100.0
                ))
            }
            ConfidenceLevel::Low => {
                Some(format!(
                    "Neural confidence low ({:.1}%) — deferring to symbolic validation",
                    adjusted_confidence * 100.0
                ))
            }
            ConfidenceLevel::Uncertain => {
                Some("Neural system uncertain — using symbolic validation only".into())
            }
        };

        RoutingDecision {
            level,
            confidence: adjusted_confidence,
            defer_to_symbolic: defer,
            explanation,
        }
    }

    /// Get the current confidence thresholds.
    pub fn thresholds(&self) -> &ConfidenceThresholds {
        &self.thresholds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::code_reasoner::{Classification, ClassificationCategory};

    fn make_classification(confidence: f32) -> Classification {
        Classification {
            category: ClassificationCategory::UnhandledError,
            confidence,
            attention_nodes: vec![],
            description: "test".into(),
        }
    }

    #[test]
    fn test_confidence_level() {
        let t = ConfidenceThresholds::default();
        assert_eq!(ConfidenceLevel::from_score(0.95, &t), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(0.8, &t), ConfidenceLevel::Moderate);
        assert_eq!(ConfidenceLevel::from_score(0.6, &t), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(0.3, &t), ConfidenceLevel::Uncertain);
    }

    #[test]
    fn test_route_high_confidence() {
        let orchestrator = NeuralOrchestrator::default();
        let classifications = vec![make_classification(0.95)];

        let decision = orchestrator.route(&classifications, &[], None);
        assert_eq!(decision.level, ConfidenceLevel::High);
        assert!(!decision.should_defer());
        assert!(decision.confidence() > 0.9);
    }

    #[test]
    fn test_route_low_confidence() {
        let orchestrator = NeuralOrchestrator::default();
        let classifications = vec![make_classification(0.3)];

        let decision = orchestrator.route(&classifications, &[], None);
        assert_eq!(decision.level, ConfidenceLevel::Uncertain);
        assert!(decision.should_defer());
    }

    #[test]
    fn test_route_empty_classifications() {
        let orchestrator = NeuralOrchestrator::default();
        let decision = orchestrator.route(&[], &[], None);

        assert_eq!(decision.level, ConfidenceLevel::Uncertain);
        assert!(decision.should_defer());
        assert!(decision.explanation().unwrap().contains("No neural"));
    }

    #[test]
    fn test_route_multiple_classifications() {
        let orchestrator = NeuralOrchestrator::default();
        let classifications = vec![
            make_classification(0.7),
            make_classification(0.85),
            make_classification(0.4),
        ];

        let decision = orchestrator.route(&classifications, &[], None);
        // Max confidence is 0.85 → Moderate
        assert_eq!(decision.level, ConfidenceLevel::Moderate);
        assert!(!decision.should_defer());
    }
}
