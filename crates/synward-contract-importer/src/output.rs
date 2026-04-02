//! Output converters for imported contracts
//!
//! Converts ImportedContract to various formats:
//! - Synward YAML format
//! - JSON for API
//! - Markdown documentation

use crate::{ImportedContract, Severity, ContractSource};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Synward contract YAML format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynwardContractFile {
    pub contracts: Vec<SynwardContract>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynwardContract {
    pub id: String,
    pub name: String,
    pub domain: String,
    #[serde(default)]
    pub severity: SynwardSeverity,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub rules: Vec<SynwardRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynwardRule {
    pub pattern: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum SynwardSeverity {
    #[serde(rename = "critical")]
    Critical,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    #[default]
    Warning,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "hint")]
    Hint,
}

impl From<Severity> for SynwardSeverity {
    fn from(s: Severity) -> Self {
        match s {
            Severity::Critical => SynwardSeverity::Critical,
            Severity::Error => SynwardSeverity::Error,
            Severity::Warning => SynwardSeverity::Warning,
            Severity::Info => SynwardSeverity::Info,
            Severity::Hint => SynwardSeverity::Hint,
        }
    }
}

/// Convert imported contracts to Synward YAML format
pub fn to_synward_yaml(contracts: Vec<ImportedContract>) -> SynwardContractFile {
    let synward_contracts: Vec<SynwardContract> = contracts
        .into_iter()
        .map(|c| {
            let rule = if let Some(pattern) = c.pattern {
                Some(SynwardRule {
                    pattern,
                    message: Some(c.description.clone()),
                    suggestion: c.suggestion,
                })
            } else {
                None
            };
            
            SynwardContract {
                id: c.id,
                name: c.name,
                domain: c.domain,
                severity: c.severity.into(),
                description: Some(c.description),
                tags: c.tags,
                rules: rule.into_iter().collect(),
            }
        })
        .collect();
    
    SynwardContractFile {
        contracts: synward_contracts,
    }
}

/// Write contracts to YAML files per language
pub fn write_yaml_files(
    contracts: Vec<ImportedContract>,
    output_dir: &Path,
) -> Result<HashMap<String, usize>> {
    fs::create_dir_all(output_dir)?;
    
    let by_lang = crate::merger::by_language(contracts);
    let mut counts = HashMap::new();
    
    for (lang, lang_contracts) in by_lang {
        let file = to_synward_yaml(lang_contracts.clone());
        let yaml = serde_yaml::to_string(&file)?;
        
        let filename = output_dir.join(format!("imported_{}.yaml", lang));
        fs::write(&filename, yaml)?;
        
        counts.insert(lang, lang_contracts.len());
    }
    
    Ok(counts)
}

/// Generate Markdown documentation
pub fn to_markdown(contracts: &[ImportedContract]) -> String {
    let mut md = String::new();
    
    md.push_str("# Imported Contracts\n\n");
    md.push_str(&format!("Total: {} contracts\n\n", contracts.len()));
    
    // Group by source
    let mut by_source: HashMap<ContractSource, Vec<&ImportedContract>> = HashMap::new();
    for c in contracts {
        by_source.entry(c.source.clone()).or_default().push(c);
    }
    
    for (source, source_contracts) in by_source {
        md.push_str(&format!("## {:?}\n\n", source));
        md.push_str("| ID | Name | Domain | Severity | Description |\n");
        md.push_str("|---|---|---|---|---|\n");
        
        for c in source_contracts {
            let sev = match c.severity {
                Severity::Critical => "**CRITICAL**",
                Severity::Error => "ERROR",
                Severity::Warning => "WARNING",
                Severity::Info => "INFO",
                Severity::Hint => "HINT",
            };
            md.push_str(&format!("| {} | {} | {} | {} | {} |\n", 
                c.id, c.name, c.domain, sev, c.description.chars().take(50).collect::<String>()
            ));
        }
        md.push('\n');
    }
    
    md
}
