//! Drift Predictor — temporal GNN for architectural drift prediction.
//!
//! Takes a sequence of CPG snapshots (from git history) and predicts:
//! - Probability of architectural drift
//! - Severity of predicted drift
//! - Affected components
//! - Suggested timeframe
//!
//! Models are exported from NexusTrain as ONNX and compiled into Rust
//! at build time via `burn-onnx::ModelGen`.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A drift prediction result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftPrediction {
    /// Probability of drift occurring (0.0 — 1.0).
    pub probability: f32,
    /// Predicted severity level.
    pub severity: DriftSeverity,
    /// Number of commits before drift is expected (if applicable).
    pub timeframe_commits: Option<usize>,
    /// Components/files most likely to be affected.
    pub affected_components: Vec<String>,
    /// Human-readable explanation of the prediction.
    pub explanation: String,
}

/// Severity levels for drift predictions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftSeverity {
    /// No significant drift expected.
    None,
    /// Minor drift — cosmetic or naming changes.
    Low,
    /// Moderate drift — structural changes within a module.
    Medium,
    /// High drift — cross-module architectural changes.
    High,
    /// Critical — fundamental architectural shift.
    Critical,
}

impl DriftSeverity {
    /// Convert to a numeric score (0.0 — 1.0).
    pub fn score(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Low => 0.25,
            Self::Medium => 0.5,
            Self::High => 0.75,
            Self::Critical => 1.0,
        }
    }
}

/// The Drift Predictor model.
///
/// Uses a Temporal GNN + sequence model to predict architectural drift
/// from git history data.
pub struct DriftPredictor {
    /// Whether the model is loaded.
    loaded: bool,
    /// Model name identifier.
    #[allow(dead_code)]
    model_name: String,
}

impl DriftPredictor {
    /// Load the Drift Predictor model from the given directory.
    ///
    /// Looks for `drift_predictor.burnpack` in the directory.
    pub fn load(models_dir: &Path) -> Result<Self> {
        let model_path = models_dir.join("drift_predictor.burnpack");

        if model_path.exists() {
            tracing::info!("Drift Predictor loaded from: {}", model_path.display());
            // TODO: Load Burn-compiled Temporal GNN model.
            Ok(Self {
                loaded: true,
                model_name: "drift_predictor".into(),
            })
        } else {
            Err(Error::ModelNotFound(format!(
                "drift_predictor.burnpack not found in {}",
                models_dir.display()
            )))
        }
    }

    /// Predict drift from a sequence of temporal snapshots.
    ///
    /// # Arguments
    ///
    /// * `snapshots` — sequence of CPG feature snapshots, ordered by time.
    ///   Each snapshot represents the CPG at a point in time (e.g., per commit).
    ///
    /// # Returns
    ///
    /// A drift prediction with probability, severity, and affected components.
    pub fn predict(&self, _snapshots: &[crate::inference::CpgTensorInput]) -> Result<DriftPrediction> {
        if !self.loaded {
            return Err(Error::ModelNotLoaded("Drift Predictor".into()));
        }

        // TODO: When the ONNX model is compiled:
        // 1. Encode each snapshot via Temporal GNN encoder
        // 2. Run sequence model on the temporal embeddings
        // 3. Parse output into DriftPrediction

        Ok(DriftPrediction {
            probability: 0.0,
            severity: DriftSeverity::None,
            timeframe_commits: None,
            affected_components: vec![],
            explanation: "Model loaded but inference not yet implemented".into(),
        })
    }

    /// Check if the model is loaded.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_severity_score() {
        assert_eq!(DriftSeverity::None.score(), 0.0);
        assert_eq!(DriftSeverity::Low.score(), 0.25);
        assert_eq!(DriftSeverity::Medium.score(), 0.5);
        assert_eq!(DriftSeverity::High.score(), 0.75);
        assert_eq!(DriftSeverity::Critical.score(), 1.0);
    }

    #[test]
    fn test_drift_predictor_load_missing() {
        let result = DriftPredictor::load(Path::new("/nonexistent"));
        assert!(result.is_err());
    }
}
