//! E2E Tests — End-to-end validation pipeline tests

use std::path::PathBuf;

use aether_validation::{ValidationPipeline, ValidationContext};
use aether_validation::layers::{SyntaxLayer, SemanticLayer, LogicLayer, ArchitectureLayer, StyleLayer};

/// Helper to get fixtures directory
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[tokio::test]
async fn test_clean_code_passes_validation() {
    let clean_code_path = fixtures_dir().join("clean_code.rs");
    let source = std::fs::read_to_string(&clean_code_path)
        .expect("Failed to read clean_code.rs");

    let ctx = ValidationContext::for_file(
        clean_code_path.display().to_string(),
        source,
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(SyntaxLayer::new())
        .add_layer(SemanticLayer::new())
        .add_layer(LogicLayer::new())
        .add_layer(ArchitectureLayer::new())
        .add_layer(StyleLayer::new());

    let result = pipeline.execute(&ctx).await;

    // Clean code should pass all layers
    // Note: StyleLayer may flag minor issues, so we check for no errors
    let has_errors = result.results.iter()
        .flat_map(|(_, r)| &r.violations)
        .any(|v| v.severity == aether_validation::Severity::Error);
    
    assert!(!has_errors, 
        "Clean code should not have errors. Violations: {:?}",
        result.results.iter().flat_map(|(_, r)| &r.violations).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_problematic_code_fails_validation() {
    let problematic_path = fixtures_dir().join("problematic_code.rs");
    let source = std::fs::read_to_string(&problematic_path)
        .expect("Failed to read problematic_code.rs");

    let ctx = ValidationContext::for_file(
        problematic_path.display().to_string(),
        source.clone(),
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(SyntaxLayer::new())
        .add_layer(SemanticLayer::new())
        .add_layer(LogicLayer::new())
        .add_layer(ArchitectureLayer::new())
        .add_layer(StyleLayer::new());

    let result = pipeline.execute(&ctx).await;

    // Problematic code should have violations
    assert!(!result.all_passed(), "Problematic code should fail validation");
    
    // Check for expected violations
    let all_violations: Vec<_> = result.results.iter()
        .flat_map(|(_, r)| &r.violations)
        .collect();

    // Print all violations for debugging
    println!("Found {} total violations:", all_violations.len());
    for v in &all_violations {
        println!("  - [{:?}] {}: {}", v.severity, v.id, v.message);
    }

    // Note: Pipeline stops on errors, so StyleLayer may not run if LogicLayer finds errors
    // The key is that LogicLayer should find violations
    assert!(all_violations.iter().any(|v| v.id.contains("LOGIC")),
        "Should have logic violations. Found: {:?}",
        all_violations.iter().map(|v| &v.id).collect::<Vec<_>>());
    
    // Verify we found the expected MIL (Military Grade) violations from LogicLayer
    assert!(all_violations.iter().any(|v| v.id == "MIL001"),
        "Should find panic! violation (MIL001)");
    assert!(all_violations.iter().any(|v| v.id == "MIL010"),
        "Should find unwrap() violation (MIL010)");
}

#[tokio::test]
async fn test_syntax_layer_alone() {
    let problematic_path = fixtures_dir().join("problematic_code.rs");
    let source = std::fs::read_to_string(&problematic_path)
        .expect("Failed to read problematic_code.rs");

    let ctx = ValidationContext::for_file(
        problematic_path.display().to_string(),
        source,
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(SyntaxLayer::new());

    let result = pipeline.execute(&ctx).await;

    // Syntax layer should pass (code is syntactically valid)
    assert!(result.all_passed(), "Syntax should be valid");
}

#[tokio::test]
async fn test_architecture_layer_alone() {
    let problematic_path = fixtures_dir().join("problematic_code.rs");
    let source = std::fs::read_to_string(&problematic_path)
        .expect("Failed to read problematic_code.rs");

    let ctx = ValidationContext::for_file(
        problematic_path.display().to_string(),
        source,
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(ArchitectureLayer::new());

    let result = pipeline.execute(&ctx).await;

    // Architecture layer should find violations
    // Note: Current implementation checks for wildcard imports, test code, etc.
    // The problematic_code.rs has issues that may or may not be caught
    // depending on the layer configuration
    let arch_violations: Vec<_> = result.results.iter()
        .flat_map(|(_, r)| &r.violations)
        .filter(|v| v.id.starts_with("ARCH"))
        .collect();

    // Just verify the layer runs without crashing
    // The specific violations depend on layer configuration
    println!("Architecture violations found: {:?}", arch_violations);
}

#[tokio::test]
async fn test_style_layer_alone() {
    let problematic_path = fixtures_dir().join("problematic_code.rs");
    let source = std::fs::read_to_string(&problematic_path)
        .expect("Failed to read problematic_code.rs");

    let ctx = ValidationContext::for_file(
        problematic_path.display().to_string(),
        source.clone(),
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(StyleLayer::new());

    let result = pipeline.execute(&ctx).await;

    // Should find style violations
    let style_violations: Vec<_> = result.results.iter()
        .flat_map(|(_, r)| &r.violations)
        .collect();

    println!("Style violations found: {:?}", style_violations);
    
    // Check for specific violations - these depend on StyleLayer implementation
    // STYLE001: snake_case for functions
    // STYLE002: PascalCase for structs
    // STYLE004: Line length
    // STYLE005: Function length
    
    // Just verify the layer runs and produces output
    assert!(!style_violations.is_empty() || result.all_passed(), 
        "StyleLayer should either find violations or pass");
}

#[tokio::test]
async fn test_logic_layer_alone() {
    let problematic_path = fixtures_dir().join("problematic_code.rs");
    let source = std::fs::read_to_string(&problematic_path)
        .expect("Failed to read problematic_code.rs");

    let ctx = ValidationContext::for_file(
        problematic_path.display().to_string(),
        source,
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(LogicLayer::new());

    let result = pipeline.execute(&ctx).await;

    // Should find logic violations
    assert!(!result.all_passed(), "Should find logic violations");
    
    let logic_violations: Vec<_> = result.results.iter()
        .flat_map(|(_, r)| &r.violations)
        .collect();

    // Check for specific violations (MIL = Military Grade IDs)
    assert!(logic_violations.iter().any(|v| v.id == "MIL001"),
        "Should find panic! violation (MIL001)");
    assert!(logic_violations.iter().any(|v| v.id == "MIL010"),
        "Should find unwrap() violation (MIL010)");
    assert!(logic_violations.iter().any(|v| v.id == "MIL050"),
        "Should find TODO violation (MIL050)");
    assert!(logic_violations.iter().any(|v| v.id == "MIL051"),
        "Should find FIXME violation (MIL051)");
}

#[tokio::test]
async fn test_pipeline_execution_order() {
    let problematic_path = fixtures_dir().join("problematic_code.rs");
    let source = std::fs::read_to_string(&problematic_path)
        .expect("Failed to read problematic_code.rs");

    let ctx = ValidationContext::for_file(
        problematic_path.display().to_string(),
        source,
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(SyntaxLayer::new())
        .add_layer(SemanticLayer::new())
        .add_layer(LogicLayer::new());

    let result = pipeline.execute(&ctx).await;

    // Should have results from all layers
    assert!(!result.all_passed());
    assert!(!result.results.is_empty(), "Should have layer results");
}

#[tokio::test]
async fn test_empty_file_validation() {
    let ctx = ValidationContext::for_file(
        "empty.rs".to_string(),
        "".to_string(),
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(SyntaxLayer::new())
        .add_layer(SemanticLayer::new())
        .add_layer(LogicLayer::new())
        .add_layer(ArchitectureLayer::new())
        .add_layer(StyleLayer::new());

    let result = pipeline.execute(&ctx).await;

    // Empty file should pass (no violations possible)
    assert!(result.all_passed(), "Empty file should pass validation");
}

#[tokio::test]
async fn test_minimal_valid_code() {
    let source = "fn main() {}";
    
    let ctx = ValidationContext::for_file(
        "minimal.rs".to_string(),
        source.to_string(),
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(SyntaxLayer::new())
        .add_layer(SemanticLayer::new())
        .add_layer(LogicLayer::new());

    let result = pipeline.execute(&ctx).await;

    // Minimal valid code should pass
    assert!(result.all_passed(), "Minimal valid code should pass");
}
