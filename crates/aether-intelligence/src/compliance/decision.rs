//! Compliance Decision - Actions and reasoning for compliance results

use serde::{Deserialize, Serialize};
use crate::compliance::ContractTier;

/// Action to take for a violation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComplianceAction {
    /// Block the validation - non-negotiable
    Block,
    /// Warn but allow continuation
    Warn,
    /// Ask user for input (Dubbioso mode)
    Ask {
        /// Question to ask
        question: String,
        /// Suggested options
        options: Vec<String>,
    },
    /// Learn this pattern for future
    Learn {
        /// Pattern to learn
        pattern: String,
        /// Confidence in pattern
        confidence: f64,
    },
    /// Accept based on learned patterns
    Accept {
        /// Reason for acceptance
        reason: String,
        /// Precedent that led to acceptance
        precedent_id: Option<String>,
    },
}

/// Reason for the decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionReason {
    /// Rule is inviolable
    InviolableRule,
    /// Similar violation was accepted before
    PrecedentMatch {
        similarity: f64,
        precedent_id: String,
    },
    /// Pattern learned from project
    LearnedPattern {
        pattern_id: String,
        occurrence_count: u32,
    },
    /// Context allows this violation
    ContextualAllowance {
        context_type: String,
        reason: String,
    },
    /// User explicitly accepted
    UserAccepted {
        reason: String,
        accepted_at: chrono::DateTime<chrono::Utc>,
    },
    /// High confidence from Dubbioso
    HighConfidence {
        confidence: f64,
        signals: Vec<String>,
    },
    /// Low confidence - needs user input
    LowConfidence {
        confidence: f64,
        uncertainty_reasons: Vec<String>,
    },
    /// No precedent found
    NoPrecedent,
}

/// Compliance decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceDecision {
    /// The action to take
    pub action: ComplianceAction,
    /// Why this decision was made
    pub reason: DecisionReason,
    /// Contract tier for this rule
    pub tier: ContractTier,
    /// Confidence in this decision (0-1)
    pub confidence: f64,
    /// Whether this decision can be overridden
    pub overridable: bool,
    /// Human-readable explanation
    pub explanation: String,
    /// Related decisions from history
    pub related_decisions: Vec<String>,
}

impl ComplianceDecision {
    /// Create a block decision
    pub fn block(tier: ContractTier, rule_id: &str, message: &str) -> Self {
        Self {
            action: ComplianceAction::Block,
            reason: DecisionReason::InviolableRule,
            tier,
            confidence: 1.0,
            overridable: false,
            explanation: format!(
                "Rule '{}' is inviolable: {}. This violation must be fixed.",
                rule_id, message
            ),
            related_decisions: vec![],
        }
    }
    
    /// Create a warn decision
    pub fn warn(tier: ContractTier, message: &str, explanation: &str) -> Self {
        Self {
            action: ComplianceAction::Warn,
            reason: DecisionReason::NoPrecedent,
            tier,
            confidence: 0.7,
            overridable: true,
            explanation: format!("{}: {}", message, explanation),
            related_decisions: vec![],
        }
    }
    
    /// Create an ask decision (for Dubbioso mode)
    pub fn ask(
        tier: ContractTier,
        question: String,
        options: Vec<String>,
        uncertainty_reasons: Vec<String>,
        confidence: f64,
    ) -> Self {
        Self {
            action: ComplianceAction::Ask { question, options },
            reason: DecisionReason::LowConfidence {
                confidence,
                uncertainty_reasons,
            },
            tier,
            confidence,
            overridable: true,
            explanation: "Confidence is low, user input needed".into(),
            related_decisions: vec![],
        }
    }
    
    /// Create a learn decision
    pub fn learn(tier: ContractTier, pattern: String, confidence: f64) -> Self {
        Self {
            action: ComplianceAction::Learn { pattern, confidence },
            reason: DecisionReason::ContextualAllowance {
                context_type: "project-pattern".into(),
                reason: "Pattern observed multiple times".into(),
            },
            tier,
            confidence,
            overridable: true,
            explanation: "New pattern detected, will learn for future".into(),
            related_decisions: vec![],
        }
    }
    
    /// Create an accept decision
    pub fn accept(
        tier: ContractTier,
        reason: String,
        precedent_id: Option<String>,
        confidence: f64,
    ) -> Self {
        let reason_enum = if let Some(ref id) = precedent_id {
            DecisionReason::PrecedentMatch {
                similarity: confidence,
                precedent_id: id.clone(),
            }
        } else {
            DecisionReason::HighConfidence {
                confidence,
                signals: vec!["learned-pattern".into()],
            }
        };
        
        Self {
            action: ComplianceAction::Accept { reason, precedent_id },
            reason: reason_enum,
            tier,
            confidence,
            overridable: true,
            explanation: "Accepted based on learned patterns or precedents".into(),
            related_decisions: vec![],
        }
    }
    
    /// Check if validation should fail
    pub fn should_fail(&self) -> bool {
        matches!(self.action, ComplianceAction::Block)
    }
    
    /// Check if user interaction is needed
    pub fn needs_user_input(&self) -> bool {
        matches!(self.action, ComplianceAction::Ask { .. })
    }
}
