//! Validation Layers — Concrete implementations
//!
//! Organized by domain:
//! - `core` - Fundamental building blocks (rules, stripper, contract)
//! - `preprocessing` - Early-stage validation (syntax, ast)
//! - `analysis` - Core validation logic (semantic, logic, complexity, clippy)
//! - `security` - Security validation (security, private, supply_chain)
//! - `architecture` - Architecture and style (architecture, style)
//! - `intelligence` - AI-powered validation (optional)

pub mod core;
pub mod preprocessing;
pub mod analysis;
pub mod security;
pub mod architecture;
#[cfg(feature = "aether-intelligence")]
pub mod intelligence;

// Re-export all layers for backward compatibility
pub use preprocessing::{SyntaxLayer, ASTLayer};
pub use analysis::{SemanticLayer, LogicLayer, ComplexityLayer, ClippyLayer};
pub use security::{SecurityLayer, FallbackSecurityLayer, PrivateLayer, SupplyChainLayer};
pub use architecture::{ArchitectureLayer, StyleLayer};
pub use core::ContractLayer;
#[cfg(feature = "aether-intelligence")]
pub use intelligence::{IntelligenceLayer, IntelligenceConfig};
