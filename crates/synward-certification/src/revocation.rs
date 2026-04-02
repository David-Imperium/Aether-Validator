//! Revocation — Certificate revocation mechanism
//!
//! This module provides:
//! - RevocationList: List of revoked certificates
//! - RevocationStatus: Status of a certificate (valid/revoked)
//! - RevocationReason: Why a certificate was revoked

use crate::certificate::CertificateId;
use crate::error::CertificationResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Reason for certificate revocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevocationReason {
    /// Key was compromised.
    KeyCompromise,
    /// Certificate was replaced.
    Superseded,
    /// Certificate is no longer needed.
    CessationOfOperation,
    /// Certificate contained invalid information.
    InvalidInformation,
    /// Other reason.
    Other,
}

impl std::fmt::Display for RevocationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeyCompromise => write!(f, "key_compromise"),
            Self::Superseded => write!(f, "superseded"),
            Self::CessationOfOperation => write!(f, "cessation_of_operation"),
            Self::InvalidInformation => write!(f, "invalid_information"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// A revoked certificate entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokedCertificate {
    /// Certificate ID.
    pub id: CertificateId,
    /// Revocation timestamp (Unix epoch).
    pub revoked_at: u64,
    /// Reason for revocation.
    pub reason: RevocationReason,
    /// Optional comment.
    pub comment: Option<String>,
}

impl RevokedCertificate {
    /// Create a new revocation entry.
    pub fn new(id: CertificateId, reason: RevocationReason) -> Self {
        Self {
            id,
            revoked_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time is before Unix epoch")
                .as_secs(),
            reason,
            comment: None,
        }
    }

    /// Add a comment to the revocation.
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}

/// Certificate Revocation List (CRL).
#[derive(Debug, Clone, Default)]
pub struct RevocationList {
    /// Revoked certificates by ID.
    revoked: HashMap<CertificateId, RevokedCertificate>,
}

impl RevocationList {
    /// Create an empty revocation list.
    pub fn new() -> Self {
        Self {
            revoked: HashMap::new(),
        }
    }

    /// Revoke a certificate.
    pub fn revoke(&mut self, id: CertificateId, reason: RevocationReason) -> CertificationResult<()> {
        let entry = RevokedCertificate::new(id.clone(), reason);
        self.revoked.insert(id, entry);
        Ok(())
    }

    /// Revoke a certificate with a comment.
    pub fn revoke_with_comment(
        &mut self,
        id: CertificateId,
        reason: RevocationReason,
        comment: impl Into<String>,
    ) -> CertificationResult<()> {
        let entry = RevokedCertificate::new(id.clone(), reason).with_comment(comment);
        self.revoked.insert(id, entry);
        Ok(())
    }

    /// Check if a certificate is revoked.
    pub fn is_revoked(&self, id: &CertificateId) -> bool {
        self.revoked.contains_key(id)
    }

    /// Get revocation details for a certificate.
    pub fn get_revocation(&self, id: &CertificateId) -> Option<&RevokedCertificate> {
        self.revoked.get(id)
    }

    /// Remove a certificate from the revocation list (unrevoke).
    pub fn unrevoke(&mut self, id: &CertificateId) -> Option<RevokedCertificate> {
        self.revoked.remove(id)
    }

    /// Get all revoked certificates.
    pub fn list_revoked(&self) -> impl Iterator<Item = &RevokedCertificate> {
        self.revoked.values()
    }

    /// Get revoked certificate count.
    pub fn len(&self) -> usize {
        self.revoked.len()
    }

    /// Check if list is empty.
    pub fn is_empty(&self) -> bool {
        self.revoked.is_empty()
    }

    /// Export to JSON.
    pub fn to_json(&self) -> CertificationResult<String> {
        serde_json::to_string(&self.revoked)
            .map_err(|e| crate::error::CertificationError::SerializationError(e.to_string()))
    }

    /// Import from JSON.
    pub fn from_json(json: &str) -> CertificationResult<Self> {
        let revoked: HashMap<CertificateId, RevokedCertificate> = serde_json::from_str(json)
            .map_err(|e| crate::error::CertificationError::SerializationError(e.to_string()))?;
        Ok(Self { revoked })
    }
}

/// Status of a certificate regarding revocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationStatus {
    /// Certificate is valid (not revoked).
    Valid,
    /// Certificate is revoked.
    Revoked,
}

impl RevocationStatus {
    /// Check if status is valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    /// Check if status is revoked.
    pub fn is_revoked(&self) -> bool {
        matches!(self, Self::Revoked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revocation_reason_display() {
        assert_eq!(RevocationReason::KeyCompromise.to_string(), "key_compromise");
        assert_eq!(RevocationReason::Superseded.to_string(), "superseded");
    }

    #[test]
    fn test_revoked_certificate_creation() {
        let id = CertificateId::new();
        let revoked = RevokedCertificate::new(id.clone(), RevocationReason::KeyCompromise);
        
        assert_eq!(revoked.id, id);
        assert_eq!(revoked.reason, RevocationReason::KeyCompromise);
        assert!(revoked.comment.is_none());
    }

    #[test]
    fn test_revoked_certificate_with_comment() {
        let id = CertificateId::new();
        let revoked = RevokedCertificate::new(id, RevocationReason::KeyCompromise)
            .with_comment("Key was leaked");
        
        assert_eq!(revoked.comment, Some("Key was leaked".to_string()));
    }

    #[test]
    fn test_revocation_list() {
        let mut crl = RevocationList::new();
        let id = CertificateId::new();
        
        assert!(crl.is_empty());
        
        crl.revoke(id.clone(), RevocationReason::KeyCompromise).unwrap();
        
        assert_eq!(crl.len(), 1);
        assert!(crl.is_revoked(&id));
        
        let revocation = crl.get_revocation(&id);
        assert!(revocation.is_some());
        assert_eq!(revocation.unwrap().reason, RevocationReason::KeyCompromise);
    }

    #[test]
    fn test_unrevoke() {
        let mut crl = RevocationList::new();
        let id = CertificateId::new();
        
        crl.revoke(id.clone(), RevocationReason::KeyCompromise).unwrap();
        assert!(crl.is_revoked(&id));
        
        crl.unrevoke(&id);
        assert!(!crl.is_revoked(&id));
    }

    #[test]
    fn test_revocation_list_json() {
        let mut crl = RevocationList::new();
        let id = CertificateId::new();
        
        crl.revoke(id.clone(), RevocationReason::KeyCompromise).unwrap();
        
        let json = crl.to_json().unwrap();
        // JSON contains the certificate ID
        assert!(json.contains(&id.to_string()));
        
        let restored = RevocationList::from_json(&json).unwrap();
        assert!(restored.is_revoked(&id));
    }

    #[test]
    fn test_revocation_status() {
        assert!(RevocationStatus::Valid.is_valid());
        assert!(!RevocationStatus::Valid.is_revoked());
        
        assert!(!RevocationStatus::Revoked.is_valid());
        assert!(RevocationStatus::Revoked.is_revoked());
    }
}
