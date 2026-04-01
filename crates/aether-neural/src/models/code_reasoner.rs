//! Code Reasoner — GNN-based code classification and analysis.
//!
//! This model takes a Code Property Graph as input and produces:
//! - **Classifications**: what kind of issues/patterns exist
//! - **Explanation embedding**: used to generate human-readable explanations
//! - **Fix suggestions**: potential fixes for detected issues
//!
//! The model is exported from NexusTrain as ONNX and compiled into
//! Rust code at build time via `burn-onnx::ModelGen`.

use crate::error::Result;
use crate::inference::CpgTensorInput;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Classification categories produced by the Code Reasoner.
///
/// Maps to the training labels in NexusTrain's GNN config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClassificationCategory {
    /// Unhandled error result (missing ?, unwrap, expect).
    UnhandledError,
    /// Resource leak (file handle, connection not closed).
    ResourceLeak,
    /// Null/None dereference risk.
    NullDereference,
    /// Incorrect error handling (swallowed errors).
    ErrorSwallowed,
    /// Thread safety issue (data race, unsafe sharing).
    ThreadSafety,
    /// Performance anti-pattern (O(n^2), unnecessary allocation).
    Performance,
    /// Security vulnerability (injection, hardcoded secrets).
    Security,
    /// Code smell (deep nesting, long function, god object).
    CodeSmell,
    /// Logic error (off-by-one, wrong comparison).
    LogicError,
    /// API misuse (incorrect trait impl, wrong generic bound).
    ApiMisuse,
    /// No issue detected — clean code pattern.
    Clean,
}

impl ClassificationCategory {
    /// Total number of classification categories.
    pub const COUNT: usize = 12;

    /// Get the index for one-hot encoding.
    pub fn index(&self) -> usize {
        match self {
            Self::UnhandledError => 0,
            Self::ResourceLeak => 1,
            Self::NullDereference => 2,
            Self::ErrorSwallowed => 3,
            Self::ThreadSafety => 4,
            Self::Performance => 5,
            Self::Security => 6,
            Self::CodeSmell => 7,
            Self::LogicError => 8,
            Self::ApiMisuse => 9,
            Self::Clean => 10,
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::UnhandledError => "unhandled_error",
            Self::ResourceLeak => "resource_leak",
            Self::NullDereference => "null_dereference",
            Self::ErrorSwallowed => "error_swallowed",
            Self::ThreadSafety => "thread_safety",
            Self::Performance => "performance",
            Self::Security => "security",
            Self::CodeSmell => "code_smell",
            Self::LogicError => "logic_error",
            Self::ApiMisuse => "api_misuse",
            Self::Clean => "clean",
        }
    }
}

/// A single classification result from the Code Reasoner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    /// The predicted category.
    pub category: ClassificationCategory,
    /// Confidence score for this classification (0.0 — 1.0).
    pub confidence: f32,
    /// Which CPG nodes contributed most (indices + attention weights).
    pub attention_nodes: Vec<(usize, f32)>,
    /// Human-readable description of the issue (template-based).
    pub description: String,
}

/// The Code Reasoner model.
///
/// Wraps the Burn-compiled GNN model for inference on Code Property Graphs.
pub struct CodeReasoner {
    /// Whether the model weights are loaded.
    loaded: bool,
    /// Model name identifier.
    model_name: String,
}

impl CodeReasoner {
    /// Load the Code Reasoner model from the given directory.
    ///
    /// Looks for `code_reasoner.burnpack` in the directory.
    /// If not found, returns an error.
    pub fn load(models_dir: &Path) -> Result<Self> {
        let model_path = models_dir.join("code_reasoner.burnpack");

        if model_path.exists() {
            tracing::info!("Code Reasoner loaded from: {}", model_path.display());
            // TODO: When the ONNX model is compiled via burn-onnx, we will
            // load the generated model struct and its weights here.
            //
            // The generated code will look like:
            //   use crate::generated::code_reasoner::Model;
            //   let model: Model<NdArray> = Model::new(weights);
            //
            // For now, mark as loaded and provide the API shape.
            Ok(Self {
                loaded: true,
                model_name: "code_reasoner".into(),
            })
        } else {
            tracing::warn!(
                "Code Reasoner model not found at: {}",
                model_path.display()
            );
            // Create an unloaded instance — classify() will return
            // placeholder results for testing the pipeline.
            Ok(Self {
                loaded: false,
                model_name: "code_reasoner".into(),
            })
        }
    }

    /// Classify a Code Property Graph.
    ///
    /// Takes the tensor representation of a CPG and returns
    /// classification results with confidence scores.
    pub fn classify(&self, input: &CpgTensorInput) -> Result<Vec<Classification>> {
        if !self.loaded {
            tracing::debug!("Code Reasoner not loaded, returning empty classifications");
            return Ok(vec![]);
        }

        // Validate input
        input.validate()?;

        // TODO: When the ONNX model is compiled, this becomes:
        //
        // let device = burn::prelude::Device::default();
        // let node_tensor = burn::tensor::Tensor::<B, 2>::from_data(
        //     burn::tensor::Data::new(input.node_features.clone(),
        //         [input.num_nodes, input.feature_dim].into()),
        //     &device,
        // );
        // let output = self.model.forward(node_tensor, edge_tensor, edge_type_tensor);
        // parse_output(&output)

        Ok(vec![])
    }

    /// Check if the model is loaded and ready for inference.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Get the model name.
    pub fn model_name(&self) -> &str {
        &self.model_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_category_index() {
        assert_eq!(ClassificationCategory::UnhandledError.index(), 0);
        assert_eq!(ClassificationCategory::Clean.index(), 10);
    }

    #[test]
    fn test_classification_category_label() {
        assert_eq!(ClassificationCategory::UnhandledError.label(), "unhandled_error");
        assert_eq!(ClassificationCategory::Clean.label(), "clean");
    }

    #[test]
    fn test_code_reasoner_load_missing() {
        let reasoner = CodeReasoner::load(Path::new("/nonexistent")).unwrap();
        assert!(!reasoner.is_loaded());
        assert!(reasoner.classify(&CpgTensorInput::empty()).unwrap().is_empty());
    }
}
