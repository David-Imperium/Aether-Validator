//! Validation Pipeline — Coordinates validation layers

use std::sync::Arc;

use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;

/// Validation pipeline that coordinates multiple layers.
pub struct ValidationPipeline {
    layers: Vec<Arc<dyn ValidationLayer>>,
}

impl ValidationPipeline {
    /// Create a new empty pipeline.
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
        }
    }

    /// Add a layer to the pipeline.
    pub fn add_layer(mut self, layer: impl ValidationLayer + 'static) -> Self {
        self.layers.push(Arc::new(layer));
        self
    }

    /// Execute all layers in order.
    pub async fn execute(&self, ctx: &ValidationContext) -> PipelineResult {
        let mut results = Vec::new();
        let mut stopped_at = None;

        for layer in &self.layers {
            let result = layer.validate(ctx).await;
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
}
