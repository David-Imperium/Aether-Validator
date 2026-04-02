//! E2E Tests — Memory-Driven Core
//!
//! Tests the complete memory-driven validation workflow:
//! 1. Load project config from .synward.toml
//! 2. Load learned config from memory
//! 3. Merge configs
//! 4. Apply to validation layers

use synward_intelligence::memory::{
    LearnedConfig, ProjectConfig, WhitelistedPattern, CustomRule,
    WhitelistSection, WhitelistEntry, RulesSection, CustomRuleEntry,
};
use synward_intelligence::MemoryStore;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_project() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

/// Test: Project config loads from .synward.toml
#[test]
fn test_project_config_loads_from_toml() {
    let dir = setup_test_project();
    let config_path = dir.path().join(".synward.toml");
    
    let toml_content = r#"
version = "1.0"

[thresholds]
"complexity.max_cyclomatic" = 20.0
"function.max_lines" = 100.0

[project]
name = "test-project"
language = "rust"
"#;
    std::fs::write(&config_path, toml_content).expect("Failed to write config");
    
    let config = ProjectConfig::load(dir.path())
        .expect("Failed to load config")
        .expect("Config not found");
    
    assert_eq!(config.version, Some("1.0".to_string()));
    assert_eq!(config.thresholds.get("complexity.max_cyclomatic"), Some(&20.0));
    assert_eq!(config.thresholds.get("function.max_lines"), Some(&100.0));
}

/// Test: Project config merges into LearnedConfig
#[test]
fn test_project_config_merges_into_learned() {
    let mut learned = LearnedConfig::default();
    
    // Set some initial thresholds
    learned.thresholds.insert("complexity.max_cyclomatic".to_string(), 10.0);
    learned.thresholds.insert("function.max_lines".to_string(), 50.0);
    
    // Create project config that overrides
    let mut project_thresholds = HashMap::new();
    project_thresholds.insert("complexity.max_cyclomatic".to_string(), 20.0);
    
    let project = ProjectConfig {
        version: Some("1.0".to_string()),
        thresholds: project_thresholds,
        ..Default::default()
    };
    
    project.merge_into(&mut learned);
    
    // Project should override
    assert_eq!(learned.thresholds.get("complexity.max_cyclomatic"), Some(&20.0));
    // Non-overridden should stay
    assert_eq!(learned.thresholds.get("function.max_lines"), Some(&50.0));
}

/// Test: Whitelist entries from config are applied
#[test]
fn test_whitelist_applied_from_config() {
    let mut learned = LearnedConfig::default();
    
    let project = ProjectConfig {
        version: Some("1.0".to_string()),
        whitelist: WhitelistSection {
            entries: vec![
                WhitelistEntry {
                    pattern_id: "SEC001".to_string(),
                    file_pattern: "legacy/**/*.rs".to_string(),
                    reason: "Legacy code".to_string(),
                    expires_at: None,
                },
            ],
        },
        ..Default::default()
    };
    
    project.merge_into(&mut learned);
    
    assert_eq!(learned.security_whitelist.len(), 1);
    assert_eq!(learned.security_whitelist[0].pattern_id, "SEC001");
}

/// Test: Custom rules from config are applied
#[test]
fn test_custom_rules_applied_from_config() {
    let mut learned = LearnedConfig::default();
    
    let project = ProjectConfig {
        version: Some("1.0".to_string()),
        rules: RulesSection {
            custom: vec![
                CustomRuleEntry {
                    id: "PROJ001".to_string(),
                    description: "No direct DB access".to_string(),
                    pattern: r"db\.".to_string(),
                    is_regex: true,
                    severity: "warning".to_string(),
                    applies_to: vec!["**/*.rs".to_string()],
                },
            ],
        },
        ..Default::default()
    };
    
    project.merge_into(&mut learned);
    
    assert_eq!(learned.custom_rules.len(), 1);
    assert_eq!(learned.custom_rules[0].id, "PROJ001");
}

/// Test: MemoryStore persists and loads LearnedConfig
#[test]
fn test_memory_store_roundtrip() {
    let dir = setup_test_project();
    let project_root = PathBuf::from(dir.path());
    let store = MemoryStore::new(Some(dir.path().join("memory")))
        .expect("Failed to create store");
    
    let mut config = LearnedConfig::default();
    config.project_root = project_root.clone();
    config.thresholds.insert("complexity.max_cyclomatic".to_string(), 15.0);
    config.security_whitelist.push(WhitelistedPattern {
        pattern_id: "SEC001".to_string(),
        file_pattern: "test_*.rs".to_string(),
        reason: "Test files".to_string(),
        approved_by: "test".to_string(),
        approved_at: chrono::Utc::now(),
        expires_at: None,
    });
    
    store.save_config(&config).expect("Failed to save");
    let loaded = store.load_config(&project_root).expect("Failed to load");
    
    assert_eq!(loaded.thresholds.get("complexity.max_cyclomatic"), Some(&15.0));
    assert_eq!(loaded.security_whitelist.len(), 1);
}

/// Test: Template generation creates valid config
#[test]
fn test_template_generation_valid() {
    let template = ProjectConfig::template();
    
    assert!(template.version.is_some());
    assert!(!template.thresholds.is_empty());
    
    // Verify it serializes to valid TOML
    let toml = toml::to_string(&template).expect("Failed to serialize");
    let parsed: ProjectConfig = toml::from_str(&toml).expect("Failed to parse");
    
    assert_eq!(parsed.version, template.version);
}

/// Test: Config file not found returns None
#[test]
fn test_config_not_found_returns_none() {
    let dir = setup_test_project();
    // No .synward.toml created
    
    let result = ProjectConfig::load(dir.path()).expect("Load failed");
    assert!(result.is_none());
}

/// Test: Full workflow - load, merge, apply
#[test]
fn test_full_memory_driven_workflow() {
    let dir = setup_test_project();
    let project_root = PathBuf::from(dir.path());
    
    // 1. Create project config
    let config_path = dir.path().join(".synward.toml");
    let toml_content = r#"
version = "1.0"

[thresholds]
"complexity.max_cyclomatic" = 25.0
"function.max_lines" = 80.0

[[whitelist.entries]]
pattern_id = "SEC001"
file_pattern = "generated/**/*.rs"
reason = "Auto-generated code"
"#;
    std::fs::write(&config_path, toml_content).expect("Failed to write config");
    
    // 2. Create memory store with existing learned config
    let store = MemoryStore::new(Some(dir.path().join("memory")))
        .expect("Failed to create store");
    let mut learned = LearnedConfig::default();
    learned.project_root = project_root.clone();
    learned.thresholds.insert("complexity.max_cyclomatic".to_string(), 10.0);
    learned.custom_rules.push(CustomRule {
        id: "LEARNED001".to_string(),
        description: "Learned rule".to_string(),
        pattern: "todo!()".to_string(),
        is_regex: false,
        severity: synward_intelligence::memory::Severity::Warning,
        applies_to: vec!["**/*.rs".to_string()],
        confidence: 0.8,
        observation_count: 10,
    });
    store.save_config(&learned).expect("Failed to save learned");
    
    // 3. Load project config
    let project = ProjectConfig::load(dir.path())
        .expect("Failed to load")
        .expect("Config not found");
    
    // 4. Merge into learned
    project.merge_into(&mut learned);
    
    // 5. Verify merge
    // Project thresholds override
    assert_eq!(learned.thresholds.get("complexity.max_cyclomatic"), Some(&25.0));
    // Learned rule preserved
    assert_eq!(learned.custom_rules.len(), 1);
    // Whitelist from project added
    assert_eq!(learned.security_whitelist.len(), 1);
}
