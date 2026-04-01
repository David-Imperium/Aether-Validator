//! Burn inference engine — abstracts backend selection and model execution.
//!
//! This module provides the core inference infrastructure:
//! - Backend selection (NdArray, WGPU, Candle)
//! - Model loading (default weights, burnpack files)
//! - Tensor conversion utilities
//!
//! The actual ONNX model is compiled into Rust code at build time by
//! `burn-onnx::ModelGen` in build.rs. This module provides the runtime
//! infrastructure to execute it.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration for the inference engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Backend to use for inference.
    pub backend: BackendType,

    /// Path to model weights directory.
    pub weights_dir: PathBuf,

    /// Enable device auto-detection (GPU if available).
    pub auto_device: bool,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            backend: BackendType::NdArray,
            weights_dir: PathBuf::from(".aether/models"),
            auto_device: true,
        }
    }
}

/// Available inference backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendType {
    /// Pure CPU backend (NdArray). Portable, no GPU dependency.
    NdArray,
    /// GPU-accelerated via Vulkan/Metal/DirectX.
    Wgpu,
    /// HuggingFace Candle backend. Good for transformer models.
    Candle,
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NdArray => write!(f, "ndarray"),
            Self::Wgpu => write!(f, "wgpu"),
            Self::Candle => write!(f, "candle"),
        }
    }
}

/// The neural inference engine.
///
/// Wraps Burn's backend with device management and model lifecycle.
/// Generic over the Burn backend trait.
pub struct NeuralInference<B: burn::prelude::Backend> {
    /// Burn device (CPU or GPU).
    device: burn::prelude::Device<B>,
    /// Path to model weights.
    weights_dir: PathBuf,
    _backend: std::marker::PhantomData<B>,
}

impl<B: burn::prelude::Backend> NeuralInference<B> {
    /// Create a new inference engine with the given weights directory.
    ///
    /// Uses the default device for the backend (CPU for NdArray,
    /// auto-detected GPU for WGPU).
    pub fn new(weights_dir: &Path) -> Result<Self>
    where
        B::Device: Default,
    {
        let device = <B::Device as Default>::default();
        Ok(Self {
            device,
            weights_dir: weights_dir.to_path_buf(),
            _backend: std::marker::PhantomData,
        })
    }

    /// Create inference engine targeting a specific device.
    pub fn with_device(weights_dir: &Path, device: burn::prelude::Device<B>) -> Self {
        Self {
            device,
            weights_dir: weights_dir.to_path_buf(),
            _backend: std::marker::PhantomData,
        }
    }

    /// Get a reference to the device.
    pub fn device(&self) -> &burn::prelude::Device<B> {
        &self.device
    }

    /// Get the weights directory path.
    pub fn weights_dir(&self) -> &Path {
        &self.weights_dir
    }

    /// Check if a model weights file exists.
    pub fn has_weights(&self, name: &str) -> bool {
        self.weights_dir.join(format!("{}.burnpack", name)).exists()
    }

    /// List all available .burnpack model files.
    pub fn available_models(&self) -> Vec<String> {
        std::fs::read_dir(&self.weights_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        e.path()
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext == "burnpack")
                            .unwrap_or(false)
                            .then(|| {
                                e.path()
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("")
                                    .to_string()
                            })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// CPG feature tensor ready for neural inference.
///
/// Contains the preprocessed node features, edge index, and edge type
/// tensors needed by the GNN models.
#[derive(Debug, Clone)]
pub struct CpgTensorInput {
    /// Node feature matrix [num_nodes, feature_dim].
    /// Each row is a 34-dim feature vector for a CPG node.
    pub node_features: Vec<f32>,
    /// Number of nodes.
    pub num_nodes: usize,
    /// Feature dimensionality (34 for standard CPG features).
    pub feature_dim: usize,
    /// Edge source indices (COO format).
    pub edge_src: Vec<usize>,
    /// Edge destination indices (COO format).
    pub edge_dst: Vec<usize>,
    /// Edge type indices (0-7 for 8 edge types).
    pub edge_types: Vec<usize>,
    /// Number of edges.
    pub num_edges: usize,
    /// Entry point mask: 1.0 for entry point nodes, 0.0 otherwise.
    pub entry_mask: Vec<f32>,
}

impl CpgTensorInput {
    /// Create an empty input (no nodes, no edges).
    pub fn empty() -> Self {
        Self {
            node_features: vec![],
            num_nodes: 0,
            feature_dim: 34,
            edge_src: vec![],
            edge_dst: vec![],
            edge_types: vec![],
            num_edges: 0,
            entry_mask: vec![],
        }
    }

    /// Validate consistency of the tensor data.
    pub fn validate(&self) -> Result<()> {
        let expected_features_len = self.num_nodes * self.feature_dim;
        if self.node_features.len() != expected_features_len {
            return Err(Error::ShapeMismatch {
                expected: format!("[{}, {}] = {} elements", self.num_nodes, self.feature_dim, expected_features_len),
                actual: format!("{} elements", self.node_features.len()),
            });
        }

        if self.edge_src.len() != self.num_edges
            || self.edge_dst.len() != self.num_edges
            || self.edge_types.len() != self.num_edges
        {
            return Err(Error::ShapeMismatch {
                expected: format!("{} edges", self.num_edges),
                actual: format!(
                    "src={}, dst={}, types={}",
                    self.edge_src.len(),
                    self.edge_dst.len(),
                    self.edge_types.len()
                ),
            });
        }

        if self.entry_mask.len() != self.num_nodes {
            return Err(Error::ShapeMismatch {
                expected: format!("{} entry mask values", self.num_nodes),
                actual: format!("{} values", self.entry_mask.len()),
            });
        }

        Ok(())
    }

    /// Total number of parameters (for logging).
    pub fn param_count(&self) -> usize {
        self.node_features.len()
            + self.edge_src.len()
            + self.edge_dst.len()
            + self.edge_types.len()
            + self.entry_mask.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type_display() {
        assert_eq!(BackendType::NdArray.to_string(), "ndarray");
        assert_eq!(BackendType::Wgpu.to_string(), "wgpu");
        assert_eq!(BackendType::Candle.to_string(), "candle");
    }

    #[test]
    fn test_cpg_tensor_input_empty() {
        let input = CpgTensorInput::empty();
        assert_eq!(input.num_nodes, 0);
        assert_eq!(input.num_edges, 0);
        assert!(input.validate().is_ok());
    }

    #[test]
    fn test_cpg_tensor_input_validation() {
        // Correct input
        let input = CpgTensorInput {
            node_features: vec![0.0; 34 * 2], // 2 nodes, 34 features
            num_nodes: 2,
            feature_dim: 34,
            edge_src: vec![0],
            edge_dst: vec![1],
            edge_types: vec![0],
            num_edges: 1,
            entry_mask: vec![1.0, 0.0],
        };
        assert!(input.validate().is_ok());

        // Mismatched features
        let bad = CpgTensorInput {
            node_features: vec![0.0; 33], // wrong count
            num_nodes: 2,
            feature_dim: 34,
            edge_src: vec![],
            edge_dst: vec![],
            edge_types: vec![],
            num_edges: 0,
            entry_mask: vec![1.0, 0.0],
        };
        assert!(bad.validate().is_err());
    }
}
