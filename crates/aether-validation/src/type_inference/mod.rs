//! Type Inference Module
//!
//! Provides basic type inference using tree-sitter AST:
//! - Variable type inference from assignments
//! - Function return type inference
//! - Type mismatch detection
//! - Implicit any detection (TypeScript)

mod types;
mod inference;
mod layer;

pub use types::{Type, TypeKind, TypeVar, TypeScheme};
pub use inference::{TypeInferenceEngine, InferenceResult, InferenceError};
pub use layer::TypeInferenceLayer;

// Language-specific type inferrers
pub mod languages;
