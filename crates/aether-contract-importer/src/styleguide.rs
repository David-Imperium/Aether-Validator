//! Style Guide Importer
//!
//! Imports rules from official style guides:
//! - Rust API Guidelines
//! - Google Style Guides (C++, Python, JavaScript)
//! - PEP 8 (Python)

use crate::{ImportedContract, ContractSource, Severity, Importer};
use anyhow::Result;

pub struct StyleGuideImporter {
    guide: String,
}

impl StyleGuideImporter {
    pub fn new(guide: &str) -> Self {
        Self {
            guide: guide.to_string(),
        }
    }
    
    /// Rust API Guidelines
    fn rust_api_guidelines() -> Vec<ImportedContract> {
        vec![
            ImportedContract {
                id: "RUST_API_NAMING".into(),
                source: ContractSource::StyleGuide("rust-api-guidelines".into()),
                name: "Naming Conventions".into(),
                domain: "style".into(),
                severity: Severity::Info,
                description: "Types use PascalCase, functions/vars use snake_case".into(),
                pattern: None,
                suggestion: Some("Follow Rust naming conventions".into()),
                references: vec!["https://rust-lang.github.io/api-guidelines/naming.html".into()],
                tags: vec!["rust".into(), "naming".into(), "style".into()],
            },
            ImportedContract {
                id: "RUST_API_DEBUG".into(),
                source: ContractSource::StyleGuide("rust-api-guidelines".into()),
                name: "Debug for Public Types".into(),
                domain: "style".into(),
                severity: Severity::Info,
                description: "Public types should implement Debug".into(),
                pattern: None,
                suggestion: Some("Derive or implement Debug for all public types".into()),
                references: vec!["https://rust-lang.github.io/api-guidelines/debuggability.html".into()],
                tags: vec!["rust".into(), "debug".into(), "style".into()],
            },
        ]
    }
    
    /// Google Style Guide rules
    fn google_style() -> Vec<ImportedContract> {
        vec![
            ImportedContract {
                id: "GOOGLE_LINE_LENGTH".into(),
                source: ContractSource::StyleGuide("google-style".into()),
                name: "Line Length Limit".into(),
                domain: "style".into(),
                severity: Severity::Info,
                description: "Lines should not exceed 80-120 characters".into(),
                pattern: None,
                suggestion: Some("Break long lines for readability".into()),
                references: vec!["https://google.github.io/styleguide/".into()],
                tags: vec!["style".into(), "formatting".into()],
            },
        ]
    }
    
    /// PEP 8 rules
    fn pep8() -> Vec<ImportedContract> {
        vec![
            ImportedContract {
                id: "PEP8_IMPORTS".into(),
                source: ContractSource::StyleGuide("pep8".into()),
                name: "Import Organization".into(),
                domain: "style".into(),
                severity: Severity::Info,
                description: "Imports should be grouped: stdlib, third-party, local".into(),
                pattern: None,
                suggestion: Some("Organize imports in three groups".into()),
                references: vec!["https://peps.python.org/pep-0008/#imports".into()],
                tags: vec!["python".into(), "style".into(), "imports".into()],
            },
            ImportedContract {
                id: "PEP8_NAMING".into(),
                source: ContractSource::StyleGuide("pep8".into()),
                name: "Naming Conventions".into(),
                domain: "style".into(),
                severity: Severity::Info,
                description: "snake_case for functions/vars, PascalCase for classes".into(),
                pattern: None,
                suggestion: Some("Follow PEP 8 naming conventions".into()),
                references: vec!["https://peps.python.org/pep-0008/#naming-conventions".into()],
                tags: vec!["python".into(), "style".into(), "naming".into()],
            },
        ]
    }
}

#[async_trait::async_trait]
impl Importer for StyleGuideImporter {
    fn source(&self) -> ContractSource {
        ContractSource::StyleGuide(self.guide.clone())
    }
    
    fn source_url(&self) -> Option<&str> {
        match self.guide.as_str() {
            "rust-api-guidelines" => Some("https://rust-lang.github.io/api-guidelines/"),
            "google-style" => Some("https://google.github.io/styleguide/"),
            "pep8" => Some("https://peps.python.org/pep-0008/"),
            _ => None,
        }
    }
    
    async fn import(&self) -> Result<Vec<ImportedContract>> {
        let contracts = match self.guide.as_str() {
            "rust-api-guidelines" => Self::rust_api_guidelines(),
            "google-style" => Self::google_style(),
            "pep8" => Self::pep8(),
            _ => vec![],
        };
        Ok(contracts)
    }
}
