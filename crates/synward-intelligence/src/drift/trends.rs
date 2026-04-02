//! Trend Analysis - Analyze trends in code metrics

use crate::drift::CodeSnapshot;
use serde::{Deserialize, Serialize};

/// A detected trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trend {
    /// Metric is declining
    Declining {
        metric: String,
        rate: f32,
        severity: String,
    },

    /// Metric is increasing
    Increasing {
        metric: String,
        rate: f32,
        severity: String,
    },

    /// Metric is stable
    Stable {
        metric: String,
        variance: f32,
    },
}

/// Report on drift for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    /// File path
    pub file_path: String,

    /// Overall drift score (0-1, higher = more drift)
    pub drift_score: f32,

    /// Detected trends
    pub trends: Vec<Trend>,

    /// Number of snapshots analyzed
    pub snapshots_analyzed: usize,
}

/// Analyze trends in metrics
pub struct TrendAnalyzer {
    /// Threshold for significant change
    change_threshold: f32,
}

impl TrendAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            change_threshold: 0.02,
        }
    }

    /// Analyze trends from snapshots
    pub fn analyze(&self, snapshots: &[CodeSnapshot]) -> Vec<Trend> {
        if snapshots.len() < 2 {
            return vec![];
        }

        let mut trends = Vec::new();

        // Get first and last metrics (safe after len check)
        let first = match snapshots.first() {
            Some(s) => &s.metrics,
            None => return vec![],
        };
        let last = match snapshots.last() {
            Some(s) => &s.metrics,
            None => return vec![],
        };

        // Analyze each metric
        trends.extend(self.analyze_metric("type_strictness", first.type_strictness, last.type_strictness, false));
        trends.extend(self.analyze_metric("naming_consistency", first.naming_consistency, last.naming_consistency, false));
        trends.extend(self.analyze_metric("error_handling", first.error_handling_quality, last.error_handling_quality, false));
        trends.extend(self.analyze_metric("complexity", first.complexity, last.complexity, true));
        trends.extend(self.analyze_metric("dead_code", first.dead_code_ratio, last.dead_code_ratio, true));

        trends
    }

    fn analyze_metric(&self, name: &str, old: f32, new: f32, inverse: bool) -> Vec<Trend> {
        let diff = new - old;
        let abs_diff = diff.abs();

        if abs_diff < self.change_threshold {
            return vec![Trend::Stable {
                metric: name.to_string(),
                variance: abs_diff,
            }];
        }

        // For inverse metrics (complexity, dead_code), increase is bad
        let is_decline = if inverse { diff > 0.0 } else { diff < 0.0 };

        let severity = if abs_diff > 0.1 {
            "high"
        } else if abs_diff > 0.05 {
            "medium"
        } else {
            "low"
        };

        if is_decline {
            vec![Trend::Declining {
                metric: name.to_string(),
                rate: abs_diff,
                severity: severity.to_string(),
            }]
        } else {
            vec![Trend::Increasing {
                metric: name.to_string(),
                rate: abs_diff,
                severity: severity.to_string(),
            }]
        }
    }
}

impl Default for TrendAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
