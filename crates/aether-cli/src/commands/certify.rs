//! Certify command

use crate::commands::CertifyArgs;
use std::path::Path;
use std::fs;

use aether_validation::{ValidationPipeline, ValidationContext};
use aether_validation::layers::{SyntaxLayer, SemanticLayer, LogicLayer, SecurityLayer, ComplexityLayer, SupplyChainLayer, ClippyLayer};
use aether_certification::{Keypair, Certificate, ValidationResult, AgentInfo};

/// Detect language from file extension
fn detect_language(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some("rust".to_string()),
        "cpp" | "cc" | "cxx" | "hpp" => Some("cpp".to_string()),
        "py" => Some("python".to_string()),
        "js" | "ts" => Some("javascript".to_string()),
        _ => None,
    }
}

pub async fn run(args: CertifyArgs) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = Path::new(&args.input);
    let output_path = Path::new(&args.output);
    
    if !input_path.exists() {
        return Err(format!("Input path does not exist: {}", args.input).into());
    }

    // Determine language
    let language = args.language.clone()
        .or_else(|| detect_language(input_path))
        .ok_or("Could not detect language. Please specify --language")?;

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
        .add_layer(ClippyLayer::new())
        .add_layer(SemanticLayer::new())
        .add_layer(LogicLayer::new())
        .add_layer(ComplexityLayer::default());

    println!("Validating: {} ({})", args.input, language);

    // Run validation
    let start = std::time::Instant::now();
    let result = pipeline.execute(&ctx).await;
    let duration = start.elapsed();

    if !result.all_passed() {
        let errors = result.results.iter()
            .flat_map(|(_, r)| &r.violations)
            .filter(|v| v.severity == aether_validation::Severity::Error)
            .count();
        println!("✗ Validation failed with {} errors", errors);
        return Err("Validation failed".into());
    }

    // Create certificate
    let file_hash = Certificate::hash_file(source.as_bytes());
    
    let cert_result = ValidationResult {
        passed: true,
        total_violations: result.total_violations(),
        errors: 0,
        warnings: result.results.iter()
            .flat_map(|(_, r)| &r.violations)
            .filter(|v| v.severity == aether_validation::Severity::Warning)
            .count(),
        duration_ms: duration.as_millis() as u64,
    };

    let mut cert = Certificate::new(
        file_hash,
        cert_result,
        AgentInfo {
            name: "aether".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    );

    // Sign certificate
    let keypair = if let Some(ref key_path) = args.key {
        // Load existing key
        let key_bytes = fs::read(key_path)?;
        let key_array: [u8; 64] = key_bytes.try_into()
            .map_err(|_| "Invalid key file: expected 64 bytes")?;
        Keypair::from_bytes(&key_array)?
    } else {
        // Generate new key
        Keypair::generate()
    };

    keypair.sign_certificate(&mut cert)?;

    // Save certificate
    let cert_json = serde_json::to_string_pretty(&cert)?;
    fs::write(output_path, &cert_json)?;

    println!("✓ Certificate created: {}", args.output);
    println!("  ID: {}", cert.id);
    println!("  Hash: {}...", &cert.file_hash[..16]);
    println!("  Duration: {}ms", duration.as_millis());

    Ok(())
}
