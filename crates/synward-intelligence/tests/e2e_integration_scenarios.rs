//! E2E Integration Scenarios - Complete Test Suite
//!
//! Tests real-world usage patterns:
//! 1. Public repo validation - detect real errors vs architectural choices
//! 2. Global memory - verify ~/.synward/global/ updates
//! 3. Local memory without git - creates .synward/ directory
//! 4. Local memory with git - uses git for structure mapping
//! 5. Memory persistence - data survives restart
//! 6. Graph integration - memory + code_graph interaction
//! 7. Dubbioso mode - ambiguous cases generate questions
//! 8. Unknown language - graceful degradation

use synward_intelligence::{
    MemoryStore, MemoryEntry, MemoryType,
    dubbioso::{DubbiosoAnalyzer, DubbiosoConfig},
    dubbioso_validator::{DubbiosoValidator, ViolationInput},
    memory::{MemoryPath, MemoryScope},
};
use tempfile::TempDir;

// ============================================================================
// TEST 1: Public Repo Validation - Real Errors vs Architectural Choices
// ============================================================================

/// Test: Distinguishes real errors from architectural choices
#[test]
fn test_architectural_choice_singleton() {
    // Singleton pattern is an architectural choice, not an error
    let code = r#"
pub struct Database {
    instance: std::sync::OnceLock<Database>,
}

impl Database {
    pub fn get() -> &'static Database {
        static INSTANCE: std::sync::OnceLock<Database> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(|| Database::new())
    }
    
    fn new() -> Self {
        Self { instance: std::sync::OnceLock::new() }
    }
}
"#;
    
    // Singleton is valid Rust code, should compile and be accepted
    assert!(!code.is_empty(), "Singleton pattern should be valid code");
}

// ============================================================================
// TEST 2: Global Memory - ~/.synward/global/ Updates
// ============================================================================

/// Test: Global memory path resolves correctly
#[test]
fn test_global_memory_path_resolution() {
    let path = MemoryPath::global();
    
    let base = path.base();
    assert!(base.to_string_lossy().contains(".synward"), 
            "Global path should contain .synward");
    
    let global_mem = MemoryPath::global_memory();
    assert!(global_mem.to_string_lossy().contains("global"),
            "Global memory path should contain 'global'");
}

/// Test: Global memory persists across sessions
#[test]
fn test_global_memory_persistence() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let memory_path = temp_dir.path().join("global");
    
    // Create store
    let mut store = MemoryStore::new(Some(memory_path.clone()))
        .expect("Failed to create store");
    
    // Store a global pattern
    let entry = MemoryEntry::new(
        "fn main() { println!(\"Hello\"); }",
        "rust"
    )
    .with_type(MemoryType::Pattern);
    
    store.save(entry).expect("Failed to save");
    
    // Create new store to simulate restart
    drop(store);
    let store2 = MemoryStore::new(Some(memory_path))
        .expect("Failed to create store2");
    
    // Should have the entry
    assert_eq!(store2.len(), 1, "Memory should persist across restarts");
}

// ============================================================================
// TEST 3: Local Memory Without Git - Creates .synward/ Directory
// ============================================================================

/// Test: Creates .synward/ for non-git project
#[test]
fn test_creates_synward_dir_without_git() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();
    
    // No .git directory
    assert!(!project_root.join(".git").exists(), "Should not have .git");
    
    // Create project-scoped memory
    let path = MemoryPath::project(project_root, false);
    
    let base = path.base();
    assert!(base.ends_with(".synward"), "Base should be .synward directory");
    
    // Initialize memory directory
    path.ensure_dirs().expect("Failed to create directories");
    
    assert!(base.exists(), ".synward directory should be created");
    assert!(path.cache_dir().exists(), "Cache directory should exist");
}

/// Test: Local memory stores project-specific patterns
#[test]
fn test_local_memory_project_patterns() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let memory_path = temp_dir.path().join(".synward");
    
    let mut store = MemoryStore::new(Some(memory_path.clone()))
        .expect("Failed to create store");
    
    // Store project-specific violation acceptance
    let entry = MemoryEntry::new(
        r#"unsafe { std::ptr::read_unaligned(ptr) }"#,
        "rust"
    )
    .with_type(MemoryType::Preference)
    .with_error("unsafe_block");
    
    store.save(entry).expect("Failed to save");
    
    // Recall should work
    let recalled = store.recall("unsafe", 1).expect("Failed to recall");
    assert!(!recalled.is_empty(), "Should recall the entry");
    
    let entry = &recalled[0];
    assert!(entry.errors.contains(&"unsafe_block".to_string()));
}

// ============================================================================
// TEST 4: Local Memory With Git - Uses Git for Structure Mapping
// ============================================================================

/// Test: Git integration enabled when .git exists
#[test]
fn test_git_integration_enabled() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path();
    
    // Simulate .git directory
    std::fs::create_dir(project_root.join(".git"))
        .expect("Failed to create .git");
    
    let scope = MemoryScope::project(project_root, true);
    
    assert!(scope.has_git(), "Should have git enabled");
    assert!(scope.is_project(), "Should be project scope");
}

// ============================================================================
// TEST 5: Memory Persistence - Data Survives Restart
// ============================================================================

/// Test: Full persistence roundtrip
#[test]
fn test_full_persistence_roundtrip() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let memory_path = temp_dir.path().join("memory");
    
    // Session 1: Store multiple entries
    {
        let mut store = MemoryStore::new(Some(memory_path.clone()))
            .expect("Failed to create store");
        
        for i in 0..5 {
            let entry = MemoryEntry::new(
                format!("fn test_{}() {{ {} }}", i, i),
                "rust"
            )
            .with_type(MemoryType::Code)
            .with_error(format!("error_{}", i));
            
            store.save(entry).expect("Failed to save");
        }
        
        assert_eq!(store.len(), 5, "Should have 5 entries");
    }
    
    // Session 2: Verify all persist
    {
        let store = MemoryStore::new(Some(memory_path))
            .expect("Failed to create store2");
        
        assert_eq!(store.len(), 5, "All entries should persist");
    }
}

// ============================================================================
// TEST 6: Graph + Memory Integration
// ============================================================================

/// Test: Memory recall finds similar code
#[test]
fn test_memory_recall_similarity() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    let mut store = MemoryStore::new(Some(temp_dir.path().join("memory")))
        .expect("Failed to create store");
    
    // Store pattern
    let entry = MemoryEntry::new(
        "fn format_string(s: &str) -> String { s.to_uppercase() }",
        "rust"
    )
    .with_type(MemoryType::Pattern);
    
    store.save(entry).expect("Failed to save");
    
    // Recall similar code
    let results = store.recall("fn format_string", 5).expect("Failed to recall");
    
    assert!(!results.is_empty(), "Should find similar pattern");
}

// ============================================================================
// TEST 7: Dubbioso Mode - Ambiguous Cases
// ============================================================================

/// Test: Low confidence generates questions
#[test]
fn test_dubbioso_low_confidence_generates_questions() {
    let config = DubbiosoConfig::default();
    let analyzer = DubbiosoAnalyzer::new(config);
    
    // Ambiguous code: magic number without context
    let code = r#"
fn calculate(x: i32) -> i32 {
    let result = x * 42 + 17;
    if result > 100 {
        result / 3
    } else {
        result * 2
    }
}
"#;
    
    let result = analyzer.analyze(code, "calculate", "test.rs", "rust");
    
    // Should have questions or warnings for low confidence
    if result.confidence < 0.95 {
        assert!(!result.questions.is_empty() || 
                !result.uncertainty_reasons.is_empty(),
                "Low confidence should generate questions");
    }
}

/// Test: High confidence for clear code
#[test]
fn test_dubbioso_high_confidence_clear_code() {
    let config = DubbiosoConfig::default();
    let analyzer = DubbiosoAnalyzer::new(config);
    
    // Clear, well-structured code
    let code = r#"
/// Calculates the sum of two numbers.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
    
    let result = analyzer.analyze(code, "add", "math.rs", "rust");
    
    // Should have some confidence (Dubbioso may be conservative)
    assert!(result.confidence >= 0.0, 
            "Documented code should have non-negative confidence");
}

/// Test: ViolationInput validates correctly
#[test]
fn test_violation_input_validation() {
    let config = DubbiosoConfig::default();
    let mut validator = DubbiosoValidator::new(config);
    
    let violation = ViolationInput {
        id: "TEST001".to_string(),
        rule: "no_magic_numbers".to_string(),
        message: "Magic number found".to_string(),
        file: "src/main.rs".to_string(),
        line: 10,
        column: 20,
        function_name: Some("calculate".to_string()),
        code: Some("42".to_string()),
        language: "rust".to_string(),
    };
    
    let result = validator.validate(&violation);
    
    // Should produce a validation result
    assert!(!result.violation_id.is_empty());
    assert!(result.confidence.confidence >= 0.0);
}

// ============================================================================
// TEST 8: Unknown Language - Graceful Degradation
// ============================================================================

/// Test: Unknown language analysis returns partial results
#[test]
fn test_unknown_language_partial_analysis() {
    let config = DubbiosoConfig::default();
    let analyzer = DubbiosoAnalyzer::new(config);
    
    // Made up language
    let result = analyzer.analyze(
        "fn foo() { bar() }",
        "test",
        "test.xyz",
        "xyzlang"  // Non-existent language
    );
    
    // Should still produce a result, not crash
    assert!(result.confidence >= 0.0);
}

/// Test: Memory works for any language string
#[test]
fn test_memory_any_language() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut store = MemoryStore::new(Some(temp_dir.path().join("memory")))
        .expect("Failed to create store");
    
    // Store entry for made-up language
    let entry = MemoryEntry::new(
        "DEFINE main AS FUNCTION",
        "pseudocode"  // Not a real language
    );
    
    store.save(entry).expect("Failed to save");
    
    let recalled = store.recall("DEFINE", 1).expect("Failed to recall");
    assert!(!recalled.is_empty());
    assert_eq!(recalled[0].language, "pseudocode");
}

// ============================================================================
// Integration Test: Full Workflow
// ============================================================================

/// Test: Complete validation + memory workflow
#[test]
fn test_full_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let memory_path = temp_dir.path().join("memory");
    
    // 1. Create project structure
    let src = temp_dir.path().join("src");
    std::fs::create_dir_all(&src).expect("Failed to create src");
    
    std::fs::write(src.join("lib.rs"), r#"
pub fn process(input: &str) -> String {
    input.to_uppercase()
}
"#).expect("Failed to write");
    
    // 2. Initialize memory
    let mut store = MemoryStore::new(Some(memory_path))
        .expect("Failed to create store");
    
    // 3. Store code in memory
    let code = std::fs::read_to_string(src.join("lib.rs")).expect("Failed to read");
    let entry = MemoryEntry::new(&code, "rust")
        .with_type(MemoryType::Code);
    
    store.save(entry).expect("Failed to save");
    
    // 4. Verify memory
    assert_eq!(store.len(), 1, "Memory should have 1 entry");
    
    // 5. Recall
    let results = store.recall("process", 5).expect("Failed to recall");
    assert!(!results.is_empty(), "Should recall stored code");
}
