//! Project Config - `.synward.toml` file parsing and merge
//!
//! This module handles the user-editable project configuration file.
//! It's loaded from `.synward.toml` in the project root and merged with
//! the learned config from memory.
//!
//! ## File Location
//!
//! - `<project_root>/.synward.toml` - Project-specific config
//! - `<project_root>/.synward/config.toml` - Alternative location (preferred for multi-file)
//!
//! ## Example .synward.toml
//!
//! ```toml
//! # Synward Project Configuration
//! # This file overrides learned defaults
//!
//! [thresholds]
//! "complexity.max_cyclomatic" = 15.0  # Allow higher complexity
//! "function.max_lines" = 100.0        # Allow longer functions
//!
//! [whitelist]
//! # Patterns that are OK in this project
//! [[whitelist.entries]]
//! pattern_id = "SEC001"
//! file_pattern = "legacy/**/*.rs"
//! reason = "Legacy code, refactoring in progress"
//!
//! [style]
//! naming.function_style = "snake_case"
//! formatting.max_line_length = 120
//!
//! [rules]
//! # Custom project-specific rules
//! [[rules.custom]]
//! id = "PROJ001"
//! description = "No direct database access in handlers"
//! pattern = "db\\."
//! is_regex = true
//! severity = "warning"
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;

use super::{LearnedConfig, WhitelistedPattern, CustomRule, StyleConventions};
use super::validation_state::Severity;
use crate::error::{Error, Result};
use chrono::Utc;

/// Project configuration loaded from `.synward.toml`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    /// Config file version for migration support
    pub version: Option<String>,

    /// Layer-specific threshold overrides
    #[serde(default)]
    pub thresholds: HashMap<String, f32>,

    /// Dubbioso Mode configuration (confidence-based validation)
    #[serde(default)]
    pub dubbioso: DubbiosoSection,

    /// Whitelisted patterns (security exceptions)
    #[serde(default)]
    pub whitelist: WhitelistSection,

    /// Style conventions
    #[serde(default)]
    pub style: StyleSection,

    /// Custom validation rules
    #[serde(default)]
    pub rules: RulesSection,

    /// Project metadata
    #[serde(default)]
    pub project: ProjectMetadataSection,
}

/// Dubbioso Mode section in .synward.toml
///
/// Configures confidence-based validation thresholds.
/// When confidence is low, questions are asked instead of failing blindly.
///
/// ## Example
///
/// ```toml
/// [dubbioso]
/// preset = "fast"  # strict, balanced, fast, turbo
/// ```
///
/// Or override specific values:
///
/// ```toml
/// [dubbioso]
/// preset = "balanced"
/// ask_threshold = 0.50  # Override preset default
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubbiosoSection {
    /// Preset level (strict, balanced, fast, turbo)
    /// If set, provides default values for all thresholds.
    #[serde(default)]
    pub preset: Option<crate::dubbioso::DubbiosoPreset>,

    /// Ask questions below this threshold (0.0-1.0)
    #[serde(default = "default_ask_threshold")]
    pub ask_threshold: f64,

    /// Warn but continue below this threshold
    #[serde(default = "default_warn_threshold")]
    pub warn_threshold: f64,

    /// Auto-accept above this threshold
    #[serde(default = "default_auto_accept_threshold")]
    pub auto_accept_threshold: f64,

    /// Make pattern permanent after N acceptances
    #[serde(default = "default_permanent_after")]
    pub permanent_after: u32,

    /// Maximum depth for graph context traversal
    #[serde(default = "default_max_context_depth")]
    pub max_context_depth: usize,
}

fn default_ask_threshold() -> f64 { 0.60 }
fn default_warn_threshold() -> f64 { 0.80 }
fn default_auto_accept_threshold() -> f64 { 0.95 }
fn default_permanent_after() -> u32 { 5 }
fn default_max_context_depth() -> usize { 3 }

impl Default for DubbiosoSection {
    fn default() -> Self {
        // Use Balanced preset as default
        let preset_config = crate::dubbioso::DubbiosoConfig::from(
            crate::dubbioso::DubbiosoPreset::Balanced
        );
        Self {
            preset: Some(crate::dubbioso::DubbiosoPreset::Balanced),
            ask_threshold: preset_config.ask_threshold,
            warn_threshold: preset_config.warn_threshold,
            auto_accept_threshold: preset_config.auto_accept_threshold,
            permanent_after: preset_config.permanent_after,
            max_context_depth: preset_config.max_context_depth,
        }
    }
}

impl From<DubbiosoSection> for crate::dubbioso::DubbiosoConfig {
    fn from(section: DubbiosoSection) -> Self {
        // Use explicit values from .synward.toml
        // (preset is just documentation/user-hint, values are authoritative)
        Self {
            ask_threshold: section.ask_threshold,
            warn_threshold: section.warn_threshold,
            auto_accept_threshold: section.auto_accept_threshold,
            permanent_after: section.permanent_after,
            max_context_depth: section.max_context_depth,
        }
    }
}

/// Whitelist section in .synward.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhitelistSection {
    /// Whitelisted pattern entries
    #[serde(default)]
    pub entries: Vec<WhitelistEntry>,
}

/// A single whitelist entry from .synward.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    /// Pattern ID (rule code)
    pub pattern_id: String,

    /// File pattern (glob)
    pub file_pattern: String,

    /// Reason for whitelisting
    pub reason: String,

    /// Expiration date (optional)
    pub expires_at: Option<String>,
}

/// Style section in .synward.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StyleSection {
    /// Naming conventions
    #[serde(default)]
    pub naming: NamingSection,

    /// Formatting conventions
    #[serde(default)]
    pub formatting: FormattingSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamingSection {
    pub function_style: Option<String>,
    pub variable_style: Option<String>,
    pub struct_style: Option<String>,
    pub constant_style: Option<String>,
    pub function_prefixes: Option<Vec<String>>,
    pub type_suffixes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormattingSection {
    pub max_line_length: Option<usize>,
    pub indent_size: Option<usize>,
    pub use_tabs: Option<bool>,
    pub max_blank_lines: Option<usize>,
    pub trailing_comma: Option<bool>,
}

/// Rules section in .synward.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RulesSection {
    /// Custom rules
    #[serde(default)]
    pub custom: Vec<CustomRuleEntry>,
}

/// A custom rule from .synward.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRuleEntry {
    pub id: String,
    pub description: String,
    pub pattern: String,
    #[serde(default)]
    pub is_regex: bool,
    pub severity: String,
    #[serde(default)]
    pub applies_to: Vec<String>,
}

/// Project metadata section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMetadataSection {
    pub name: Option<String>,
    pub language: Option<String>,
    pub framework: Option<String>,
}

impl ProjectConfig {
    /// Load project config from `.synward.toml` in project root
    pub fn load(project_root: &Path) -> Result<Option<Self>> {
        // Try primary location
        let config_path = project_root.join(".synward.toml");
        
        // Try alternative location
        let alt_path = project_root.join(".synward").join("config.toml");
        
        let (path, content) = if config_path.exists() {
            let content = fs::read_to_string(&config_path).map_err(Error::Io)?;
            (config_path, content)
        } else if alt_path.exists() {
            let content = fs::read_to_string(&alt_path).map_err(Error::Io)?;
            (alt_path, content)
        } else {
            return Ok(None);
        };

        let config: ProjectConfig = toml::from_str(&content)
            .map_err(|e| Error::Toml(format!("Failed to parse {}: {}", path.display(), e)))?;

        tracing::info!("Loaded project config from {:?}", path);
        Ok(Some(config))
    }

    /// Save project config to `.synward.toml`
    pub fn save(&self, project_root: &Path) -> Result<()> {
        let config_path = project_root.join(".synward.toml");
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Toml(e.to_string()))?;
        
        fs::write(&config_path, content).map_err(Error::Io)?;
        
        tracing::info!("Saved project config to {:?}", config_path);
        Ok(())
    }

    /// Merge project config into learned config
    ///
    /// Project config takes precedence for explicitly set values.
    /// Learned config provides defaults for unset values.
    pub fn merge_into(&self, learned: &mut LearnedConfig) {
        // Merge thresholds (project config overrides)
        for (key, value) in &self.thresholds {
            learned.thresholds.insert(key.clone(), *value);
        }

        // Merge whitelist
        for entry in &self.whitelist.entries {
            let whitelisted = WhitelistedPattern {
                pattern_id: entry.pattern_id.clone(),
                file_pattern: entry.file_pattern.clone(),
                reason: entry.reason.clone(),
                approved_by: "project-config".to_string(),
                approved_at: Utc::now(),
                expires_at: entry.expires_at.as_ref().and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                }),
            };
            
            // Remove existing entry for same pattern+file
            learned.security_whitelist.retain(|w| {
                !(w.pattern_id == whitelisted.pattern_id && w.file_pattern == whitelisted.file_pattern)
            });
            learned.security_whitelist.push(whitelisted);
        }

        // Merge style conventions
        self.merge_style(&mut learned.conventions);

        // Merge custom rules
        for rule in &self.rules.custom {
            let custom = CustomRule {
                id: rule.id.clone(),
                description: rule.description.clone(),
                pattern: rule.pattern.clone(),
                is_regex: rule.is_regex,
                severity: parse_severity(&rule.severity),
                applies_to: if rule.applies_to.is_empty() {
                    vec!["*".to_string()]
                } else {
                    rule.applies_to.clone()
                },
                observation_count: 0,
                confidence: 1.0, // User-defined rules have max confidence
            };
            
            // Remove existing rule with same ID
            learned.custom_rules.retain(|r| r.id != custom.id);
            learned.custom_rules.push(custom);
        }
    }

    /// Merge style section into conventions
    fn merge_style(&self, conventions: &mut StyleConventions) {
        // Naming
        if let Some(style) = &self.style.naming.function_style {
            conventions.naming.function_style = Some(style.clone());
        }
        if let Some(style) = &self.style.naming.variable_style {
            conventions.naming.variable_style = Some(style.clone());
        }
        if let Some(style) = &self.style.naming.struct_style {
            conventions.naming.struct_style = Some(style.clone());
        }
        if let Some(style) = &self.style.naming.constant_style {
            conventions.naming.constant_style = Some(style.clone());
        }
        if let Some(prefixes) = &self.style.naming.function_prefixes {
            conventions.naming.function_prefixes = prefixes.clone();
        }
        if let Some(suffixes) = &self.style.naming.type_suffixes {
            conventions.naming.type_suffixes = suffixes.clone();
        }

        // Formatting
        if let Some(len) = self.style.formatting.max_line_length {
            conventions.formatting.max_line_length = Some(len);
        }
        if let Some(size) = self.style.formatting.indent_size {
            conventions.formatting.indent_size = Some(size);
        }
        if let Some(tabs) = self.style.formatting.use_tabs {
            conventions.formatting.use_tabs = Some(tabs);
        }
        if let Some(lines) = self.style.formatting.max_blank_lines {
            conventions.formatting.max_blank_lines = Some(lines);
        }
        if let Some(comma) = self.style.formatting.trailing_comma {
            conventions.formatting.trailing_comma = Some(comma);
        }
    }

    /// Create default project config template
    pub fn template() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("complexity.max_cyclomatic".to_string(), 10.0);
        thresholds.insert("function.max_lines".to_string(), 50.0);

        Self {
            version: Some("1.0".to_string()),
            thresholds,
            dubbioso: DubbiosoSection::default(),
            whitelist: WhitelistSection::default(),
            style: StyleSection::default(),
            rules: RulesSection::default(),
            project: ProjectMetadataSection::default(),
        }
    }

    /// Convert project config to a LearnedConfig for export
    pub fn to_learned_config(&self) -> LearnedConfig {
        let mut config = LearnedConfig::defaults();
        self.merge_into(&mut config);
        config
    }
}

/// Parse severity string into enum
fn parse_severity(s: &str) -> Severity {
    match s.to_lowercase().as_str() {
        "info" | "note" => Severity::Info,
        "warning" | "warn" => Severity::Warning,
        "error" | "err" => Severity::Error,
        _ => Severity::Warning,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let toml = r#"
        version = "1.0"
        
        [thresholds]
        "complexity.max_cyclomatic" = 15.0
        "function.max_lines" = 100.0
        
        [project]
        name = "test-project"
        language = "rust"
        "#;

        let config: ProjectConfig = toml::from_str(toml).unwrap();
        
        assert_eq!(config.version, Some("1.0".to_string()));
        assert_eq!(config.thresholds.get("complexity.max_cyclomatic"), Some(&15.0));
        assert_eq!(config.thresholds.get("function.max_lines"), Some(&100.0));
        assert_eq!(config.project.name, Some("test-project".to_string()));
    }

    #[test]
    fn test_parse_whitelist() {
        let toml = r#"
        [[whitelist.entries]]
        pattern_id = "SEC001"
        file_pattern = "test_*.rs"
        reason = "Test files"
        
        [[whitelist.entries]]
        pattern_id = "SEC002"
        file_pattern = "legacy/**/*.rs"
        reason = "Legacy code"
        expires_at = "2025-12-31T00:00:00Z"
        "#;

        let config: ProjectConfig = toml::from_str(toml).unwrap();
        
        assert_eq!(config.whitelist.entries.len(), 2);
        assert_eq!(config.whitelist.entries[0].pattern_id, "SEC001");
        assert_eq!(config.whitelist.entries[1].expires_at, Some("2025-12-31T00:00:00Z".to_string()));
    }

    #[test]
    fn test_merge_into_learned_config() {
        let mut learned = LearnedConfig::defaults();
        
        let project = ProjectConfig {
            thresholds: {
                let mut m = HashMap::new();
                m.insert("complexity.max_cyclomatic".to_string(), 20.0);
                m
            },
            ..Default::default()
        };

        project.merge_into(&mut learned);
        
        // Project threshold overrides learned
        assert_eq!(learned.thresholds.get("complexity.max_cyclomatic"), Some(&20.0));
        // Learned default still present for others
        assert_eq!(learned.thresholds.get("function.max_lines"), Some(&50.0));
    }

    #[test]
    fn test_template_generation() {
        let template = ProjectConfig::template();
        
        assert!(template.version.is_some());
        assert!(!template.thresholds.is_empty());
    }
}
