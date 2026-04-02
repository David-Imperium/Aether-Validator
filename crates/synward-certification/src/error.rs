//! Certification errors

use thiserror::Error;

/// Result type for certification operations.
pub type CertificationResult<T> = std::result::Result<T, CertificationError>;

/// Errors that can occur during certification.
#[derive(Debug, Error)]
pub enum CertificationError {
    /// Certificate not found.
    #[error("Certificate not found: {0}")]
    NotFound(String),

    /// Certificate not signed.
    #[error("Certificate is not signed")]
    NotSigned,

    /// Invalid signature.
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Signing error.
    #[error("Signing error: {0}")]
    SigningError(String),

    /// Storage error.
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Verification failed.
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// Invalid certificate chain.
    #[error("Invalid certificate chain: {0}")]
    InvalidChain(String),

    /// Untrusted root certificate.
    #[error("Untrusted root certificate: {0}")]
    UntrustedRoot(crate::certificate::CertificateId),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_not_found() {
        let err = CertificationError::NotFound("cert-001".to_string());
        assert!(err.to_string().contains("cert-001"));
    }
}
