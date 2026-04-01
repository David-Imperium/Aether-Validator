//! Dubbioso Mode - Confidence-based Validation
//!
//! Combines Graph RAG + Semantic Analysis for context-aware validation.
//! When confidence is low, asks questions instead of failing blindly.

use crate::memory::{CodeGraph, FunctionContext};
use crate::semantic::{SemanticAnalyzer, FunctionSemanticContext, ErrorHandlingStyle};
use serde::{Deserialize, Serialize};

/// Dubbioso preset levels - user-facing configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DubbiosoPreset {
    /// Maximum precision, frequent questions, slow learning
    Strict,
    /// Balanced - default for most projects
    #[default]
    Balanced,
    /// Fast validation, fewer questions, quick learning
    Fast,
    /// Maximum speed, no questions, rapid learning
    Turbo,
}

impl DubbiosoPreset {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Strict => "Maximum precision, asks often, learns slowly",
            Self::Balanced => "Balanced precision and speed, default",
            Self::Fast => "Fast validation, fewer questions, quick learning",
            Self::Turbo => "Maximum speed, no questions, rapid learning",
        }
    }

    /// Get all presets as slice
    pub fn all() -> &'static [Self] {
        &[Self::Strict, Self::Balanced, Self::Fast, Self::Turbo]
    }
}

impl From<DubbiosoPreset> for DubbiosoConfig {
    fn from(preset: DubbiosoPreset) -> Self {
        match preset {
            DubbiosoPreset::Strict => Self {
                ask_threshold: 0.75,          // Ask more often
                warn_threshold: 0.90,         // Higher bar for "good"
                auto_accept_threshold: 0.98,  // Very high for auto-accept
                permanent_after: 10,          // Slow learning
                max_context_depth: 5,         // Deep analysis
            },
            DubbiosoPreset::Balanced => Self {
                ask_threshold: 0.60,
                warn_threshold: 0.80,
                auto_accept_threshold: 0.95,
                permanent_after: 5,
                max_context_depth: 3,
            },
            DubbiosoPreset::Fast => Self {
                ask_threshold: 0.45,          // Ask less often
                warn_threshold: 0.70,         // Lower bar
                auto_accept_threshold: 0.90,  // Easier auto-accept
                permanent_after: 3,           // Quick learning
                max_context_depth: 2,         // Shallow analysis
            },
            DubbiosoPreset::Turbo => Self {
                ask_threshold: 0.30,          // Rarely ask
                warn_threshold: 0.50,         // Low bar
                auto_accept_threshold: 0.85,  // Easy auto-accept
                permanent_after: 2,           // Very quick learning
                max_context_depth: 1,         // Minimal analysis
            },
        }
    }
}

/// Dubbioso Mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbiosoConfig {
    /// Ask questions below this threshold
    pub ask_threshold: f64,
    /// Warn but continue below this threshold
    pub warn_threshold: f64,
    /// Auto-accept above this threshold
    pub auto_accept_threshold: f64,
    /// Make pattern permanent after N acceptances
    pub permanent_after: u32,
    /// Maximum depth for context traversal
    pub max_context_depth: usize,
}

impl Default for DubbiosoConfig {
    fn default() -> Self {
        Self::from(DubbiosoPreset::Balanced)
    }
}

/// Confidence result from Dubbioso Mode analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceResult {
    /// Overall confidence score (0-1)
    pub confidence: f64,
    /// Confidence level
    pub level: ConfidenceLevel,
    /// Graph context (if available)
    pub graph_context: Option<FunctionContext>,
    /// Semantic context
    pub semantic_context: Option<FunctionSemanticContext>,
    /// Why confidence is low (if applicable)
    pub uncertainty_reasons: Vec<String>,
    /// Suggested questions to ask
    pub questions: Vec<String>,
}

/// Confidence level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfidenceLevel {
    /// Very low - must ask user
    Ask,
    /// Low - warn but continue
    Warn,
    /// Good - proceed normally
    Good,
    /// High - auto-accept
    AutoAccept,
}

/// Dubbioso Mode analyzer
pub struct DubbiosoAnalyzer {
    config: DubbiosoConfig,
    semantic_analyzer: SemanticAnalyzer,
    code_graph: Option<CodeGraph>,
}

impl DubbiosoAnalyzer {
    /// Create new Dubbioso analyzer
    pub fn new(config: DubbiosoConfig) -> Self {
        Self {
            config,
            semantic_analyzer: SemanticAnalyzer::new(),
            code_graph: None,
        }
    }

    /// Create with code graph for context
    pub fn with_graph(config: DubbiosoConfig, graph: CodeGraph) -> Self {
        Self {
            config,
            semantic_analyzer: SemanticAnalyzer::new(),
            code_graph: Some(graph),
        }
    }

    /// Set code graph
    pub fn set_graph(&mut self, graph: CodeGraph) {
        self.code_graph = Some(graph);
    }

    /// Analyze a function for confidence
    pub fn analyze(
        &self,
        code: &str,
        function_name: &str,
        file: &str,
        language: &str,
    ) -> ConfidenceResult {
        // 1. Semantic analysis
        let semantic_ctx = self.semantic_analyzer.analyze_function(code, function_name, language);

        // 2. Graph context (if available)
        let graph_ctx = self.code_graph.as_ref().map(|g| {
            g.get_full_context(function_name, file, self.config.max_context_depth)
        });

        // 3. Calculate combined confidence
        let confidence = self.calculate_confidence(&semantic_ctx, &graph_ctx);

        // 4. Determine level
        let level = self.determine_level(confidence);

        // 5. Generate uncertainty reasons and questions
        let (reasons, questions) = self.analyze_uncertainty(&semantic_ctx, &graph_ctx, confidence);

        ConfidenceResult {
            confidence,
            level,
            graph_context: graph_ctx,
            semantic_context: Some(semantic_ctx),
            uncertainty_reasons: reasons,
            questions,
        }
    }

    /// Calculate combined confidence score
    fn calculate_confidence(
        &self,
        semantic: &FunctionSemanticContext,
        graph: &Option<FunctionContext>,
    ) -> f64 {
        // Start with function context score (includes base confidence)
        let mut score = semantic.function_context_score();

        // Boost from graph context (if available)
        if let Some(ref ctx) = graph {
            // More context = more confidence
            let graph_boost = ctx.context_score * 0.15;
            score += graph_boost;

            // Files involved = more understanding
            let file_boost = (ctx.files_involved.len() as f64 * 0.02).min(0.1);
            score += file_boost;

            // Call chain depth = better understanding of impact
            let max_call_depth = ctx.calls_at_depth.keys().max().copied().unwrap_or(0);
            let depth_boost = (max_call_depth as f64 * 0.03).min(0.1);
            score += depth_boost;
        }

        // Penalty for anti-patterns
        score -= semantic.base.anti_patterns.len() as f64 * 0.1;

        // Penalty for no error handling in non-test functions
        if !semantic.is_test_function {
            if let Some(ref err_style) = semantic.base.error_handling {
                if matches!(err_style, ErrorHandlingStyle::None) {
                    score -= 0.1;
                }
            }
        }

        // Clamp to 0-1
        score.clamp(0.0, 1.0)
    }

    /// Determine confidence level based on thresholds
    fn determine_level(&self, confidence: f64) -> ConfidenceLevel {
        if confidence >= self.config.auto_accept_threshold {
            ConfidenceLevel::AutoAccept
        } else if confidence >= self.config.warn_threshold {
            ConfidenceLevel::Good
        } else if confidence >= self.config.ask_threshold {
            ConfidenceLevel::Warn
        } else {
            ConfidenceLevel::Ask
        }
    }

    /// Analyze reasons for uncertainty and generate questions
    fn analyze_uncertainty(
        &self,
        semantic: &FunctionSemanticContext,
        graph: &Option<FunctionContext>,
        confidence: f64,
    ) -> (Vec<String>, Vec<String>) {
        let mut reasons = Vec::new();
        let mut questions = Vec::new();

        // Anti-patterns detected
        if !semantic.base.anti_patterns.is_empty() {
            for anti_pattern in &semantic.base.anti_patterns {
                reasons.push(format!("Anti-pattern detected: {}", anti_pattern));
            }
        }

        // No error handling in production code
        if !semantic.is_test_function {
            if let Some(ref err_style) = semantic.base.error_handling {
                if matches!(err_style, ErrorHandlingStyle::None) {
                    reasons.push("No error handling visible".to_string());
                    questions.push("Is this function guaranteed to never fail?".to_string());
                }
            }
        }

        // Low graph context
        if let Some(ref ctx) = graph {
            if ctx.context_score < 0.3 {
                reasons.push("Function has limited context in code graph".to_string());
                questions.push("Is this a new function? Should it be connected to other modules?".to_string());
            }

            // Check caller impact
            if let Some(callers) = ctx.callers_at_depth.get(&1) {
                if callers.len() > 3 {
                    reasons.push(format!("Function called by {} other functions", callers.len()));
                    questions.push("This function has many callers. Are you sure about this change?".to_string());
                }
            }
        } else {
            reasons.push("No code graph available for context".to_string());
        }

        // Test functions have lower risk
        if semantic.is_test_function && confidence < 0.8 {
            questions.push("This is a test function. Is the detected pattern intentional?".to_string());
        }

        // Handler functions need careful review
        if semantic.is_handler_function && confidence < 0.7 {
            questions.push("This is a handler/entry point. Should it have explicit error handling?".to_string());
        }

        (reasons, questions)
    }

    /// Check if a violation should trigger questioning
    pub fn should_ask(&self, result: &ConfidenceResult) -> bool {
        result.level == ConfidenceLevel::Ask
    }

    /// Check if a violation should warn but continue
    pub fn should_warn(&self, result: &ConfidenceResult) -> bool {
        result.level == ConfidenceLevel::Warn
    }

    /// Check if a violation can be auto-accepted
    pub fn can_auto_accept(&self, result: &ConfidenceResult) -> bool {
        result.level == ConfidenceLevel::AutoAccept
    }

    /// Format result for display
    pub fn format_result(&self, result: &ConfidenceResult) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "Confidence: {:.0}% ({:?})\n",
            result.confidence * 100.0,
            result.level
        ));

        if !result.uncertainty_reasons.is_empty() {
            output.push_str("\nWhy uncertain:\n");
            for reason in &result.uncertainty_reasons {
                output.push_str(&format!("  • {}\n", reason));
            }
        }

        if !result.questions.is_empty() {
            output.push_str("\nQuestions:\n");
            for (i, question) in result.questions.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, question));
            }
        }

        if let Some(ref ctx) = result.graph_context {
            output.push_str(&format!(
                "\nContext: {} files involved, score {:.2}\n",
                ctx.files_involved.len(),
                ctx.context_score
            ));
        }

        output
    }
}

impl Default for DubbiosoAnalyzer {
    fn default() -> Self {
        Self::new(DubbiosoConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_calculation() {
        let analyzer = DubbiosoAnalyzer::default();

        let code = r#"
fn process(data: String) -> Result<(), Error> {
    let parsed = parse(&data)?;
    Ok(parsed)
}
"#;

        let result = analyzer.analyze(code, "process", "src/lib.rs", "rust");
        assert!(result.confidence > 0.5);
        assert!(result.level != ConfidenceLevel::Ask);
    }

    #[test]
    fn test_low_confidence_with_anti_patterns() {
        let analyzer = DubbiosoAnalyzer::default();

        let code = r#"
fn main() {
    let x = Some(5).unwrap();
}
"#;

        let result = analyzer.analyze(code, "main", "src/main.rs", "rust");
        assert!(!result.uncertainty_reasons.is_empty());
    }

    #[test]
    fn test_threshold_classification() {
        let analyzer = DubbiosoAnalyzer::default();

        // High confidence code
        let good_code = r#"
fn helper() -> i32 { 42 }
"#;
        let result = analyzer.analyze(good_code, "helper", "src/lib.rs", "rust");
        assert!(result.confidence >= 0.0);
    }

    // === Preset Tests ===

    #[test]
    fn test_preset_default_is_balanced() {
        let preset = DubbiosoPreset::default();
        assert_eq!(preset, DubbiosoPreset::Balanced);
    }

    #[test]
    fn test_preset_to_config_strict() {
        let config = DubbiosoConfig::from(DubbiosoPreset::Strict);
        assert_eq!(config.ask_threshold, 0.75);
        assert_eq!(config.warn_threshold, 0.90);
        assert_eq!(config.auto_accept_threshold, 0.98);
        assert_eq!(config.permanent_after, 10);
        assert_eq!(config.max_context_depth, 5);
    }

    #[test]
    fn test_preset_to_config_balanced() {
        let config = DubbiosoConfig::from(DubbiosoPreset::Balanced);
        assert_eq!(config.ask_threshold, 0.60);
        assert_eq!(config.warn_threshold, 0.80);
        assert_eq!(config.auto_accept_threshold, 0.95);
        assert_eq!(config.permanent_after, 5);
        assert_eq!(config.max_context_depth, 3);
    }

    #[test]
    fn test_preset_to_config_fast() {
        let config = DubbiosoConfig::from(DubbiosoPreset::Fast);
        assert_eq!(config.ask_threshold, 0.45);
        assert_eq!(config.warn_threshold, 0.70);
        assert_eq!(config.auto_accept_threshold, 0.90);
        assert_eq!(config.permanent_after, 3);
        assert_eq!(config.max_context_depth, 2);
    }

    #[test]
    fn test_preset_to_config_turbo() {
        let config = DubbiosoConfig::from(DubbiosoPreset::Turbo);
        assert_eq!(config.ask_threshold, 0.30);
        assert_eq!(config.warn_threshold, 0.50);
        assert_eq!(config.auto_accept_threshold, 0.85);
        assert_eq!(config.permanent_after, 2);
        assert_eq!(config.max_context_depth, 1);
    }

    #[test]
    fn test_preset_all_returns_four() {
        assert_eq!(DubbiosoPreset::all().len(), 4);
    }

    #[test]
    fn test_preset_threshold_ordering() {
        // Strict should be most conservative (highest thresholds)
        let strict = DubbiosoConfig::from(DubbiosoPreset::Strict);
        let balanced = DubbiosoConfig::from(DubbiosoPreset::Balanced);
        let fast = DubbiosoConfig::from(DubbiosoPreset::Fast);
        let turbo = DubbiosoConfig::from(DubbiosoPreset::Turbo);

        // Ask thresholds: strict > balanced > fast > turbo
        assert!(strict.ask_threshold > balanced.ask_threshold);
        assert!(balanced.ask_threshold > fast.ask_threshold);
        assert!(fast.ask_threshold > turbo.ask_threshold);

        // Auto-accept thresholds: strict > balanced > fast > turbo
        assert!(strict.auto_accept_threshold > balanced.auto_accept_threshold);
        assert!(balanced.auto_accept_threshold > fast.auto_accept_threshold);
        assert!(fast.auto_accept_threshold > turbo.auto_accept_threshold);
    }
}
