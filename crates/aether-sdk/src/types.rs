//! SDK Types — Request/Response types for the SDK

use serde::{Deserialize, Serialize};

/// Validation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOptions {
    /// Language of the source code
    pub language: String,
    /// Contracts to enforce (empty = all)
    #[serde(default)]
    pub contracts: Vec<String>,
    /// Include suggestions in results
    #[serde(default = "default_true")]
    pub include_suggestions: bool,
    /// Maximum violations to report
    #[serde(default = "default_max_violations")]
    pub max_violations: usize,
}

fn default_true() -> bool { true }
fn default_max_violations() -> usize { 100 }

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            language: "rust".to_string(),
            contracts: vec![],
            include_suggestions: true,
            max_violations: 100,
        }
    }
}

/// Certification options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationOptions {
    /// Language of the source code
    pub language: String,
    /// Signer name/identifier
    pub signer: String,
    /// Contracts to enforce
    #[serde(default)]
    pub contracts: Vec<String>,
    /// Include full validation results
    #[serde(default = "default_true")]
    pub include_validation: bool,
}

impl Default for CertificationOptions {
    fn default() -> Self {
        Self {
            language: "rust".to_string(),
            signer: "default".to_string(),
            contracts: vec![],
            include_validation: true,
        }
    }
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Total violations found
    pub violation_count: usize,
    /// Violations by layer
    pub layers: Vec<LayerResult>,
    /// Processing time in milliseconds
    pub duration_ms: u64,
}

/// Layer result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerResult {
    /// Layer name
    pub name: String,
    /// Whether layer passed
    pub passed: bool,
    /// Violations in this layer
    pub violations: Vec<Violation>,
}

/// Violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// Violation message
    pub message: String,
    /// Severity level
    pub severity: String,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Column number (if applicable)
    pub column: Option<usize>,
    /// Suggestion for fixing
    pub suggestion: Option<String>,
}

/// Certification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationResult {
    /// Certificate ID
    pub certificate_id: String,
    /// Code hash
    pub code_hash: String,
    /// Signature (base64)
    pub signature: String,
    /// Signer public key
    pub public_key: String,
    /// Validation result (if requested)
    pub validation: Option<ValidationResult>,
}

/// Analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// AST statistics
    pub stats: AstStats,
    /// Prompt analysis (if available)
    pub prompt_analysis: Option<PromptAnalysis>,
}

/// AST statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstStats {
    pub functions: usize,
    pub structs: usize,
    pub enums: usize,
    pub traits: usize,
    pub modules: usize,
    pub total_lines: usize,
}

/// Prompt analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAnalysis {
    pub intent: String,
    pub confidence: f32,
    pub scope: String,
    pub domains: Vec<String>,
    pub ambiguities: Vec<String>,
}
