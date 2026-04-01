//! Contract Layer — Load and evaluate YAML contracts
//!
//! This layer loads validation contracts from YAML files and evaluates
//! them against source code. Contracts are organized by language:
//! - contracts/python/*.yaml
//! - contracts/rust/*.yaml
//! - contracts/javascript/*.yaml
//! - etc.

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity, Span};
use regex::Regex;
use std::path::PathBuf;
use std::collections::HashMap;

/// Contract layer - YAML-based rule evaluation.
pub struct ContractLayer {
    /// Cache of loaded contracts by language
    cache: HashMap<String, Vec<ContractDef>>,
    /// Base path for contracts directory
    contracts_path: PathBuf,
}

/// Contract definition from YAML
#[derive(Debug, Clone, serde::Deserialize)]
struct ContractsFile {
    contracts: Vec<ContractDef>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ContractDef {
    id: String,
    name: String,
    #[allow(dead_code)]
    domain: String,
    #[serde(default)]
    severity: YamlSeverity,
    #[serde(default)]
    #[allow(dead_code)]
    description: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    rules: Vec<RuleDef>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RuleDef {
    pattern: String,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum YamlSeverity {
    #[default]
    Warning,
    Error,
    Info,
    Hint,
    Critical,
}

impl From<YamlSeverity> for Severity {
    fn from(s: YamlSeverity) -> Self {
        match s {
            YamlSeverity::Critical => Severity::Critical,
            YamlSeverity::Error => Severity::Error,
            YamlSeverity::Warning => Severity::Warning,
            YamlSeverity::Info => Severity::Info,
            YamlSeverity::Hint => Severity::Hint,
        }
    }
}

impl ContractLayer {
    /// Create a new contract layer with default path
    pub fn new() -> Self {
        let contracts_path = std::env::current_dir()
            .map(|p| p.join("contracts"))
            .unwrap_or_else(|_| PathBuf::from("contracts"));
        
        Self {
            cache: HashMap::new(),
            contracts_path,
        }
    }

    /// Create with custom contracts path
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            cache: HashMap::new(),
            contracts_path: path.into(),
        }
    }

    /// Load contracts for a specific language
    fn load_contracts(&mut self, language: &str) -> Vec<ContractDef> {
        if let Some(cached) = self.cache.get(language) {
            return cached.clone();
        }

        let mut contracts = Vec::new();

        // 1. Load from language-specific directory
        let lang_path = self.contracts_path.join(language);
        if let Ok(entries) = std::fs::read_dir(&lang_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "yaml").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(file) = serde_yaml::from_str::<ContractsFile>(&content) {
                            contracts.extend(file.contracts);
                        }
                    }
                }
            }
        }

        // 2. Load from imported directory (imported_{language}.yaml)
        let imported_path = self.contracts_path.join("imported");
        let imported_file = imported_path.join(format!("imported_{}.yaml", language));
        if imported_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&imported_file) {
                if let Ok(file) = serde_yaml::from_str::<ContractsFile>(&content) {
                    contracts.extend(file.contracts);
                }
            }
        }

        // 3. Also load imported_all.yaml for cross-language rules
        let all_file = imported_path.join("imported_all.yaml");
        if all_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&all_file) {
                if let Ok(file) = serde_yaml::from_str::<ContractsFile>(&content) {
                    // Filter by tag matching language
                    let lang_tag = language.to_lowercase();
                    let filtered: Vec<ContractDef> = file.contracts
                        .into_iter()
                        .filter(|c| c.tags.iter().any(|t| t.to_lowercase() == lang_tag))
                        .collect();
                    contracts.extend(filtered);
                }
            }
        }

        self.cache.insert(language.to_string(), contracts.clone());
        contracts
    }

    /// Evaluate a pattern against source
    fn evaluate_pattern(&self, pattern: &str, source: &str) -> Vec<PatternMatch> {
        // All patterns from YAML are regex by default
        // Support explicit "regex:" prefix for clarity, but treat all as regex
        let regex_pattern = pattern.strip_prefix("regex:").unwrap_or(pattern);

        // Try as regex first
        if let Ok(regex) = Regex::new(regex_pattern) {
            return regex.find_iter(source)
                .map(|m| PatternMatch {
                    start: m.start(),
                    end: m.end(),
                    matched: m.as_str().to_string(),
                })
                .collect();
        }

        // Fallback to simple text pattern if regex fails
        let mut matches = Vec::new();
        let mut start = 0;
        while let Some(pos) = source[start..].find(pattern) {
            let abs_start = start + pos;
            matches.push(PatternMatch {
                start: abs_start,
                end: abs_start + pattern.len(),
                matched: pattern.to_string(),
            });
            start = abs_start + pattern.len();
        }
        matches
    }

    /// Convert position to line number
    fn line_from_pos(&self, source: &str, pos: usize) -> usize {
        source[..pos].chars().filter(|&c| c == '\n').count() + 1
    }
}

struct PatternMatch {
    start: usize,
    #[allow(dead_code)]
    end: usize,
    matched: String,
}

impl Default for ContractLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for ContractLayer {
    fn name(&self) -> &str {
        "contract"
    }

    fn priority(&self) -> u8 {
        35 // After logic (30), before style (40)
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let language = ctx.language.to_lowercase();
        let mut layer = self.clone();
        let contracts = layer.load_contracts(&language);

        if contracts.is_empty() {
            return LayerResult::pass();
        }

        let mut violations = Vec::new();

        for contract in contracts {
            for rule in &contract.rules {
                let matches = self.evaluate_pattern(&rule.pattern, &ctx.source);
                
                for m in matches {
                    let line = self.line_from_pos(&ctx.source, m.start);
                    let message = rule.message.clone()
                        .unwrap_or_else(|| format!("{}: {}", contract.name, m.matched));
                    
                    let mut violation = match Severity::from(contract.severity) {
                        Severity::Critical => Violation::critical(&contract.id, message),
                        Severity::Error => Violation::error(&contract.id, message),
                        Severity::Warning => Violation::warning(&contract.id, message),
                        Severity::Info => Violation::info(&contract.id, message),
                        Severity::Hint => {
                            let mut v = Violation::new(&contract.id, message);
                            v.severity = Severity::Hint;
                            v
                        }
                    };
                    
                    violation.span = Some(Span {
                        line,
                        column: 1,
                    });
                    
                    if let Some(suggestion) = &rule.suggestion {
                        violation = violation.suggest(suggestion);
                    }
                    
                    violations.push(violation);
                }
            }
        }

        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }
}

impl Clone for ContractLayer {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            contracts_path: self.contracts_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_layer_creation() {
        let layer = ContractLayer::new();
        assert_eq!(layer.name(), "contract");
    }

    #[test]
    fn test_text_pattern_matching() {
        let layer = ContractLayer::new();
        let source = "let x = option.unwrap();";
        let matches = layer.evaluate_pattern("unwrap()", source);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_regex_pattern_matching() {
        let layer = ContractLayer::new();
        let source = "except Exception as e:";
        let matches = layer.evaluate_pattern("regex:except\\s+Exception", source);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_line_calculation() {
        let layer = ContractLayer::new();
        let source = "line1\nline2\nline3";
        assert_eq!(layer.line_from_pos(source, 0), 1);
        assert_eq!(layer.line_from_pos(source, 6), 2);
        assert_eq!(layer.line_from_pos(source, 12), 3);
    }
}
