//! Audit Log — Certificate audit trail

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Audit entry for certificate operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Entry ID.
    pub id: String,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Operation type.
    pub operation: AuditOperation,
    /// Certificate ID.
    pub certificate_id: String,
    /// File hash.
    pub file_hash: String,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

/// Type of audit operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuditOperation {
    /// Certificate created.
    Create,
    /// Certificate signed.
    Sign,
    /// Certificate verified.
    Verify,
    /// Certificate stored.
    Store,
    /// Certificate retrieved.
    Retrieve,
}

/// Audit log for tracking certificate operations.
#[derive(Debug, Default)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    /// Create a new empty audit log.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add an entry to the log.
    pub fn add(&mut self, entry: AuditEntry) {
        self.entries.push(entry);
    }

    /// Create and add a new entry.
    pub fn log(&mut self, operation: AuditOperation, certificate_id: &str, file_hash: &str) {
        self.add(AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            operation,
            certificate_id: certificate_id.to_string(),
            file_hash: file_hash.to_string(),
            metadata: HashMap::new(),
        });
    }

    /// Get all entries.
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Get entries for a certificate.
    pub fn for_certificate(&self, id: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.certificate_id == id)
            .collect()
    }

    /// Clear the log.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log() {
        let mut log = AuditLog::new();
        
        log.log(
            AuditOperation::Create,
            "cert-001",
            "hash-001",
        );
        log.log(
            AuditOperation::Sign,
            "cert-001",
            "hash-001",
        );
        
        assert_eq!(log.entries().len(), 2);
        assert_eq!(log.for_certificate("cert-001").len(), 2);
    }
}
