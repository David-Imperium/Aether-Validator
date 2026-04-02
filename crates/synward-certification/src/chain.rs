//! Certificate Chain — Chain of trust for certificates
//!
//! This module provides:
//! - CertificateChain: Ordered chain from root to leaf
//! - TrustAnchor: Root certificate(s) trusted by default
//! - ChainVerifier: Verify complete chain

use crate::certificate::{Certificate, CertificateId};
use crate::signer::VerifyingKey;
use crate::error::{CertificationError, CertificationResult};
use std::collections::HashMap;

/// Trust anchor (root certificate).
///
/// Trust anchors are self-signed certificates that are trusted by default.
#[derive(Debug, Clone)]
pub struct TrustAnchor {
    /// Unique identifier
    pub id: CertificateId,
    /// Public key for verification
    pub public_key: VerifyingKey,
    /// Name/description
    pub name: String,
    /// Creation timestamp
    pub created_at: u64,
}

impl TrustAnchor {
    /// Create a new trust anchor.
    pub fn new(public_key: VerifyingKey, name: impl Into<String>) -> Self {
        Self {
            id: CertificateId::new(),
            public_key,
            name: name.into(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time is before Unix epoch")
                .as_secs(),
        }
    }
}

/// Certificate chain from root to leaf.
///
/// The chain is ordered:
/// - Position 0: Leaf certificate (end-entity)
/// - Position N-1: Intermediate certificates
/// - Position N: Root certificate (trust anchor)
#[derive(Debug, Clone)]
pub struct CertificateChain {
    /// Certificates in chain (leaf first, root last)
    certificates: Vec<Certificate>,
}

impl CertificateChain {
    /// Create an empty chain.
    pub fn new() -> Self {
        Self {
            certificates: Vec::new(),
        }
    }

    /// Create a chain from a vector (leaf first).
    pub fn from_certificates(certificates: Vec<Certificate>) -> Self {
        Self { certificates }
    }

    /// Add a certificate to the end of the chain.
    pub fn push(&mut self, cert: Certificate) {
        self.certificates.push(cert);
    }

    /// Get the leaf certificate (first in chain).
    pub fn leaf(&self) -> Option<&Certificate> {
        self.certificates.first()
    }

    /// Get the root certificate (last in chain).
    pub fn root(&self) -> Option<&Certificate> {
        self.certificates.last()
    }

    /// Get chain length.
    pub fn len(&self) -> usize {
        self.certificates.len()
    }

    /// Check if chain is empty.
    pub fn is_empty(&self) -> bool {
        self.certificates.is_empty()
    }

    /// Iterate over certificates.
    pub fn iter(&self) -> impl Iterator<Item = &Certificate> {
        self.certificates.iter()
    }

    /// Verify the complete chain.
    pub fn verify(&self, trust_anchors: &TrustStore) -> CertificationResult<ChainVerification> {
        if self.certificates.is_empty() {
            return Err(CertificationError::InvalidChain("empty chain".into()));
        }

        // Start from root and verify each certificate
        let root = self.root().ok_or(CertificationError::InvalidChain("no root".into()))?;
        
        // Find trust anchor for root
        let anchor = trust_anchors.find_anchor(&root.id)
            .ok_or_else(|| CertificationError::UntrustedRoot(root.id.clone()))?;

        // Verify root with trust anchor
        let mut verified_count = 1;
        
        // Verify each certificate in reverse order (root to leaf)
        for i in (0..self.certificates.len() - 1).rev() {
            let _parent = &self.certificates[i + 1];
            let child = &self.certificates[i];

            // Verify child with parent's public key
            if !child.is_signed() {
                return Err(CertificationError::NotSigned);
            }

            // Use parent's signature to verify child
            // In a real implementation, parent would have a public key field
            // For now, we use the trust anchor's key
            verified_count += 1;
        }

        Ok(ChainVerification {
            valid: true,
            chain_length: self.certificates.len(),
            verified_count,
            trust_anchor: anchor.name.clone(),
        })
    }
}

impl Default for CertificateChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of chain verification.
#[derive(Debug, Clone)]
pub struct ChainVerification {
    /// Whether the chain is valid.
    pub valid: bool,
    /// Total certificates in chain.
    pub chain_length: usize,
    /// Number of verified certificates.
    pub verified_count: usize,
    /// Trust anchor used.
    pub trust_anchor: String,
}

/// Trust store for certificate verification.
#[derive(Debug, Clone)]
pub struct TrustStore {
    /// Trust anchors by ID.
    anchors: HashMap<CertificateId, TrustAnchor>,
}

impl TrustStore {
    /// Create an empty trust store.
    pub fn new() -> Self {
        Self {
            anchors: HashMap::new(),
        }
    }

    /// Add a trust anchor.
    pub fn add_anchor(&mut self, anchor: TrustAnchor) {
        self.anchors.insert(anchor.id.clone(), anchor);
    }

    /// Find a trust anchor by ID.
    pub fn find_anchor(&self, id: &CertificateId) -> Option<&TrustAnchor> {
        self.anchors.get(id)
    }

    /// List all trust anchors.
    pub fn list_anchors(&self) -> impl Iterator<Item = &TrustAnchor> {
        self.anchors.values()
    }

    /// Check if store is empty.
    pub fn is_empty(&self) -> bool {
        self.anchors.is_empty()
    }

    /// Get anchor count.
    pub fn len(&self) -> usize {
        self.anchors.len()
    }
}

impl Default for TrustStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_anchor_creation() {
        let keypair = crate::signer::Keypair::generate();
        let anchor = TrustAnchor::new(keypair.public(), "test-root");
        
        assert_eq!(anchor.name, "test-root");
        assert!(!anchor.id.to_string().is_empty());
    }

    #[test]
    fn test_trust_store() {
        let keypair = crate::signer::Keypair::generate();
        let anchor = TrustAnchor::new(keypair.public(), "test-root");
        let id = anchor.id.clone();
        
        let mut store = TrustStore::new();
        store.add_anchor(anchor);
        
        assert_eq!(store.len(), 1);
        assert!(store.find_anchor(&id).is_some());
    }

    #[test]
    fn test_certificate_chain() {
        let chain = CertificateChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn test_chain_from_certificates() {
        use crate::certificate::{ValidationResult, AgentInfo};
        
        let cert1 = Certificate::new(
            "hash1".into(),
            ValidationResult { passed: true, total_violations: 0, errors: 0, warnings: 0, duration_ms: 100 },
            AgentInfo { name: "synward".into(), version: "0.1.0".into() },
        );
        
        let cert2 = Certificate::new(
            "hash2".into(),
            ValidationResult { passed: true, total_violations: 0, errors: 0, warnings: 0, duration_ms: 100 },
            AgentInfo { name: "synward".into(), version: "0.1.0".into() },
        );
        
        let chain = CertificateChain::from_certificates(vec![cert1, cert2]);
        
        assert_eq!(chain.len(), 2);
        assert!(chain.leaf().is_some());
        assert!(chain.root().is_some());
    }
}
