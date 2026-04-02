//! OWASP Top 10 Importer
//!
//! Imports security patterns from OWASP Top 10.
//! Source: https://owasp.org/Top10/

use crate::{ImportedContract, ContractSource, Severity, Importer};
use anyhow::Result;
use reqwest::Client;

pub struct OWASPImporter {
    #[allow(dead_code)]
    client: Client,
}

impl OWASPImporter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
    
    /// OWASP Top 10 (2021)
    fn top_10() -> Vec<ImportedContract> {
        vec![
            // A01:2021 - Broken Access Control
            ImportedContract {
                id: "OWASP_A01_ACCESS_CONTROL".into(),
                source: ContractSource::OWASP,
                name: "Broken Access Control".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Restrictions on authenticated users not properly enforced".into(),
                pattern: None,
                suggestion: Some("Implement proper authorization checks on every endpoint".into()),
                references: vec!["https://owasp.org/Top10/A01_2021-Broken_Access_Control/".into()],
                tags: vec!["security".into(), "access-control".into(), "owasp".into()],
            },
            // A02:2021 - Cryptographic Failures
            ImportedContract {
                id: "OWASP_A02_CRYPTO".into(),
                source: ContractSource::OWASP,
                name: "Cryptographic Failures".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Failures related to cryptography leading to sensitive data exposure".into(),
                pattern: Some("md5(".into()),
                suggestion: Some("Use strong encryption (AES-256, SHA-256+)".into()),
                references: vec!["https://owasp.org/Top10/A02_2021-Cryptographic_Failures/".into()],
                tags: vec!["security".into(), "cryptography".into(), "owasp".into()],
            },
            // A03:2021 - Injection
            ImportedContract {
                id: "OWASP_A03_INJECTION".into(),
                source: ContractSource::OWASP,
                name: "Injection".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "User-supplied data not validated, filtered, or sanitized".into(),
                pattern: Some("eval(".into()),
                suggestion: Some("Use parameterized queries, sanitize all input".into()),
                references: vec!["https://owasp.org/Top10/A03_2021-Injection/".into()],
                tags: vec!["security".into(), "injection".into(), "owasp".into()],
            },
            // A04:2021 - Insecure Design
            ImportedContract {
                id: "OWASP_A04_INSECURE_DESIGN".into(),
                source: ContractSource::OWASP,
                name: "Insecure Design".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Missing or ineffective control design".into(),
                pattern: None,
                suggestion: Some("Use threat modeling, secure design patterns".into()),
                references: vec!["https://owasp.org/Top10/A04_2021-Insecure_Design/".into()],
                tags: vec!["security".into(), "design".into(), "owasp".into()],
            },
            // A05:2021 - Security Misconfiguration
            ImportedContract {
                id: "OWASP_A05_MISCONFIG".into(),
                source: ContractSource::OWASP,
                name: "Security Misconfiguration".into(),
                domain: "security".into(),
                severity: Severity::Error,
                description: "Missing appropriate security hardening".into(),
                pattern: Some("debug = true".into()),
                suggestion: Some("Disable debug modes, remove default credentials".into()),
                references: vec!["https://owasp.org/Top10/A05_2021-Security_Misconfiguration/".into()],
                tags: vec!["security".into(), "configuration".into(), "owasp".into()],
            },
            // A06:2021 - Vulnerable Components
            ImportedContract {
                id: "OWASP_A06_VULN_COMPONENTS".into(),
                source: ContractSource::OWASP,
                name: "Vulnerable and Outdated Components".into(),
                domain: "security".into(),
                severity: Severity::Error,
                description: "Using components with known vulnerabilities".into(),
                pattern: None,
                suggestion: Some("Regularly update dependencies, use dependency scanning".into()),
                references: vec!["https://owasp.org/Top10/A06_2021-Vulnerable_and_Outdated_Components/".into()],
                tags: vec!["security".into(), "supply-chain".into(), "owasp".into()],
            },
            // A07:2021 - Authentication Failures
            ImportedContract {
                id: "OWASP_A07_AUTH_FAILURES".into(),
                source: ContractSource::OWASP,
                name: "Identification and Authentication Failures".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Confirmation of user identity not properly implemented".into(),
                pattern: None,
                suggestion: Some("Implement MFA, use secure session management".into()),
                references: vec!["https://owasp.org/Top10/A07_2021-Identification_and_Authentication_Failures/".into()],
                tags: vec!["security".into(), "authentication".into(), "owasp".into()],
            },
            // A08:2021 - Software Integrity Failures
            ImportedContract {
                id: "OWASP_A08_INTEGRITY".into(),
                source: ContractSource::OWASP,
                name: "Software and Data Integrity Failures".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Code and infrastructure not protected from integrity violations".into(),
                pattern: Some("| bash".into()),
                suggestion: Some("Verify signatures, use secure CD/CI pipelines".into()),
                references: vec!["https://owasp.org/Top10/A08_2021-Software_and_Data_Integrity_Failures/".into()],
                tags: vec!["security".into(), "integrity".into(), "owasp".into()],
            },
            // A09:2021 - Logging Failures
            ImportedContract {
                id: "OWASP_A09_LOGGING".into(),
                source: ContractSource::OWASP,
                name: "Security Logging and Monitoring Failures".into(),
                domain: "security".into(),
                severity: Severity::Error,
                description: "Insufficient logging and monitoring".into(),
                pattern: None,
                suggestion: Some("Log security events, set up alerting".into()),
                references: vec!["https://owasp.org/Top10/A09_2021-Security_Logging_and_Monitoring_Failures/".into()],
                tags: vec!["security".into(), "logging".into(), "owasp".into()],
            },
            // A10:2021 - SSRF
            ImportedContract {
                id: "OWASP_A10_SSRF".into(),
                source: ContractSource::OWASP,
                name: "Server-Side Request Forgery".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Server fetches remote resource without validating the URL".into(),
                pattern: Some("curl(".into()),
                suggestion: Some("Validate and whitelist allowed URLs".into()),
                references: vec!["https://owasp.org/Top10/A10_2021-Server-Side_Request_Forgery_SSRF/".into()],
                tags: vec!["security".into(), "ssrf".into(), "owasp".into()],
            },
        ]
    }
}

impl Default for OWASPImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Importer for OWASPImporter {
    fn source(&self) -> ContractSource {
        ContractSource::OWASP
    }
    
    fn source_url(&self) -> Option<&str> {
        Some("https://owasp.org/Top10/")
    }
    
    async fn import(&self) -> Result<Vec<ImportedContract>> {
        Ok(Self::top_10())
    }
}
