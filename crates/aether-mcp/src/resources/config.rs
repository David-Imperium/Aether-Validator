//! Config Resources - Expose Aether configuration via MCP

use anyhow::Result;
use rmcp::model::RawResource;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Aether configuration resource
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResource {
    pub compliance: ComplianceConfigInfo,
    pub dubbioso: DubbiosoConfigInfo,
    pub validation: ValidationConfigInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceConfigInfo {
    pub auto_accept_threshold: f64,
    pub ask_threshold: f64,
    pub learn_after_occurrences: u32,
    pub use_dubbioso: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DubbiosoConfigInfo {
    pub preset: String,
    pub ask_threshold: f64,
    pub warn_threshold: f64,
    pub auto_accept_threshold: f64,
    pub permanent_after: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationConfigInfo {
    pub severity_filter: String,
    pub max_violations: usize,
    pub enable_experimental: bool,
}

/// List config resources
pub fn list_config_resources() -> Vec<RawResource> {
    vec![
        RawResource::new(
            "aether://config/main".to_string(),
            "Aether Configuration".to_string(),
        ),
        RawResource::new(
            "aether://config/compliance".to_string(),
            "Compliance Engine Configuration".to_string(),
        ),
        RawResource::new(
            "aether://config/dubbioso".to_string(),
            "Dubbioso Mode Configuration".to_string(),
        ),
    ]
}

/// Read main configuration
pub fn read_config_resource(uri: &str) -> Result<ConfigResource> {
    // Default configuration (would load from .aether/config.toml if exists)
    Ok(ConfigResource {
        compliance: ComplianceConfigInfo {
            auto_accept_threshold: 0.90,
            ask_threshold: 0.60,
            learn_after_occurrences: 3,
            use_dubbioso: true,
        },
        dubbioso: DubbiosoConfigInfo {
            preset: "balanced".to_string(),
            ask_threshold: 0.60,
            warn_threshold: 0.80,
            auto_accept_threshold: 0.95,
            permanent_after: 5,
        },
        validation: ValidationConfigInfo {
            severity_filter: "all".to_string(),
            max_violations: 1000,
            enable_experimental: false,
        },
    })
}
