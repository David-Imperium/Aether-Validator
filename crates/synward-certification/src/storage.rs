//! Certificate Storage — Persistent certificate storage

use std::path::PathBuf;
use std::collections::HashMap;
use std::fs;

use crate::certificate::Certificate;
use crate::error::{CertificationError, CertificationResult};

/// In-memory certificate store.
#[derive(Debug, Default)]
pub struct CertificateStore {
    certificates: HashMap<String, Certificate>,
    storage_path: Option<PathBuf>,
}

impl CertificateStore {
    /// Create a new in-memory store.
    pub fn new() -> Self {
        Self {
            certificates: HashMap::new(),
            storage_path: None,
        }
    }

    /// Create a store with persistent storage.
    pub fn with_storage(path: impl Into<PathBuf>) -> Self {
        Self {
            certificates: HashMap::new(),
            storage_path: Some(path.into()),
        }
    }

    /// Store a certificate.
    pub fn store(&mut self, cert: &Certificate) -> CertificationResult<()> {
        let id = cert.id.to_string();
        self.certificates.insert(id.clone(), cert.clone());
        
        if self.storage_path.is_some() {
            self.persist(&id, cert)?;
        }
        
        Ok(())
    }

    /// Retrieve a certificate by ID.
    pub fn get(&self, id: &str) -> Option<&Certificate> {
        self.certificates.get(id)
    }

    /// Check if a certificate exists.
    pub fn exists(&self, id: &str) -> bool {
        self.certificates.contains_key(id)
    }

    /// Remove a certificate.
    pub fn remove(&mut self, id: &str) -> Option<Certificate> {
        let cert = self.certificates.remove(id);
        
        if let Some(ref path) = self.storage_path {
            let file = path.join(format!("{}.json", id));
            let _ = fs::remove_file(file);
        }
        
        cert
    }

    /// List all certificate IDs.
    pub fn list(&self) -> Vec<&str> {
        self.certificates.keys().map(|s| s.as_str()).collect()
    }

    /// Persist a certificate to disk.
    fn persist(&self, id: &str, cert: &Certificate) -> CertificationResult<()> {
        if let Some(ref path) = self.storage_path {
            fs::create_dir_all(path)
                .map_err(|e| CertificationError::StorageError(e.to_string()))?;
            
            let file = path.join(format!("{}.json", id));
            let content = serde_json::to_string_pretty(cert)
                .map_err(|e| CertificationError::StorageError(e.to_string()))?;
            
            fs::write(&file, content)
                .map_err(|e| CertificationError::StorageError(e.to_string()))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::{ValidationResult, AgentInfo};

    fn test_cert() -> Certificate {
        Certificate::new(
            "hash-001".to_string(),
            ValidationResult {
                passed: true,
                total_violations: 0,
                errors: 0,
                warnings: 0,
                duration_ms: 100,
            },
            AgentInfo {
                name: "synward".to_string(),
                version: "0.1.0".to_string(),
            },
        )
    }

    #[test]
    fn test_store_and_get() {
        let mut store = CertificateStore::new();
        let cert = test_cert();
        let id = cert.id.to_string();
        
        store.store(&cert).unwrap();
        assert!(store.exists(&id));
        
        let retrieved = store.get(&id).unwrap();
        assert_eq!(retrieved.file_hash, "hash-001");
    }

    #[test]
    fn test_remove() {
        let mut store = CertificateStore::new();
        let cert = test_cert();
        let id = cert.id.to_string();
        
        store.store(&cert).unwrap();
        let removed = store.remove(&id).unwrap();
        
        assert!(!store.exists(&id));
        assert_eq!(removed.file_hash, "hash-001");
    }
}
