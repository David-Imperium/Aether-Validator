//! Validate command

use crate::commands::ValidateArgs;
use std::path::Path;
use std::fs;

use aether_validation::{ValidationPipeline, ValidationContext};
use aether_validation::layers::{SyntaxLayer, SemanticLayer, LogicLayer, SecurityLayer, ComplexityLayer, SupplyChainLayer};

/// Detect language from file extension
fn detect_language(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some("rust".to_string()),
        "py" => Some("python".to_string()),
        "js" | "ts" => Some("javascript".to_string()),
        _ => None,
    }
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
    let pipeline = ValidationPipeline::new()
        .add_layer(SupplyChainLayer::new())
        .add_layer(SecurityLayer::new())
        .add_layer(SyntaxLayer::new())
        .add_layer(SemanticLayer::new())
        .add_layer(LogicLayer::new())
        .add_layer(ComplexityLayer::default());

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
            println!("  [{}] {} {}: {}", layer_name, severity, violation.id, violation.message);
            
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

    Ok(())
}
