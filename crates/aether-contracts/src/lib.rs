//! Aether Contracts — Contract engine for validation
//!
//! This crate provides:
//! - Contract trait abstraction
//! - Contract registry for rule lookup
//! - YAML-based contract definitions
//! - Rule evaluation engine
//! - Composite patterns (AND, OR, NOT)

mod contract;
mod registry;
mod loader;
mod evaluator;
mod pattern;
mod error;

pub use contract::{Contract, ContractMeta, Severity};
pub use registry::ContractRegistry;
pub use loader::{ContractLoader, ContractDefinition, RuleDefinition};
pub use evaluator::RuleEvaluator;
pub use pattern::{Pattern, PatternFactory, PatternMatch, TextPattern, RegexPattern, AndPattern, OrPattern, NotPattern};
pub use error::{ContractError, ContractResult};
