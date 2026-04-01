//! Aether Validation — Validation layer pipeline
//!
//! This crate provides:
//! - ValidationLayer trait abstraction
//! - Layer pipeline coordination
//! - Violation collection and reporting
//! - AI-friendly feedback for violations
//! - Built-in layers: Syntax, Semantic, Logic, Architecture, Style, AST
//! - Prompt analysis for AI code generation
//! - Memory-Driven validation via LearnedConfig

mod layer;
mod pipeline;
mod context;
mod violation;
mod feedback;
pub mod layers;
pub mod prompt;
pub mod scope;
pub mod type_inference;
pub mod lsp;

pub use layer::{ValidationLayer, LayerResult, LayerConfig};
pub use pipeline::{ValidationPipeline, PipelineResult};
pub use context::ValidationContext;
pub use violation::{Violation, Severity, Span, deduplicate_violations};
pub use feedback::{FeedbackProvider, ViolationFeedback, FeedbackLevel};
pub use layers::{SyntaxLayer, SemanticLayer, LogicLayer, ArchitectureLayer, StyleLayer, ASTLayer, SecurityLayer, FallbackSecurityLayer, PrivateLayer, ComplexityLayer, SupplyChainLayer, ClippyLayer, ContractLayer};
pub use scope::{ScopeAnalysisLayer, ScopeTree, ScopeNode, ScopeKind, Symbol, SymbolKind, Reference, ReferenceKind, ScopeExtractor};
pub use type_inference::{TypeInferenceLayer, TypeInferenceEngine, Type, TypeKind};
pub use lsp::{LspAnalysisLayer, LspClient, LspClientPool, LspError, LspDiagnostic, LspPosition, JsonRpcTransport, LspServerConfig};

#[cfg(feature = "aether-intelligence")]
pub use layers::IntelligenceLayer;
pub use prompt::{PromptAnalyzer, PromptAnalysis};
