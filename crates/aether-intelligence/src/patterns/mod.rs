//! Layer 3: Pattern Discovery
//!
//! Rule-based pattern discovery for finding anti-patterns.

mod features;
mod anomaly;
mod rule_gen;

pub use features::{CodeFeatures, FeatureExtractor};
pub use anomaly::{Anomaly, AnomalyDetector, AnomalyType};
pub use rule_gen::{CandidateRule, RuleGenerator};

use serde::{Deserialize, Serialize};

/// A discovered pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPattern {
    /// Pattern identifier
    pub id: String,

    /// Pattern description
    pub description: String,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Number of occurrences found
    pub occurrences: usize,

    /// Example code snippets
    pub examples: Vec<String>,

    /// Whether this is approved as a rule
    pub approved: bool,
}
