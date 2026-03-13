//! Aether Client — High-level client for validation and certification

use crate::error::{SdkError, SdkResult};
use crate::types::{
    ValidationOptions, ValidationResult,
    CertificationOptions, CertificationResult,
    AnalysisResult,
};
use aether_certification::{Certificate, Keypair};
use aether_validation::{ValidationPipeline, ValidationContext};
use aether_validation::layers::{SyntaxLayer, SemanticLayer, LogicLayer};
use aether_parsers::{RustParser, Parser, AST};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::path::Path;
use std::sync::Arc;

/// Aether client for validation and certification
pub struct AetherClient {
    /// Validation pipeline
    pipeline: Arc<ValidationPipeline>,
    /// Certificate signer (optional)
    signer: Option<Keypair>,
}

impl AetherClient {
    /// Create a new Aether client with default configuration
    pub fn new() -> Self {
        Self {
            pipeline: Arc::new(
                ValidationPipeline::new()
                    .add_layer(SyntaxLayer::new())
                    .add_layer(SemanticLayer::new())
                    .add_layer(LogicLayer::new())
            ),
            signer: None,
        }
    }

    /// Create client with custom pipeline
    pub fn with_pipeline(pipeline: ValidationPipeline) -> Self {
        Self {
            pipeline: Arc::new(pipeline),
            signer: None,
        }
    }

    /// Set up certificate signing
    pub fn with_signing(mut self, keypair: Keypair, _signer_name: String) -> Self {
        self.signer = Some(keypair);
        self
    }

    /// Generate a new keypair for signing
    pub fn generate_keypair() -> Keypair {
        Keypair::generate()
    }

    /// Validate source code (synchronous wrapper)
    pub fn validate(&self, code: &str, options: &ValidationOptions) -> SdkResult<ValidationResult> {
        // Use tokio runtime for async operation
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| SdkError::Internal(format!("Runtime error: {}", e)))?;
        rt.block_on(self.validate_async(code, options))
    }

    /// Validate source code (async)
    pub async fn validate_async(&self, code: &str, options: &ValidationOptions) -> SdkResult<ValidationResult> {
        // Parse based on language (optional - some validations don't need AST)
        let _ast = match options.language.as_str() {
            "rust" => {
                let parser = RustParser::new();
                parser.parse(code)
                    .await
                    .ok() // Don't fail on parse errors - validation can still run
            }
            "cpp" | "c++" => {
                // C++ parsing not implemented yet - validation runs on text patterns
                None
            }
            _ => {
                // Unknown language - validation runs on text patterns only
                None
            }
        };

        // Create context
        let context = ValidationContext::for_file(
            "inline".to_string(),
            code.to_string(),
            options.language.clone(),
        );

        // Run validation
        let start = std::time::Instant::now();
        let results = self.pipeline.execute(&context).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Convert results
        let mut total_violations = 0;
        let layers: Vec<crate::types::LayerResult> = results
            .results
            .into_iter()
            .map(|(name, layer_result)| {
                let violations: Vec<crate::types::Violation> = layer_result.violations
                    .iter()
                    .take(options.max_violations)
                    .map(|v| crate::types::Violation {
                        message: v.message.clone(),
                        severity: format!("{:?}", v.severity),
                        line: v.span.as_ref().map(|s| s.line),
                        column: v.span.as_ref().map(|s| s.column),
                        suggestion: if options.include_suggestions {
                            v.suggestion.clone()
                        } else {
                            None
                        },
                    })
                    .collect();
                
                total_violations += violations.len();
                
                crate::types::LayerResult {
                    name,
                    passed: layer_result.passed,
                    violations,
                }
            })
            .collect();

        Ok(ValidationResult {
            passed: total_violations == 0,
            violation_count: total_violations,
            layers,
            duration_ms,
        })
    }

    /// Validate a file
    pub fn validate_file(&self, path: &Path, options: &ValidationOptions) -> SdkResult<ValidationResult> {
        let code = std::fs::read_to_string(path)
            .map_err(|e| SdkError::InvalidInput(format!("Failed to read file: {}", e)))?;
        
        self.validate(&code, options)
    }

    /// Certify source code (validate + sign)
    pub fn certify(&self, code: &str, options: &CertificationOptions) -> SdkResult<CertificationResult> {
        // First validate
        let validation_options = ValidationOptions {
            language: options.language.clone(),
            contracts: options.contracts.clone(),
            include_suggestions: true,
            max_violations: 100,
        };
        
        let validation_result = self.validate(code, &validation_options)?;
        
        if !validation_result.passed {
            return Err(SdkError::Certification(
                "Code failed validation".to_string()
            ));
        }

        // Create certificate
        let keypair = self.signer.as_ref()
            .ok_or_else(|| SdkError::Certification("No signer configured".to_string()))?;

        let code_hash = Certificate::hash_file(code.as_bytes());
        
        let mut certificate = Certificate::new(
            code_hash.clone(),
            aether_certification::ValidationResult {
                passed: true,
                total_violations: 0,
                errors: 0,
                warnings: 0,
                duration_ms: validation_result.duration_ms,
            },
            aether_certification::AgentInfo {
                name: options.signer.clone(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        );
        
        keypair.sign_certificate(&mut certificate)
            .map_err(|e| SdkError::Certification(e.to_string()))?;
        
        let signature = certificate.signature.clone().unwrap_or_default();
        let public_key = BASE64.encode(keypair.public().as_bytes());
        
        Ok(CertificationResult {
            certificate_id: certificate.id.to_string(),
            code_hash,
            signature,
            public_key,
            validation: if options.include_validation {
                Some(validation_result)
            } else {
                None
            },
        })
    }

    /// Certify a file
    pub fn certify_file(&self, path: &Path, options: &CertificationOptions) -> SdkResult<CertificationResult> {
        let code = std::fs::read_to_string(path)
            .map_err(|e| SdkError::InvalidInput(format!("Failed to read file: {}", e)))?;
        
        self.certify(&code, options)
    }

    /// Analyze source code (AST stats) - synchronous wrapper
    pub fn analyze(&self, code: &str, language: &str) -> SdkResult<AnalysisResult> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| SdkError::Internal(format!("Runtime error: {}", e)))?;
        rt.block_on(self.analyze_async(code, language))
    }

    /// Analyze source code (async)
    pub async fn analyze_async(&self, code: &str, language: &str) -> SdkResult<AnalysisResult> {
        let ast = match language {
            "rust" => {
                let parser = RustParser::new();
                parser.parse(code)
                    .await
                    .map_err(|e| SdkError::Internal(format!("Parse error: {}", e)))?
            }
            lang => return Err(SdkError::InvalidInput(format!("Unsupported language: {}", lang))),
        };

        // Count AST nodes by traversing the tree
        let stats = count_ast_nodes(&ast, code.lines().count());

        Ok(AnalysisResult {
            stats,
            prompt_analysis: None, // TODO: Integrate with PromptAnalyzer
        })
    }
}

/// Count AST nodes by traversing the tree
fn count_ast_nodes(ast: &AST, total_lines: usize) -> crate::types::AstStats {
    let mut stats = crate::types::AstStats {
        functions: 0,
        structs: 0,
        enums: 0,
        traits: 0,
        modules: 0,
        total_lines,
    };
    
    count_nodes(&ast.root, &mut stats);
    stats
}

fn count_nodes(node: &aether_parsers::ASTNode, stats: &mut crate::types::AstStats) {
    use aether_parsers::NodeKind;
    
    match node.kind {
        NodeKind::Function => stats.functions += 1,
        NodeKind::Struct => stats.structs += 1,
        NodeKind::Enum => stats.enums += 1,
        NodeKind::Trait => stats.traits += 1,
        NodeKind::Module => stats.modules += 1,
        _ => {}
    }
    
    for child in &node.children {
        count_nodes(child, stats);
    }
}

impl Default for AetherClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = AetherClient::new();
        assert!(client.signer.is_none());
    }

    #[test]
    fn test_client_with_signer() {
        let keypair = AetherClient::generate_keypair();
        let client = AetherClient::new()
            .with_signing(keypair, "test".to_string());
        
        assert!(client.signer.is_some());
    }

    #[test]
    fn test_validate_simple_code() {
        let client = AetherClient::new();
        let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        let options = ValidationOptions::default();
        let result = client.validate(code, &options);
        
        // Should succeed parsing
        assert!(result.is_ok());
    }
}
