//! CLI commands module
//!
//! This module contains the command implementations for the Synward CLI.

pub mod executor;
pub mod rag;
pub mod hooks;
pub mod state;
pub mod postprocess;

#[cfg(feature = "intelligence")]
pub mod memory;

#[cfg(feature = "intelligence")]
pub mod drift;

#[cfg(feature = "intelligence")]
pub mod learn;

// Re-export main command functions
pub use executor::{
    validate, self_validate, analyze, certify, verify, list, generate_keypair, init,
    contracts_check, contracts_update,
};
pub use rag::SynwardRag;

#[cfg(feature = "intelligence")]
pub use learn::learn;

