//! Code Property Graph — AST + CFG + DFG unified representation.
//!
//! The CPG extends the simple function-level call graph with full structural,
//! control-flow, and data-flow information extracted via tree-sitter.

pub mod builder;
pub mod types;

pub use types::{
    CPGEdge, CPGEdgeType, CPGNode, CPGNodeType, CodePropertyGraph, EdgeIndex,
};
pub use builder::CPGBuilder;
