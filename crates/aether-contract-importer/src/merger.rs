//! Contract Merger and Deduplication
//!
//! Merges contracts from multiple sources and removes duplicates.

use crate::{ImportedContract, Severity, ContractSource};
use std::collections::HashMap;

/// Deduplicate contracts by pattern + domain
pub fn deduplicate(contracts: Vec<ImportedContract>) -> Vec<ImportedContract> {
    let mut seen: HashMap<(Option<String>, String), ImportedContract> = HashMap::new();
    
    for contract in contracts {
        let key = (contract.pattern.clone(), contract.domain.clone());
        
        seen.entry(key)
            .and_modify(|existing| {
                // Keep the one with higher severity
                if contract.severity < existing.severity {
                    *existing = contract.clone();
                }
                // Merge references
                existing.references.extend(contract.references.clone());
                existing.references.sort();
                existing.references.dedup();
            })
            .or_insert(contract);
    }
    
    seen.into_values().collect()
}

/// Merge contracts by language
pub fn by_language(contracts: Vec<ImportedContract>) -> HashMap<String, Vec<ImportedContract>> {
    let mut groups: HashMap<String, Vec<ImportedContract>> = HashMap::new();
    
    for contract in contracts {
        for tag in &contract.tags {
            if matches!(tag.as_str(), "rust" | "python" | "javascript" | "typescript" | "cpp" | "go" | "java") {
                groups.entry(tag.clone())
                    .or_default()
                    .push(contract.clone());
            }
        }
        
        // Also group security contracts for all languages
        if contract.domain == "security" {
            groups.entry("all".into())
                .or_default()
                .push(contract);
        }
    }
    
    groups
}

/// Merge contracts by domain
pub fn by_domain(contracts: Vec<ImportedContract>) -> HashMap<String, Vec<ImportedContract>> {
    let mut groups: HashMap<String, Vec<ImportedContract>> = HashMap::new();
    
    for contract in contracts {
        groups.entry(contract.domain.clone())
            .or_default()
            .push(contract);
    }
    
    groups
}

/// Statistics about imported contracts
#[derive(Debug, Clone)]
pub struct ImportStats {
    pub total: usize,
    pub by_source: HashMap<ContractSource, usize>,
    pub by_severity: HashMap<Severity, usize>,
    pub by_domain: HashMap<String, usize>,
}

impl ImportStats {
    pub fn from_contracts(contracts: &[ImportedContract]) -> Self {
        let mut by_source = HashMap::new();
        let mut by_severity = HashMap::new();
        let mut by_domain = HashMap::new();
        
        for c in contracts {
            *by_source.entry(c.source.clone()).or_insert(0) += 1;
            *by_severity.entry(c.severity).or_insert(0) += 1;
            *by_domain.entry(c.domain.clone()).or_insert(0) += 1;
        }
        
        Self {
            total: contracts.len(),
            by_source,
            by_severity,
            by_domain,
        }
    }
}
