//! Prompt Analyzer
//!
//! Combines all analysis components into a unified analyzer.

use std::time::Instant;

use super::intent::{IntentClassifier, IntentResult};
use super::scope::{ScopeExtractor, ScopeResult};
use super::domain::{DomainMapper, DomainResult};
use super::ambiguity::{Ambiguity, AmbiguityDetector, ClarificationRequest};

/// Complete prompt analysis result.
#[derive(Debug, Clone)]
pub struct PromptAnalysis {
    /// Original prompt.
    pub original_prompt: String,
    /// Intent classification.
    pub intent: IntentResult,
    /// Scope extraction.
    pub scope: ScopeResult,
    /// Domain mapping.
    pub domain: DomainResult,
    /// Detected ambiguities.
    pub ambiguities: Vec<Ambiguity>,
    /// Whether clarification is needed.
    pub needs_clarification: bool,
    /// Clarification request (if needed).
    pub clarification: Option<ClarificationRequest>,
    /// Enhanced prompt with context.
    pub enhanced_prompt: Option<String>,
    /// Processing time in milliseconds.
    pub processing_time_ms: u64,
}

impl PromptAnalysis {
    /// Check if the analysis has ambiguities.
    pub fn has_ambiguities(&self) -> bool {
        !self.ambiguities.is_empty()
    }

    /// Get high-severity ambiguities (> 0.5).
    pub fn critical_ambiguities(&self) -> Vec<&Ambiguity> {
        self.ambiguities.iter()
            .filter(|a| a.severity > 0.5)
            .collect()
    }
}

/// Main prompt analyzer.
pub struct PromptAnalyzer {
    /// Intent classifier.
    intent_classifier: IntentClassifier,
    /// Scope extractor.
    scope_extractor: ScopeExtractor,
    /// Domain mapper.
    domain_mapper: DomainMapper,
    /// Ambiguity detector.
    ambiguity_detector: AmbiguityDetector,
}

impl PromptAnalyzer {
    /// Create a new prompt analyzer.
    pub fn new() -> Self {
        Self {
            intent_classifier: IntentClassifier::new(),
            scope_extractor: ScopeExtractor::new(),
            domain_mapper: DomainMapper::new(),
            ambiguity_detector: AmbiguityDetector::new(),
        }
    }

    /// Analyze a prompt.
    pub fn analyze(&self, prompt: &str) -> PromptAnalysis {
        let start = Instant::now();

        // Step 1: Classify intent
        let intent = self.intent_classifier.classify(prompt);

        // Step 2: Extract scope
        let scope = self.scope_extractor.extract(prompt);

        // Step 3: Map domain
        let domain = self.domain_mapper.map(prompt);

        // Step 4: Detect ambiguities
        let ambiguities = self.ambiguity_detector.detect(
            prompt,
            intent.primary,
            &scope,
        );

        // Step 5: Create clarification if needed
        let needs_clarification = !ambiguities.is_empty();
        let clarification = if needs_clarification {
            self.ambiguity_detector.create_clarification(ambiguities.clone())
        } else {
            None
        };

        // Step 6: Generate enhanced prompt
        let enhanced_prompt = self.generate_enhanced_prompt(
            prompt,
            &intent,
            &scope,
            &domain,
        );

        let processing_time_ms = start.elapsed().as_millis() as u64;

        PromptAnalysis {
            original_prompt: prompt.to_string(),
            intent,
            scope,
            domain,
            ambiguities,
            needs_clarification,
            clarification,
            enhanced_prompt,
            processing_time_ms,
        }
    }

    /// Generate an enhanced prompt with context.
    fn generate_enhanced_prompt(
        &self,
        prompt: &str,
        intent: &IntentResult,
        scope: &ScopeResult,
        domain: &DomainResult,
    ) -> Option<String> {
        let mut enhanced = String::new();

        // Add intent context
        enhanced.push_str(&format!("[Intent: {}] ", intent.primary));

        // Add scope context
        if !scope.entities.is_empty() {
            enhanced.push_str(&format!("[Scope: {}] ", scope.level));
            for entity in &scope.entities {
                enhanced.push_str(&format!("{}: {}, ", 
                    entity.entity_type_as_str(), entity.name));
            }
        }

        // Add domain context
        enhanced.push_str(&format!("[Domain: {}] ", domain.primary));

        // Add original prompt
        enhanced.push_str(prompt);

        Some(enhanced)
    }

    /// Analyze and return a summary string.
    pub fn analyze_summary(&self, prompt: &str) -> String {
        let analysis = self.analyze(prompt);
        format!(
            "Intent: {} ({:.0}%)\nScope: {}\nDomain: {}\nAmbiguities: {}",
            analysis.intent.primary,
            analysis.intent.confidence * 100.0,
            analysis.scope.level,
            analysis.domain.primary,
            analysis.ambiguities.len()
        )
    }
}

impl Default for PromptAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::Intent;

    #[test]
    fn test_analyze_create_intent() {
        let analyzer = PromptAnalyzer::new();
        let analysis = analyzer.analyze("Add a new enemy class with patrol behavior");
        
        assert_eq!(analysis.intent.primary, Intent::Create);
        assert_eq!(analysis.domain.primary, "gameplay");
    }

    #[test]
    fn test_analyze_fix_intent() {
        let analyzer = PromptAnalyzer::new();
        let analysis = analyzer.analyze("Fix the crash in enemy.rs");
        
        assert_eq!(analysis.intent.primary, Intent::Fix);
        assert!(analysis.scope.entities.iter().any(|e| e.name.contains("enemy.rs")));
    }

    #[test]
    fn test_analyze_with_ambiguity() {
        let analyzer = PromptAnalyzer::new();
        let analysis = analyzer.analyze("Fix the bug");
        
        assert!(analysis.has_ambiguities());
        assert!(analysis.needs_clarification);
        assert!(analysis.clarification.is_some());
    }

    #[test]
    fn test_analyze_no_ambiguity() {
        let analyzer = PromptAnalyzer::new();
        let analysis = analyzer.analyze("Set the player speed to 10 in player.rs");
        
        // Should have file scope
        assert!(!analysis.scope.entities.is_empty());
        // Should not have critical ambiguities since value is specified
        assert!(analysis.critical_ambiguities().is_empty() || !analysis.critical_ambiguities().iter().any(|a| a.ambiguity_type == super::super::ambiguity::AmbiguityType::Value));
    }

    #[test]
    fn test_enhanced_prompt() {
        let analyzer = PromptAnalyzer::new();
        let analysis = analyzer.analyze("Add enemy patrol");
        
        assert!(analysis.enhanced_prompt.is_some());
        let enhanced = analysis.enhanced_prompt.unwrap();
        assert!(enhanced.contains("[Intent:"));
        assert!(enhanced.contains("[Domain:"));
    }

    #[test]
    fn test_analyze_summary() {
        let analyzer = PromptAnalyzer::new();
        let summary = analyzer.analyze_summary("Add enemy patrol");
        
        assert!(summary.contains("Intent: CREATE"));
        assert!(summary.contains("Domain:"));
    }

    #[test]
    fn test_processing_time() {
        let analyzer = PromptAnalyzer::new();
        let analysis = analyzer.analyze("Test prompt");
        
        // Processing should be fast (< 100ms)
        assert!(analysis.processing_time_ms < 100);
    }
}
