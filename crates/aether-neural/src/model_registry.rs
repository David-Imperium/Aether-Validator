//! Model Registry — tracks available models and their versions.
//!
//! Manages the lifecycle of neural models:
//! - Scanning for available model files
//! - Versioning (each model has a version, timestamp, metrics)
//! - Rollback (revert to a previous model version)
//!
//! Integrates with Aether's Temporal Memory for model version history.

use crate::error::Result;
use chrono;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Information about a loaded model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier (e.g., "code_reasoner").
    pub name: String,
    /// Model version (semver-like).
    pub version: ModelVersion,
    /// Path to the model weights file.
    pub path: PathBuf,
    /// File size in bytes.
    pub size_bytes: u64,
    /// When the model was last modified.
    pub modified: String,
}

/// Model version identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    /// Major version (breaking changes).
    pub major: u32,
    /// Minor version (new features).
    pub minor: u32,
    /// Patch version (bug fixes).
    pub patch: u32,
    /// Build metadata (commit hash, timestamp).
    pub build: String,
}

impl ModelVersion {
    /// Create a new model version.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            build: String::new(),
        }
    }

    /// Create version 0.1.0 (initial release).
    pub fn initial() -> Self {
        Self::new(0, 1, 0)
    }

    /// Display as version string.
    pub fn display(&self) -> String {
        if self.build.is_empty() {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        } else {
            format!(
                "{}.{}.{}+{}",
                self.major, self.minor, self.patch, self.build
            )
        }
    }
}

impl std::fmt::Display for ModelVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Model registry — tracks all available models and their versions.
pub struct ModelRegistry {
    /// Known models by name.
    models: HashMap<String, ModelInfo>,
    /// Registry directory path.
    #[allow(dead_code)]
    registry_dir: PathBuf,
}

impl ModelRegistry {
    /// Scan a directory for model files (.burnpack) and build the registry.
    pub fn scan(models_dir: &Path) -> Result<Self> {
        let mut models = HashMap::new();

        if models_dir.exists() {
            for entry in std::fs::read_dir(models_dir)? {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let path = entry.path();

                // Only consider .burnpack files
                if path.extension().and_then(|e| e.to_str()) != Some("burnpack") {
                    continue;
                }

                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let metadata = match std::fs::metadata(&path) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let modified = metadata
                    .modified()
                    .ok()
                    .map(|t| {
                        let dt: chrono::DateTime<chrono::Local> = t.into();
                        dt.format("%Y-%m-%d %H:%M:%S").to_string()
                    })
                    .unwrap_or_else(|| "unknown".into());

                models.insert(
                    name.clone(),
                    ModelInfo {
                        name,
                        version: ModelVersion::initial(),
                        path: path.clone(),
                        size_bytes: metadata.len(),
                        modified,
                    },
                );
            }
        }

        tracing::info!(
            "Model registry: {} models found in {}",
            models.len(),
            models_dir.display()
        );

        Ok(Self {
            models,
            registry_dir: models_dir.to_path_buf(),
        })
    }

    /// Get info about a specific model.
    pub fn get(&self, name: &str) -> Option<&ModelInfo> {
        self.models.get(name)
    }

    /// Check if a model exists in the registry.
    pub fn has(&self, name: &str) -> bool {
        self.models.contains_key(name)
    }

    /// List all registered models.
    pub fn list_models(&self) -> Vec<ModelInfo> {
        self.models.values().cloned().collect()
    }

    /// Get model names.
    pub fn model_names(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }

    /// Number of registered models.
    pub fn count(&self) -> usize {
        self.models.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_version_display() {
        let v = ModelVersion::new(1, 2, 3);
        assert_eq!(v.display(), "1.2.3");

        let v2 = ModelVersion {
            major: 1,
            minor: 0,
            patch: 0,
            build: "abc123".into(),
        };
        assert_eq!(v2.display(), "1.0.0+abc123");
    }

    #[test]
    fn test_model_version_initial() {
        let v = ModelVersion::initial();
        assert_eq!(v.display(), "0.1.0");
    }

    #[test]
    fn test_scan_nonexistent_dir() {
        let registry = ModelRegistry::scan(Path::new("/nonexistent/path")).unwrap();
        assert_eq!(registry.count(), 0);
    }
}
