//! Contract Registry — Contract lookup and management

use std::collections::HashMap;
use std::sync::Arc;

use crate::contract::Contract;
use crate::error::{ContractError, ContractResult};

/// Registry for validation contracts.
///
/// The registry allows:
/// - Registering contracts by ID
/// - Looking up contracts by domain
/// - Bulk registration from YAML files
pub struct ContractRegistry {
    contracts: HashMap<String, Arc<dyn Contract>>,
    by_domain: HashMap<String, Vec<String>>,
}

impl ContractRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
            by_domain: HashMap::new(),
        }
    }

    /// Register a contract.
    pub fn register(&mut self, contract: impl Contract + 'static) {
        let id = contract.id().to_string();
        let domain = contract.domain().to_string();
        
        self.contracts.insert(id.clone(), Arc::new(contract));
        self.by_domain.entry(domain).or_default().push(id);
    }

    /// Get a contract by ID.
    pub fn get(&self, id: &str) -> ContractResult<Arc<dyn Contract>> {
        self.contracts
            .get(id)
            .cloned()
            .ok_or_else(|| ContractError::NotFound(id.to_string()))
    }

    /// Get all contracts in a domain.
    pub fn by_domain(&self, domain: &str) -> Vec<Arc<dyn Contract>> {
        self.by_domain
            .get(domain)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.contracts.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// List all contract IDs.
    pub fn list(&self) -> Vec<&str> {
        self.contracts.keys().map(|s| s.as_str()).collect()
    }

    /// Get total contract count.
    pub fn count(&self) -> usize {
        self.contracts.len()
    }
}

impl Default for ContractRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{ContractMeta, Severity};
    use async_trait::async_trait;
    use synward_validation::Violation;

    struct TestContract {
        meta: ContractMeta,
    }

    #[async_trait]
    impl Contract for TestContract {
        fn meta(&self) -> &ContractMeta {
            &self.meta
        }

        async fn evaluate(&self, _source: &str) -> ContractResult<Vec<Violation>> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ContractRegistry::new();
        
        registry.register(TestContract {
            meta: ContractMeta {
                id: "TEST001".into(),
                name: "Test Contract".into(),
                domain: "test".into(),
                severity: Severity::Warning,
                description: None,
                tags: Vec::new(),
            },
        });
        
        assert!(registry.get("TEST001").is_ok());
        assert_eq!(registry.count(), 1);
    }
}
