//! Aether Validation — Validation layer pipeline
//!
//! This crate provides:
//! - ValidationLayer trait abstraction
//! - Layer pipeline coordination
//! - Violation collection and reporting
//! - AI-friendly feedback for violations
//! - Built-in layers: Syntax, Semantic, Logic, Architecture, Style, AST
//! - Prompt analysis for AI code generation

mod layer;
mod pipeline;
mod context;
mod violation;
mod feedback;
pub mod layers;
pub mod prompt;

pub use layer::{ValidationLayer, LayerResult};
pub use pipeline::{ValidationPipeline, PipelineResult};
pub use context::ValidationContext;
pub use violation::{Violation, Severity, Span};
pub use feedback::{FeedbackProvider, ViolationFeedback, FeedbackLevel};
pub use layers::{SyntaxLayer, SemanticLayer, LogicLayer, ArchitectureLayer, StyleLayer, ASTLayer, SecurityLayer, PrivateLayer, ComplexityLayer, SupplyChainLayer};
pub use prompt::{PromptAnalyzer, PromptAnalysis};
