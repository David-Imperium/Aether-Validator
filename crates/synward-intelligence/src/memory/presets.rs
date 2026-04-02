//! Bundled Presets — Pre-configured validation settings for common languages
//!
//! Presets provide sensible defaults for different languages and project types.
//! They can be imported and customized per-project.
//!
//! ## Preset Structure
//!
//! ```toml
//! # ~/.synward/presets/rust.toml
//! name = "rust-strict"
//! description = "Strict Rust validation preset"
//! language = "rust"
//! version = "1.0"
//!
//! [thresholds]
//! "complexity.max_cyclomatic" = 10.0
//! "function.max_lines" = 50.0
//! "file.max_lines" = 500.0
//!
//! [[rules.custom]]
//! id = "RUST001"
//! description = "No unwrap in production code"
//! pattern = "\.unwrap\(\)"
//! is_regex = true
//! severity = "warning"
//! applies_to = ["src/**/*.rs", "!**/test*.rs"]
//!
//! [style.naming]
//! function_style = "snake_case"
//! struct_style= "PascalCase"
//! const_style= "SCREAMING_SNAKE_CASE"
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use super::{LearnedConfig, ProjectConfig, Severity};
use super::scope::MemoryPath;
use crate::error::{Error, Result};

/// A bundled preset for a specific language/project type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    /// Preset name (e.g., "rust-strict", "python-relaxed")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Target language
    pub language: String,

    /// Preset version
    pub version: String,

    /// Threshold overrides
    #[serde(default)]
    pub thresholds: HashMap<String, f32>,

    /// Custom rules
    #[serde(default)]
    pub rules: PresetRules,

    /// Style conventions
    #[serde(default)]
    pub style: PresetStyle,

    /// Preset metadata
    #[serde(default)]
    pub meta: PresetMeta,
}

/// Rules section in a preset
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetRules {
    /// Custom rules
    #[serde(default)]
    pub custom: Vec<PresetRule>,
}

/// A custom rule in a preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetRule {
    /// Rule ID
    pub id: String,

    /// Description
    pub description: String,

    /// Pattern to match
    pub pattern: String,

    /// Is regex pattern
    #[serde(default)]
    pub is_regex: bool,

    /// Severity
    pub severity: String,

    /// File patterns to apply to
    #[serde(default)]
    pub applies_to: Vec<String>,
}

/// Style section in a preset
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetStyle {
    /// Naming conventions
    #[serde(default)]
    pub naming: PresetNaming,

    /// Formatting conventions
    #[serde(default)]
    pub formatting: PresetFormatting,
}

/// Naming conventions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetNaming {
    /// Function naming style
    #[serde(default)]
    pub function_style: Option<String>,

    /// Struct/Class naming style
    #[serde(default)]
    pub struct_style: Option<String>,

    /// Constant naming style
    #[serde(default)]
    pub const_style: Option<String>,

    /// Variable naming style
    #[serde(default)]
    pub variable_style: Option<String>,
}

/// Formatting conventions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetFormatting {
    /// Max line length
    #[serde(default)]
    pub max_line_length: Option<u32>,

    /// Indent size
    #[serde(default)]
    pub indent_size: Option<u32>,

    /// Use tabs or spaces
    #[serde(default)]
    pub indent_style: Option<String>,
}

/// Preset metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetMeta {
    /// Author
    #[serde(default)]
    pub author: Option<String>,

    /// Tags for search
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Preset manager - handles loading, saving, and applying presets
pub struct PresetManager {
    /// Preset directory path
    preset_dir: PathBuf,

    /// Loaded presets cache
    presets: HashMap<String, Preset>,
}

impl PresetManager {
    /// Create a new preset manager
    pub fn new(preset_dir: Option<PathBuf>) -> Result<Self> {
        let preset_dir = preset_dir.unwrap_or_else(|| {
            MemoryPath::global_base()
                .join("presets")
        });

        let mut manager = Self {
            preset_dir: preset_dir.clone(),
            presets: HashMap::new(),
        };

        // Ensure directory exists
        if !manager.preset_dir.exists() {
            fs::create_dir_all(&manager.preset_dir)
                .map_err(Error::Io)?;
        }

        // Load existing presets
        manager.load_all()?;

        // If no presets exist, create defaults
        if manager.presets.is_empty() {
            manager.create_default_presets()?;
        }

        Ok(manager)
    }

    /// Load all presets from directory
    pub fn load_all(&mut self) -> Result<()> {
        if !self.preset_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.preset_dir)
            .map_err(Error::Io)?
        {
            let entry = entry.map_err(Error::Io)?;
            let path = entry.path();

            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Ok(preset) = self.load_preset(&path) {
                    self.presets.insert(preset.name.clone(), preset);
                }
            }
        }

        Ok(())
    }

    /// Load a single preset from file
    pub fn load_preset(&self, path: &Path) -> Result<Preset> {
        let content = fs::read_to_string(path)
            .map_err(Error::Io)?;

        toml::from_str(&content)
            .map_err(|e| Error::Toml(e.to_string()))
    }

    /// Save a preset to file
    pub fn save_preset(&self, preset: &Preset) -> Result<()> {
        let path = self.preset_dir.join(format!("{}.toml", preset.name));
        let content = toml::to_string_pretty(preset)
            .map_err(|e| Error::Toml(e.to_string()))?;

        fs::write(&path, content)
            .map_err(Error::Io)
    }

    /// Get a preset by name
    pub fn get(&self, name: &str) -> Option<&Preset> {
        self.presets.get(name)
    }

    /// List all available presets
    pub fn list(&self) -> Vec<&Preset> {
        self.presets.values().collect()
    }

    /// List presets for a specific language
    pub fn list_for_language(&self, language: &str) -> Vec<&Preset> {
        self.presets.values()
            .filter(|p| p.language.to_lowercase() == language.to_lowercase())
            .collect()
    }

    /// Apply a preset to a LearnedConfig
    pub fn apply_to(&self, name: &str, config: &mut LearnedConfig) -> Result<()> {
        let preset = self.presets.get(name)
            .ok_or_else(|| Error::NotFound(format!("Preset '{}' not found", name)))?;

        // Apply thresholds
        for (key, value) in &preset.thresholds {
            config.thresholds.insert(key.clone(), *value);
        }

        // Apply custom rules
        for rule in &preset.rules.custom {
            let severity = match rule.severity.to_lowercase().as_str() {
                "error" => Severity::Error,
                "warning" => Severity::Warning,
                "info" | "note" => Severity::Info,
                "style" => Severity::Style,
                _ => Severity::Warning,
            };

            config.custom_rules.push(super::CustomRule {
                id: rule.id.clone(),
                description: rule.description.clone(),
                pattern: rule.pattern.clone(),
                is_regex: rule.is_regex,
                severity,
                applies_to: rule.applies_to.clone(),
                confidence: 1.0, // Presets have full confidence
                observation_count: 0, // Not learned from observation
            });
        }

        Ok(())
    }

    /// Apply a preset to a ProjectConfig
    pub fn apply_to_project(&self, name: &str, config: &mut ProjectConfig) -> Result<()> {
        let preset = self.presets.get(name)
            .ok_or_else(|| Error::NotFound(format!("Preset '{}' not found", name)))?;

        // Apply thresholds
        for (key, value) in &preset.thresholds {
                config.thresholds.insert(key.clone(), *value);
            }

        Ok(())
    }

    /// Create default presets for common languages
    fn create_default_presets(&mut self) -> Result<()> {
        let defaults = vec![
            Self::rust_strict(),
            Self::rust_relaxed(),
            Self::python_strict(),
            Self::python_relaxed(),
            Self::typescript_strict(),
            Self::typescript_relaxed(),
            Self::javascript_relaxed(),
            Self::go_strict(),
            Self::cpp_strict(),
            Self::cpp_relaxed(),
        ];

        for preset in defaults {
            self.save_preset(&preset)?;
            self.presets.insert(preset.name.clone(), preset);
        }

        Ok(())
    }

    // === Default Presets ===

    fn rust_strict() -> Preset {
        let mut thresholds = HashMap::new();
        thresholds.insert("complexity.max_cyclomatic".to_string(), 10.0);
        thresholds.insert("function.max_lines".to_string(), 40.0);
        thresholds.insert("file.max_lines".to_string(), 400.0);
        thresholds.insert("nesting.max_depth".to_string(), 3.0);

        Preset {
            name: "rust-strict".to_string(),
            description: "Strict Rust validation with Clippy-inspired rules".to_string(),
            language: "rust".to_string(),
            version: "1.0".to_string(),
            thresholds,
            rules: PresetRules {
                custom: vec![
                    PresetRule {
                        id: "RUST001".to_string(),
                        description: "Avoid unwrap() in production code".to_string(),
                        pattern: r#"\.unwrap\(\)"#.to_string(),
                        is_regex: true,
                        severity: "warning".to_string(),
                        applies_to: vec!["src/**/*.rs".to_string(), "!**/test*.rs".to_string()],
                    },
                    PresetRule {
                        id: "RUST002".to_string(),
                        description: "Avoid expect() with generic message".to_string(),
                        pattern: r#"\.expect\(".*"\)"#.to_string(),
                        is_regex: true,
                        severity: "info".to_string(),
                        applies_to: vec!["src/**/*.rs".to_string()],
                    },
                    PresetRule {
                        id: "RUST003".to_string(),
                        description: "Prefer Result over panic!".to_string(),
                        pattern: "panic!".to_string(),
                        is_regex: false,
                        severity: "warning".to_string(),
                        applies_to: vec!["src/**/*.rs".to_string(), "!**/test*.rs".to_string()],
                    },
                ],
            },
            style: PresetStyle {
                naming: PresetNaming {
                    function_style: Some("snake_case".to_string()),
                    struct_style: Some("PascalCase".to_string()),
                    const_style: Some("SCREAMING_SNAKE_CASE".to_string()),
                    variable_style: Some("snake_case".to_string()),
                },
                formatting: PresetFormatting {
                    max_line_length: Some(100),
                    indent_size: Some(4),
                    indent_style: Some("spaces".to_string()),
                },
            },
            meta: PresetMeta {
                author: Some("synward".to_string()),
                tags: vec!["rust".to_string(), "strict".to_string(), "clippy".to_string()],
            },
        }
    }

    fn rust_relaxed() -> Preset {
        let mut thresholds = HashMap::new();
        thresholds.insert("complexity.max_cyclomatic".to_string(), 15.0);
        thresholds.insert("function.max_lines".to_string(), 60.0);
        thresholds.insert("file.max_lines".to_string(), 600.0);

        Preset {
            name: "rust-relaxed".to_string(),
            description: "Relaxed Rust validation for rapid development".to_string(),
            language: "rust".to_string(),
            version: "1.0".to_string(),
            thresholds,
            rules: PresetRules::default(),
            style: PresetStyle {
                naming: PresetNaming {
                    function_style: Some("snake_case".to_string()),
                    struct_style: Some("PascalCase".to_string()),
                    const_style: Some("SCREAMING_SNAKE_CASE".to_string()),
                    variable_style: None,
                },
                formatting: PresetFormatting {
                    max_line_length: Some(120),
                    indent_size: Some(4),
                    indent_style: None,
                },
            },
            meta: PresetMeta {
                author: Some("synward".to_string()),
                tags: vec!["rust".to_string(), "relaxed".to_string()],
            },
        }
    }

    fn python_strict() -> Preset {
        let mut thresholds = HashMap::new();
        thresholds.insert("complexity.max_cyclomatic".to_string(), 10.0);
        thresholds.insert("function.max_lines".to_string(), 50.0);
        thresholds.insert("file.max_lines".to_string(), 500.0);

        Preset {
            name: "python-strict".to_string(),
            description: "Strict Python validation with PEP8 and beyond".to_string(),
            language: "python".to_string(),
            version: "1.0".to_string(),
            thresholds,
            rules: PresetRules {
                custom: vec![
                    PresetRule {
                        id: "PY001".to_string(),
                        description: "Avoid bare except".to_string(),
                        pattern: r#"except:"#.to_string(),
                        is_regex: true,
                        severity: "warning".to_string(),
                        applies_to: vec!["**/*.py".to_string()],
                    },
                    PresetRule {
                        id: "PY002".to_string(),
                        description: "Avoid mutable default arguments".to_string(),
                        pattern: r#"def\s+\w+\([^)]*=\s*\[\]"#.to_string(),
                        is_regex: true,
                        severity: "warning".to_string(),
                        applies_to: vec!["**/*.py".to_string()],
                    },
                    PresetRule {
                        id: "PY003".to_string(),
                        description: "Use f-strings for formatting".to_string(),
                        pattern: ".format(".to_string(),
                        is_regex: false,
                        severity: "info".to_string(),
                        applies_to: vec!["**/*.py".to_string()],
                    },
                ],
            },
            style: PresetStyle {
                naming: PresetNaming {
                    function_style: Some("snake_case".to_string()),
                    struct_style: Some("PascalCase".to_string()),
                    const_style: Some("SCREAMING_SNAKE_CASE".to_string()),
                    variable_style: Some("snake_case".to_string()),
                },
                formatting: PresetFormatting {
                    max_line_length: Some(88), // Black default
                    indent_size: Some(4),
                    indent_style: Some("spaces".to_string()),
                },
            },
            meta: PresetMeta {
                author: Some("synward".to_string()),
                tags: vec!["python".to_string(), "strict".to_string(), "pep8".to_string()],
            },
        }
    }

    fn python_relaxed() -> Preset {
        let mut thresholds = HashMap::new();
        thresholds.insert("complexity.max_cyclomatic".to_string(), 15.0);
        thresholds.insert("function.max_lines".to_string(), 80.0);

        Preset {
            name: "python-relaxed".to_string(),
            description: "Relaxed Python validation for scripts and notebooks".to_string(),
            language: "python".to_string(),
            version: "1.0".to_string(),
            thresholds,
            rules: PresetRules::default(),
            style: PresetStyle {
                naming: PresetNaming {
                    function_style: Some("snake_case".to_string()),
                    struct_style: None,
                    const_style: None,
                    variable_style: None,
                },
                formatting: PresetFormatting {
                    max_line_length: Some(100),
                    indent_size: Some(4),
                    indent_style: None,
                },
            },
            meta: PresetMeta {
                author: Some("synward".to_string()),
                tags: vec!["python".to_string(), "relaxed".to_string()],
            },
        }
    }

    fn typescript_strict() -> Preset {
        let mut thresholds = HashMap::new();
        thresholds.insert("complexity.max_cyclomatic".to_string(), 10.0);
        thresholds.insert("function.max_lines".to_string(), 50.0);
        thresholds.insert("file.max_lines".to_string(), 500.0);

        Preset {
            name: "typescript-strict".to_string(),
            description: "Strict TypeScript validation with ESLint-inspired rules".to_string(),
            language: "typescript".to_string(),
            version: "1.0".to_string(),
            thresholds,
            rules: PresetRules {
                custom: vec![
                    PresetRule {
                        id: "TS001".to_string(),
                        description: "Avoid any type".to_string(),
                        pattern: ": any".to_string(),
                        is_regex: false,
                        severity: "warning".to_string(),
                        applies_to: vec!["**/*.ts".to_string(), "!**/*.d.ts".to_string()],
                    },
                    PresetRule {
                        id: "TS002".to_string(),
                        description: "Prefer const over let".to_string(),
                        pattern: "let ".to_string(),
                        is_regex: false,
                        severity: "info".to_string(),
                        applies_to: vec!["**/*.ts".to_string(), "**/*.tsx".to_string()],
                    },
                    PresetRule {
                        id: "TS003".to_string(),
                        description: "Avoid non-null assertion".to_string(),
                        pattern: "!.".to_string(),
                        is_regex: false,
                        severity: "warning".to_string(),
                        applies_to: vec!["**/*.ts".to_string()],
                    },
                ],
            },
            style: PresetStyle {
                naming: PresetNaming {
                    function_style: Some("camelCase".to_string()),
                    struct_style: Some("PascalCase".to_string()),
                    const_style: Some("SCREAMING_SNAKE_CASE".to_string()),
                    variable_style: Some("camelCase".to_string()),
                },
                formatting: PresetFormatting {
                    max_line_length: Some(100),
                    indent_size: Some(2),
                    indent_style: Some("spaces".to_string()),
                },
            },
            meta: PresetMeta {
                author: Some("synward".to_string()),
                tags: vec!["typescript".to_string(), "strict".to_string(), "eslint".to_string()],
            },
        }
    }

    fn typescript_relaxed() -> Preset {
        let mut thresholds = HashMap::new();
        thresholds.insert("complexity.max_cyclomatic".to_string(), 15.0);
        thresholds.insert("function.max_lines".to_string(), 80.0);

        Preset {
            name: "typescript-relaxed".to_string(),
            description: "Relaxed TypeScript validation".to_string(),
            language: "typescript".to_string(),
            version: "1.0".to_string(),
            thresholds,
            rules: PresetRules::default(),
            style: PresetStyle {
                naming: PresetNaming {
                    function_style: Some("camelCase".to_string()),
                    struct_style: Some("PascalCase".to_string()),
                    const_style: None,
                    variable_style: Some("camelCase".to_string()),
                },
                formatting: PresetFormatting {
                    max_line_length: Some(120),
                    indent_size: Some(2),
                    indent_style: None,
                },
            },
            meta: PresetMeta {
                author: Some("synward".to_string()),
                tags: vec!["typescript".to_string(), "relaxed".to_string()],
            },
        }
    }

    fn javascript_relaxed() -> Preset {
        let mut thresholds = HashMap::new();
        thresholds.insert("complexity.max_cyclomatic".to_string(), 15.0);
        thresholds.insert("function.max_lines".to_string(), 60.0);

        Preset {
            name: "javascript-relaxed".to_string(),
            description: "Relaxed JavaScript validation".to_string(),
            language: "javascript".to_string(),
            version: "1.0".to_string(),
            thresholds,
            rules: PresetRules {
                custom: vec![
                    PresetRule {
                        id: "JS001".to_string(),
                        description: "Avoid var keyword".to_string(),
                        pattern: "var ".to_string(),
                        is_regex: false,
                        severity: "warning".to_string(),
                        applies_to: vec!["**/*.js".to_string(), "**/*.jsx".to_string()],
                    },
                    PresetRule {
                        id: "JS002".to_string(),
                        description: "Prefer const over let".to_string(),
                        pattern: "let ".to_string(),
                        is_regex: false,
                        severity: "info".to_string(),
                        applies_to: vec!["**/*.js".to_string(), "**/*.jsx".to_string()],
                    },
                ],
            },
            style: PresetStyle {
                naming: PresetNaming {
                    function_style: Some("camelCase".to_string()),
                    struct_style: None,
                    const_style: None,
                    variable_style: Some("camelCase".to_string()),
                },
                formatting: PresetFormatting {
                    max_line_length: Some(100),
                    indent_size: Some(2),
                    indent_style: None,
                },
            },
            meta: PresetMeta {
                author: Some("synward".to_string()),
                tags: vec!["javascript".to_string(), "relaxed".to_string()],
            },
        }
    }

    fn go_strict() -> Preset {
        let mut thresholds = HashMap::new();
        thresholds.insert("complexity.max_cyclomatic".to_string(), 15.0);
        thresholds.insert("function.max_lines".to_string(), 60.0);

        Preset {
            name: "go-strict".to_string(),
            description: "Strict Go validation with gofmt and golint rules".to_string(),
            language: "go".to_string(),
            version: "1.0".to_string(),
            thresholds,
            rules: PresetRules {
                custom: vec![
                    PresetRule {
                        id: "GO001".to_string(),
                        description: "Handle all errors explicitly".to_string(),
                        pattern: " = nil".to_string(),
                        is_regex: false,
                        severity: "info".to_string(),
                        applies_to: vec!["**/*.go".to_string()],
                    },
                ],
            },
            style: PresetStyle {
                naming: PresetNaming {
                    function_style: Some("PascalCase".to_string()), // Exported
                    struct_style: Some("PascalCase".to_string()),
                    const_style: Some("PascalCase".to_string()),
                    variable_style: Some("camelCase".to_string()),
                },
                formatting: PresetFormatting {
                    max_line_length: None, // Go has no line limit
                    indent_size: Some(4),
                    indent_style: Some("tabs".to_string()),
                },
            },
            meta: PresetMeta {
                author: Some("synward".to_string()),
                tags: vec!["go".to_string(), "strict".to_string(), "gofmt".to_string()],
            },
        }
    }

    fn cpp_strict() -> Preset {
        let mut thresholds = HashMap::new();
        thresholds.insert("complexity.max_cyclomatic".to_string(), 12.0);
        thresholds.insert("function.max_lines".to_string(), 50.0);
        thresholds.insert("file.max_lines".to_string(), 800.0);

        Preset {
            name: "cpp-strict".to_string(),
            description: "Strict C++ validation with modern C++ guidelines".to_string(),
            language: "cpp".to_string(),
            version: "1.0".to_string(),
            thresholds,
            rules: PresetRules {
                custom: vec![
                    PresetRule {
                        id: "CPP001".to_string(),
                        description: "Avoid raw new/delete".to_string(),
                        pattern: "new ".to_string(),
                        is_regex: false,
                        severity: "warning".to_string(),
                        applies_to: vec!["**/*.cpp".to_string(), "**/*.hpp".to_string()],
                    },
                    PresetRule {
                        id: "CPP002".to_string(),
                        description: "Avoid C-style casts".to_string(),
                        pattern: r#"\(\w+\)"#.to_string(),
                        is_regex: true,
                        severity: "warning".to_string(),
                        applies_to: vec!["**/*.cpp".to_string()],
                    },
                    PresetRule {
                        id: "CPP003".to_string(),
                        description: "Use nullptr instead of NULL".to_string(),
                        pattern: "NULL".to_string(),
                        is_regex: false,
                        severity: "warning".to_string(),
                        applies_to: vec!["**/*.cpp".to_string(), "**/*.hpp".to_string()],
                    },
                ],
            },
            style: PresetStyle {
                naming: PresetNaming {
                    function_style: Some("snake_case".to_string()),
                    struct_style: Some("PascalCase".to_string()),
                    const_style: Some("kCamelCase".to_string()),
                    variable_style: Some("snake_case".to_string()),
                },
                formatting: PresetFormatting {
                    max_line_length: Some(120),
                    indent_size: Some(4),
                    indent_style: Some("spaces".to_string()),
                },
            },
            meta: PresetMeta {
                author: Some("synward".to_string()),
                tags: vec!["cpp".to_string(), "strict".to_string(), "modern".to_string()],
            },
        }
    }

    fn cpp_relaxed() -> Preset {
        let mut thresholds = HashMap::new();
        thresholds.insert("complexity.max_cyclomatic".to_string(), 20.0);
        thresholds.insert("function.max_lines".to_string(), 100.0);

        Preset {
            name: "cpp-relaxed".to_string(),
            description: "Relaxed C++ validation for legacy code".to_string(),
            language: "cpp".to_string(),
            version: "1.0".to_string(),
            thresholds,
            rules: PresetRules::default(),
            style: PresetStyle::default(),
            meta: PresetMeta {
                author: Some("synward".to_string()),
                tags: vec!["cpp".to_string(), "relaxed".to_string()],
            },
        }
    }
}

/// Export a LearnedConfig as a clean preset (without personal info)
pub fn export_as_preset(config: &LearnedConfig, name: &str, description: &str) -> Preset {
    let mut thresholds = HashMap::new();
    for (k, v) in &config.thresholds {
        thresholds.insert(k.clone(), *v);
    }

    let custom_rules = config.custom_rules.iter().map(|r| {
        PresetRule {
            id: r.id.clone(),
            description: r.description.clone(),
            pattern: r.pattern.clone(),
            is_regex: r.is_regex,
            severity: match r.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
                Severity::Style => "style",
            }.to_string(),
            applies_to: r.applies_to.clone(),
        }
    }).collect();

    Preset {
        name: name.to_string(),
        description: description.to_string(),
        language: "multi".to_string(), // Could be multi-language
        version: "1.0".to_string(),
        thresholds,
        rules: PresetRules { custom: custom_rules },
        style: PresetStyle::default(),
        meta: PresetMeta {
            author: None, // No personal info
            tags: vec![],
        },
    }
}

/// Import a preset file into the preset directory
pub fn import_preset(path: &Path, preset_dir: &Path) -> Result<Preset> {
    let content = fs::read_to_string(path)
        .map_err(Error::Io)?;

    let preset: Preset = toml::from_str(&content)
        .map_err(|e| Error::Toml(e.to_string()))?;

    // Save to preset directory
    let dest = preset_dir.join(format!("{}.toml", preset.name));
    fs::write(&dest, content)
        .map_err(Error::Io)?;

    Ok(preset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    

    #[test]
    fn test_preset_manager_creates_defaults() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let manager = PresetManager::new(Some(dir.path().join("presets")))
            .expect("Failed to create manager");

        // Should have default presets
        assert!(manager.get("rust-strict").is_some());
        assert!(manager.get("python-strict").is_some());
        assert!(manager.get("typescript-strict").is_some());
    }

    #[test]
    fn test_preset_apply_to_config() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let manager = PresetManager::new(Some(dir.path().join("presets")))
            .expect("Failed to create manager");

        let mut config = LearnedConfig::default();
        manager.apply_to("rust-strict", &mut config).expect("Failed to apply");

        assert_eq!(config.thresholds.get("complexity.max_cyclomatic"), Some(&10.0));
        assert!(!config.custom_rules.is_empty());
    }

    #[test]
    fn test_preset_serialization() {
        let preset = PresetManager::rust_strict();

        let toml = toml::to_string(&preset).expect("Failed to serialize");
        let parsed: Preset = toml::from_str(&toml).expect("Failed to parse");

        assert_eq!(parsed.name, preset.name);
        assert_eq!(parsed.thresholds.len(), preset.thresholds.len());
    }

    #[test]
    fn test_export_as_preset() {
        let mut config = LearnedConfig::default();
        config.thresholds.insert("complexity.max_cyclomatic".to_string(), 15.0);

        let preset = export_as_preset(&config, "my-preset", "Custom preset");

        assert_eq!(preset.name, "my-preset");
        assert_eq!(preset.description, "Custom preset");
        assert!(preset.meta.author.is_none()); // No personal info
    }

    #[test]
    fn test_list_for_language() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let manager = PresetManager::new(Some(dir.path().join("presets")))
            .expect("Failed to create manager");

        let rust_presets = manager.list_for_language("rust");
        assert!(rust_presets.len() >= 2); // strict + relaxed
    }
}
