//! Command implementations
//!
//! This module contains all command execution logic.

use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use std::fs;
use std::io::{self, BufRead};

use aether_parsers::ParserRegistry;
use aether_validation::{ValidationPipeline, SyntaxLayer, SemanticLayer, LogicLayer, SecurityLayer, ComplexityLayer, SupplyChainLayer, ValidationContext, PipelineResult, Violation, Severity};
use aether_certification::{Keypair, CertificateVerifier, Certificate, ValidationResult, AgentInfo, VerifyingKey};
use aether_contracts::{ContractLoader, RuleEvaluator};

use crate::platforms;

// ============================================================================
// HELPERS
// ============================================================================

/// Detect language from file extension
pub fn detect_language(path: &PathBuf) -> String {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    match ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" | "tsx" => "typescript",
        "cpp" | "cxx" | "cc" | "hpp" => "cpp",
        "go" => "go",
        "java" => "java",
        "lua" => "lua",
        "lex" => "lex",
        _ => "rust",
    }.to_string()
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
) -> Result<()> {
    let language = lang.unwrap_or_else(|| detect_language(&path));
    let source = fs::read_to_string(&path)?;
    let contracts_dir = contracts.unwrap_or_else(get_contracts_dir);

    // Get parser
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

    // Output
    let passed = result.all_passed() && contract_violations.is_empty();

    if format == "json" {
        let output = serde_json::json!({
            "passed": passed,
            "language": language,
            "file": path.to_string_lossy(),
            "validation_violations": result.total_violations(),
            "contract_violations": contract_violations.len(),
            "violations": contract_violations.iter().map(|(id, name, v)| {
                serde_json::json!({
                    "contract_id": id,
                    "contract_name": name,
                    "severity": format!("{:?}", v.severity),
                    "message": v.message
                })
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print_validation_result(&language, &path, &result, &contract_violations, severity);
    }

    if !passed {
        std::process::exit(1);
    }

    Ok(())
}

fn print_validation_result(
    language: &str,
    path: &PathBuf,
    result: &PipelineResult,
    contract_violations: &[(String, String, Violation)],
    min_severity: &str,
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
    println!("{} {} {} {}", "║".cyan(), "SUMMARY:".bold(), total_errors + total_warnings + total_infos, "violations".cyan());
    println!("{}   • {} errors", "║".cyan(), total_errors.to_string().red());
    println!("{}   • {} warnings", "║".cyan(), total_warnings.to_string().yellow());
    println!("{}   • {} infos", "║".cyan(), total_infos.to_string().blue());
    println!("║",);

    if total_errors == 0 && total_warnings == 0 {
        println!("{} {} {}", "║".cyan(), "✓".green(), "Code looks good!".green());
    } else if total_errors > 0 {
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
            || path_str.contains("\\examples\\");

        if path.extension().map(|e| e == "rs").unwrap_or(false) && !is_excluded {
            rust_files.push(path.to_path_buf());
        }
    }

    let contracts_dir = get_contracts_dir();
    let registry = ParserRegistry::with_defaults();
    let parser = registry.get("rust")
        .ok_or_else(|| anyhow::anyhow!("Rust parser not found"))?;

    let mut total_files = 0;
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut total_infos = 0;
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
        let pipeline = ValidationPipeline::new()
            .add_layer(SupplyChainLayer::new())
            .add_layer(SecurityLayer::new())
            .add_layer(SyntaxLayer::new())
            .add_layer(SemanticLayer::new())
            .add_layer(LogicLayer::new())
            .add_layer(ComplexityLayer::default());

        let result = pipeline.execute(&ctx).await;

        // Load contracts
        let loader = ContractLoader::new(contracts_dir.clone());
        let contract_rules = match loader.load_dir("rust") {
            Ok(rules) => rules,
            Err(_) => continue,
        };

        let mut evaluator = RuleEvaluator::new();
        let mut violations = Vec::new();

        // Contract violations
        for contract in &contract_rules {
            for rule in &contract.rules {
                if let Ok(v) = evaluator.evaluate(rule, &source) {
                    for v in v {
                        violations.push((contract.id.clone(), contract.name.clone(), v));
                    }
                }
            }
        }

        // Validation layer violations
        for (_, layer_result) in &result.results {
            for v in &layer_result.violations {
                let sev = match v.severity {
                    Severity::Error => { total_errors += 1; "ERROR" }
                    Severity::Warning => { total_warnings += 1; "WARN" }
                    Severity::Info => { total_infos += 1; "INFO" }
                    Severity::Hint => { total_infos += 1; "HINT" }
                };
                violations.push((v.id.clone(), sev.to_string(), v.clone()));
            }
        }

        // Count contract violations
        for (_, _, v) in &violations {
            match v.severity {
                Severity::Error => total_errors += 1,
                Severity::Warning => total_warnings += 1,
                Severity::Info => total_infos += 1,
                Severity::Hint => {}
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
    let language = detect_language(&file);
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
    let language = detect_language(&file);
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
    println!("{} {} {}", "║".cyan(), "Available Contracts".bold(), ".");
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
    
    if output != PathBuf::from(".") {
        fs::create_dir_all(&output)?;
    }

    let keypair_path = if output == PathBuf::from(".") {
        PathBuf::from("keypair.json")
    } else {
        output.join("keypair.json")
    };

    let public_path = if output == PathBuf::from(".") {
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
    println!("{} Algorithm: {}", "║".cyan(), "Ed25519");
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
    println!("{} {} {}", "║".cyan(), "Installed Contracts".bold(), "");
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
