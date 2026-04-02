//! Analysis layers
//!
//! Core validation logic:
//! - `semantic` - Semantic analysis (types, scopes)
//! - `logic` - Contract pattern evaluation
//! - `complexity` - Complexity metrics
//! - `clippy` - Clippy lint integration

mod semantic;
mod logic;
mod complexity;
mod clippy;

pub use semantic::SemanticLayer;
pub use logic::LogicLayer;
pub use complexity::ComplexityLayer;
pub use clippy::ClippyLayer;
