//! Contract trait — Abstraction for validation contracts

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Contract metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMeta {
    /// Unique contract ID (e.g., "RUST001").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Domain (e.g., "memory-safety", "error-handling").
    pub domain: String,
    /// Severity level.
    pub severity: Severity,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Tags for filtering.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Severity level for contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Severity {
    Critical,
    Error,
    #[default]
    Warning,
    Info,
    Hint,
}


/// Contract trait for validation rules.
///
/// Each contract represents a single validation rule that:
/// - Has a unique ID and name
/// - Belongs to a domain (memory-safety, error-handling, etc.)
/// - Can evaluate AST to find violations
#[async_trait]
pub trait Contract: Send + Sync {
    /// Get contract metadata.
    fn meta(&self) -> &ContractMeta;

    /// Get the contract ID.
    fn id(&self) -> &str {
        &self.meta().id
    }

    /// Get the contract name.
    fn name(&self) -> &str {
        &self.meta().name
    }

    /// Get the domain.
    fn domain(&self) -> &str {
        &self.meta().domain
    }

    /// Get the severity.
    fn severity(&self) -> Severity {
        self.meta().severity
    }

    /// Evaluate the contract against the AST.
    /// Returns a list of violations found.
    async fn evaluate(&self, source: &str) -> ContractResult<Vec<Violation>>;
}

use crate::error::ContractResult;
use synward_validation::Violation;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_meta() {
        let meta = ContractMeta {
            id: "RUST001".into(),
            name: "No unwrap without context".into(),
            domain: "error-handling".into(),
            severity: Severity::Warning,
            description: Some("Prevent unwrap() without context message".into()),
            tags: vec!["rust".into(), "error-handling".into()],
        };
        
        assert_eq!(meta.id, "RUST001");
        assert_eq!(meta.severity, Severity::Warning);
    }
}
