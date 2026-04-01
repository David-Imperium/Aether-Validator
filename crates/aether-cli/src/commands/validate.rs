//! Validate command

use crate::commands::ValidateArgs;
use std::path::{Path, PathBuf};
use std::fs;

use aether_validation::{ValidationPipeline, ValidationContext, Violation};
use aether_validation::layers::{SyntaxLayer, SemanticLayer, LogicLayer, SecurityLayer, ComplexityLayer, SupplyChainLayer, StyleLayer, ClippyLayer, ContractLayer};
use aether_validation::ScopeAnalysisLayer;
use aether_contracts::{ContractLoader, RuleEvaluator};

/// Detect language from file extension
fn detect_language(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some("rust".to_string()),
        "py" => Some("python".to_string()),
        "js" | "ts" => Some("javascript".to_string()),
        "glsl" | "frag" | "vert" | "comp" => Some("glsl".to_string()),
        _ => None,
    }
}

/// Get contracts directory
fn get_contracts_dir() -> PathBuf {
    // First check local .factory/contracts
    let local = std::env::current_dir()
        .map(|c| c.join(".factory/contracts"))
        .unwrap_or_default();
    if local.exists() {
        return local;
    }
    
    // Check for contracts/ in current directory
    let local_contracts = std::env::current_dir()
        .map(|c| c.join("contracts"))
        .unwrap_or_default();
    if local_contracts.exists() {
        return local_contracts;
    }
    
    // Then check home directory
    dirs::home_dir()
        .map(|h| h.join(".aether/contracts"))
        .unwrap_or_else(|| PathBuf::from("contracts"))
}

pub async fn run(args: ValidateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = Path::new(&args.input);
    
    if !input_path.exists() {
        return Err(format!("Input path does not exist: {}", args.input).into());
    }

    // Determine language
    let language = args.language.clone()
        .or_else(|| detect_language(input_path))
        .ok_or("Could not detect language. Specify with --language")?;

    // Read source
    let source = fs::read_to_string(input_path)?;
    
    // Create validation context
    let ctx = ValidationContext::for_file(
        input_path.display().to_string(),
        source.clone(),
        language.clone(),
    );

    // Build validation pipeline
    let contracts_dir = get_contracts_dir();
    
    let pipeline = ValidationPipeline::new()
        .add_layer(SupplyChainLayer::new())
        .add_layer(SecurityLayer::new())
        .add_layer(SyntaxLayer::new())
        .add_layer(ClippyLayer::new())
        .add_layer(SemanticLayer::new())
        .add_layer(ScopeAnalysisLayer::new())
        .add_layer(LogicLayer::new())
        .add_layer(ContractLayer::with_path(&contracts_dir))
        .add_layer(ComplexityLayer::default())
        .add_layer(StyleLayer::new());

    println!("Validating: {} ({})", args.input, language);

    // Run validation
    let result = pipeline.execute(&ctx).await;

    // Report results
    let errors = result.results.iter()
        .flat_map(|(_, r)| &r.violations)
        .filter(|v| v.severity == aether_validation::Severity::Error)
        .count();
    let warnings = result.results.iter()
        .flat_map(|(_, r)| &r.violations)
        .filter(|v| v.severity == aether_validation::Severity::Warning)
        .count();

    // Output violations
    for (layer_name, layer_result) in &result.results {
        for violation in &layer_result.violations {
            let severity = match violation.severity {
                aether_validation::Severity::Error => "ERROR",
                aether_validation::Severity::Warning => "WARN",
                aether_validation::Severity::Info => "INFO",
                aether_validation::Severity::Hint => "HINT",
            };
            let count_str = if violation.count > 1 {
                format!(" ({} times)", violation.count)
            } else {
                String::new()
            };
            println!("  [{}] {} {}: {}{}", layer_name, severity, violation.id, violation.message, count_str);
            
            if args.verbose {
                if let Some(ref suggestion) = violation.suggestion {
                    println!("         Suggestion: {}", suggestion);
                }
            }
        }
    }

    // Summary
    println!();
    if result.all_passed() {
        println!("✓ Validation passed (0 violations)");
    } else {
        println!("✗ Validation failed: {} errors, {} warnings", errors, warnings);
    }

    if let Some(layer) = result.stopped_at {
        println!("  Pipeline stopped at layer: {}", layer);
    }

    // Load and evaluate contracts
    let contracts_dir = get_contracts_dir();
    
    // Load contracts from multiple sources:
    // 1. {language}.yaml in root
    // 2. {language}/ directory
    // 3. imported/imported_{language}.yaml
    // 4. imported/imported_all.yaml (security rules)
    let mut all_contract_rules = Vec::new();
    
    let loader = ContractLoader::new(&contracts_dir);
    
    // 1. Root language file
    if contracts_dir.join(format!("{}.yaml", language)).exists() {
        if let Ok(rules) = loader.load(format!("{}.yaml", language)) {
            all_contract_rules.extend(rules);
        }
    }
    
    // 2. Language subdirectory
    let lang_dir = contracts_dir.join(&language);
    if lang_dir.exists() && lang_dir.is_dir() {
        if let Ok(rules) = loader.load_dir(&language) {
            all_contract_rules.extend(rules);
        }
    }
    
    // 3. Imported contracts for this language
    let imported_lang_path = PathBuf::from("imported").join(format!("imported_{}.yaml", language));
    if contracts_dir.join(&imported_lang_path).exists() {
        if let Ok(rules) = loader.load(&imported_lang_path) {
            all_contract_rules.extend(rules);
        }
    }
    
    // 4. Imported universal security rules
    let imported_all_path = PathBuf::from("imported").join("imported_all.yaml");
    if contracts_dir.join(&imported_all_path).exists() {
        if let Ok(rules) = loader.load(&imported_all_path) {
            all_contract_rules.extend(rules);
        }
    }
    
    // Deduplicate contracts by ID
    {
        use std::collections::HashSet;
        let mut seen_ids = HashSet::new();
        let mut unique_rules = Vec::new();
        for contract in all_contract_rules {
            if seen_ids.insert(contract.id.clone()) {
                unique_rules.push(contract);
            }
        }
        all_contract_rules = unique_rules;
    }
    
    if !all_contract_rules.is_empty() {
        let mut evaluator = RuleEvaluator::new();
        let mut contract_violations = Vec::new();
        
        for contract in &all_contract_rules {
            for rule in &contract.rules {
                if let Ok(violations) = evaluator.evaluate(rule, &source) {
                    for v in violations {
                        contract_violations.push((contract.id.clone(), contract.name.clone(), v));
                    }
                }
            }
        }
        
        // Deduplicate
        contract_violations = deduplicate_contract_violations(contract_violations);
        
        if !contract_violations.is_empty() {
            println!("\nContract Violations ({} rules loaded):", all_contract_rules.len());
            for (id, name, v) in &contract_violations {
                let count_str = if v.count > 1 {
                    format!(" ({} times)", v.count)
                } else {
                    String::new()
                };
                println!("  ⚠ [{}] {} - {}{}", id, name, v.message, count_str);
                
                if args.verbose {
                    if let Some(ref suggestion) = v.suggestion {
                        println!("         Suggestion: {}", suggestion);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Deduplicate contract violations by pattern
fn deduplicate_contract_violations(
    violations: Vec<(String, String, Violation)>
) -> Vec<(String, String, Violation)> {
    use std::collections::HashMap;

    // Normalize pattern for comparison
    fn normalize_pattern(p: &str) -> String {
        let p = p.to_lowercase();
        
        // Extract core pattern from regex
        if p.starts_with("regex:") {
            let regex = p.strip_prefix("regex:").unwrap_or(&p);
            let core = regex
                .replace("\\b", "")
                .replace("\\w+", "")
                .replace("\\.", ".")
                .replace("\\(", "(")
                .replace("\\)", ")")
                .replace(".*", "")
                .replace(".+", "");
            return core.chars().filter(|c| c.is_alphanumeric()).collect();
        }
        
        // Remove composite pattern wrappers
        if p.starts_with("and:[") || p.starts_with("or:[") {
            let inner = p.trim_start_matches("and:[").trim_start_matches("or:[");
            if let Some(first) = inner.split(',').next() {
                return normalize_pattern(first.trim());
            }
        }
        
        // Simple pattern: keep only alphanumeric
        p.chars().filter(|c| c.is_alphanumeric()).collect()
    }

    let mut groups: HashMap<String, (String, String, Violation)> = HashMap::new();

    for (contract_id, contract_name, mut v) in violations {
        let key = normalize_pattern(&v.id);

        groups.entry(key)
            .and_modify(|(_, _, existing)| {
                existing.count += 1;
            })
            .or_insert((contract_id, contract_name, v));
    }

    groups.into_values().collect()
}
