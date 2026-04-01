//! # Contract Generator - Genera Contracts da Pattern Appresi
//!
//! Questo modulo genera file YAML di contract Aether a partire dai
//! pattern estratti dal PatternLearner.
//!
//! ## Panoramica
//!
//! Quando `aether learn` estrae convenzioni da un progetto, questo modulo
//! può convertirle automaticamente in contracts utilizzabili dalla
//! validazione.
//!
//! ## Esempio
//!
//! ```rust,ignore
//! use aether_intelligence::learner::PatternLearner;
//! use aether_intelligence::contract_generator::ContractGenerator;
//!
//! let mut learner = PatternLearner::new("my-project");
//! learner.analyze_file(source_code)?;
//! let patterns = learner.finalize();
//!
//! let generator = ContractGenerator::from_patterns(&patterns);
//! generator.save("./.aether/contracts/learned.yaml")?;
//! ```

use crate::error::{Error, Result};
use crate::learner::{LearnedPatterns, NamingPatterns, DerivePatterns, DocPatterns};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Generatore di contracts da pattern appresi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractGenerator {
    /// Nome del progetto
    pub project: String,
    /// Linguaggio target
    pub language: String,
    /// Regole di naming generate
    pub naming_rules: Vec<NamingRule>,
    /// Regole di derive generate
    pub derive_rules: Vec<DeriveRule>,
    /// Regole di documentazione generate
    pub doc_rules: Vec<DocRule>,
    /// Metadati del contract
    pub metadata: ContractMetadata,
}

/// Regola di naming estratta dai pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingRule {
    /// ID univoco della regola
    pub id: String,
    /// Tipo di regola
    pub rule_type: NamingRuleType,
    /// Pattern permessi (dal learned)
    pub allowed: Vec<String>,
    /// Pattern vietati (deviazioni comuni)
    pub forbidden: Vec<String>,
    /// Confidence della regola (0.0 - 1.0)
    pub confidence: f64,
    /// Severity se violata
    pub severity: String,
    /// Descrizione
    pub description: String,
}

/// Tipo di regola naming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamingRuleType {
    StructSuffix,
    EnumSuffix,
    FunctionPrefix,
    VariableStyle,
    ModuleNaming,
}

/// Regola di derive estratta dai pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeriveRule {
    /// ID univoco della regola
    pub id: String,
    /// Derive trait richiesto
    pub derive: String,
    /// Soglia percentuale sotto cui warning
    pub threshold: f64,
    /// Confidence della regola
    pub confidence: f64,
    /// Severity
    pub severity: String,
    /// Descrizione
    pub description: String,
}

/// Regola di documentazione estratta dai pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocRule {
    /// ID univoco della regola
    pub id: String,
    /// Tipo di regola doc
    pub rule_type: DocRuleType,
    /// Soglia percentuale
    pub threshold: f64,
    /// Confidence della regola
    pub confidence: f64,
    /// Severity
    pub severity: String,
    /// Descrizione
    pub description: String,
}

/// Tipo di regola documentazione
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocRuleType {
    PublicDocCoverage,
    CommentStyle,
    ExampleCoverage,
    ModuleDocPresent,
}

/// Metadati del contract generato
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    /// Versione del formato contract
    pub version: String,
    /// Data generazione
    pub generated_at: String,
    /// Files analizzati
    pub files_analyzed: usize,
    /// Se è auto-generato da learning
    pub learned: bool,
    /// Sorgente del learning
    pub source: String,
}

impl ContractGenerator {
    /// Crea un generatore dai pattern appresi
    pub fn from_patterns(patterns: &LearnedPatterns) -> Self {
        let mut generator = Self {
            project: patterns.project.clone(),
            language: patterns.language.clone(),
            naming_rules: Vec::new(),
            derive_rules: Vec::new(),
            doc_rules: Vec::new(),
            metadata: ContractMetadata {
                version: "1.0".into(),
                generated_at: patterns.analyzed_at.clone(),
                files_analyzed: patterns.files_analyzed,
                learned: true,
                source: "aether-learn".into(),
            },
        };

        generator.generate_naming_rules(&patterns.naming, patterns.confidence.naming);
        generator.generate_derive_rules(&patterns.derives, patterns.confidence.derives);
        generator.generate_doc_rules(&patterns.documentation, patterns.confidence.documentation);

        generator
    }

    /// Genera regole naming dai pattern
    fn generate_naming_rules(&mut self, naming: &NamingPatterns, confidence: f64) {
        // Struct suffixes
        if !naming.struct_suffixes.is_empty() {
            let mut suffixes: Vec<_> = naming.struct_suffixes.iter().collect();
            suffixes.sort_by(|a, b| b.1.cmp(a.1));

            // Top suffixes sono allowed
            let allowed: Vec<String> = suffixes.iter()
                .take(5)
                .map(|(s, _)| s.to_string())
                .collect();

            self.naming_rules.push(NamingRule {
                id: format!("{}-struct-suffix", self.project),
                rule_type: NamingRuleType::StructSuffix,
                allowed,
                forbidden: vec!["Impl".into(), "Helper".into()], // anti-pattern comuni
                confidence,
                severity: if confidence >= 0.8 { "warning".into() } else { "info".into() },
                description: format!(
                    "Struct suffixes learned from {} samples",
                    naming.struct_suffixes.len()
                ),
            });
        }

        // Function prefixes
        if !naming.function_prefixes.is_empty() {
            let mut prefixes: Vec<_> = naming.function_prefixes.iter().collect();
            prefixes.sort_by(|a, b| b.1.cmp(a.1));

            let allowed: Vec<String> = prefixes.iter()
                .take(8)
                .map(|(p, _)| p.to_string())
                .collect();

            self.naming_rules.push(NamingRule {
                id: format!("{}-fn-prefix", self.project),
                rule_type: NamingRuleType::FunctionPrefix,
                allowed,
                forbidden: vec![], // nessun prefix vietato
                confidence,
                severity: if confidence >= 0.8 { "info".into() } else { "hint".into() },
                description: format!(
                    "Function prefixes learned from {} samples",
                    naming.function_prefixes.len()
                ),
            });
        }

        // Enum suffixes
        if !naming.enum_suffixes.is_empty() {
            let mut suffixes: Vec<_> = naming.enum_suffixes.iter().collect();
            suffixes.sort_by(|a, b| b.1.cmp(a.1));

            let allowed: Vec<String> = suffixes.iter()
                .take(5)
                .map(|(s, _)| s.to_string())
                .collect();

            self.naming_rules.push(NamingRule {
                id: format!("{}-enum-suffix", self.project),
                rule_type: NamingRuleType::EnumSuffix,
                allowed,
                forbidden: vec![],
                confidence,
                severity: "info".into(),
                description: format!(
                    "Enum suffixes learned from {} samples",
                    naming.enum_suffixes.len()
                ),
            });
        }
    }

    /// Genera regole derive dai pattern
    fn generate_derive_rules(&mut self, derives: &DerivePatterns, confidence: f64) {
        // Debug derive rule
        if derives.debug_percentage > 50.0 {
            self.derive_rules.push(DeriveRule {
                id: format!("{}-derive-debug", self.project),
                derive: "Debug".into(),
                threshold: derives.debug_percentage,
                confidence,
                severity: if derives.debug_percentage > 80.0 { "warning".into() } else { "info".into() },
                description: format!(
                    "Debug derive expected ({}% of structs have it)",
                    derives.debug_percentage.round()
                ),
            });
        }

        // Clone derive rule
        if derives.clone_percentage > 50.0 {
            self.derive_rules.push(DeriveRule {
                id: format!("{}-derive-clone", self.project),
                derive: "Clone".into(),
                threshold: derives.clone_percentage,
                confidence,
                severity: if derives.clone_percentage > 80.0 { "info".into() } else { "hint".into() },
                description: format!(
                    "Clone derive expected ({}% of structs have it)",
                    derives.clone_percentage.round()
                ),
            });
        }

        // Default derive rule
        if derives.default_percentage > 30.0 {
            self.derive_rules.push(DeriveRule {
                id: format!("{}-derive-default", self.project),
                derive: "Default".into(),
                threshold: derives.default_percentage,
                confidence,
                severity: "hint".into(),
                description: format!(
                    "Default derive common ({}% of structs have it)",
                    derives.default_percentage.round()
                ),
            });
        }
    }

    /// Genera regole doc dai pattern
    fn generate_doc_rules(&mut self, docs: &DocPatterns, confidence: f64) {
        // Public doc coverage
        self.doc_rules.push(DocRule {
            id: format!("{}-doc-coverage", self.project),
            rule_type: DocRuleType::PublicDocCoverage,
            threshold: docs.public_doc_percentage,
            confidence,
            severity: if docs.public_doc_percentage < 30.0 { "warning".into() } else { "info".into() },
            description: format!(
                "Public documentation coverage: {}%",
                docs.public_doc_percentage.round()
            ),
        });
    }

    /// Converte in formato YAML per Aether contracts
    pub fn to_yaml(&self) -> Result<String> {
        let yaml = serde_yaml::to_string(&self)
            .map_err(|e| Error::Other(format!("YAML serialization failed: {}", e)))?;
        Ok(yaml)
    }

    /// Salva in file YAML
    pub fn save(&self, path: &Path) -> Result<()> {
        let yaml = self.to_yaml()?;

        // Crea directory parent se necessario
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Other(format!("Failed to create directory: {}", e)))?;
        }

        std::fs::write(path, yaml)
            .map_err(|e| Error::Other(format!("Failed to write file: {}", e)))?;

        tracing::info!("Contract saved to {}", path.display());
        Ok(())
    }

    /// Carica da file YAML
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Other(format!("Failed to read file: {}", e)))?;

        let generator: Self = serde_yaml::from_str(&content)
            .map_err(|e| Error::Other(format!("YAML parsing failed: {}", e)))?;

        Ok(generator)
    }

    /// Aggiunge una regola custom
    pub fn add_naming_rule(&mut self, rule: NamingRule) -> &mut Self {
        self.naming_rules.push(rule);
        self
    }

    /// Aggiunge una regola derive custom
    pub fn add_derive_rule(&mut self, rule: DeriveRule) -> &mut Self {
        self.derive_rules.push(rule);
        self
    }

    /// Aggiunge una regola doc custom
    pub fn add_doc_rule(&mut self, rule: DocRule) -> &mut Self {
        self.doc_rules.push(rule);
        self
    }

    /// Genera contract in formato Aether nativo
    pub fn to_aether_contract(&self) -> AetherContract {
        AetherContract {
            api_version: "aether.dev/v1".into(),
            kind: "NamingContract".into(),
            metadata: AetherContractMeta {
                name: format!("{}-learned", self.project),
                learned: true,
            },
            spec: AetherContractSpec {
                rules: self.naming_rules.iter()
                    .map(|r| AetherContractRule {
                        pattern: format!("{:?}", r.rule_type),
                        allowed: r.allowed.clone(),
                        confidence: r.confidence,
                    })
                    .collect(),
            },
        }
    }
}

/// Contract formato nativo Aether
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherContract {
    pub api_version: String,
    pub kind: String,
    pub metadata: AetherContractMeta,
    pub spec: AetherContractSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherContractMeta {
    pub name: String,
    pub learned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherContractSpec {
    pub rules: Vec<AetherContractRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherContractRule {
    pub pattern: String,
    pub allowed: Vec<String>,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let patterns = LearnedPatterns::default();
        let generator = ContractGenerator::from_patterns(&patterns);

        assert_eq!(generator.metadata.learned, true);
        assert_eq!(generator.metadata.source, "aether-learn");
    }

    #[test]
    fn test_yaml_output() {
        let patterns = LearnedPatterns::default();
        let generator = ContractGenerator::from_patterns(&patterns);

        let yaml = generator.to_yaml().unwrap();
        assert!(yaml.contains("project:"));
        assert!(yaml.contains("learned: true"));
    }
}
