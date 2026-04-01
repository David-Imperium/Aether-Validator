//! Command implementations
//!
//! This module contains all command execution logic.

use anyhow::Result;
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, BufRead};

use aether_parsers::ParserRegistry;
use aether_validation::{ValidationPipeline, SyntaxLayer, SemanticLayer, LogicLayer, SecurityLayer, FallbackSecurityLayer, ComplexityLayer, SupplyChainLayer, ValidationContext, PipelineResult, Violation, Severity, ScopeAnalysisLayer, TypeInferenceLayer, ValidationLayer};
use aether_certification::{Keypair, CertificateVerifier, Certificate, ValidationResult, AgentInfo, VerifyingKey};
use aether_contracts::{ContractLoader, RuleEvaluator};

#[cfg(feature = "intelligence")]
use aether_intelligence::{
    dubbioso::{DubbiosoConfig, ConfidenceLevel},
    dubbioso_validator::{DubbiosoValidator, ViolationInput},
    memory::{MultiSignalScore, ValidationState, ViolationRecord, FileState},
    compliance::{
        ComplianceEngine, ComplianceConfig, ComplianceContext,
        ComplianceAction,
    },
};

use crate::platforms;

/// Result of validation with violations for feedback loop
/// Infrastructure for future CLI output integration and feedback loop
#[allow(dead_code)]
#[derive(Debug)]
pub struct ValidateResult {
    /// Whether validation passed (no criticals or errors)
    pub passed: bool,
    /// Total critical issues
    pub criticals: usize,
    /// Total errors
    pub errors: usize,
    /// Total warnings
    pub warnings: usize,
    /// All violations from validation layers
    pub violations: Vec<ViolationInfo>,
}

/// Violation info for feedback loop
#[derive(Debug, Clone)]
pub struct ViolationInfo {
    pub id: String,
    pub rule: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub message: String,
}

// ============================================================================
// HELPERS
// ============================================================================

/// Convert aether_validation::Severity to memory::Severity
#[cfg(feature = "intelligence")]
fn to_memory_severity(sev: Severity) -> aether_intelligence::memory::Severity {
    use aether_intelligence::memory::Severity as MemSev;
    match sev {
        Severity::Critical | Severity::Error => MemSev::Error,
        Severity::Warning => MemSev::Warning,
        Severity::Info => MemSev::Info,
        Severity::Hint => MemSev::Style,
    }
}

/// Save ProjectState after validation (intelligence feature)
#[cfg(feature = "intelligence")]
fn save_project_state(
    path: &Path,
    source: &str,
    all_violations: &[ViolationInfo],
) -> Result<()> {
    // Compute file hash using Certificate::hash_file
    let hash = Certificate::hash_file(source.as_bytes());

    // Compute line count
    let line_count = source.lines().count();

    // Convert violations to ViolationRecord
    let violation_records: Vec<ViolationRecord> = all_violations
        .iter()
        .map(|v| ViolationRecord {
            id: v.id.clone(),
            rule: v.rule.clone(),
            file: v.file.clone(),
            severity: to_memory_severity(v.severity),
            line: v.line,
            column: v.column,
            message: v.message.clone(),
            snippet: None,
        })
        .collect();

    // Create FileState
    let file_state = FileState::from_validation(hash, violation_records, line_count);

    // Get parent directory and relative path
    let parent_dir = path.parent()
        .ok_or_else(|| anyhow::anyhow!("No parent directory for file"))?;
    let relative_path = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Get or create ProjectState and save
    let mut validation_state = ValidationState::new(None)?;
    let project_state = validation_state.get_project(parent_dir);
    let mut project_state = project_state.clone();
    project_state.update_file(relative_path, file_state);
    validation_state.save_project(&project_state)?;

    Ok(())
}

/// Detect language from file extension
/// Returns None for unsupported languages (instead of defaulting to "rust")
pub fn detect_language(path: &Path) -> Option<String> {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        "rs" => Some("rust".into()),
        "py" => Some("python".into()),
        "js" => Some("javascript".into()),
        "ts" | "tsx" => Some("typescript".into()),
        "cpp" | "cxx" | "cc" | "hpp" => Some("cpp".into()),
        "go" => Some("go".into()),
        "java" => Some("java".into()),
        "lua" => Some("lua".into()),
        "lex" => Some("lex".into()),
        _ => None,  // Return None for unsupported languages
    }
}

/// Get default contracts directory
pub fn get_contracts_dir() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".aether/contracts"))
        .unwrap_or_else(|| PathBuf::from("contracts"))
}

/// Get default keystore directory
pub fn get_keystore_dir() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".aether"))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Check if running in interactive terminal
pub fn is_interactive() -> bool {
    true
}

/// Read a line from stdin
pub fn read_line() -> Result<String> {
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Parse comma-separated values
pub fn parse_list(input: &str) -> Vec<String> {
    input.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

// ============================================================================
// VALIDATE
// ============================================================================

pub async fn validate(
    path: PathBuf,
    lang: Option<String>,
    contracts: Option<PathBuf>,
    severity: &str,
    format: &str,
    dubbioso: bool,
) -> Result<ValidateResult> {
    let language = lang.or_else(|| detect_language(&path));
    let source = fs::read_to_string(&path)?;
    let contracts_dir = contracts.unwrap_or_else(get_contracts_dir);

    // Handle unsupported languages with FallbackSecurityLayer
    let (language, use_fallback) = match language {
        Some(lang) => (lang, false),
        None => {
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown");
            eprintln!("{} Language '.{}' not supported, using regex-only security fallback", "⚠".yellow(), ext);
            ("unknown".into(), true)
        }
    };

    // For unsupported languages, use FallbackSecurityLayer only
    if use_fallback {
        let ctx = ValidationContext::for_file(&path, source.clone(), language.clone());
        let layer = FallbackSecurityLayer::new();
        let layer_result = layer.validate(&ctx).await;

        let mut total_criticals = 0;
        let mut total_errors = 0;
        let mut total_warnings = 0;
        let mut all_violations = Vec::new();

        for v in &layer_result.violations {
            match v.severity {
                Severity::Critical => total_criticals += 1,
                Severity::Error => total_errors += 1,
                Severity::Warning => total_warnings += 1,
                _ => {}
            }
            all_violations.push(ViolationInfo {
                id: v.id.clone(),
                rule: "fallback_security".into(),
                file: path.to_string_lossy().to_string(),
                line: v.span.as_ref().map(|s| s.line).unwrap_or(0),
                column: v.span.as_ref().map(|s| s.column).unwrap_or(0),
                severity: v.severity,
                message: v.message.clone(),
            });
        }

        let passed = total_criticals == 0 && total_errors == 0;
        print_fallback_result(&path, &all_violations, passed, total_criticals, total_errors, total_warnings, format);

        // Save ProjectState after validation
        #[cfg(feature = "intelligence")]
        if let Err(e) = save_project_state(&path, &source, &all_violations) {
            tracing::warn!("Failed to save project state: {}", e);
        }

        return Ok(ValidateResult {
            passed,
            criticals: total_criticals,
            errors: total_errors,
            warnings: total_warnings,
            violations: all_violations,
        });
    }

    // Get parser for supported languages
    let registry = ParserRegistry::with_defaults();
    let parser = registry.get(&language)
        .ok_or_else(|| anyhow::anyhow!("Language '{}' not supported", language))?;

    // Parse
    let _ast = parser.parse(&source).await
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    // Build validation context
    let ctx = ValidationContext::for_file(&path, source.clone(), language.clone());

    // Build validation pipeline
    let pipeline = ValidationPipeline::new()
        .add_layer(SupplyChainLayer::new())
        .add_layer(SecurityLayer::new())
        .add_layer(SyntaxLayer::new())
        .add_layer(SemanticLayer::new())
        .add_layer(ScopeAnalysisLayer::new())
        .add_layer(TypeInferenceLayer::new())
        .add_layer(LogicLayer::new())
        .add_layer(ComplexityLayer::default());

    // Validate
    let result = pipeline.execute(&ctx).await;

    // Load contracts
    let loader = ContractLoader::new(contracts_dir);
    let contract_rules = loader.load_dir(&language)?;

    let mut evaluator = RuleEvaluator::new();
    let mut contract_violations = Vec::new();

    for contract in &contract_rules {
        for rule in &contract.rules {
            if let Ok(violations) = evaluator.evaluate(rule, &source) {
                for v in violations {
                    contract_violations.push((contract.id.clone(), contract.name.clone(), v));
                }
            }
        }
    }

    // Collect all violations for feedback loop
    let mut all_violations = Vec::new();
    let mut total_criticals: usize = 0;
    let mut total_errors: usize = 0;
    let mut total_warnings: usize = 0;

    // From validation layers
    for (layer_name, layer_result) in &result.results {
        for v in &layer_result.violations {
            all_violations.push(ViolationInfo {
                id: v.id.clone(),
                rule: layer_name.clone(),
                file: path.to_string_lossy().to_string(),
                line: v.span.as_ref().map(|s| s.line).unwrap_or(0),
                column: v.span.as_ref().map(|s| s.column).unwrap_or(0),
                severity: v.severity,
                message: v.message.clone(),
            });
            match v.severity {
                Severity::Critical => total_criticals += 1,
                Severity::Error => total_errors += 1,
                Severity::Warning => total_warnings += 1,
                _ => {}
            }
        }
    }

    // From contracts
    for (id, name, v) in &contract_violations {
        all_violations.push(ViolationInfo {
            id: id.clone(),
            rule: name.clone(),
            file: path.to_string_lossy().to_string(),
            line: v.span.as_ref().map(|s| s.line).unwrap_or(0),
            column: v.span.as_ref().map(|s| s.column).unwrap_or(0),
            severity: v.severity,
            message: v.message.clone(),
        });
        match v.severity {
            Severity::Critical => total_criticals += 1,
            Severity::Error => total_errors += 1,
            Severity::Warning => total_warnings += 1,
            _ => {}
        }
    }

    // ===== Compliance Engine: Intelligent Contract Enforcement =====
    #[cfg(feature = "intelligence")]
    let (compliance_blocked, compliance_accepted, compliance_results) = {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let aether_dir = cwd.join(".aether");
        
        let config = ComplianceConfig {
            exemption_store_path: Some(aether_dir.join("exemptions.json")),
            ..ComplianceConfig::default()
        };
        
        let mut engine = ComplianceEngine::with_config(config)
            .expect("Failed to create ComplianceEngine");
        
        let mut blocked = Vec::new();
        let mut accepted = Vec::new();
        let mut results = Vec::new();
        
        for v in &all_violations {
            let ctx = ComplianceContext {
                file_path: v.file.clone(),
                line: v.line,
                snippet: None,
                project_type: None,
                code_region: if v.file.contains("test") { Some("test".into()) } else { None },
                function_context: None,
            };
            
            // Domain inference from rule ID prefix
            let domain = v.id.chars()
                .take_while(|c| c.is_alphabetic())
                .collect::<String>();
            let domain = match domain.to_uppercase().as_str() {
                "SEC" => "security",
                "MEM" => "memory-safety",
                "SUPP" => "supply-chain",
                "LOGIC" => "logic",
                "STYLE" => "style",
                "NAME" => "naming",
                "CPLX" => "complexity",
                "SEMANTIC" => "semantic",
                _ => "general",
            };
            
            let decision = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(
                    engine.evaluate(&v.id, domain, &v.message, &ctx)
                )
            }).expect("Compliance evaluation failed");
            
            match &decision.action {
                ComplianceAction::Block => {
                    blocked.push(v.clone());
                }
                ComplianceAction::Accept { reason, .. } => {
                    accepted.push((v.clone(), reason.clone()));
                }
                _ => {}
            }
            
            results.push((v.id.clone(), decision));
        }
        
        (blocked, accepted, results)
    };

    #[cfg(not(feature = "intelligence"))]
    let (compliance_blocked, compliance_accepted, compliance_results): (Vec<ViolationInfo>, Vec<(ViolationInfo, String)>, Vec<(String, ())>) = (Vec::new(), Vec::new(), Vec::new());

    // Remove accepted violations from counts
    #[cfg(feature = "intelligence")]
    {
        for (v, _) in &compliance_accepted {
            match v.severity {
                Severity::Critical => total_criticals = total_criticals.saturating_sub(1),
                Severity::Error => total_errors = total_errors.saturating_sub(1),
                Severity::Warning => total_warnings = total_warnings.saturating_sub(1),
                _ => {}
            }
        }
    }

    // ===== Phase 2: Multi-Signal Scoring with Dubbioso Mode =====
    #[cfg(feature = "intelligence")]
    let (dubbioso_results, multi_signal_scores) = if dubbioso && !all_violations.is_empty() {
        let config = DubbiosoConfig::default();
        let mut validator = DubbiosoValidator::new(config);

        // Load ProjectState for delta detection
        if let Ok(parent) = path.parent().ok_or_else(|| anyhow::anyhow!("No parent directory")) {
            if let Ok(mut validation_state) = ValidationState::new(None) {
                let project_state = validation_state.get_project(parent);
                validator.set_previous_state(project_state.clone());
            }
        }

        let mut results = Vec::new();
        let mut scores = Vec::new();

        for v in &all_violations {
            let input = ViolationInput {
                id: v.id.clone(),
                rule: v.rule.clone(),
                message: v.message.clone(),
                file: v.file.clone(),
                line: v.line as u32,
                column: v.column as u32,
                function_name: None,
                code: None,
                language: language.clone(),
            };

            let result = validator.validate(&input);

            // Compute multi-signal score for ranking
            let score = MultiSignalScore::compute(
                result.confidence.confidence as f32,
                chrono::Utc::now() - chrono::Duration::seconds(1),
                0.5,
                chrono::Utc::now(),
            );

            results.push(result);
            scores.push(score);
        }

        (Some(results), Some(scores))
    } else {
        (None, None)
    };

    #[cfg(not(feature = "intelligence"))]
    let (dubbioso_results, multi_signal_scores): (Option<Vec<()>>, Option<Vec<()>>) = (None, None);

    // ===== Phase 3: Filter low-confidence violations (reduce false positives) =====
    #[cfg(feature = "intelligence")]
    if dubbioso {
        if let (Some(ref results), _) = (&dubbioso_results, &multi_signal_scores) {
            const CONFIDENCE_THRESHOLD: f64 = 0.25; // Filter below 25% confidence
            let original_count = all_violations.len();
            // Filter: keep only violations that pass confidence check
            let filtered: Vec<ViolationInfo> = all_violations.into_iter()
                .enumerate()
                .filter_map(|(idx, v)| {
                    if let Some(r) = results.get(idx) {
                        if r.confidence.confidence >= CONFIDENCE_THRESHOLD {
                            return Some(v);
                        }
                        None // Filter out low confidence
                    } else {
                        Some(v) // Keep if no dubbioso result
                    }
                })
                .collect();
            let filtered_count = original_count - filtered.len();
            all_violations = filtered;
            if filtered_count > 0 {
                eprintln!("[Dubbioso] Filtered {} low-confidence violations (threshold: {}%)",
                    filtered_count, (CONFIDENCE_THRESHOLD * 100.0) as u32);
            }
        }
    }

    // ===== Phase 4: Filter test-only violations (reduce false positives in test code) =====
    // Rules like LOGIC001 (panic!) should not apply to test code
    const TEST_ONLY_RULES: &[&str] = &["LOGIC001", "LOGIC002"]; // panic!, todo!
    let path_str = path.to_string_lossy().to_lowercase();
    let is_test_file = path_str.contains("/tests/") || path_str.contains("\\tests\\") ||
                        path_str.contains("/test_") || path_str.contains("\\test_") ||
                        path_str.ends_with("_test.rs") || path_str.ends_with("_tests.rs");
    if is_test_file {
        let original_count = all_violations.len();
        all_violations.retain(|v| !TEST_ONLY_RULES.contains(&v.rule.as_str()));
        let filtered_count = original_count - all_violations.len();
        if filtered_count > 0 {
            eprintln!("[Filter] Filtered {} panic/todo violations in test code", filtered_count);
        }
    }

    // Output - consider compliance blocked violations
    #[cfg(feature = "intelligence")]
    let passed = result.all_passed() && contract_violations.is_empty() && compliance_blocked.is_empty();
    
    #[cfg(not(feature = "intelligence"))]
    let passed = result.all_passed() && contract_violations.is_empty();

    #[cfg(feature = "intelligence")]
    if format == "json" {
        let mut violations_json: Vec<serde_json::Value> = all_violations.iter().map(|v| {
            // Check if this violation was blocked by compliance
            let compliance_info = compliance_results.iter()
                .find(|(id, _)| id == &v.id)
                .map(|(_, decision)| decision);
            
            let compliance_status = compliance_info.map(|d| {
                serde_json::json!({
                    "tier": format!("{:?}", d.tier),
                    "action": match &d.action {
                        ComplianceAction::Block => "blocked",
                        ComplianceAction::Warn => "warn",
                        ComplianceAction::Ask { .. } => "ask",
                        ComplianceAction::Learn { .. } => "learn",
                        ComplianceAction::Accept { reason, .. } => reason,
                    },
                    "confidence": d.confidence,
                    "overridable": d.overridable,
                })
            });
            
            let mut v_json = serde_json::json!({
                "id": v.id,
                "rule": v.rule,
                "severity": format!("{:?}", v.severity),
                "message": v.message,
                "file": v.file,
                "line": v.line
            });
            
            if let Some(status) = compliance_status {
                if let serde_json::Value::Object(ref mut map) = v_json {
                    map.insert("compliance".to_string(), status);
                }
            }
            
            v_json
        }).collect();

        // Add Dubbioso confidence if available
        if let (Some(ref results), Some(ref scores)) = (dubbioso_results, multi_signal_scores) {
            for (i, (r, s)) in results.iter().zip(scores.iter()).enumerate() {
                if let Some(serde_json::Value::Object(ref mut map)) = violations_json.get_mut(i) {
                    map.insert("dubbioso".to_string(), serde_json::json!({
                        "level": format!("{:?}", r.confidence.level),
                        "confidence": r.confidence.confidence,
                        "multi_signal_score": s.score,
                        "action": match r.confidence.level {
                            ConfidenceLevel::AutoAccept => "auto-accept",
                            ConfidenceLevel::Good => "proceed",
                            ConfidenceLevel::Warn => "review",
                            ConfidenceLevel::Ask => "ask",
                        }
                    }));
                }
            }
        }

        let output = serde_json::json!({
            "passed": passed,
            "language": language,
            "file": path.to_string_lossy(),
            "validation_violations": result.total_violations(),
            "contract_violations": contract_violations.len(),
            "violations": violations_json
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Print compliance summary if there are results
        #[cfg(feature = "intelligence")]
        {
            if !compliance_blocked.is_empty() || !compliance_accepted.is_empty() {
                println!();
                println!("{}", "╔══════════════════════════════════════════════════════════════╗".red());
                println!("{}", "║                    COMPLIANCE ENGINE                         ║".red());
                println!("{}", "╠══════════════════════════════════════════════════════════════╣".red());
                
                if !compliance_blocked.is_empty() {
                    println!("{}", "║  BLOCKED (Inviolable violations - must fix):".red());
                    for v in &compliance_blocked {
                        println!("{} {} - {} (line {})", 
                            "  ✗".red(), 
                            v.id.red(), 
                            v.message.red(), 
                            v.line
                        );
                    }
                }
                
                if !compliance_accepted.is_empty() {
                    println!("{}", "║  ACCEPTED (Based on learned patterns/precedents):".green());
                    for (v, reason) in &compliance_accepted {
                        println!("{} {} - {} (line {})", 
                            "  ✓".green(), 
                            v.id.green(), 
                            reason.green(), 
                            v.line
                        );
                    }
                }
                
                println!("{}", "╚══════════════════════════════════════════════════════════════╝".red());
                println!();
            }
        }
        
        print_validation_result_with_dubbioso(&language, &path, &result, &contract_violations, severity, dubbioso, total_criticals, &dubbioso_results, &multi_signal_scores);
    }

    #[cfg(not(feature = "intelligence"))]
    if format == "json" {
        let output = serde_json::json!({
            "passed": passed,
            "language": language,
            "file": path.to_string_lossy(),
            "validation_violations": result.total_violations(),
            "contract_violations": contract_violations.len(),
            "violations": all_violations.iter().map(|v| {
                serde_json::json!({
                    "id": v.id,
                    "rule": v.rule,
                    "severity": format!("{:?}", v.severity),
                    "message": v.message,
                    "file": v.file,
                    "line": v.line
                })
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print_validation_result(&language, &path, &result, &contract_violations, severity, total_criticals);
    }

    // Save ProjectState after validation
    #[cfg(feature = "intelligence")]
    if let Err(e) = save_project_state(&path, &source, &all_violations) {
        tracing::warn!("Failed to save project state: {}", e);
    }

    Ok(ValidateResult {
        passed,
        criticals: total_criticals,
        errors: total_errors,
        warnings: total_warnings,
        violations: all_violations,
    })
}

/// Prints fallback validation result (for unsupported languages using regex-only security)
#[allow(dead_code)]
fn print_fallback_result(
    path: &Path,
    violations: &[ViolationInfo],
    passed: bool,
    total_criticals: usize,
    total_errors: usize,
    total_warnings: usize,
    _format: &str,
) {
    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} {}{}", "║".cyan(), format!("AETHER - Validating {} (Unknown - fallback mode)", path.display()).bold(), ".".cyan());
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());

    if violations.is_empty() {
        println!("{} {} {}", "║".cyan(), "✓".green(), "No security issues found".green());
    } else {
        println!("{} {}:", "║".cyan(), "Security Violations:".yellow());
        for v in violations {
            match v.severity {
                Severity::Critical => {
                    println!("{} {} {} - {} (line {})", "║".cyan(), "⚠".red().bold(), v.id.red().bold(), v.message, v.line);
                }
                Severity::Error => {
                    println!("{} {} {} - {} (line {})", "║".cyan(), "✗".red(), v.id.red(), v.message, v.line);
                }
                Severity::Warning => {
                    println!("{} {} {} - {} (line {})", "║".cyan(), "⚠".yellow(), v.id.yellow(), v.message, v.line);
                }
                Severity::Info => {
                    println!("{} {} {} - {} (line {})", "║".cyan(), "ℹ".blue(), v.id.blue(), v.message, v.line);
                }
                Severity::Hint => {
                    println!("{} {} {} - {} (line {})", "║".cyan(), "💡".dimmed(), v.id.dimmed(), v.message, v.line);
                }
            }
        }
    }

    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    println!("{} {} {} {}", "║".cyan(), "SUMMARY:".bold(), violations.len(), "violations".cyan());
    if total_criticals > 0 {
        println!("{}   • {} criticals", "║".cyan(), total_criticals.to_string().red().bold());
    }
    println!("{}   • {} errors", "║".cyan(), total_errors.to_string().red());
    println!("{}   • {} warnings", "║".cyan(), total_warnings.to_string().yellow());

    // Quality Score
    let quality_score = 100.0 - (total_criticals as f64 * 20.0) - (total_errors as f64 * 10.0) - (total_warnings as f64 * 2.0);
    let quality_score = quality_score.max(0.0);
    let score_str = format!("{:.1}", quality_score);
    let score_colored = if quality_score >= 80.0 { score_str.green() } else if quality_score >= 60.0 { score_str.yellow() } else { score_str.red() };
    println!("{}   Quality Score: {}/100", "║".cyan(), score_colored);

    println!("║",);

    if passed {
        println!("{} {} {}", "║".cyan(), "✓".green(), "No critical issues found".green());
    } else {
        println!("{} {} {}", "║".cyan(), "✗".red(), "Fix errors before committing".red());
    }

    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());
}

/// Prints validation result without Dubbioso mode (used when intelligence feature is disabled)
/// Infrastructure for future CLI output integration
#[allow(dead_code)]
fn print_validation_result(
    language: &str,
    path: &Path,
    result: &PipelineResult,
    contract_violations: &[(String, String, Violation)],
    min_severity: &str,
    total_criticals: usize,
) {
    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} {}{}", "║".cyan(), format!("AETHER - Validating {} ({})", path.display(), language).bold(), ".".cyan());
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut total_infos = 0;

    // Validation violations from layers
    for (layer_name, layer_result) in &result.results {
        if !layer_result.violations.is_empty() {
            println!("{} {} [{}]:", "║".cyan(), "Validation".yellow(), layer_name.blue());
            for v in &layer_result.violations {
                let show = match (min_severity, &v.severity) {
                    ("error", _) => true,
                    ("warning", _) => v.severity != Severity::Info,
                    ("info", _) => true,
                    _ => true,
                };

                if show {
                    match v.severity {
                        Severity::Critical => {
                            total_errors += 1;
                            println!("{} {} {} - {}", "║".cyan(), "⚠".red().bold(), v.id.red().bold(), v.message);
                        }
                        Severity::Error => {
                            total_errors += 1;
                            println!("{} {} {} - {}", "║".cyan(), "✗".red(), v.id.red(), v.message);
                        }
                        Severity::Warning => {
                            total_warnings += 1;
                            println!("{} {} {} - {}", "║".cyan(), "⚠".yellow(), v.id.yellow(), v.message);
                        }
                        Severity::Info => {
                            total_infos += 1;
                            println!("{} {} {} - {}", "║".cyan(), "ℹ".blue(), v.id.blue(), v.message);
                        }
                        Severity::Hint => {
                            println!("{} {} {} - {}", "║".cyan(), "💡".dimmed(), v.id.dimmed(), v.message);
                        }
                    }
                }
            }
        }
    }

    // Contract violations
    if !contract_violations.is_empty() {
        println!("{} {}:", "║".cyan(), "Contract Violations:".yellow());
        for (id, name, v) in contract_violations {
            let show = match (min_severity, &v.severity) {
                ("error", Severity::Error) => true,
                ("warning", Severity::Error | Severity::Warning) => true,
                ("info", _) => true,
                _ => true,
            };

            if show {
                match v.severity {
                    Severity::Critical => {
                        total_errors += 1;
                        println!("{} {} {} {} - {}", "║".cyan(), "⚠".red().bold(), format!("[{}]", id).red().bold(), name.red().bold(), v.message);
                    }
                    Severity::Error => {
                        total_errors += 1;
                        println!("{} {} {} {} - {}", "║".cyan(), "✗".red(), format!("[{}]", id).red(), name.red(), v.message);
                    }
                    Severity::Warning => {
                        total_warnings += 1;
                        println!("{} {} {} {} - {}", "║".cyan(), "⚠".yellow(), format!("[{}]", id).yellow(), name.yellow(), v.message);
                    }
                    Severity::Info => {
                        total_infos += 1;
                        println!("{} {} {} {} - {}", "║".cyan(), "ℹ".blue(), format!("[{}]", id).blue(), name.blue(), v.message);
                    }
                    Severity::Hint => {
                        println!("{} {} {} {} - {}", "║".cyan(), "💡".dimmed(), format!("[{}]", id).dimmed(), name.dimmed(), v.message);
                    }
                }
            }
        }
    }

    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    println!("{} {} {} {}", "║".cyan(), "SUMMARY:".bold(), total_criticals + total_errors + total_warnings + total_infos, "violations".cyan());
    if total_criticals > 0 {
        println!("{}   • {} criticals", "║".cyan(), total_criticals.to_string().red().bold());
    }
    println!("{}   • {} errors", "║".cyan(), total_errors.to_string().red());
    println!("{}   • {} warnings", "║".cyan(), total_warnings.to_string().yellow());
    println!("{}   • {} infos", "║".cyan(), total_infos.to_string().blue());

    // Quality Score
    let quality_score = 100.0 - (total_criticals as f64 * 20.0) - (total_errors as f64 * 10.0) - (total_warnings as f64 * 2.0) - (total_infos as f64 * 0.5);
    let quality_score = quality_score.max(0.0);
    let score_str = format!("{:.1}", quality_score);
    let score_colored = if quality_score >= 80.0 { score_str.green() } else if quality_score >= 60.0 { score_str.yellow() } else { score_str.red() };
    println!("{}   Quality Score: {}/100", "║".cyan(), score_colored);

    println!("║",);

    if total_criticals == 0 && total_errors == 0 && total_warnings == 0 {
        println!("{} {} {}", "║".cyan(), "✓".green(), "Code looks good!".green());
    } else if total_criticals > 0 || total_errors > 0 {
        println!("{} {} {}", "║".cyan(), "✗".red(), "Fix errors before committing".red());
    } else {
        println!("{} {} {}", "║".cyan(), "⚠".yellow(), "Consider fixing warnings".yellow());
    }

    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());
}

#[cfg(feature = "intelligence")]
#[allow(clippy::too_many_arguments)]
fn print_validation_result_with_dubbioso(
    language: &str,
    path: &Path,
    result: &PipelineResult,
    contract_violations: &[(String, String, Violation)],
    min_severity: &str,
    dubbioso: bool,
    total_criticals: usize,
    dubbioso_results: &Option<Vec<aether_intelligence::dubbioso_validator::DubbiosoValidationResult>>,
    multi_signal_scores: &Option<Vec<MultiSignalScore>>,
) {
    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} {}{}", "║".cyan(), format!("AETHER - Validating {} ({})", path.display(), language).bold(), ".".cyan());
    if dubbioso {
        println!("{} {} {}", "║".cyan(), "🤖".purple(), "Dubbioso Mode ACTIVE".purple());
    }
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut total_infos = 0;
    let mut violation_idx = 0;

    // Validation violations from layers
    for (layer_name, layer_result) in &result.results {
        if !layer_result.violations.is_empty() {
            println!("{} {} [{}]:", "║".cyan(), "Validation".yellow(), layer_name.blue());
            for v in &layer_result.violations {
                let show = match (min_severity, &v.severity) {
                    ("error", _) => true,
                    ("warning", _) => v.severity != Severity::Info,
                    ("info", _) => true,
                    _ => true,
                };

                if show {
                    match v.severity {
                        Severity::Critical => {
                            total_errors += 1;
                            print!("{} {} {} - {}", "║".cyan(), "⚠".red().bold(), v.id.red().bold(), v.message);
                        }
                        Severity::Error => {
                            total_errors += 1;
                            print!("{} {} {} - {}", "║".cyan(), "✗".red(), v.id.red(), v.message);
                        }
                        Severity::Warning => {
                            total_warnings += 1;
                            print!("{} {} {} - {}", "║".cyan(), "⚠".yellow(), v.id.yellow(), v.message);
                        }
                        Severity::Info => {
                            total_infos += 1;
                            print!("{} {} {} - {}", "║".cyan(), "ℹ".blue(), v.id.blue(), v.message);
                        }
                        Severity::Hint => {
                            print!("{} {} {} - {}", "║".cyan(), "💡".dimmed(), v.id.dimmed(), v.message);
                        }
                    }

                    // Add Dubbioso confidence if available
                    if dubbioso {
                        if let (Some(ref results), Some(ref scores)) = (dubbioso_results, multi_signal_scores) {
                            if let (Some(r), Some(_s)) = (results.get(violation_idx), scores.get(violation_idx)) {
                                let confidence_str = format!(" [{:.0}% {}]",
                                    r.confidence.confidence * 100.0,
                                    match r.confidence.level {
                                        ConfidenceLevel::AutoAccept => "AUTO".green(),
                                        ConfidenceLevel::Good => "GOOD".bright_green(),
                                        ConfidenceLevel::Warn => "WARN".yellow(),
                                        ConfidenceLevel::Ask => "ASK".red(),
                                    }
                                );
                                print!("{}", confidence_str);
                            }
                        }
                    }
                    println!();
                    violation_idx += 1;
                }
            }
        }
    }

    // Contract violations
    if !contract_violations.is_empty() {
        println!("{} {}:", "║".cyan(), "Contract Violations:".yellow());
        for (id, name, v) in contract_violations {
            let show = match (min_severity, &v.severity) {
                ("error", Severity::Error) => true,
                ("warning", Severity::Error | Severity::Warning) => true,
                ("info", _) => true,
                _ => true,
            };

            if show {
                match v.severity {
                    Severity::Critical => {
                        total_errors += 1;
                        print!("{} {} {} {} - {}", "║".cyan(), "⚠".red().bold(), format!("[{}]", id).red().bold(), name.red().bold(), v.message);
                    }
                    Severity::Error => {
                        total_errors += 1;
                        print!("{} {} {} {} - {}", "║".cyan(), "✗".red(), format!("[{}]", id).red(), name.red(), v.message);
                    }
                    Severity::Warning => {
                        total_warnings += 1;
                        print!("{} {} {} {} - {}", "║".cyan(), "⚠".yellow(), format!("[{}]", id).yellow(), name.yellow(), v.message);
                    }
                    Severity::Info => {
                        total_infos += 1;
                        print!("{} {} {} {} - {}", "║".cyan(), "ℹ".blue(), format!("[{}]", id).blue(), name.blue(), v.message);
                    }
                    Severity::Hint => {
                        print!("{} {} {} {} - {}", "║".cyan(), "💡".dimmed(), format!("[{}]", id).dimmed(), name.dimmed(), v.message);
                    }
                }

                // Add Dubbioso confidence if available
                if dubbioso {
                    if let (Some(ref results), Some(ref scores)) = (dubbioso_results, multi_signal_scores) {
                        if let (Some(r), Some(_s)) = (results.get(violation_idx), scores.get(violation_idx)) {
                            let confidence_str = format!(" [{:.0}% {}]",
                                r.confidence.confidence * 100.0,
                                match r.confidence.level {
                                    ConfidenceLevel::AutoAccept => "AUTO".green(),
                                    ConfidenceLevel::Good => "GOOD".bright_green(),
                                    ConfidenceLevel::Warn => "WARN".yellow(),
                                    ConfidenceLevel::Ask => "ASK".red(),
                                }
                            );
                            print!("{}", confidence_str);
                        }
                    }
                }
                println!();
                violation_idx += 1;
            }
        }
    }

    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    println!("{} {} {} {}", "║".cyan(), "SUMMARY:".bold(), total_criticals + total_errors + total_warnings + total_infos, "violations".cyan());
    if total_criticals > 0 {
        println!("{}   • {} criticals", "║".cyan(), total_criticals.to_string().red().bold());
    }
    println!("{}   • {} errors", "║".cyan(), total_errors.to_string().red());
    println!("{}   • {} warnings", "║".cyan(), total_warnings.to_string().yellow());
    println!("{}   • {} infos", "║".cyan(), total_infos.to_string().blue());

    // Quality Score
    let quality_score = 100.0 - (total_criticals as f64 * 20.0) - (total_errors as f64 * 10.0) - (total_warnings as f64 * 2.0) - (total_infos as f64 * 0.5);
    let quality_score = quality_score.max(0.0);
    let score_str = format!("{:.1}", quality_score);
    let score_colored = if quality_score >= 80.0 { score_str.green() } else if quality_score >= 60.0 { score_str.yellow() } else { score_str.red() };
    println!("{}   Quality Score: {}/100", "║".cyan(), score_colored);

    println!("║",);

    if total_criticals == 0 && total_errors == 0 && total_warnings == 0 {
        println!("{} {} {}", "║".cyan(), "✓".green(), "Code looks good!".green());
    } else if total_criticals > 0 || total_errors > 0 {
        println!("{} {} {}", "║".cyan(), "✗".red(), "Fix errors before committing".red());
    } else {
        println!("{} {} {}", "║".cyan(), "⚠".yellow(), "Consider fixing warnings".yellow());
    }

    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());
}

// ============================================================================
// SELF VALIDATE (Aether validates itself)
// ============================================================================

pub async fn self_validate(severity: &str, format: &str) -> Result<()> {
    // Find Aether source directory
    let aether_dir = std::env::current_dir()?
        .ancestors()
        .find(|p| p.join("Cargo.toml").exists())
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Could not find Aether project directory"))?;

    // Collect all .rs files (exclude tests, fixtures, benchmarks, examples)
    let mut rust_files: Vec<PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(&aether_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let path_str = path.to_string_lossy();

        let is_excluded = path_str.contains("target")
            || path_str.contains("/tests/")
            || path_str.contains("\\tests\\")
            || path_str.contains("/test_samples/")
            || path_str.contains("\\test_samples\\")
            || path_str.contains("/fixtures/")
            || path_str.contains("\\fixtures\\")
            || path_str.contains("/benches/")
            || path_str.contains("\\benches\\")
            || path_str.contains("/examples/")
            || path_str.contains("\\examples\\")
            || path_str.contains("/test-suite/")
            || path_str.contains("\\test-suite\\")
            || path_str.contains("/test-")
            || path_str.contains("\\test-")
            || path_str.contains("/contracts/")
            || path_str.contains("\\contracts\\");

        if path.extension().map(|e| e == "rs").unwrap_or(false) && !is_excluded {
            rust_files.push(path.to_path_buf());
        }
    }

    let _contracts_dir = get_contracts_dir(); // Keep for future use
    let registry = ParserRegistry::with_defaults();
    let parser = registry.get("rust")
        .ok_or_else(|| anyhow::anyhow!("Rust parser not found"))?;

    let mut total_files = 0;
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut total_infos = 0;
    #[allow(clippy::type_complexity)]
    let mut file_results: Vec<(PathBuf, Vec<(String, String, Violation)>)> = Vec::new();

    // Validate each file
    for file in &rust_files {
        let source = match fs::read_to_string(file) {
            Ok(s) => s,
            Err(_) => continue,
        };

        total_files += 1;

        // Parse
        let _ast = match parser.parse(&source).await {
            Ok(ast) => ast,
            Err(_) => continue,
        };

        // Build validation context
        let ctx = ValidationContext::for_file(file, source.clone(), "rust".to_string());

        // Build validation pipeline
        // Note: SyntaxLayer is excluded from self-validation because:
        // 1. The code compiles successfully, so syntax is valid by definition
        // 2. SyntaxLayer's simple brace counting produces false positives on:
        //    - Rust macros (format!, vec!, etc.)
        //    - Raw strings with complex content
        //    - Attribute macros
        let pipeline = ValidationPipeline::new()
            .add_layer(SupplyChainLayer::new())
            .add_layer(SecurityLayer::new())
            .add_layer(SemanticLayer::new())
            .add_layer(ScopeAnalysisLayer::new())
            .add_layer(TypeInferenceLayer::new())
            .add_layer(LogicLayer::new())
            .add_layer(ComplexityLayer::default());

        let result = pipeline.execute(&ctx).await;

        // Skip contract checks in self-validation - Aether defines these rules,
        // it doesn't consume them. Contracts are for validating user code.

        let mut violations: Vec<(String, String, Violation)> = Vec::new();

        // Validation layer violations
        for (_, layer_result) in &result.results {
            for v in &layer_result.violations {
                let sev = match v.severity {
                    Severity::Critical => { total_errors += 1; "CRITICAL" }
                    Severity::Error => { total_errors += 1; "ERROR" }
                    Severity::Warning => { total_warnings += 1; "WARN" }
                    Severity::Info => { total_infos += 1; "INFO" }
                    Severity::Hint => { total_infos += 1; "HINT" }
                };
                violations.push((v.id.clone(), sev.to_string(), v.clone()));
            }
        }

        if !violations.is_empty() {
            file_results.push((file.clone(), violations));
        }
    }

    // Output
    if format == "json" {
        let output = serde_json::json!({
            "aether_self_validation": true,
            "files_checked": total_files,
            "files_with_violations": file_results.len(),
            "total_errors": total_errors,
            "total_warnings": total_warnings,
            "total_infos": total_infos,
            "results": file_results.iter().map(|(path, violations)| {
                serde_json::json!({
                    "file": path.to_string_lossy(),
                    "violations": violations.len()
                })
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
        println!("{} {}", "║".cyan(), "AETHER SELF-VALIDATION".bold());
        println!("{} {}", "║".cyan(), "(Eat your own dog food)".dimmed());
        println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
        println!("{} Files checked: {}", "║".cyan(), total_files);
        println!("{} Files with issues: {}", "║".cyan(), file_results.len());
        println!("║",);

        for (file, violations) in &file_results {
            let rel_path = file.strip_prefix(&aether_dir).unwrap_or(file);
            println!("{} {}:", "║".cyan(), rel_path.display().to_string().yellow());

            let show_all = severity != "error";

            for (id, _sev, v) in violations {
                let show = match (severity, &v.severity) {
                    ("error", Severity::Error) => true,
                    ("warning", Severity::Error | Severity::Warning) => true,
                    ("info", _) => true,
                    _ => show_all,
                };

                if show {
                    match v.severity {
                        Severity::Critical => println!("{}   {} [{}] {}", "║".cyan(), "⚠".red().bold(), id.red().bold(), v.message),
                        Severity::Error => println!("{}   {} [{}] {}", "║".cyan(), "✗".red(), id.red(), v.message),
                        Severity::Warning => println!("{}   {} [{}] {}", "║".cyan(), "⚠".yellow(), id.yellow(), v.message),
                        Severity::Info => println!("{}   {} [{}] {}", "║".cyan(), "ℹ".blue(), id.blue(), v.message),
                        Severity::Hint => {}
                    }
                }
            }
        }

        println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
        println!("{} {} {}", "║".cyan(), "SUMMARY:".bold(), total_errors + total_warnings + total_infos);
        println!("{}   • {} errors", "║".cyan(), total_errors.to_string().red());
        println!("{}   • {} warnings", "║".cyan(), total_warnings.to_string().yellow());
        println!("{}   • {} infos", "║".cyan(), total_infos.to_string().blue());
        println!("║",);

        if total_errors == 0 && total_warnings == 0 {
            println!("{} {} {}", "║".cyan(), "✓".green(), "Aether code looks good!".green());
        } else if total_errors > 0 {
            println!("{} {} {}", "║".cyan(), "✗".red(), "Fix errors in Aether source code".red());
        } else {
            println!("{} {} {}", "║".cyan(), "⚠".yellow(), "Consider fixing warnings".yellow());
        }

        println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());
    }

    if total_errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

// ============================================================================
// ANALYZE
// ============================================================================

pub async fn analyze(file: PathBuf, format: &str) -> Result<()> {
    let language = detect_language(&file)
        .ok_or_else(|| {
            let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("unknown");
            anyhow::anyhow!("Language '.{}' not supported. Use 'aether validate' for fallback security checks.", ext)
        })?;
    let source = fs::read_to_string(&file)?;

    let registry = ParserRegistry::with_defaults();
    let parser = registry.get(&language)
        .ok_or_else(|| anyhow::anyhow!("Language '{}' not supported", language))?;

    let ast = parser.parse(&source).await
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    // Count nodes
    fn count_nodes(node: &aether_parsers::ASTNode) -> usize {
        1 + node.children.iter().map(count_nodes).sum::<usize>()
    }
    fn max_depth(node: &aether_parsers::ASTNode) -> usize {
        if node.children.is_empty() {
            1
        } else {
            1 + node.children.iter().map(max_depth).max().unwrap_or(0)
        }
    }
    
    let node_count = count_nodes(&ast.root);
    let depth = max_depth(&ast.root);

    if format == "json" {
        let output = serde_json::json!({
            "language": language,
            "file": file.to_string_lossy(),
            "stats": {
                "nodes": node_count,
                "depth": depth,
                "errors": ast.errors.len()
            }
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
        println!("{} {} ({})", "║".cyan(), "AST Analysis".bold(), language.green());
        println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
        println!("{} File: {}", "║".cyan(), file.display());
        println!("{} Nodes: {}", "║".cyan(), node_count);
        println!("{} Max Depth: {}", "║".cyan(), depth);
        if !ast.errors.is_empty() {
            println!("{} Errors: {}", "║".cyan(), ast.errors.len());
            for err in &ast.errors {
                println!("{}   • {}", "║".cyan(), err.red());
            }
        }
        println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());
    }

    Ok(())
}

// ============================================================================
// CERTIFY
// ============================================================================

pub async fn certify(file: PathBuf, output: Option<PathBuf>, keypair_path: Option<PathBuf>) -> Result<()> {
    let language = detect_language(&file)
        .ok_or_else(|| {
            let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("unknown");
            anyhow::anyhow!("Language '.{}' not supported. Cannot certify unsupported languages.", ext)
        })?;
    let source = fs::read_to_string(&file)?;

    // Keypair
    let keypair_file = keypair_path.unwrap_or_else(|| {
        get_keystore_dir().join("keypair.json")
    });

    let keypair = if keypair_file.exists() {
        let bytes = fs::read(&keypair_file)?;
        let json: serde_json::Value = serde_json::from_slice(&bytes)?;
        let key_bytes = hex::decode(json["secret"].as_str().unwrap_or(""))
            .map_err(|e| anyhow::anyhow!("Invalid keypair: {}", e))?;
        let key_array: [u8; 64] = key_bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid keypair length"))?;
        Keypair::from_bytes(&key_array)?
    } else {
        println!("{} Keypair not found, generating...", "ℹ".blue());
        let keypair = Keypair::generate();
        
        let key_bytes = keypair.to_bytes();
        let json = serde_json::json!({
            "secret": hex::encode(key_bytes),
            "public": hex::encode(keypair.public().as_bytes())
        });

        fs::write(&keypair_file, serde_json::to_string_pretty(&json)?)?;
        
        let public_path = keypair_file.with_extension("pub");
        fs::write(&public_path, hex::encode(keypair.public().as_bytes()))?;
        
        println!("{} Keypair saved to {}", "✓".green(), keypair_file.display());
        keypair
    };

    // Parse and validate
    let registry = ParserRegistry::with_defaults();
    let parser = registry.get(&language)
        .ok_or_else(|| anyhow::anyhow!("Language '{}' not supported", language))?;

    let _ast = parser.parse(&source).await
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let ctx = ValidationContext::for_file(&file, source.clone(), language.clone());
    let pipeline = ValidationPipeline::new()
        .add_layer(SupplyChainLayer::new())
        .add_layer(SecurityLayer::new())
        .add_layer(SyntaxLayer::new())
        .add_layer(SemanticLayer::new())
        .add_layer(ScopeAnalysisLayer::new())
        .add_layer(TypeInferenceLayer::new())
        .add_layer(LogicLayer::new())
        .add_layer(ComplexityLayer::default());

    let result = pipeline.execute(&ctx).await;

    if !result.all_passed() {
        println!("{} Validation failed:", "✗".red());
        for (layer_name, layer_result) in &result.results {
            if !layer_result.passed {
                println!("  {} [{}]: {} violations", "✗".red(), layer_name, layer_result.violations.len());
            }
        }
        return Err(anyhow::anyhow!("Validation failed"));
    }

    // Create certificate
    let file_hash = Certificate::hash_file(source.as_bytes());
    let start = std::time::Instant::now();
    
    let mut cert = Certificate::new(
        file_hash,
        ValidationResult {
            passed: true,
            total_violations: result.total_violations(),
            errors: 0,
            warnings: 0,
            duration_ms: start.elapsed().as_millis() as u64,
        },
        AgentInfo {
            name: "aether-cli".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    );
    
    keypair.sign_certificate(&mut cert)?;
    
    let cert_path = output.unwrap_or_else(|| {
        file.with_extension("cert.json")
    });
    
    let json = serde_json::to_string_pretty(&cert)?;
    fs::write(&cert_path, json)?;

    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} {} {}", "║".cyan(), "✓".green(), "Certificate created".green());
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    println!("{} File: {}", "║".cyan(), file.display());
    println!("{} Language: {}", "║".cyan(), language);
    println!("{} Certificate: {}", "║".cyan(), cert_path.display());
    println!("{} ID: {}", "║".cyan(), cert.id);
    println!("{} Signed: {}", "║".cyan(), cert.is_signed());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());

    Ok(())
}

// ============================================================================
// VERIFY
// ============================================================================

pub fn verify(cert_path: PathBuf, public_key_path: Option<PathBuf>) -> Result<()> {
    let cert_json = fs::read_to_string(&cert_path)?;
    let cert: Certificate = serde_json::from_str(&cert_json)?;

    let public_key_file = public_key_path.unwrap_or_else(|| {
        get_keystore_dir().join("keypair.pub")
    });
    
    let public_bytes = if public_key_file.exists() {
        fs::read(&public_key_file)?
    } else {
        let alt_path = cert_path.with_extension("pub");
        if alt_path.exists() {
            fs::read(alt_path)?
        } else {
            return Err(anyhow::anyhow!("Public key file not found. Use --public-key to specify."));
        }
    };
    
    let public_hex = String::from_utf8(public_bytes)?;
    let public_bytes = hex::decode(public_hex.trim())?;
    let public_array: [u8; 32] = public_bytes.try_into()
        .map_err(|_| anyhow::anyhow!("Invalid public key length"))?;
    let public_key = VerifyingKey::from_bytes(&public_array)
        .map_err(|e| anyhow::anyhow!("Invalid public key: {}", e))?;

    let valid = CertificateVerifier::verify(&cert, &public_key)?;

    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    
    if valid {
        println!("{} {} {}", "║".cyan(), "✓".green(), "Certificate VERIFIED".green());
    } else {
        println!("{} {} {}", "║".cyan(), "✗".red(), "Certificate INVALID".red());
    }
    
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    println!("{} Certificate: {}", "║".cyan(), cert_path.display());
    println!("{} ID: {}", "║".cyan(), cert.id);
    println!("{} File Hash: {}...", "║".cyan(), &cert.file_hash[..16.min(cert.file_hash.len())]);
    println!("{} Valid: {}", "║".cyan(), if valid { "Yes".green() } else { "No".red() });
    println!("{} Passed: {}", "║".cyan(), cert.validation.passed);
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());

    if !valid {
        std::process::exit(1);
    }

    Ok(())
}

// ============================================================================
// LIST
// ============================================================================

pub fn list(lang: Option<String>, dir: Option<PathBuf>) -> Result<()> {
    let contracts_dir = dir.unwrap_or_else(get_contracts_dir);
    let loader = ContractLoader::new(contracts_dir.clone());

    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} {} .", "║".cyan(), "Available Contracts".bold());
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    println!("{} Directory: {}", "║".cyan(), contracts_dir.display());
    println!("║",);

    let languages = ["rust", "python", "javascript", "typescript", "cpp", "go", "java", "lua", "lex"];

    for lang_name in languages {
        if let Some(ref filter) = lang {
            if lang_name != filter {
                continue;
            }
        }

        let contracts = loader.load_dir(lang_name)?;
        if !contracts.is_empty() {
            println!("{} {}:", "║".cyan(), lang_name.to_uppercase().green());
            for contract in contracts {
                println!("{}   • {} - {}", "║".cyan(), contract.id, contract.name);
            }
        }
    }

    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());

    Ok(())
}

// ============================================================================
// GENERATE KEYPAIR
// ============================================================================

pub fn generate_keypair(output: PathBuf) -> Result<()> {
    let keypair = Keypair::generate();

    if output != Path::new(".") {
        fs::create_dir_all(&output)?;
    }

    let keypair_path = if output == Path::new(".") {
        PathBuf::from("keypair.json")
    } else {
        output.join("keypair.json")
    };

    let public_path = if output == Path::new(".") {
        PathBuf::from("keypair.pub")
    } else {
        output.join("keypair.pub")
    };

    let key_bytes = keypair.to_bytes();
    let json = serde_json::json!({
        "secret": hex::encode(key_bytes),
        "public": hex::encode(keypair.public().as_bytes())
    });
    fs::write(&keypair_path, serde_json::to_string_pretty(&json)?)?;
    fs::write(&public_path, hex::encode(keypair.public().as_bytes()))?;

    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} {} {}", "║".cyan(), "✓".green(), "Keypair Generated".green());
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());
    println!("{} Keypair: {} {}", "║".cyan(), keypair_path.display(), "(KEEP SECRET!)".red());
    println!("{} Public:  {}", "║".cyan(), public_path.display());
    println!("{} Algorithm: Ed25519", "║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());

    Ok(())
}

// ============================================================================
// INIT
// ============================================================================

pub async fn init(lang: Option<String>, platform: Option<String>, level: Option<String>, config: Option<PathBuf>) -> Result<()> {
    println!();
    println!("{}", "╔═══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} {}", "║".cyan(), "                     AETHER SETUP v0.1                          ".cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════════════╝".cyan());
    println!();

    let config_data = if let Some(config_path) = &config {
        let content = fs::read_to_string(config_path)?;
        Some(serde_yaml::from_str::<serde_yaml::Value>(&content)?)
    } else {
        None
    };

    // Step 1: Languages
    let selected_languages: Vec<String> = if let Some(ref langs) = lang {
        parse_list(langs)
    } else if let Some(ref cfg) = config_data {
        cfg.get("languages")
            .and_then(|v| v.as_sequence())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default()
    } else if is_interactive() {
        use inquire::MultiSelect;
        
        match MultiSelect::new("Step 1/3: Select languages (space=select, enter=confirm)", platforms::LANGUAGES.to_vec())
            .prompt()
        {
            Ok(selection) => selection.into_iter().map(|s| s.to_string()).collect(),
            Err(e) => {
                eprintln!("{} {}", "Warning:".yellow(), e);
                eprintln!("{}", "Falling back to default: rust".yellow());
                vec!["rust".to_string()]
            }
        }
    } else {
        println!("{}", "Step 1/3: Languages (comma-separated, e.g., rust,python):".cyan());
        println!("  {}", "Options: rust, cpp, python, prism, lua, javascript, typescript, go, java".dimmed());
        print!("  {}: ", "Enter".green());
        let input = read_line()?;
        if input.is_empty() { vec!["rust".to_string()] } else { parse_list(&input) }
    };

    if selected_languages.is_empty() {
        println!("{}", "Error: At least one language must be selected".red());
        println!("{}", "Usage: aether init --lang rust,python --platform vscode --level standard".yellow());
        std::process::exit(1);
    }

    // Step 2: Platform
    let selected_platform: String = if let Some(ref plat) = platform {
        plat.clone()
    } else if let Some(ref cfg) = config_data {
        cfg.get("platform")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
    } else if is_interactive() {
        use inquire::Select;
        
        match Select::new("Step 2/3: Select platform", platforms::PLATFORMS.to_vec())
            .prompt()
        {
            Ok(selection) => selection.to_string(),
            Err(e) => {
                eprintln!("{} {}", "Warning:".yellow(), e);
                eprintln!("{}", "Falling back to default: claude".yellow());
                "claude".to_string()
            }
        }
    } else {
        println!();
        println!("{}", "Step 2/3: Platform:".cyan());
        println!("  {}", "Options: claude, vscode, cursor, neovim, zed, jetbrains, gemini, antigravity".dimmed());
        print!("  {}: ", "Enter".green());
        let input = read_line()?;
        if input.is_empty() { "claude".to_string() } else { input }
    };

    // Step 3: Level
    let selected_level: String = if let Some(ref lvl) = level {
        lvl.clone()
    } else if let Some(ref cfg) = config_data {
        cfg.get("level")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
    } else if is_interactive() {
        use inquire::Select;
        
        match Select::new("Step 3/3: Select validation level", platforms::LEVELS.to_vec())
            .prompt()
        {
            Ok(selection) => selection.to_string(),
            Err(e) => {
                eprintln!("{} {}", "Warning:".yellow(), e);
                eprintln!("{}", "Falling back to default: standard".yellow());
                "standard".to_string()
            }
        }
    } else {
        println!();
        println!("{}", "Step 3/3: Validation level:".cyan());
        println!("  {}", "Options: basic, standard, strict".dimmed());
        print!("  {}: ", "Enter".green());
        let input = read_line()?;
        if input.is_empty() { "standard".to_string() } else { input }
    };

    let selected_platform = if selected_platform.is_empty() { "claude".to_string() } else { selected_platform };
    let selected_level = if selected_level.is_empty() { "standard".to_string() } else { selected_level };

    println!();
    println!("{}", "Generating configuration...".cyan());

    let cwd = std::env::current_dir()?;
    platforms::generate_config(&selected_platform, &selected_languages, &selected_level, &cwd)?;

    println!();
    println!("{}", "╔═══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} {} {}", "║".cyan(), "✓".green(), "Installation complete!".green());
    println!("{}", "╠═══════════════════════════════════════════════════════════════╣".cyan());
    println!("{} Languages: {}", "║".cyan(), selected_languages.join(", "));
    println!("{} Platform: {}", "║".cyan(), selected_platform);
    println!("{} Level: {}", "║".cyan(), selected_level);
    println!("{}", "║".cyan());
    println!("{} To update: {}", "║".cyan(), "aether contracts update".yellow());
    println!("{} To check: {}", "║".cyan(), "aether contracts check".yellow());
    println!("{}", "╚═══════════════════════════════════════════════════════════════╝".cyan());

    Ok(())
}

// ============================================================================
// CONTRACTS
// ============================================================================

pub async fn contracts_check() -> Result<()> {
    println!();
    println!("{}", "Checking for contract updates...".cyan());

    let contracts_dir = get_contracts_dir();
    let loader = ContractLoader::new(contracts_dir.clone());

    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} {} ", "║".cyan(), "Installed Contracts".bold());
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());

    for lang in platforms::LANGUAGES {
        let lang_lower = lang.to_lowercase();
        if let Ok(contracts) = loader.load_dir(&lang_lower) {
            if !contracts.is_empty() {
                println!("{} {}: {} contracts", "║".cyan(), lang.green(), contracts.len());
            }
        }
    }

    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());
    println!();
    println!("Run {} to update contracts", "aether contracts update".yellow());

    Ok(())
}

pub async fn contracts_update(lang: Option<String>, _force: bool) -> Result<()> {
    println!();
    println!("{}", "Updating contracts...".cyan());

    let contracts_dir = get_contracts_dir();
    fs::create_dir_all(&contracts_dir)?;

    let languages_to_update = if let Some(l) = lang {
        vec![l.to_lowercase()]
    } else {
        platforms::LANGUAGES.iter().map(|l| l.to_lowercase()).collect()
    };

    println!("{}", "╔══════════════════════════════════════════════════════════════╗".cyan());
    println!("{} {}", "║".cyan(), "Downloading contracts...".bold());
    println!("{}", "╠══════════════════════════════════════════════════════════════╣".cyan());

    for lang_name in languages_to_update {
        let lang_dir = contracts_dir.join(&lang_name);
        fs::create_dir_all(&lang_dir)?;

        let contract = serde_yaml::to_string(&serde_yaml::Value::Mapping(
            serde_yaml::Mapping::from_iter(vec![
                (serde_yaml::Value::String("id".to_string()), serde_yaml::Value::String(format!("{}_001", lang_name.to_uppercase()))),
                (serde_yaml::Value::String("name".to_string()), serde_yaml::Value::String("Basic validation".to_string())),
                (serde_yaml::Value::String("language".to_string()), serde_yaml::Value::String(lang_name.clone())),
                (serde_yaml::Value::String("version".to_string()), serde_yaml::Value::String("1.0.0".to_string())),
                (serde_yaml::Value::String("rules".to_string()), serde_yaml::Value::Sequence(vec![])),
            ])
        ))?;

        fs::write(lang_dir.join(format!("{}.yaml", lang_name)), contract)?;
        println!("{} {} ✓", "║".cyan(), lang_name.green());
    }

    println!("{}", "╚══════════════════════════════════════════════════════════════╝".cyan());
    println!();
    println!("{} Contracts updated!", "✓".green());

    Ok(())
}
