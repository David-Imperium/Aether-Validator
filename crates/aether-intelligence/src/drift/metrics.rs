//! Drift Metrics - Metrics for tracking code quality over time

use serde::{Deserialize, Serialize};

/// Overall drift metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DriftMetrics {
    /// Overall drift score (0-1, higher = more drift)
    pub score: f32,

    /// Rate of change
    pub rate: f32,

    /// Confidence in the measurement
    pub confidence: f32,
}

impl Default for DriftMetrics {
    fn default() -> Self {
        Self {
            score: 0.0,
            rate: 0.0,
            confidence: 1.0,
        }
    }
}

/// Metrics at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetrics {
    /// Type strictness score (0-1)
    pub type_strictness: f32,

    /// Naming consistency score (0-1)
    pub naming_consistency: f32,

    /// Error handling quality (0-1)
    pub error_handling_quality: f32,

    /// Code complexity score (0-1, lower = better)
    pub complexity: f32,

    /// Dead code ratio (0-1)
    pub dead_code_ratio: f32,

    /// Documentation coverage (0-1)
    pub doc_coverage: f32,

    /// Test coverage estimate (0-1)
    pub test_coverage: f32,
}

impl Default for SnapshotMetrics {
    fn default() -> Self {
        Self {
            type_strictness: 1.0,
            naming_consistency: 1.0,
            error_handling_quality: 1.0,
            complexity: 0.0,
            dead_code_ratio: 0.0,
            doc_coverage: 0.0,
            test_coverage: 0.0,
        }
    }
}

impl SnapshotMetrics {
    /// Create from feature extraction
    pub fn from_features(features: &crate::patterns::CodeFeatures) -> Self {
        Self {
            type_strictness: 1.0,
            naming_consistency: 1.0,
            error_handling_quality: if features.unwrap_count > 0 {
                (1.0 - features.unwrap_count as f32 / 10.0).max(0.0)
            } else {
                1.0
            },
            complexity: (features.cyclomatic_complexity as f32 / 20.0).min(1.0),
            dead_code_ratio: 0.0,
            doc_coverage: if features.comment_count > 0 { 0.5 } else { 0.0 },
            test_coverage: 0.0,
        }
    }
}
