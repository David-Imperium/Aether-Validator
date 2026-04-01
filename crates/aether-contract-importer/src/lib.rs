//! Aether Contract Importer
//!
//! Import validation contracts from human-authored sources:
//! - Clippy lints (Rust)
//! - ESLint rules (JavaScript/TypeScript)
//! - Pylint checks (Python)
//! - CWE/OWASP patterns (Security)
//! - Official style guides

pub mod clippy;
pub mod eslint;
pub mod pylint;
pub mod cwe;
pub mod owasp;
pub mod styleguide;
pub mod merger;
pub mod output;

use serde::{Deserialize, Serialize};

/// Imported contract ready for conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedContract {
    pub id: String,
    pub source: ContractSource,
    pub name: String,
    pub domain: String,
    pub severity: Severity,
    pub description: String,
    pub pattern: Option<String>,
    pub suggestion: Option<String>,
    pub references: Vec<String>,
    pub tags: Vec<String>,
}

/// Source of the contract
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContractSource {
    Clippy,
    ESLint,
    Pylint,
    CWE,
    OWASP,
    StyleGuide(String), // "rust-api-guidelines", "google-style", etc.
    Manual,
}

/// Severity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Severity {
    Critical,
    Error,
    Warning,
    Info,
    Hint,
}

/// Importer trait for all sources
#[async_trait::async_trait]
pub trait Importer {
    /// Source name
    fn source(&self) -> ContractSource;
    
    /// Import all contracts from source
    async fn import(&self) -> anyhow::Result<Vec<ImportedContract>>;
    
    /// Source URL for fetching data
    fn source_url(&self) -> Option<&str>;
}

/// Import options
#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub include_deprecated: bool,
    pub min_severity: Severity,
    pub languages: Vec<String>,
    pub domains: Vec<String>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            include_deprecated: false,
            min_severity: Severity::Info,
            languages: vec!["rust".into(), "python".into(), "javascript".into()],
            domains: vec!["security".into(), "correctness".into(), "performance".into()],
        }
    }
}

/// Run all importers and merge results
pub async fn import_all(options: ImportOptions) -> anyhow::Result<Vec<ImportedContract>> {
    let mut all_contracts = Vec::new();
    
    // Create importers first
    let clippy_importer = clippy::ClippyImporter::new();
    let eslint_importer = eslint::ESLintImporter::new();
    let pylint_importer = pylint::PylintImporter::new();
    let cwe_importer = cwe::CWEImporter::new();
    let owasp_importer = owasp::OWASPImporter::new();
    
    // Import from each source in parallel
    let (clippy, eslint, pylint, cwe, owasp) = tokio::join!(
        clippy_importer.import(),
        eslint_importer.import(),
        pylint_importer.import(),
        cwe_importer.import(),
        owasp_importer.import(),
    );
    
    if let Ok(c) = clippy { all_contracts.extend(c); }
    if let Ok(c) = eslint { all_contracts.extend(c); }
    if let Ok(c) = pylint { all_contracts.extend(c); }
    if let Ok(c) = cwe { all_contracts.extend(c); }
    if let Ok(c) = owasp { all_contracts.extend(c); }
    
    // Filter by options
    all_contracts.retain(|c| {
        c.severity <= options.min_severity
    });
    
    // Deduplicate
    all_contracts = merger::deduplicate(all_contracts);
    
    Ok(all_contracts)
}
