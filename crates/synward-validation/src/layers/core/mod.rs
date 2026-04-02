//! Core validation components
//!
//! Contains fundamental building blocks:
//! - `rules` - Pattern matching rules and context checks
//! - `stripper` - Source code preprocessing (remove tests, strings)
//! - `loop_detection` - Loop boundary detection utilities
//! - `contract` - Contract validation primitives

mod rules;
mod stripper;
mod loop_detection;
mod contract;

pub use rules::{PatternRule, ContextCheck, precision_rules};
pub use stripper::{
    strip_test_blocks, strip_string_literals,
    find_checked_unwrap_lines, find_safety_comment_lines,
    has_nearby_safety, has_inline_safety, check_long_functions, check_deep_nesting
};
pub use loop_detection::find_loop_lines;
pub use contract::ContractLayer;
