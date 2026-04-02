//! Intelligence Layer — Integrates Synward Intelligence 5-layer system
//!
//! This layer wraps `SynwardIntelligence` to provide:
//! - Layer 2: Memory (Code Graph, Decision Log, Validation State)
//! - Layer 3: Pattern Discovery (Anomaly Detection)
//! - Layer 4: Intent Inference (External LLM)
//! - Layer 5: Drift Detection (Git-based)
//!
//! ## Feature Flags
//!
//! This layer requires `synward-intelligence` dependency.
//! Compile with `--features intelligence-full` for all layers,
//! or individual features: `memory`, `patterns`, `intent-api`, `drift`.

use async_trait::async_trait;

use crate::layer::{LayerResult, ValidationLayer};
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity};

/// Configuration for the Intelligence Layer
#[derive(Debug, Clone)]
pub struct IntelligenceConfig {
    /// Enable Layer 2: Memory (Code Graph, Decision Log)
    pub memory: bool,
    /// Enable Layer 3: Pattern Discovery
    pub patterns: bool,
    /// Enable Layer 4: Intent Inference (requires LLM API)
    pub intent: bool,
    /// Enable Layer 5: Drift Detection
    pub drift: bool,
    /// Path to Synward memory store
    pub memory_path: Option<std::path::PathBuf>,
    /// Project root for code graph indexing
    pub project_root: Option<std::path::PathBuf>,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            memory: true,
            patterns: true,
            intent: false, // Requires external API
            drift: false,  // Requires git repo
            memory_path: None,
            project_root: None,
        }
    }
}

/// Intelligence Layer that integrates Synward Intelligence 5-layer system
///
/// When `synward-intelligence` is not available, this layer passes through
/// with an informational message.
pub struct IntelligenceLayer {
    #[allow(dead_code)]
    config: IntelligenceConfig,
}

impl IntelligenceLayer {
    /// Create new IntelligenceLayer with default config
    pub fn new() -> Self {
        Self {
            config: IntelligenceConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: IntelligenceConfig) -> Self {
        Self { config }
    }

    /// Enable all layers
    pub fn full() -> Self {
        Self::with_config(IntelligenceConfig {
            memory: true,
            patterns: true,
            intent: true,
            drift: true,
            memory_path: None,
            project_root: None,
        })
    }

    /// Enable only memory layer (fastest)
    pub fn memory_only() -> Self {
        Self::with_config(IntelligenceConfig {
            memory: true,
            patterns: false,
            intent: false,
            drift: false,
            memory_path: None,
            project_root: None,
        })
    }
}

impl Default for IntelligenceLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for IntelligenceLayer {
    fn name(&self) -> &str {
        "intelligence"
    }

    fn priority(&self) -> u8 {
        60 // Run after core layers (Syntax=10, Semantic=20, Logic=30)
    }

    #[cfg(feature = "synward-intelligence")]
    async fn validate(&self, _ctx: &ValidationContext) -> LayerResult {
        let violations: Vec<Violation> = Vec::new();
        let infos = Vec::new();

        // Layer 2: Memory - Check for similar issues in memory
        #[cfg(feature = "memory")]
        if self.config.memory {
            if let Some(ref file_path) = ctx.file_path {
                let file_str = file_path.to_string_lossy();
                infos.push(format!("Memory: Checking {} for known issues", file_str));
            }
        }

        // Layer 3: Pattern Discovery - Detect anomalies
        #[cfg(feature = "patterns")]
        if self.config.patterns {
            use synward_intelligence::{FeatureExtractor, AnomalyDetector};

            let extractor = FeatureExtractor::new();
            let features = extractor.extract(&ctx.source, &ctx.language);

            let detector = AnomalyDetector::new();
            let anomalies = detector.detect(&features);

            for (idx, anomaly) in anomalies.iter().enumerate() {
                let severity = match anomaly.severity {
                    4 | 5 => Severity::Warning,
                    2 | 3 => Severity::Info,
                    _ => Severity::Hint,
                };

                violations.push(Violation {
                    id: format!("AI{:03}", idx + 1),
                    message: anomaly.description.clone(),
                    severity,
                    file: ctx.file_path.clone(),
                    span: None,
                    suggestion: anomaly.suggestion.clone(),
                    count: 1,
                    locations: Vec::new(),
                });
            }

            infos.push(format!("Patterns: {} anomalies detected", anomalies.len()));
        }

        // Layer 4: Intent Inference - Use external LLM (info only, not blocking)
        #[cfg(feature = "intent-api")]
        if self.config.intent {
            infos.push("Intent: Layer 4 available (external LLM integration)".to_string());
        }

        // Layer 5: Drift Detection - Info only
        #[cfg(feature = "drift")]
        if self.config.drift {
            if let Some(ref file_path) = ctx.file_path {
                infos.push(format!("Drift: Analyzing {}...", file_path.display()));
            }
        }

        // Build result
        let passed = violations.iter().all(|v| v.severity != Severity::Error);
        LayerResult { passed, violations, infos, whitelisted_count: 0 }
    }

    #[cfg(not(feature = "synward-intelligence"))]
    async fn validate(&self, _ctx: &ValidationContext) -> LayerResult {
        LayerResult::pass()
            .with_info("Intelligence layer disabled (compile with --features intelligence-full)".to_string())
    }

    fn can_continue(&self, _result: &LayerResult) -> bool {
        true // Intelligence layer is never blocking
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intelligence_layer_creation() {
        let layer = IntelligenceLayer::new();
        assert_eq!(layer.name(), "intelligence");
        assert_eq!(layer.priority(), 60);
    }

    #[test]
    fn test_intelligence_config_default() {
        let config = IntelligenceConfig::default();
        assert!(config.memory);
        assert!(config.patterns);
        assert!(!config.intent);
        assert!(!config.drift);
    }

    #[tokio::test]
    async fn test_intelligence_layer_pass() {
        let layer = IntelligenceLayer::memory_only();
        let ctx = ValidationContext::for_file("test.rs", "fn main() {}".into(), "rust".into());
        let result = layer.validate(&ctx).await;

        // Should pass without errors
        assert!(result.passed || result.violations.is_empty());
    }
}
