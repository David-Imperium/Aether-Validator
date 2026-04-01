//! Validation Pipeline — Coordinates validation layers

use std::sync::Arc;

use crate::layer::{ValidationLayer, LayerResult, LayerConfig};
use crate::context::ValidationContext;

/// Validation pipeline that coordinates multiple layers.
///
/// ## Memory-Driven Mode
///
/// When `config` is set via `with_config()`, all layers receive
/// the learned configuration enabling dynamic behavior:
/// - Thresholds adjusted based on project history
/// - Custom rules from discovered patterns
/// - Whitelisted patterns from user acceptance
/// - Style conventions from codebase analysis
pub struct ValidationPipeline {
    layers: Vec<Arc<dyn ValidationLayer>>,
    config: Option<LayerConfig>,
}

impl ValidationPipeline {
    /// Create a new empty pipeline.
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            config: None,
        }
    }

    /// Add a layer to the pipeline.
    pub fn add_layer(mut self, layer: impl ValidationLayer + 'static) -> Self {
        self.layers.push(Arc::new(layer));
        self
    }

    /// Set the learned configuration (Memory-Driven mode).
    ///
    /// This enables all layers to receive dynamic configuration.
    pub fn with_config(mut self, config: LayerConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Get the current configuration.
    pub fn config(&self) -> Option<&LayerConfig> {
        self.config.as_ref()
    }

    /// Execute all layers in order.
    pub async fn execute(&self, ctx: &ValidationContext) -> PipelineResult {
        let mut results = Vec::new();
        let mut stopped_at = None;

        for layer in &self.layers {
            // Use config-aware validation if config is set
            let result = match &self.config {
                Some(cfg) => layer.validate_with_config(ctx, Some(cfg)).await,
                None => layer.validate(ctx).await,
            };

            let can_continue = layer.can_continue(&result);

            if !can_continue {
                stopped_at = Some(layer.name().to_string());
                results.push((layer.name().to_string(), result));
                break;
            }

            results.push((layer.name().to_string(), result));
        }

        PipelineResult {
            results,
            stopped_at,
            config_used: self.config.clone(),
        }
    }

    /// Get all registered layers.
    pub fn layers(&self) -> &[Arc<dyn ValidationLayer>] {
        &self.layers
    }
}

impl Default for ValidationPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Result from pipeline execution.
#[derive(Debug)]
pub struct PipelineResult {
    /// Results from each layer.
    pub results: Vec<(String, LayerResult)>,
    /// If stopped early, which layer stopped it.
    pub stopped_at: Option<String>,
    /// Configuration used (if any).
    pub config_used: Option<LayerConfig>,
}

impl PipelineResult {
    /// Check if all layers passed.
    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|(_, r)| r.passed)
    }

    /// Get total violations across all layers.
    pub fn total_violations(&self) -> usize {
        self.results.iter().map(|(_, r)| r.violations.len()).sum()
    }

    /// Check if Memory-Driven mode was used.
    pub fn was_memory_driven(&self) -> bool {
        self.config_used.is_some()
    }

    /// Get all violations as a flat list.
    pub fn all_violations(&self) -> Vec<&crate::violation::Violation> {
        self.results
            .iter()
            .flat_map(|(_, r)| &r.violations)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::violation::Violation;

    struct TestLayer {
        name: String,
        should_pass: bool,
    }

    #[async_trait]
    impl ValidationLayer for TestLayer {
        fn name(&self) -> &str {
            &self.name
        }

        fn can_continue(&self, result: &LayerResult) -> bool {
            // Stop on errors (like Syntax layer does)
            !result.has_errors()
        }

        async fn validate(&self, _ctx: &ValidationContext) -> LayerResult {
            if self.should_pass {
                LayerResult::pass()
            } else {
                LayerResult::fail(vec![Violation::error("test", "test error")])
            }
        }
    }

    #[tokio::test]
    async fn test_pipeline_all_pass() {
        let pipeline = ValidationPipeline::new()
            .add_layer(TestLayer { name: "layer1".into(), should_pass: true })
            .add_layer(TestLayer { name: "layer2".into(), should_pass: true });

        let ctx = ValidationContext::default();
        let result = pipeline.execute(&ctx).await;

        assert!(result.all_passed());
        assert!(result.stopped_at.is_none());
    }

    #[tokio::test]
    async fn test_pipeline_stop_on_error() {
        let pipeline = ValidationPipeline::new()
            .add_layer(TestLayer { name: "layer1".into(), should_pass: true })
            .add_layer(TestLayer { name: "layer2".into(), should_pass: false })
            .add_layer(TestLayer { name: "layer3".into(), should_pass: true });

        let ctx = ValidationContext::default();
        let result = pipeline.execute(&ctx).await;

        assert!(!result.all_passed());
        assert_eq!(result.stopped_at, Some("layer2".to_string()));
    }

    #[tokio::test]
    async fn test_pipeline_with_config() {
        let pipeline = ValidationPipeline::new()
            .with_config(serde_json::json!({"threshold": 10}))
            .add_layer(TestLayer { name: "layer1".into(), should_pass: true });

        let ctx = ValidationContext::default();
        let result = pipeline.execute(&ctx).await;

        assert!(result.all_passed());
        assert!(result.was_memory_driven());
        assert!(result.config_used.is_some());
    }

    #[tokio::test]
    async fn test_all_violations() {
        let pipeline = ValidationPipeline::new()
            .add_layer(TestLayer { name: "layer1".into(), should_pass: false })
            .add_layer(TestLayer { name: "layer2".into(), should_pass: false });

        let ctx = ValidationContext::default();
        let result = pipeline.execute(&ctx).await;

        // layer2 won't run because layer1 stops on error
        assert_eq!(result.all_violations().len(), 1);
    }
}
