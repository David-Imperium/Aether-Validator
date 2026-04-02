//! LearnedConfig — Dynamic configuration learned from validation history
//!
//! This is the core of Memory-Driven validation: instead of static rules,
//! layers receive configuration that evolves based on project patterns.
//!
//! ## What Memory CAN Configure
//!
//! - **Thresholds**: complexity limits, max function length, nesting depth
//! - **Custom Rules**: project-specific patterns discovered over time
//! - **Security Whitelist**: accepted patterns that would normally flag
//! - **Style Conventions**: naming, formatting preferences learned from codebase
//!
//! ## What Memory CANNOT Touch
//!
//! - Parser/AST structure (syntax is non-negotiable)
//! - Base syntax validation (must always pass)
//! - Security hard limits (critical vulnerabilities always flagged)
//! - Pipeline execution order (layers run in fixed sequence)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Unique identifier for a learned configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigId(pub String);

impl Default for ConfigId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

/// Learned configuration that modifies validation behavior
///
/// This is loaded from memory at the start of each validation session
/// and applied to all relevant layers.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearnedConfig {
    /// Unique identifier for this config
    pub id: ConfigId,

    /// Project this config belongs to
    pub project_root: PathBuf,

    /// When this config was last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,

    /// Confidence level (0.0-1.0) based on sample size
    pub confidence: f32,

    /// Layer-specific thresholds
    /// Keys: "complexity.max", "function.length", "nesting.depth", etc.
    pub thresholds: HashMap<String, f32>,

    /// Custom rules discovered from project patterns
    pub custom_rules: Vec<CustomRule>,

    /// Patterns whitelisted by user acceptance
    pub security_whitelist: Vec<WhitelistedPattern>,

    /// Style conventions learned from codebase
    pub conventions: StyleConventions,

    /// Statistics about how this config was learned
    pub stats: ConfigStats,
}

/// A custom validation rule discovered from project patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    /// Unique rule ID
    pub id: String,

    /// Human-readable description
    pub description: String,

    /// Pattern to match (regex or simple string)
    pub pattern: String,

    /// Whether pattern is regex
    pub is_regex: bool,

    /// Severity when violated
    pub severity: super::validation_state::Severity,

    /// Layers this rule applies to
    pub applies_to: Vec<String>,

    /// How many times this pattern was observed
    pub observation_count: usize,

    /// Confidence in this rule (0.0-1.0)
    pub confidence: f32,
}

/// A pattern that was explicitly whitelisted by user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistedPattern {
    /// Pattern ID (e.g., rule code)
    pub pattern_id: String,

    /// File path (glob pattern)
    pub file_pattern: String,

    /// Reason for whitelisting
    pub reason: String,

    /// Who approved this
    pub approved_by: String,

    /// When it was approved
    pub approved_at: chrono::DateTime<chrono::Utc>,

    /// Expiration (optional)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Style conventions learned from analyzing the codebase
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StyleConventions {
    /// Naming conventions
    pub naming: NamingConventions,

    /// Formatting preferences
    pub formatting: FormattingConventions,

    /// Idiom preferences (e.g., prefer `if let` over `match`)
    pub idioms: HashMap<String, String>,

    /// Import organization
    pub imports: ImportConventions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamingConventions {
    /// snake_case, camelCase, PascalCase, kebab-case
    pub function_style: Option<String>,

    /// snake_case, camelCase, PascalCase
    pub variable_style: Option<String>,

    /// snake_case, PascalCase
    pub struct_style: Option<String>,

    /// SCREAMING_SNAKE_CASE, camelCase
    pub constant_style: Option<String>,

    /// Prefix patterns (e.g., "get_", "set_", "is_")
    pub function_prefixes: Vec<String>,

    /// Suffix patterns (e.g., "Error", "Result", "Builder")
    pub type_suffixes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormattingConventions {
    /// Max line length
    pub max_line_length: Option<usize>,

    /// Indent size (spaces)
    pub indent_size: Option<usize>,

    /// Use tabs instead of spaces
    pub use_tabs: Option<bool>,

    /// Max blank lines between items
    pub max_blank_lines: Option<usize>,

    /// Trailing comma preference
    pub trailing_comma: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportConventions {
    /// Group order: std, external, crate, local
    pub group_order: Vec<String>,

    /// Sort alphabetically within groups
    pub sort_alphabetically: Option<bool>,

    /// Separate groups with blank lines
    pub separate_groups: Option<bool>,
}

/// Statistics about config learning
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigStats {
    /// Total validations that contributed to this config
    pub sample_count: usize,

    /// How many times config was applied
    pub application_count: usize,

    /// How many times config was updated
    pub update_count: usize,

    /// Average violations per validation (baseline)
    pub baseline_violation_rate: f32,

    /// Current violation rate (after applying config)
    pub current_violation_rate: f32,
}

impl LearnedConfig {
    /// Create a new empty config for a project
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            id: ConfigId::default(),
            project_root,
            last_updated: chrono::Utc::now(),
            confidence: 0.0,
            thresholds: HashMap::new(),
            custom_rules: Vec::new(),
            security_whitelist: Vec::new(),
            conventions: StyleConventions::default(),
            stats: ConfigStats::default(),
        }
    }

    /// Get a threshold value, with fallback to default
    pub fn threshold(&self, key: &str, default: f32) -> f32 {
        self.thresholds.get(key).copied().unwrap_or(default)
    }

    /// Check if a pattern is whitelisted
    pub fn is_whitelisted(&self, pattern_id: &str, file_path: &str) -> bool {
        self.security_whitelist.iter().any(|w| {
            w.pattern_id == pattern_id
                && glob_match(&w.file_pattern, file_path)
                && w.expires_at.map(|e| e > chrono::Utc::now()).unwrap_or(true)
        })
    }

    /// Get custom rules for a specific layer
    pub fn rules_for_layer(&self, layer_name: &str) -> Vec<&CustomRule> {
        self.custom_rules
            .iter()
            .filter(|r| r.applies_to.iter().any(|l| l == layer_name || l == "*"))
            .collect()
    }

    /// Update confidence based on sample size
    pub fn update_confidence(&mut self) {
        // Confidence grows logarithmically with sample size
        // 10 samples = 0.5, 100 samples = 0.75, 1000 samples = 0.9
        let samples = self.stats.sample_count as f32;
        self.confidence = (1.0 + samples.log10() / 4.0).min(1.0);
    }

    /// Merge with another config (for combining project + user defaults)
    pub fn merge(&mut self, other: &LearnedConfig) {
        // Thresholds: take the stricter value
        for (key, value) in &other.thresholds {
            self.thresholds
                .entry(key.clone())
                .and_modify(|v| *v = v.min(*value))
                .or_insert(*value);
        }

        // Custom rules: add if not present
        for rule in &other.custom_rules {
            if !self.custom_rules.iter().any(|r| r.id == rule.id) {
                self.custom_rules.push(rule.clone());
            }
        }

        // Whitelist: merge
        for w in &other.security_whitelist {
            if !self.security_whitelist.iter().any(|existing| {
                existing.pattern_id == w.pattern_id && existing.file_pattern == w.file_pattern
            }) {
                self.security_whitelist.push(w.clone());
            }
        }

        // Conventions: take non-None values from other
        if other.conventions.naming.function_style.is_some() {
            self.conventions.naming.function_style = other.conventions.naming.function_style.clone();
        }
        if other.conventions.formatting.max_line_length.is_some() {
            self.conventions.formatting.max_line_length = other.conventions.formatting.max_line_length;
        }
    }
}

/// Simple glob pattern matching
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            return text.starts_with(prefix) && text.ends_with(suffix);
        }
    }

    pattern == text
}

/// Default thresholds for new projects
impl LearnedConfig {
    /// Get default config with sensible baseline thresholds
    pub fn defaults() -> Self {
        let mut thresholds = HashMap::new();

        // Complexity thresholds
        thresholds.insert("complexity.max_cyclomatic".to_string(), 10.0);
        thresholds.insert("complexity.max_cognitive".to_string(), 15.0);
        thresholds.insert("complexity.max_nesting".to_string(), 4.0);

        // Function/Method thresholds
        thresholds.insert("function.max_lines".to_string(), 50.0);
        thresholds.insert("function.max_params".to_string(), 5.0);
        thresholds.insert("function.max_returns".to_string(), 3.0);

        // File thresholds
        thresholds.insert("file.max_lines".to_string(), 500.0);
        thresholds.insert("file.max_functions".to_string(), 30.0);

        // Security thresholds
        thresholds.insert("security.max_severity".to_string(), 2.0); // 0=Info, 1=Warning, 2=Error

        Self {
            thresholds,
            confidence: 0.5, // Default config has medium confidence
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_fallback() {
        let config = LearnedConfig::new(PathBuf::from("/test"));
        assert_eq!(config.threshold("complexity.max", 15.0), 15.0);

        let mut config_with_threshold = config;
        config_with_threshold
            .thresholds
            .insert("complexity.max".to_string(), 8.0);
        assert_eq!(
            config_with_threshold.threshold("complexity.max", 15.0),
            8.0
        );
    }

    #[test]
    fn test_whitelist_check() {
        let mut config = LearnedConfig::new(PathBuf::from("/test"));
        config.security_whitelist.push(WhitelistedPattern {
            pattern_id: "SEC001".to_string(),
            file_pattern: "test_*.rs".to_string(),
            reason: "Test file".to_string(),
            approved_by: "admin".to_string(),
            approved_at: chrono::Utc::now(),
            expires_at: None,
        });

        assert!(config.is_whitelisted("SEC001", "test_foo.rs"));
        assert!(!config.is_whitelisted("SEC001", "src/main.rs"));
        assert!(!config.is_whitelisted("SEC002", "test_foo.rs"));
    }

    #[test]
    fn test_confidence_update() {
        let mut config = LearnedConfig::new(PathBuf::from("/test"));

        config.stats.sample_count = 10;
        config.update_confidence();
        assert!(config.confidence >= 0.5);

        config.stats.sample_count = 100;
        config.update_confidence();
        assert!(config.confidence >= 0.75);

        config.stats.sample_count = 1000;
        config.update_confidence();
        assert!(config.confidence >= 0.9);
    }

    #[test]
    fn test_merge_configs() {
        let mut config1 = LearnedConfig::new(PathBuf::from("/project"));
        config1
            .thresholds
            .insert("complexity.max".to_string(), 10.0);

        let mut config2 = LearnedConfig::defaults();
        config2
            .thresholds
            .insert("complexity.max".to_string(), 15.0);
        config2
            .thresholds
            .insert("function.max_lines".to_string(), 50.0);

        config1.merge(&config2);

        // Takes stricter (lower) threshold
        assert_eq!(config1.thresholds.get("complexity.max"), Some(&10.0));
        // Adds missing threshold
        assert_eq!(config1.thresholds.get("function.max_lines"), Some(&50.0));
    }

    #[test]
    fn test_defaults() {
        let defaults = LearnedConfig::defaults();

        assert_eq!(
            defaults.threshold("complexity.max_cyclomatic", 0.0),
            10.0
        );
        assert_eq!(defaults.threshold("function.max_lines", 0.0), 50.0);
    }
}
