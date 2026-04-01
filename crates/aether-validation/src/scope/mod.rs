//! Scope Analysis Module
//!
//! Provides scope-aware validation using tree-sitter AST:
//! - Variable definitions and references
//! - Scope boundary detection
//! - Unused variable detection
//! - Variable shadowing detection
//! - Undefined reference detection

mod tree;
mod symbol;
mod extractor;
mod layer;

pub use tree::{ScopeTree, ScopeNode, ScopeKind};
pub use symbol::{Symbol, SymbolKind, Reference, ReferenceKind};
pub use extractor::{ScopeExtractor, ExtractionResult, ExtractionStats};
pub use layer::ScopeAnalysisLayer;

// Language-specific extractors
pub mod languages;
