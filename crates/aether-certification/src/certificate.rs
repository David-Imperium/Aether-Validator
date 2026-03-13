//! Certificate — Validation certificate with signature

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// Unique certificate ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CertificateId(String);

impl CertificateId {
    /// Generate a new unique certificate ID.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create from string.
    pub fn from_string(s: String) -> Self {
        Self(s)
    }
}

impl Default for CertificateId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CertificateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validation result summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed.
    pub passed: bool,
    /// Total violations found.
    pub total_violations: usize,
    /// Errors count.
    pub errors: usize,
    /// Warnings count.
    pub warnings: usize,
    /// Validation duration in milliseconds.
    pub duration_ms: u64,
}

/// Agent information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent name/version.
    pub name: String,
    /// Agent version.
    pub version: String,
}

/// A validation certificate.
///
/// Certificates are signed proof that validation was performed
/// and the code passed all checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// Unique certificate ID.
    pub id: CertificateId,
    /// Hash of the validated file(s).
    pub file_hash: String,
    /// Validation result summary.
    pub validation: ValidationResult,
    /// Timestamp (Unix epoch).
    pub timestamp: u64,
    /// Agent that created the certificate.
    pub agent: AgentInfo,
    /// Ed25519 signature (Base64).
    pub signature: Option<String>,
}

impl Certificate {
    /// Create a new unsigned certificate.
    pub fn new(file_hash: String, validation: ValidationResult, agent: AgentInfo) -> Self {
        Self {
            id: CertificateId::new(),
            file_hash,
            validation,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time is before Unix epoch")
                .as_secs(),
            agent,
            signature: None,
        }
    }

    /// Compute SHA-256 hash of a file.
    pub fn hash_file(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Get canonical form for signing.
    pub fn canonical_form(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.id,
            self.file_hash,
            self.validation.passed,
            self.timestamp,
            self.agent.name
        )
    }

    /// Check if the certificate is signed.
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_creation() {
        let cert = Certificate::new(
            "abc123".to_string(),
            ValidationResult {
                passed: true,
                total_violations: 0,
                errors: 0,
                warnings: 0,
                duration_ms: 100,
            },
            AgentInfo {
                name: "aether".to_string(),
                version: "0.1.0".to_string(),
            },
        );
        
        assert!(!cert.is_signed());
        assert!(cert.id.to_string().len() > 0);
    }

    #[test]
    fn test_file_hash() {
        let content = b"fn main() {}";
        let hash = Certificate::hash_file(content);
        assert_eq!(hash.len(), 64); // SHA-256 hex length
    }
}
