//! Contract Loader — Load contracts from YAML files

use std::path::Path;

use crate::contract::{ContractMeta, Severity};
use crate::error::{ContractError, ContractResult};

/// Contract loader for YAML files.
pub struct ContractLoader {
    base_path: std::path::PathBuf,
}

impl ContractLoader {
    /// Create a new loader with base path.
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Load a contract file (supports both wrapped and unwrapped formats).
    pub fn load(&self, path: impl AsRef<Path>) -> ContractResult<Vec<ContractDefinition>> {
        let path = self.base_path.join(path.as_ref());
        let content = std::fs::read_to_string(&path)
            .map_err(|e| ContractError::LoadError(path.display().to_string(), e.to_string()))?;
        
        // Try wrapped format first (contracts: [...])
        if let Ok(wrapped) = serde_yaml::from_str::<ContractsFile>(&content) {
            return Ok(wrapped.contracts);
        }
        
        // Try single contract format
        if let Ok(single) = serde_yaml::from_str::<ContractDefinition>(&content) {
            return Ok(vec![single]);
        }
        
        Err(ContractError::ParseError(path.display().to_string(), "Invalid contract format".to_string()))
    }

    /// Load all contracts from a directory.
    pub fn load_dir(&self, dir: impl AsRef<Path>) -> ContractResult<Vec<ContractDefinition>> {
        let dir_path = self.base_path.join(dir.as_ref());
        let mut all_contracts = Vec::new();

        for entry in std::fs::read_dir(&dir_path)
            .map_err(|e| ContractError::LoadError(dir_path.display().to_string(), e.to_string()))?
        {
            let entry = entry.map_err(|e| ContractError::LoadError(dir_path.display().to_string(), e.to_string()))?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "yaml").unwrap_or(false) {
                // Read and parse directly to avoid double path joining
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| ContractError::LoadError(path.display().to_string(), e.to_string()))?;
                
                // Try wrapped format first (contracts: [...])
                match serde_yaml::from_str::<ContractsFile>(&content) {
                    Ok(wrapped) => {
                        all_contracts.extend(wrapped.contracts);
                        continue;
                    }
                    Err(e) => {
                        eprintln!("DEBUG: Failed to parse as wrapped format: {}", e);
                    }
                }
                
                // Try single contract format
                match serde_yaml::from_str::<ContractDefinition>(&content) {
                    Ok(single) => {
                        all_contracts.push(single);
                        continue;
                    }
                    Err(e) => {
                        eprintln!("DEBUG: Failed to parse as single contract: {}", e);
                    }
                }
                
                return Err(ContractError::ParseError(path.display().to_string(), "Invalid contract format".to_string()));
            }
        }

        Ok(all_contracts)
    }
}

/// Wrapper for contracts file format.
#[derive(Debug, Clone, serde::Deserialize)]
struct ContractsFile {
    contracts: Vec<ContractDefinition>,
}

/// Contract definition from YAML.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContractDefinition {
    pub id: String,
    pub name: String,
    pub domain: String,
    #[serde(default)]
    pub severity: Severity,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub rules: Vec<RuleDefinition>,
}

/// Rule definition from YAML.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RuleDefinition {
    pub pattern: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub suggestion: Option<String>,
}

impl From<ContractDefinition> for ContractMeta {
    fn from(def: ContractDefinition) -> Self {
        Self {
            id: def.id,
            name: def.name,
            domain: def.domain,
            severity: def.severity,
            description: def.description,
            tags: def.tags,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_creation() {
        let loader = ContractLoader::new("/contracts");
        assert!(loader.base_path.to_str().unwrap().contains("contracts"));
    }
}
