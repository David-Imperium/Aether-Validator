//! Signer — Ed25519 signing and verification

use ed25519_dalek::{Signature, Signer, Verifier as Ed25519Verifier, SigningKey};
use rand::rngs::OsRng;

use crate::certificate::Certificate;
use crate::error::{CertificationError, CertificationResult};

// Re-export VerifyingKey for public use
pub use ed25519_dalek::VerifyingKey;

/// Ed25519 keypair for signing.
pub struct Keypair {
    signing_key: SigningKey,
}

impl Keypair {
    /// Generate a new random keypair.
    pub fn generate() -> Self {
        let mut rng = OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        Self { signing_key }
    }

    /// Get the public key (verifying key).
    pub fn public(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Sign a message.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Get the keypair as bytes (secret key + public key).
    pub fn to_bytes(&self) -> [u8; 64] {
        self.signing_key.to_keypair_bytes()
    }

    /// Create from bytes.
    pub fn from_bytes(bytes: &[u8; 64]) -> CertificationResult<Self> {
        let signing_key = SigningKey::from_keypair_bytes(bytes)
            .map_err(|e| CertificationError::SigningError(e.to_string()))?;
        Ok(Self { signing_key })
    }

    /// Sign a certificate in-place.
    pub fn sign_certificate(&self, cert: &mut Certificate) -> CertificationResult<()> {
        let message = cert.canonical_form();
        let signature = self.sign(message.as_bytes());
        cert.signature = Some(base64_encode(&signature.to_bytes()));
        Ok(())
    }
}

/// Certificate signer trait (alias for Keypair).
pub type CertificateSigner = Keypair;

/// Verifier for certificate verification.
pub struct CertificateVerifier;

impl CertificateVerifier {
    /// Verify a certificate signature.
    pub fn verify(cert: &Certificate, public_key: &VerifyingKey) -> CertificationResult<bool> {
        let signature_bytes = cert.signature
            .as_ref()
            .ok_or(CertificationError::NotSigned)?;
        
        let signature_bytes = base64_decode(signature_bytes)?;
        let signature = Signature::from_bytes(&signature_bytes.try_into()
            .map_err(|_| CertificationError::InvalidSignature("invalid signature length".into()))?);
        
        let message = cert.canonical_form();
        Ok(public_key.verify(message.as_bytes(), &signature).is_ok())
    }
}

// Backward compatibility alias
pub use CertificateVerifier as Verifier;

fn base64_encode(bytes: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
}

fn base64_decode(s: &str) -> CertificationResult<Vec<u8>> {
    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)
        .map_err(|e| CertificationError::InvalidSignature(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::{ValidationResult, AgentInfo};

    #[test]
    fn test_keypair_generation() {
        let keypair = Keypair::generate();
        let public = keypair.public();
        
        assert!(!public.as_bytes().iter().all(|&b| b == 0));
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = Keypair::generate();
        let public = keypair.public();
        
        let mut cert = Certificate::new(
            "test".to_string(),
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
        );
        
        keypair.sign_certificate(&mut cert).unwrap();
        assert!(cert.is_signed());
        
        let verified = Verifier::verify(&cert, &public).unwrap();
        assert!(verified);
    }
}
