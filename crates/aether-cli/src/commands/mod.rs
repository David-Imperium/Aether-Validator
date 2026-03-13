//! CLI commands module
//!
//! This module contains the command implementations for the Aether CLI.

pub mod executor;
pub mod rag;

// Re-export main command functions
pub use executor::{
    validate, self_validate, analyze, certify, verify, list, generate_keypair, init,
    contracts_check, contracts_update,
};
pub use rag::{AetherRag, CorrectionEntry, SearchResult, RagStats};
