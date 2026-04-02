//! Compliance Engine - Intelligent Contract Enforcement
//!
//! Combines strict contract enforcement with context-aware learning.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    COMPLIANCE ENGINE                     │
//! ├─────────────────────────────────────────────────────────┤
//! │  Contract Tier: INVIOLABLE > STRICT > FLEXIBLE          │
//! │  Context: Project type, code region, history            │
//! │  Decision: BLOCK | WARN | ASK | LEARN | ACCEPT          │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use synward_intelligence::compliance::{ComplianceEngine, ContractTier};
//!
//! let engine = ComplianceEngine::new()?;
//! let decision = engine.evaluate(violation, context).await?;
//!
//! match decision.action {
//!     ComplianceAction::Block => { /* non-negotiable */ }
//!     ComplianceAction::Ask(reason) => { /* use Dubbioso */ }
//!     ComplianceAction::Learn(pattern) => { /* update patterns */ }
//!     _ => {}
//! }
//! ```

mod classifier;
mod engine;
mod decision;
mod exemptions;

pub use classifier::{ContractClassifier, ContractTier, TierMetadata};
pub use engine::{ComplianceEngine, ComplianceConfig, ComplianceContext, ComplianceStats};
pub use decision::{ComplianceDecision, ComplianceAction, DecisionReason};
pub use exemptions::{ExemptionStore, Exemption, ExemptionScope, ExemptionSource};
