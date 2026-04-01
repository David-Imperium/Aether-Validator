//! Error types for the Aether Neural crate.

use thiserror::Error;

/// Result type for the neural crate.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    /// A required model file was not found.
    #[error("model not found: {0}")]
    ModelNotFound(String),

    /// A model is not loaded (optional network).
    #[error("model not loaded: {0}")]
    ModelNotLoaded(String),

    /// Error during model loading (weights, parsing).
    #[error("failed to load model: {0}")]
    LoadFailed(String),

    /// Error during inference.
    #[error("inference error: {0}")]
    InferenceFailed(String),

    /// Error in feature extraction (CPG, vectorization).
    #[error("feature extraction error: {0}")]
    FeatureExtraction(String),

    /// Invalid input shape or dimensions.
    #[error("shape mismatch: expected {expected}, got {actual}")]
    ShapeMismatch { expected: String, actual: String },

    /// Burn framework error.
    #[error("burn error: {0}")]
    Burn(String),

    /// ONNX Runtime fallback error.
    #[cfg(feature = "onnx-fallback")]
    #[error("ONNX Runtime error: {0}")]
    OnnxRuntime(#[from] ort::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Create a Burn error from a string message.
    pub fn burn_msg(msg: impl Into<String>) -> Self {
        Self::Burn(msg.into())
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::InferenceFailed(err.to_string())
    }
}
