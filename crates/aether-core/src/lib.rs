//! Aether Core — Orchestrator and Session management
//!
//! This crate provides the main orchestration logic for Aether validation:
//! - Session management for validation runs
//! - Pipeline coordination across validation layers
//! - Result aggregation and reporting

mod orchestrator;
mod session;
mod pipeline;
mod config;
mod error;

pub use orchestrator::Orchestrator;
pub use session::{Session, SessionId};
pub use pipeline::Pipeline;
pub use config::Config;
pub use error::{Error, Result};
