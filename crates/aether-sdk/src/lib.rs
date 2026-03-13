//! Aether SDK — Client library for code validation and certification
//!
//! This crate provides a high-level SDK for integrating Aether validation
//! into external applications, CLIs, and language bindings.

mod client;
mod error;
mod types;

pub use client::AetherClient;
pub use error::{SdkError, SdkResult};
pub use types::{
    ValidationOptions,
    CertificationOptions,
    ValidationResult,
    CertificationResult,
    AnalysisResult,
};

/// SDK version
pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default server address
pub const DEFAULT_SERVER: &str = "http://127.0.0.1:3000";
