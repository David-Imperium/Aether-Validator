//! Compliance Layer — Integrates ComplianceEngine for intelligent contract enforcement
//!
//! This layer wraps `ComplianceEngine` to provide:
//! - Contract tier classification (INVIOLABLE, STRICT, FLEXIBLE)
//! - Context-aware violation evaluation
//! - Auto-learning for FLEXIBLE tier patterns
//! - Dubbioso mode integration for low-confidence cases

use async_trait::async_trait;

use crate::layer::{LayerResult, ValidationLayer, LayerConfig};
use crate::context::ValidationContext;
use crate::violation::Violation;

/// Configuration for the Compliance Layer
#[derive(Debug, Clone)]
pub struct ComplianceLayerConfig {
    /// Minimum confidence to auto-accept violations
    pub auto_accept_threshold: f64,
    /// Confidence below which to use Dubbioso mode
    pub dubbioso_threshold: f64,
    /// How many occurrences before learning a pattern
    pub learn_after_occurrences: u32,
    /// Path to store exemptions
    pub exemption_store_path: Option<std::path::PathBuf>,
}

impl Default for ComplianceLayerConfig {
    fn default() -> Self {
        Self {
            auto_accept_threshold: 0.90,
            dubbioso_threshold: 0.60,
            learn_after_occurrences: 3,
            exemption_store_path: None,
        }
    }
}

/// Result of compliance evaluation for a single violation
#[derive(Debug, Clone)]
pub struct ComplianceResult {
    /// Original violation
    pub violation: Violation,
    /// Compliance decision
    pub decision: synward_intelligence::compliance::ComplianceDecision,
    /// Whether this violation should block validation
    pub should_block: bool,
    /// Whether user input is needed
    pub needs_input: bool,
    /// Message for Dubbioso mode (if applicable)
    pub dubbioso_message: Option<String>,
}

/// Compliance Layer that integrates ComplianceEngine
///
/// This layer runs AFTER other layers to evaluate violations
/// using intelligent contract enforcement rules.
pub struct ComplianceLayer {
    config: ComplianceLayerConfig,
    /// Cached compliance engine
    engine: Option<synward_intelligence::compliance::ComplianceEngine>,
}

impl ComplianceLayer {
    /// Create new ComplianceLayer with default config
    pub fn new() -> Self {
        Self {
            config: ComplianceLayerConfig::default(),
            engine: None,
        }
    }

    /// Create with custom config
    pub fn with_config(config: ComplianceLayerConfig) -> Self {
        Self {
            config,
            engine: None,
        }
    }

    /// Initialize the compliance engine (lazy initialization)
    fn ensure_engine(&mut self) -> &mut synward_intelligence::compliance::ComplianceEngine {
        if self.engine.is_none() {
            let engine_config = synward_intelligence::compliance::ComplianceConfig {
                auto_accept_threshold: self.config.auto_accept_threshold,
                ask_threshold: self.config.dubbioso_threshold,
                learn_after_occurrences: self.config.learn_after_occurrences,
                use_dubbioso: true,
                exemption_store_path: self.config.exemption_store_path.clone(),
            };
            self.engine = Some(
                synward_intelligence::compliance::ComplianceEngine::with_config(engine_config)
                    .expect("Failed to create ComplianceEngine")
            );
        }
        self.engine.as_mut().unwrap()
    }

    /// Evaluate a single violation through the compliance engine
    pub async fn evaluate_violation(
        &mut self,
        violation: &Violation,
        ctx: &ValidationContext,
    ) -> ComplianceResult {
        use synward_intelligence::compliance::ComplianceContext;

        // Build compliance context from validation context
        let compliance_ctx = ComplianceContext {
            file_path: ctx.file_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            line: violation.span.as_ref().map(|s| s.line).unwrap_or(0),
            snippet: None,
            project_type: ctx.metadata.get("project_type").cloned(),
            code_region: ctx.metadata.get("code_region").cloned(),
            function_context: ctx.metadata.get("function").cloned(),
        };

        // Determine domain from violation ID prefix
        let domain = Self::infer_domain(&violation.id);

        // Evaluate through compliance engine
        let decision = self.ensure_engine()
            .evaluate(&violation.id, &domain, &violation.message, &compliance_ctx)
            .await
            .expect("Compliance evaluation failed");

        let should_block = decision.should_fail();
        let needs_input = decision.needs_user_input();
        
        // Generate Dubbioso message if needed
        let dubbioso_message = if needs_input {
            match &decision.action {
                synward_intelligence::compliance::ComplianceAction::Ask { question, options } => {
                    Some(format!("{} Options: {:?}", question, options))
                }
                _ => None,
            }
        } else {
            None
        };

        ComplianceResult {
            violation: violation.clone(),
            decision,
            should_block,
            needs_input,
            dubbioso_message,
        }
    }

    /// Evaluate multiple violations and return compliance results
    pub async fn evaluate_violations(
        &mut self,
        violations: &[Violation],
        ctx: &ValidationContext,
    ) -> Vec<ComplianceResult> {
        let mut results = Vec::with_capacity(violations.len());
        for v in violations {
            results.push(self.evaluate_violation(v, ctx).await);
        }
        results
    }

    /// Infer domain from violation ID (e.g., "SEC001" → "security")
    fn infer_domain(id: &str) -> String {
        let prefix = id.chars().take_while(|c| c.is_alphabetic()).collect::<String>();
        match prefix.to_uppercase().as_str() {
            "SEC" => "security",
            "MEM" => "memory-safety",
            "SUPP" => "supply-chain",
            "LOGIC" => "logic",
            "STYLE" => "style",
            "NAME" => "naming",
            "FMT" => "formatting",
            "DOC" => "documentation",
            "CPLX" => "complexity",
            "ARCH" => "architecture",
            "SEMANTIC" => "semantic",
            _ => "general",
        }.to_string()
    }
}

impl Default for ComplianceLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for ComplianceLayer {
    fn name(&self) -> &str {
        "compliance"
    }

    fn priority(&self) -> u8 {
        70 // Run after intelligence layer (60) and before final output
    }

    async fn validate(&self, _ctx: &ValidationContext) -> LayerResult {
        // Compliance layer doesn't generate violations on its own
        // It evaluates violations from other layers
        LayerResult::pass()
    }

    async fn validate_with_config(
        &self,
        _ctx: &ValidationContext,
        config: Option<&LayerConfig>,
    ) -> LayerResult {
        // Check for pre-evaluated compliance results in config
        if let Some(_cfg) = config {
            // Results would be processed by the executor
        }
        
        LayerResult::pass()
    }

    fn can_continue(&self, result: &LayerResult) -> bool {
        // Compliance layer can block on INVIOLABLE violations
        !result.has_errors()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_layer_creation() {
        let layer = ComplianceLayer::new();
        assert_eq!(layer.name(), "compliance");
        assert_eq!(layer.priority(), 70);
    }

    #[test]
    fn test_infer_domain() {
        assert_eq!(ComplianceLayer::infer_domain("SEC001"), "security");
        assert_eq!(ComplianceLayer::infer_domain("MEM002"), "memory-safety");
        assert_eq!(ComplianceLayer::infer_domain("STYLE003"), "style");
        assert_eq!(ComplianceLayer::infer_domain("UNKNOWN"), "general");
    }
}
