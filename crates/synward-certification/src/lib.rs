//! Synward Certification — Certificate generation and verification
//!
//! This module provides:
//! - Ed25519-based certificate signing
//! - Certificate chain verification
//! - Certificate revocation (CRL)
//! - Audit logging
//! - Certificate storage

mod certificate;
mod signer;
mod chain;
mod revocation;
mod audit;
mod storage;
mod error;

pub use certificate::{Certificate, CertificateId, ValidationResult, AgentInfo};
pub use signer::{Keypair, Verifier, CertificateSigner, CertificateVerifier, VerifyingKey};
pub use chain::{TrustAnchor, TrustStore, CertificateChain, ChainVerification};
pub use revocation::{RevocationList, RevokedCertificate, RevocationReason, RevocationStatus};
pub use audit::AuditLog;
pub use storage::CertificateStore;
pub use error::{CertificationError, CertificationResult};
