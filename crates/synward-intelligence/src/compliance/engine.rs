//! Compliance Engine - Main engine for intelligent contract enforcement
//!
//! Integrates:
//! - Contract classification (inviolable vs flexible)
//! - Context analysis (project type, code region)
//! - Decision history (precedents, learned patterns)
//! - Dubbioso mode for low-confidence cases

use serde::{Deserialize, Serialize};

use crate::compliance::{
    ContractClassifier, ContractTier,
    ComplianceDecision,
    ExemptionStore, Exemption, ExemptionScope, ExemptionSource,
};
use crate::error::Result;

/// Configuration for the compliance engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Minimum confidence to auto-accept
    pub auto_accept_threshold: f64,
    /// Confidence below which to ask user
    pub ask_threshold: f64,
    /// How many occurrences before learning a pattern
    pub learn_after_occurrences: u32,
    /// Whether to use Dubbioso for uncertain cases
    pub use_dubbioso: bool,
    /// Path to store exemptions
    pub exemption_store_path: Option<std::path::PathBuf>,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            auto_accept_threshold: 0.90,
            ask_threshold: 0.60,
            learn_after_occurrences: 3,
            use_dubbioso: true,
            exemption_store_path: None,
        }
    }
}

/// Context for compliance evaluation
#[derive(Debug, Clone)]
pub struct ComplianceContext {
    /// File being validated
    pub file_path: String,
    /// Line number
    pub line: usize,
    /// Code snippet
    pub snippet: Option<String>,
    /// Project type (cli, library, embedded, web, etc.)
    pub project_type: Option<String>,
    /// Code region type (main, test, example, benchmark)
    pub code_region: Option<String>,
    /// Function or module context
    pub function_context: Option<String>,
}

/// The compliance engine
pub struct ComplianceEngine {
    /// Contract classifier
    classifier: ContractClassifier,
    /// Exemption store
    exemptions: ExemptionStore,
    /// Configuration
    config: ComplianceConfig,
    /// Occurrence tracking for learning
    occurrences: std::collections::HashMap<String, u32>,
}

impl ComplianceEngine {
    /// Create a new compliance engine
    pub fn new() -> Result<Self> {
        Self::with_config(ComplianceConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(config: ComplianceConfig) -> Result<Self> {
        let exemptions = if let Some(ref path) = config.exemption_store_path {
            ExemptionStore::with_path(path.clone())
        } else {
            ExemptionStore::new()
        };
        
        Ok(Self {
            classifier: ContractClassifier::new(),
            exemptions,
            config,
            occurrences: std::collections::HashMap::new(),
        })
    }
    
    /// Evaluate a violation and determine the action
    pub async fn evaluate(
        &mut self,
        rule_id: &str,
        domain: &str,
        message: &str,
        ctx: &ComplianceContext,
    ) -> Result<ComplianceDecision> {
        // Step 1: Classify the contract
        let tier = self.classifier.classify(rule_id, domain);
        
        // Step 2: Check if inviolable - immediate block
        if tier == ContractTier::Inviolable {
            return Ok(ComplianceDecision::block(tier, rule_id, message));
        }
        
        // Step 3: Check for existing exemption
        let exemption_match = self.exemptions.find(rule_id, &ctx.file_path).map(|e| {
            (e.id.clone(), e.reason.clone(), e.confidence)
        });
        
        if let Some((id, reason, confidence)) = exemption_match {
            self.exemptions.record_application(&id);
            return Ok(ComplianceDecision::accept(tier, reason, Some(id), confidence));
        }
        
        // Step 4: Track occurrences for learning
        let occurrence_key = format!("{}:{}", rule_id, ctx.file_path);
        let count = {
            let count = self.occurrences.entry(occurrence_key).or_insert(0);
            *count += 1;
            *count
        };
        
        // Step 5: Check if we should learn this pattern
        if tier.supports_learning() && count >= self.config.learn_after_occurrences {
            let scope = Self::infer_scope_static(&ctx.file_path);
            let confidence = 0.85 + (count as f64 * 0.02).min(0.15);
            let exemption = Exemption::learned(rule_id.to_string(), scope, confidence);
            let learned_confidence = exemption.confidence;
            self.exemptions.add(exemption);
            
            return Ok(ComplianceDecision::learn(
                tier,
                format!("Auto-learned after {} occurrences", count),
                learned_confidence,
            ));
        }
        
        // Step 6: Check for similar precedents in the same file
        let file_exemptions = self.exemptions.get_for_rule(rule_id);
        let similar = file_exemptions.into_iter()
            .filter(|e| Self::is_similar_context_static(e, ctx))
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some(precedent) = similar {
            if precedent.confidence >= self.config.auto_accept_threshold {
                return Ok(ComplianceDecision::accept(
                    tier,
                    "Based on similar precedent".into(),
                    Some(precedent.id.clone()),
                    precedent.confidence,
                ));
            }
        }
        
        // Step 7: Context-based analysis
        let confidence = self.compute_confidence(rule_id, ctx);
        
        if confidence < self.config.ask_threshold && self.config.use_dubbioso {
            // Low confidence - ask user
            return Ok(ComplianceDecision::ask(
                tier,
                format!("Is this {} violation acceptable in this context?", rule_id),
                vec![
                    "Yes, accept for this file".into(),
                    "Yes, accept for all similar cases".into(),
                    "No, this should be fixed".into(),
                ],
                vec![
                    "No similar pattern found".into(),
                    "Low confidence in context".into(),
                ],
                confidence,
            ));
        }
        
        // Step 8: Default to warn
        Ok(ComplianceDecision::warn(
            tier,
            rule_id,
            message,
        ))
    }
    
    /// Accept a violation with a reason
    pub fn accept_violation(
        &mut self,
        rule_id: &str,
        file_path: &str,
        reason: String,
    ) -> Result<()> {
        let exemption = Exemption::new(
            rule_id.to_string(),
            ExemptionScope::File { path: file_path.to_string() },
            reason,
            ExemptionSource::UserCreated,
        );
        self.exemptions.add(exemption);
        Ok(())
    }
    
    /// Get statistics about the engine
    pub fn stats(&self) -> ComplianceStats {
        ComplianceStats {
            exemptions: self.exemptions.stats(),
            occurrence_tracking: self.occurrences.len(),
        }
    }
    
    // Helper methods
    
    fn infer_scope_static(file_path: &str) -> ExemptionScope {
        if file_path.contains("/test") || file_path.contains("_test.") || file_path.contains("_spec.") {
            return ExemptionScope::Pattern { pattern: "*test*".into() };
        }
        if file_path.contains("/example") || file_path.contains("/examples") {
            return ExemptionScope::Directory { path: "examples".into() };
        }
        ExemptionScope::File { path: file_path.to_string() }
    }
    
    fn is_similar_context_static(exemption: &Exemption, ctx: &ComplianceContext) -> bool {
        match &exemption.scope {
            ExemptionScope::File { path } => path == &ctx.file_path,
            ExemptionScope::Directory { path } => ctx.file_path.starts_with(path),
            ExemptionScope::Pattern { pattern } => {
                ctx.file_path.contains(&pattern.replace('*', ""))
            }
            ExemptionScope::Project => true,
        }
    }
    
    fn compute_confidence(&self, rule_id: &str, ctx: &ComplianceContext) -> f64 {
        let mut confidence = 0.5;
        
        // Boost for test files
        if let Some(ref region) = ctx.code_region {
            if region == "test" {
                confidence += 0.15;
            }
        }
        
        // Boost for example files
        if ctx.file_path.contains("example") || ctx.file_path.contains("demo") {
            confidence += 0.10;
        }
        
        // Check occurrence count
        let key = format!("{}:{}", rule_id, ctx.file_path);
        if let Some(&count) = self.occurrences.get(&key) {
            confidence += (count as f64 * 0.05).min(0.20);
        }
        
        confidence.min(1.0)
    }
}

/// Statistics about the compliance engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStats {
    pub exemptions: super::exemptions::ExemptionStats,
    pub occurrence_tracking: usize,
}

impl Default for ComplianceEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create ComplianceEngine")
    }
}
