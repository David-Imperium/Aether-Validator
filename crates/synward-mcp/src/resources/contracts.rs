//! Contract Resources - Expose validation contracts via MCP

use anyhow::Result;
use rmcp::model::{RawResource, ResourceContents, AnnotateAble};
use std::path::PathBuf;

/// List available contract resources
pub fn list_contract_resources() -> Vec<RawResource> {
    let contracts_dir = get_contracts_dir();
    let mut resources = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&contracts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "yaml").unwrap_or(false) {
                if let Some(name) = path.file_stem() {
                    resources.push(RawResource::new(
                        format!("synward://contracts/{}.yaml", name.to_string_lossy()),
                        format!("Contract: {}", name.to_string_lossy()),
                    ));
                }
            }
        }
    }
    
    resources
}

/// Read a specific contract resource
pub fn read_contract_resource(uri: &str) -> Result<String> {
    // Parse URI: synward://contracts/rust.yaml
    let name = uri
        .strip_prefix("synward://contracts/")
        .and_then(|s| s.strip_suffix(".yaml"))
        .ok_or_else(|| anyhow::anyhow!("Invalid contract URI: {}", uri))?;
    
    let contracts_dir = get_contracts_dir();
    let path = contracts_dir.join(format!("{}.yaml", name));
    
    std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read contract {}: {}", name, e))
}

fn get_contracts_dir() -> PathBuf {
    // Same logic as main.rs
    let local = std::env::current_dir()
        .map(|c| c.join(".factory/contracts"))
        .unwrap_or_default();
    if local.exists() {
        return local;
    }
    
    dirs::home_dir()
        .map(|h| h.join(".synward/contracts"))
        .unwrap_or_else(|| PathBuf::from("contracts"))
}
