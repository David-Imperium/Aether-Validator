//! Aether Neural — Burn-based inference for neuro-symbolic code analysis.
//!
//! This crate provides the neural layer of the Aether system, running
//! ONNX models exported from NexusTrain via the Burn framework.
//!
//! # Architecture
//!
//! Three neural networks collaborate through an orchestrator:
//! - **Code Reasoner** (GNN): classifies code patterns from Code Property Graphs
//! - **Pattern Memory** (TreeFFN + Hopfield): semantic similarity and experience retrieval
//! - **Drift Predictor** (Temporal GNN): predicts architectural drift
//!
//! # Feature Flags
//!
//! - `backend-ndarray` (default): CPU inference, portable
//! - `backend-wgpu`: GPU-accelerated via Vulkan/Metal/DirectX
//! - `backend-candle`: HuggingFace Candle backend
//! - `onnx-fallback`: ONNX Runtime for ops unsupported by Burn codegen
//! - `tree-sitter`: runtime CPG extraction
//!
//! # Example
//!
//! ```ignore
//! use aether_neural::{AetherNeural, NeuralConfig};
//!
//! let neural = AetherNeural::load(NeuralConfig::default())?;
//! let result = neural.analyze("fn main() {}", "rust")?;
//! println!("Classification: {:?}", result.classifications);
//! ```

pub mod error;
pub mod inference;
pub mod model_registry;

pub mod models;
pub mod features;

pub mod orchestrator;

pub use error::{Error, Result};
pub use inference::{NeuralInference, InferenceConfig};
pub use model_registry::{ModelRegistry, ModelInfo, ModelVersion};
pub use orchestrator::{NeuralOrchestrator, ConfidenceLevel, ConfidenceThresholds, RoutingDecision};

pub use models::code_reasoner::{CodeReasoner, Classification, ClassificationCategory};
pub use models::pattern_memory::{PatternMemory, PatternMatch, ExperienceMeta};
pub use models::drift_predictor::{DriftPredictor, DriftPrediction, DriftSeverity};

pub use features::cpg::CpgFeatureExtractor;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the Aether Neural system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralConfig {
    /// Directory containing model files (.onnx, .burnpack).
    pub models_dir: PathBuf,

    /// Maximum number of nodes per CPG for inference (truncates large graphs).
    pub max_nodes: usize,

    /// Maximum number of edges per CPG for inference.
    pub max_edges: usize,

    /// Whether to enable GPU acceleration (if available).
    pub use_gpu: bool,

    /// Enable ONNX Runtime fallback for unsupported ops.
    pub enable_onnx_fallback: bool,
}

impl Default for NeuralConfig {
    fn default() -> Self {
        Self {
            models_dir: PathBuf::from(".aether/models"),
            max_nodes: 500,
            max_edges: 2000,
            use_gpu: false,
            enable_onnx_fallback: false,
        }
    }
}

/// Main entry point for the Aether Neural system.
///
/// Coordinates all three neural networks and the orchestrator
/// for unified neuro-symbolic analysis.
pub struct AetherNeural {
    config: NeuralConfig,
    code_reasoner: CodeReasoner,
    pattern_memory: Option<PatternMemory>,
    drift_predictor: Option<DriftPredictor>,
    orchestrator: NeuralOrchestrator,
    registry: ModelRegistry,
}

impl AetherNeural {
    /// Load the neural system from the configured models directory.
    ///
    /// Returns an error if no models are found. Individual networks
    /// are loaded lazily — missing model files only log warnings.
    pub fn load(config: NeuralConfig) -> Result<Self> {
        let registry = ModelRegistry::scan(&config.models_dir)?;
        let code_reasoner = CodeReasoner::load(&config.models_dir)?;

        // Optional networks — don't fail if models are missing
        let pattern_memory = PatternMemory::load(&config.models_dir)
            .map_err(|e| {
                tracing::warn!("Pattern Memory model not loaded: {}", e);
                e
            })
            .ok();

        let drift_predictor = DriftPredictor::load(&config.models_dir)
            .map_err(|e| {
                tracing::warn!("Drift Predictor model not loaded: {}", e);
                e
            })
            .ok();

        let orchestrator = NeuralOrchestrator::default();

        tracing::info!(
            "Aether Neural loaded: Code Reasoner={}, Pattern Memory={}, Drift Predictor={}",
            "active",
            if pattern_memory.is_some() { "active" } else { "unavailable" },
            if drift_predictor.is_some() { "active" } else { "unavailable" },
        );

        Ok(Self {
            config,
            code_reasoner,
            pattern_memory,
            drift_predictor,
            orchestrator,
            registry,
        })
    }

    /// Analyze source code through the neural pipeline.
    ///
    /// 1. Extract CPG features
    /// 2. Run Code Reasoner (classification + explanation)
    /// 3. Run Pattern Memory (similarity search)
    /// 4. Run Drift Predictor (if temporal data available)
    /// 5. Orchestrate: fuse signals, produce result
    pub fn analyze(&self, source: &str, language: &str) -> Result<NeuralResult> {
        let extractor = CpgFeatureExtractor::new(self.config.max_nodes, self.config.max_edges);

        // 1. Feature extraction
        let features = extractor.extract(source, language)?;

        // 2. Code Reasoner — always available
        let classifications = self.code_reasoner.classify(&features)?;

        // 3. Pattern Memory — optional
        let similar_patterns = self
            .pattern_memory
            .as_ref()
            .map(|pm| pm.find_similar(&features))
            .transpose()?
            .unwrap_or_default();

        // 4. Drift — optional, needs temporal data
        let drift_warning = None; // TODO: wire when temporal data available

        // 5. Orchestration
        let routing = self.orchestrator.route(
            &classifications,
            &similar_patterns,
            drift_warning.as_ref(),
        );

        Ok(NeuralResult {
            classifications,
            explanation: routing.explanation().map(String::from),
            similar_patterns,
            fix_suggestions: vec![],
            drift_warning,
            confidence: routing.confidence(),
            should_defer_to_symbolic: routing.should_defer(),
        })
    }

    /// Query the Pattern Memory for similar code patterns.
    pub fn recall_similar(&self, source: &str, k: usize) -> Result<Vec<PatternMatch>> {
        match &self.pattern_memory {
            Some(pm) => {
                let extractor = CpgFeatureExtractor::new(self.config.max_nodes, self.config.max_edges);
                let features = extractor.extract(source, "auto")?;
                let matches = pm.find_similar_top_k(&features, k)?;
                Ok(matches)
            }
            None => Err(Error::ModelNotLoaded("Pattern Memory".into())),
        }
    }

    /// Store an experience embedding in the Pattern Memory.
    ///
    /// This is the primary API for populating the pattern memory from
    /// external sources (e.g. validation results, TreeFFN sidecar).
    /// The embedding must have dimensionality matching the store (default 256).
    pub fn store_experience(
        &mut self,
        embedding: &[f32],
        meta: models::pattern_memory::ExperienceMeta,
    ) -> Result<()> {
        let pm = self.pattern_memory.as_mut()
            .ok_or_else(|| Error::ModelNotLoaded("Pattern Memory".into()))?;
        pm.store_embedding(embedding, meta);
        Ok(())
    }

    /// Store a batch of experience embeddings in the Pattern Memory.
    pub fn store_experience_batch(
        &mut self,
        embeddings: &[f32],
        metas: Vec<models::pattern_memory::ExperienceMeta>,
    ) -> Result<()> {
        let pm = self.pattern_memory.as_mut()
            .ok_or_else(|| Error::ModelNotLoaded("Pattern Memory".into()))?;
        pm.store_batch(embeddings, metas);
        Ok(())
    }

    /// Search the Pattern Memory with a pre-computed embedding vector.
    ///
    /// Unlike `recall_similar`, this does not require the TreeFFN encoder —
    /// useful when embeddings come from an external source.
    pub fn search_memory(&self, query: &[f32], k: usize) -> Result<Vec<PatternMatch>> {
        let pm = self.pattern_memory.as_ref()
            .ok_or_else(|| Error::ModelNotLoaded("Pattern Memory".into()))?;
        Ok(pm.search(query, k))
    }

    /// Persist the Pattern Memory state to disk.
    pub fn persist_pattern_memory(&self) -> Result<()> {
        let pm = self.pattern_memory.as_ref()
            .ok_or_else(|| Error::ModelNotLoaded("Pattern Memory".into()))?;
        pm.persist(&self.config.models_dir)
    }

    /// Number of patterns stored in the Pattern Memory.
    pub fn pattern_memory_size(&self) -> usize {
        self.pattern_memory.as_ref().map(|pm| pm.num_stored()).unwrap_or(0)
    }

    /// Get the current status of the neural system.
    pub fn status(&self) -> NeuralStatus {
        NeuralStatus {
            code_reasoner_loaded: true,
            pattern_memory_loaded: self.pattern_memory.is_some(),
            drift_predictor_loaded: self.drift_predictor.is_some(),
            models: self.registry.list_models(),
        }
    }
}

/// Result of neural analysis on a source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralResult {
    /// Classifications from Code Reasoner (Rete A).
    pub classifications: Vec<Classification>,

    /// Human-readable explanation (if confidence is high enough).
    pub explanation: Option<String>,

    /// Similar patterns from Pattern Memory (Rete B).
    pub similar_patterns: Vec<PatternMatch>,

    /// Fix suggestions from Code Reasoner + Pattern Memory.
    pub fix_suggestions: Vec<FixSuggestion>,

    /// Drift prediction warning (Rete C), if temporal data available.
    pub drift_warning: Option<DriftPrediction>,

    /// Overall confidence score (0.0 — 1.0) from the orchestrator.
    pub confidence: f32,

    /// Whether the neural system is uncertain and should defer to symbolic validation.
    pub should_defer_to_symbolic: bool,
}

/// A fix suggestion produced by the neural system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixSuggestion {
    /// Description of the suggested fix.
    pub description: String,
    /// Confidence in this fix (0.0 — 1.0).
    pub confidence: f32,
    /// Region of code this fix applies to.
    pub region: CodeRegion,
}

/// A region of source code (line-based).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRegion {
    pub start_line: usize,
    pub end_line: usize,
    pub file_path: String,
}

/// Status of the neural system.
#[derive(Debug, Serialize)]
pub struct NeuralStatus {
    pub code_reasoner_loaded: bool,
    pub pattern_memory_loaded: bool,
    pub drift_predictor_loaded: bool,
    pub models: Vec<ModelInfo>,
}
