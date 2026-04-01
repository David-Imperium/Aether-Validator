//! CWE (Common Weakness Enumeration) Importer
//!
//! Imports security patterns from MITRE CWE database.
//! Source: https://cwe.mitre.org/

use crate::{ImportedContract, ContractSource, Severity, Importer};
use anyhow::Result;
use reqwest::Client;

pub struct CWEImporter {
    #[allow(dead_code)]
    client: Client,
}

impl CWEImporter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
    
    /// Top CWE patterns (most common vulnerabilities)
    fn top_cwes() -> Vec<ImportedContract> {
        vec![
            // OWASP Top 10 mapped to CWE
            ImportedContract {
                id: "CWE_079".into(),
                source: ContractSource::CWE,
                name: "Cross-site Scripting (XSS)".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Improper neutralization of input during web page generation".into(),
                pattern: Some("innerHTML".into()),
                suggestion: Some("Sanitize/escape user input before rendering".into()),
                references: vec!["https://cwe.mitre.org/data/definitions/79.html".into()],
                tags: vec!["security".into(), "xss".into(), "web".into()],
            },
            ImportedContract {
                id: "CWE_089".into(),
                source: ContractSource::CWE,
                name: "SQL Injection".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Improper neutralization of SQL commands".into(),
                pattern: Some(r#"format!.*SELECT"#.into()),
                suggestion: Some("Use parameterized queries".into()),
                references: vec!["https://cwe.mitre.org/data/definitions/89.html".into()],
                tags: vec!["security".into(), "sql-injection".into(), "database".into()],
            },
            ImportedContract {
                id: "CWE_078".into(),
                source: ContractSource::CWE,
                name: "OS Command Injection".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Improper neutralization of OS commands".into(),
                pattern: Some("os.system(".into()),
                suggestion: Some("Use subprocess with list args, never shell=True".into()),
                references: vec!["https://cwe.mitre.org/data/definitions/78.html".into()],
                tags: vec!["security".into(), "command-injection".into()],
            },
            ImportedContract {
                id: "CWE_022".into(),
                source: ContractSource::CWE,
                name: "Path Traversal".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Improper limitation of pathname to restricted directory".into(),
                pattern: Some("..".into()),
                suggestion: Some("Validate and sanitize file paths".into()),
                references: vec!["https://cwe.mitre.org/data/definitions/22.html".into()],
                tags: vec!["security".into(), "path-traversal".into()],
            },
            ImportedContract {
                id: "CWE_094".into(),
                source: ContractSource::CWE,
                name: "Code Injection".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Improper control of code generation (eval/exec)".into(),
                pattern: Some("eval(".into()),
                suggestion: Some("Never use eval/exec with user input".into()),
                references: vec!["https://cwe.mitre.org/data/definitions/94.html".into()],
                tags: vec!["security".into(), "code-injection".into()],
            },
            ImportedContract {
                id: "CWE_200".into(),
                source: ContractSource::CWE,
                name: "Information Exposure".into(),
                domain: "security".into(),
                severity: Severity::Warning,
                description: "Exposure of sensitive information".into(),
                pattern: Some("password".into()),
                suggestion: Some("Never log/echo sensitive data".into()),
                references: vec!["https://cwe.mitre.org/data/definitions/200.html".into()],
                tags: vec!["security".into(), "information-disclosure".into()],
            },
            ImportedContract {
                id: "CWE_311".into(),
                source: ContractSource::CWE,
                name: "Missing Encryption".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Missing encryption of sensitive data".into(),
                pattern: Some("password = \"".into()),
                suggestion: Some("Encrypt sensitive data at rest and in transit".into()),
                references: vec!["https://cwe.mitre.org/data/definitions/311.html".into()],
                tags: vec!["security".into(), "encryption".into()],
            },
            ImportedContract {
                id: "CWE_798".into(),
                source: ContractSource::CWE,
                name: "Hardcoded Credentials".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Use of hardcoded credentials".into(),
                pattern: Some(r#"password\s*=\s*""#.into()),
                suggestion: Some("Use environment variables or secrets manager".into()),
                references: vec!["https://cwe.mitre.org/data/definitions/798.html".into()],
                tags: vec!["security".into(), "credentials".into()],
            },
            ImportedContract {
                id: "CWE_125".into(),
                source: ContractSource::CWE,
                name: "Buffer Overflow".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Out-of-bounds read/write".into(),
                pattern: None,
                suggestion: Some("Use bounds-checked containers".into()),
                references: vec!["https://cwe.mitre.org/data/definitions/125.html".into()],
                tags: vec!["security".into(), "memory".into(), "buffer-overflow".into()],
            },
            ImportedContract {
                id: "CWE_416".into(),
                source: ContractSource::CWE,
                name: "Use After Free".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Dereferencing freed memory".into(),
                pattern: None,
                suggestion: Some("Use ownership systems or garbage collection".into()),
                references: vec!["https://cwe.mitre.org/data/definitions/416.html".into()],
                tags: vec!["security".into(), "memory".into(), "use-after-free".into()],
            },
        ]
    }
}

impl Default for CWEImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Importer for CWEImporter {
    fn source(&self) -> ContractSource {
        ContractSource::CWE
    }
    
    fn source_url(&self) -> Option<&str> {
        Some("https://cwe.mitre.org/")
    }
    
    async fn import(&self) -> Result<Vec<ImportedContract>> {
        // Use top CWEs for now
        Ok(Self::top_cwes())
    }
}
