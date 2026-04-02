//! Contract Classifier - Determines contract enforcement tier
//!
//! Classifies contracts into enforcement tiers:
//! - **Inviolable**: Security, safety - never bypassed
//! - **Strict**: Memory, logic - requires explicit acceptance
//! - **Flexible**: Style, naming - learns from patterns

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Contract enforcement tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContractTier {
    /// Never bypassed. Security vulnerabilities, memory safety, undefined behavior.
    /// Examples: SQL injection, use-after-free, buffer overflow
    Inviolable,
    
    /// Requires explicit acceptance with reason. Logic errors, resource leaks.
    /// Examples: Deep nesting, long functions, missing error handling
    Strict,
    
    /// Can be learned/auto-accepted based on project patterns.
    /// Examples: Naming conventions, line length, import order
    Flexible,
}

/// Metadata for tier classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierMetadata {
    /// The tier
    pub tier: ContractTier,
    /// Human-readable reason for tier assignment
    pub reason: String,
    /// Domains that map to this tier
    pub domains: Vec<String>,
    /// Rule IDs that are always this tier
    pub rule_ids: Vec<String>,
}

impl ContractTier {
    /// Get all tiers in enforcement order (most to least strict)
    pub fn in_order() -> &'static [Self] {
        &[Self::Inviolable, Self::Strict, Self::Flexible]
    }
    
    /// Check if this tier can be bypassed
    pub fn is_bypassable(&self) -> bool {
        matches!(self, Self::Strict | Self::Flexible)
    }
    
    /// Check if this tier supports learning
    pub fn supports_learning(&self) -> bool {
        matches!(self, Self::Flexible)
    }
}

/// Contract classifier - determines tier for rules
pub struct ContractClassifier {
    /// Domains that are inviolable
    inviolable_domains: HashSet<String>,
    /// Domains that are strict
    strict_domains: HashSet<String>,
    /// Specific rule IDs that are inviolable (override domain)
    inviolable_rules: HashSet<String>,
    /// Rule IDs that are flexible (override domain)
    flexible_rules: HashSet<String>,
}

impl Default for ContractClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ContractClassifier {
    /// Create a new classifier with default rules
    pub fn new() -> Self {
        let inviolable_domains = HashSet::from([
            // Security
            "security".into(),
            "injection".into(),
            "xss".into(),
            "authentication".into(),
            "authorization".into(),
            "cryptography".into(),
            // Memory safety
            "memory-safety".into(),
            "undefined-behavior".into(),
            "data-race".into(),
            // Supply chain
            "supply-chain".into(),
            // Safety-critical
            "safety-critical".into(),
        ]);
        
        let strict_domains = HashSet::from([
            // Logic
            "logic".into(),
            "error-handling".into(),
            "resource-management".into(),
            // Memory
            "memory".into(),
            "leak".into(),
            // Concurrency
            "concurrency".into(),
            "deadlock".into(),
        ]);
        
        // Specific rules that are always inviolable regardless of domain
        let inviolable_rules = HashSet::from([
            // SQL Injection
            "SEC001".into(), "SEC002".into(), "SEC013".into(),
            // XSS
            "SEC003".into(), "SEC004".into(),
            // Path traversal
            "SEC005".into(),
            // Command injection
            "SEC006".into(),
            // Hardcoded secrets
            "SEC007".into(), "SEC008".into(),
            // Use after free, buffer overflow
            "MEM001".into(), "MEM002".into(), "MEM003".into(),
            // Supply chain
            "SUPP001".into(), "SUPP002".into(), "SUPP003".into(),
            "SUPP004".into(), "SUPP005".into(),
        ]);
        
        // Rules that are always flexible
        let flexible_rules = HashSet::from([
            // Style
            "STYLE001".into(), "STYLE002".into(), "STYLE003".into(),
            // Naming
            "NAME001".into(), "NAME002".into(),
            // Formatting
            "FMT001".into(), "FMT002".into(),
            // Comments
            "DOC001".into(), "DOC002".into(),
        ]);
        
        Self {
            inviolable_domains,
            strict_domains,
            inviolable_rules,
            flexible_rules,
        }
    }
    
    /// Classify a rule by ID and domain
    pub fn classify(&self, rule_id: &str, domain: &str) -> ContractTier {
        // Rule ID takes precedence over domain
        if self.inviolable_rules.contains(rule_id) {
            return ContractTier::Inviolable;
        }
        if self.flexible_rules.contains(rule_id) {
            return ContractTier::Flexible;
        }
        
        // Check domain
        let domain_lower = domain.to_lowercase();
        
        if self.inviolable_domains.iter().any(|d| domain_lower.contains(d)) {
            return ContractTier::Inviolable;
        }
        
        if self.strict_domains.iter().any(|d| domain_lower.contains(d)) {
            return ContractTier::Strict;
        }
        
        // Default to flexible for unknown domains
        ContractTier::Flexible
    }
    
    /// Get metadata for a classification
    pub fn get_metadata(&self, rule_id: &str, domain: &str) -> TierMetadata {
        let tier = self.classify(rule_id, domain);
        
        let reason = match tier {
            ContractTier::Inviolable => 
                format!("Rule '{}' is security/safety critical and cannot be bypassed", rule_id),
            ContractTier::Strict =>
                format!("Rule '{}' requires explicit acceptance with documented reason", rule_id),
            ContractTier::Flexible =>
                format!("Rule '{}' can be auto-learned based on project patterns", rule_id),
        };
        
        TierMetadata {
            tier,
            reason,
            domains: match tier {
                ContractTier::Inviolable => self.inviolable_domains.iter().cloned().collect(),
                ContractTier::Strict => self.strict_domains.iter().cloned().collect(),
                ContractTier::Flexible => vec!["style".into(), "naming".into(), "formatting".into()],
            },
            rule_ids: match tier {
                ContractTier::Inviolable => self.inviolable_rules.iter().cloned().collect(),
                ContractTier::Strict => vec![],
                ContractTier::Flexible => self.flexible_rules.iter().cloned().collect(),
            },
        }
    }
    
    /// Add a custom inviolable rule
    pub fn add_inviolable_rule(&mut self, rule_id: String) {
        self.inviolable_rules.insert(rule_id);
    }
    
    /// Add a custom flexible rule
    pub fn add_flexible_rule(&mut self, rule_id: String) {
        self.flexible_rules.insert(rule_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_rules_are_inviolable() {
        let classifier = ContractClassifier::new();
        assert_eq!(classifier.classify("SEC001", "security"), ContractTier::Inviolable);
        assert_eq!(classifier.classify("SEC013", "sql"), ContractTier::Inviolable);
    }
    
    #[test]
    fn test_style_rules_are_flexible() {
        let classifier = ContractClassifier::new();
        assert_eq!(classifier.classify("STYLE001", "style"), ContractTier::Flexible);
    }
    
    #[test]
    fn test_domain_classification() {
        let classifier = ContractClassifier::new();
        // Unknown rule in security domain
        assert_eq!(classifier.classify("CUSTOM001", "security"), ContractTier::Inviolable);
        // Unknown rule in unknown domain
        assert_eq!(classifier.classify("CUSTOM002", "custom"), ContractTier::Flexible);
    }
}
