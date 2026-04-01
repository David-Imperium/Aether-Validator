//! Layer 5: Drift Detection
//!
//! Temporal analysis for detecting code quality degradation.

mod metrics;
mod git_integration;
mod trends;
mod detector;

pub use metrics::{DriftMetrics, SnapshotMetrics};
pub use git_integration::GitIntegration;
pub use trends::{Trend, TrendAnalyzer, DriftReport};
pub use detector::DriftDetector;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A snapshot of code at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnapshot {
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,

    /// Commit hash (if from git)
    pub commit: Option<String>,

    /// File path
    pub file_path: String,

    /// Metrics at this point
    pub metrics: SnapshotMetrics,
}
