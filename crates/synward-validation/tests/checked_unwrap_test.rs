//! Test cases for checked unwrap detection (should reduce false positives)

use std::path::PathBuf;
use synward_validation::{ValidationPipeline, ValidationContext};
use synward_validation::layers::LogicLayer;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[tokio::test]
async fn test_checked_unwrap_not_flagged() {
    let checked_path = fixtures_dir().join("checked_unwrap_cases.rs");
    let source = std::fs::read_to_string(&checked_path)
        .expect("Failed to read checked_unwrap_cases.rs");

    let ctx = ValidationContext::for_file(
        checked_path.display().to_string(),
        source.clone(),
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(LogicLayer::new());

    let result = pipeline.execute(&ctx).await;

    // Get all LOGIC010 violations (unwrap violations)
    let unwrap_violations: Vec<_> = result.results.iter()
        .flat_map(|(_, r)| &r.violations)
        .filter(|v| v.id == "LOGIC010")
        .collect();

    println!("Unwrap violations found: {:?}", 
        unwrap_violations.iter().map(|v| (v.span.as_ref().map(|s| s.line), &v.message)).collect::<Vec<_>>());

    // We should have very few or no unwrap violations for this code
    // All the unwrap calls are either:
    // 1. In test code (stripped)
    // 2. Have SAFETY comments
    // 3. Have explicit checks (is_some, is_ok, etc.)
    // 4. Are infallible operations
    // 5. Use expect() instead
    
    // Allow some violations for patterns we haven't fully implemented yet
    // But should be significantly less than 11 (number of unwrap calls)
    assert!(unwrap_violations.len() <= 3, 
        "Should have at most 3 unwrap violations for checked patterns, found {}.\n\
        Violations: {:?}",
        unwrap_violations.len(),
        unwrap_violations.iter().map(|v| (v.span.as_ref().map(|s| s.line), &v.message)).collect::<Vec<_>>()
    );
}

#[tokio::test]  
async fn test_unsafe_with_safety_comment_not_flagged() {
    let source = r#"
fn safe_wrapper(data: &[u8]) -> u32 {
    // SAFETY: data.len() >= 4 guaranteed by caller validation
    unsafe {
        std::ptr::read_unaligned(data.as_ptr() as *const u32)
    }
}
"#;

    let ctx = ValidationContext::for_file(
        "safe_unsafe.rs".to_string(),
        source.to_string(),
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(LogicLayer::new());

    let result = pipeline.execute(&ctx).await;

    // Should NOT have unsafe violation due to SAFETY comment
    let unsafe_violations: Vec<_> = result.results.iter()
        .flat_map(|(_, r)| &r.violations)
        .filter(|v| v.id == "LOGIC030")
        .collect();

    assert!(unsafe_violations.is_empty(),
        "Should not flag unsafe block with SAFETY comment. Violations: {:?}",
        unsafe_violations
    );
}

#[tokio::test]
async fn test_unsafe_without_safety_comment_flagged() {
    let source = r#"
fn unsafe_wrapper(data: &[u8]) -> u32 {
    unsafe {
        std::ptr::read_unaligned(data.as_ptr() as *const u32)
    }
}
"#;

    let ctx = ValidationContext::for_file(
        "unsafe_without_comment.rs".to_string(),
        source.to_string(),
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(LogicLayer::new());

    let result = pipeline.execute(&ctx).await;

    // SHOULD have unsafe violation (no SAFETY comment)
    let unsafe_violations: Vec<_> = result.results.iter()
        .flat_map(|(_, r)| &r.violations)
        .filter(|v| v.id == "LOGIC030")
        .collect();

    assert!(!unsafe_violations.is_empty(),
        "Should flag unsafe block without SAFETY comment"
    );
}

#[tokio::test]
async fn test_unwrap_in_test_code_not_flagged() {
    let source = r#"
fn production_code() -> Option<i32> { Some(42) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production() {
        let result = production_code().unwrap();  // OK in tests
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_async() {
        let result = production_code().unwrap();  // OK in tests
        assert_eq!(result, 42);
    }
}
"#;

    let ctx = ValidationContext::for_file(
        "test_code.rs".to_string(),
        source.to_string(),
        "rust".to_string(),
    );

    let pipeline = ValidationPipeline::new()
        .add_layer(LogicLayer::new());

    let result = pipeline.execute(&ctx).await;

    // Should NOT have unwrap violations in test code
    let unwrap_violations: Vec<_> = result.results.iter()
        .flat_map(|(_, r)| &r.violations)
        .filter(|v| v.id == "LOGIC010")
        .collect();

    assert!(unwrap_violations.is_empty(),
        "Should not flag unwrap in test code. Violations: {:?}",
        unwrap_violations
    );
}
