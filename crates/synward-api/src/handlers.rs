//! HTTP handlers for API endpoints

use crate::auth::ApiKey;
use crate::error::ApiResult;
use crate::state::AppState;
use axum::{
    extract::{Extension, Json, State},
    response::IntoResponse,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use synward_validation::ValidationContext;
use synward_parsers::{RustParser, Parser};
use synward_certification::Certificate;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Validate request
#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    /// Source code to validate
    pub code: String,
    /// Language (rust, lex, etc.)
    pub language: String,
    /// Contracts to check (optional)
    #[serde(default)]
    #[allow(dead_code)]
    pub contracts: Vec<String>,
}

/// Validate response
#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    /// Whether validation passed
    pub passed: bool,
    /// Validation results
    pub results: Vec<LayerResult>,
    /// Total violations
    pub violation_count: usize,
    /// Processing time in ms
    pub duration_ms: u64,
}

/// Layer result
#[derive(Debug, Serialize)]
pub struct LayerResult {
    pub layer: String,
    pub passed: bool,
    pub violations: Vec<ViolationInfo>,
}

/// Violation info
#[derive(Debug, Serialize)]
pub struct ViolationInfo {
    pub message: String,
    pub severity: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub suggestion: Option<String>,
}

/// Certify request
#[derive(Debug, Deserialize)]
pub struct CertifyRequest {
    /// Source code to certify
    pub code: String,
    /// Language
    pub language: String,
    /// Signer name
    pub signer: String,
    /// Contracts to enforce
    #[serde(default)]
    #[allow(dead_code)]
    pub contracts: Vec<String>,
}

/// Certify response
#[derive(Debug, Serialize)]
pub struct CertifyResponse {
    /// Certificate ID
    pub certificate_id: String,
    /// Code hash
    pub code_hash: String,
    /// Signature (base64)
    pub signature: String,
    /// Signer public key
    pub public_key: String,
    /// Validation results
    pub validation: ValidateResponse,
}

/// Verify request
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    /// Certificate ID
    #[serde(default)]
    pub certificate_id: Option<String>,
    /// Certificate data (for inline verification)
    #[serde(default)]
    #[allow(dead_code)]
    pub certificate: Option<serde_json::Value>,
}

/// Verify response
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    /// Whether verification passed
    pub valid: bool,
    /// Certificate ID
    pub certificate_id: String,
    /// Signer public key
    pub public_key: String,
    /// Verification details
    pub details: String,
}

/// Analyze request
#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    /// Source code to analyze
    pub code: String,
    /// Language
    pub language: String,
}

/// Analyze response
#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    /// AST statistics
    pub stats: AstStats,
    /// Prompt analysis (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_analysis: Option<PromptAnalysisResult>,
}

/// AST statistics
#[derive(Debug, Serialize)]
pub struct AstStats {
    pub functions: usize,
    pub structs: usize,
    pub enums: usize,
    pub traits: usize,
    pub modules: usize,
    pub total_lines: usize,
}

/// Prompt analysis result
#[derive(Debug, Serialize)]
pub struct PromptAnalysisResult {
    pub intent: String,
    pub confidence: f32,
    pub scope: String,
    pub domains: Vec<String>,
    pub ambiguities: Vec<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/v1/validate
pub async fn validate(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<Arc<ApiKey>>,
    Json(request): Json<ValidateRequest>,
) -> ApiResult<Json<ValidateResponse>> {
    let start = std::time::Instant::now();

    // Parse code based on language
    let _ast = match request.language.as_str() {
        "rust" => {
            let parser = RustParser::new();
            parser.parse(&request.code)
                .await
                .map_err(|e| crate::error::ApiError::BadRequest(format!("Parse error: {}", e)))?
        }
        lang => return Err(crate::error::ApiError::BadRequest(format!("Unsupported language: {}", lang))),
    };

    // Create validation context
    let context = ValidationContext::for_file(
        "inline".to_string(),
        request.code.clone(),
        request.language.clone(),
    );

    // Run validation pipeline
    let results = state.pipeline.execute(&context).await;

    // Convert results
    let mut total_violations = 0;
    let layers: Vec<LayerResult> = results
        .results
        .into_iter()
        .map(|(name, layer_result)| {
            let violations: Vec<ViolationInfo> = layer_result.violations
                .iter()
                .map(|v| ViolationInfo {
                    message: v.message.clone(),
                    severity: format!("{:?}", v.severity),
                    line: v.span.as_ref().map(|s| s.line),
                    column: v.span.as_ref().map(|s| s.column),
                    suggestion: v.suggestion.clone(),
                })
                .collect();
            
            total_violations += violations.len();
            
            LayerResult {
                layer: name,
                passed: layer_result.violations.is_empty(),
                violations,
            }
        })
        .collect();

    let response = ValidateResponse {
        passed: total_violations == 0,
        results: layers,
        violation_count: total_violations,
        duration_ms: start.elapsed().as_millis() as u64,
    };

    Ok(Json(response))
}

/// POST /api/v1/certify
pub async fn certify(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<Arc<ApiKey>>,
    Json(request): Json<CertifyRequest>,
) -> ApiResult<Json<CertifyResponse>> {
    let start = std::time::Instant::now();

    // First validate
    let context = ValidationContext::for_file(
        "inline".to_string(),
        request.code.clone(),
        request.language.clone(),
    );

    let results = state.pipeline.execute(&context).await;
    
    if !results.all_passed() {
        let total_violations = results.total_violations();
        return Err(crate::error::ApiError::BadRequest(
            format!("Validation failed with {} violations", total_violations)
        ));
    }

    // Create certificate
    let keypair = state.keypair.as_ref()
        .ok_or_else(|| crate::error::ApiError::Internal("No signing key configured".to_string()))?;

    let code_hash = Certificate::hash_file(request.code.as_bytes());
    
    let mut certificate = Certificate::new(
        code_hash.clone(),
        synward_certification::ValidationResult {
            passed: true,
            total_violations: 0,
            errors: 0,
            warnings: 0,
            duration_ms: start.elapsed().as_millis() as u64,
        },
        synward_certification::AgentInfo {
            name: request.signer.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    );
    
    keypair.sign_certificate(&mut certificate)
        .map_err(|e| crate::error::ApiError::Internal(e.to_string()))?;
    
    let signature = certificate.signature.clone().unwrap_or_default();
    let public_key = BASE64.encode(keypair.public().as_bytes());

    // Build layer results
    let mut total_violations = 0;
    let layers: Vec<LayerResult> = results
        .results
        .into_iter()
        .map(|(name, layer_result)| {
            let violations: Vec<ViolationInfo> = layer_result.violations
                .iter()
                .map(|v| ViolationInfo {
                    message: v.message.clone(),
                    severity: format!("{:?}", v.severity),
                    line: v.span.as_ref().map(|s| s.line),
                    column: v.span.as_ref().map(|s| s.column),
                    suggestion: v.suggestion.clone(),
                })
                .collect();
            
            total_violations += violations.len();
            
            LayerResult {
                layer: name,
                passed: layer_result.violations.is_empty(),
                violations,
            }
        })
        .collect();
    
    let response = CertifyResponse {
        certificate_id: certificate.id.to_string(),
        code_hash,
        signature,
        public_key,
        validation: ValidateResponse {
            passed: true,
            results: layers,
            violation_count: total_violations,
            duration_ms: start.elapsed().as_millis() as u64,
        },
    };

    Ok(Json(response))
}

/// POST /api/v1/verify
pub async fn verify(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<Arc<ApiKey>>,
    Json(request): Json<VerifyRequest>,
) -> ApiResult<Json<VerifyResponse>> {
    // Get certificate from request
    let certificate_id = request.certificate_id.unwrap_or_else(|| "unknown".to_string());
    
    // TODO: Load certificate from store by ID
    // For now, return basic verification status
    let keypair = state.keypair.as_ref();
    
    let response = VerifyResponse {
        valid: keypair.is_some(),
        certificate_id,
        public_key: keypair
            .map(|k| BASE64.encode(k.public().as_bytes()))
            .unwrap_or_else(|| "no_key_configured".to_string()),
        details: if keypair.is_some() {
            "Server has signing capability".to_string()
        } else {
            "No signing key configured".to_string()
        },
    };

    Ok(Json(response))
}

/// POST /api/v1/analyze
pub async fn analyze(
    State(_state): State<Arc<AppState>>,
    Extension(_auth): Extension<Arc<ApiKey>>,
    Json(request): Json<AnalyzeRequest>,
) -> ApiResult<Json<AnalyzeResponse>> {
    // Parse code based on language
    let ast = match request.language.as_str() {
        "rust" => {
            let parser = RustParser::new();
            parser.parse(&request.code)
                .await
                .map_err(|e| crate::error::ApiError::BadRequest(format!("Parse error: {}", e)))?
        }
        lang => return Err(crate::error::ApiError::BadRequest(format!("Unsupported language: {}", lang))),
    };

    // Count AST nodes
    let stats = count_ast_nodes(&ast, request.code.lines().count());

    let response = AnalyzeResponse {
        stats,
        prompt_analysis: None, // TODO: Integrate with PromptAnalyzer
    };

    Ok(Json(response))
}

/// Count AST nodes by traversing the tree
fn count_ast_nodes(ast: &synward_parsers::AST, total_lines: usize) -> AstStats {
    let mut stats = AstStats {
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

fn count_nodes(node: &synward_parsers::ASTNode, stats: &mut AstStats) {
    use synward_parsers::NodeKind;
    
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

/// GET /api/v1/health
pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": crate::API_VERSION,
    }))
}

/// GET /api/v1/keys
pub async fn list_keys() -> ApiResult<Json<Vec<ApiKey>>> {
    // TODO: Implement actual key listing
    Ok(Json(vec![]))
}
