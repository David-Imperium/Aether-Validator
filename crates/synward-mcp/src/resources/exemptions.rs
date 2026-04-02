//! Exemption Resources - Expose compliance exemptions via MCP

use anyhow::Result;
use rmcp::model::RawResource;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Exemption resource info
#[derive(Debug, Serialize, Deserialize)]
pub struct ExemptionResource {
    pub id: String,
    pub rule_id: String,
    pub scope: String,
    pub reason: String,
    pub confidence: f64,
    pub source: String,
    pub application_count: u32,
}

/// List exemption resources
pub fn list_exemption_resources() -> Vec<RawResource> {
    let exemptions_path = get_exemptions_path();
    
    if exemptions_path.exists() {
        vec![RawResource::new(
            "synward://exemptions/all".to_string(),
            "Compliance Exemptions".to_string(),
        )]
    } else {
        vec![]
    }
}

/// Read exemptions
pub fn read_exemptions_resource() -> Result<Vec<ExemptionResource>> {
    let path = get_exemptions_path();
    
    if !path.exists() {
        return Ok(vec![]);
    }
    
    let content = std::fs::read_to_string(&path)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;
    
    let mut exemptions = Vec::new();
    
    if let serde_json::Value::Object(map) = data {
        for (_rule_id, items) in map {
            if let serde_json::Value::Array(arr) = items {
                for item in arr {
                    if let serde_json::Value::Object(obj) = item {
                        exemptions.push(ExemptionResource {
                            id: obj.get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            rule_id: obj.get("rule_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            scope: obj.get("scope")
                                .map(|v| v.to_string())
                                .unwrap_or_default(),
                            reason: obj.get("reason")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            confidence: obj.get("confidence")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.5),
                            source: obj.get("source")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            application_count: obj.get("application_count")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0) as u32,
                        });
                    }
                }
            }
        }
    }
    
    Ok(exemptions)
}

fn get_exemptions_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.join(".synward/exemptions.json")
}
