//! Project Context - Contextual information about the project being validated

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context about the project being validated
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Project name
    pub project_name: Option<String>,

    /// Root directory path
    pub root_path: Option<String>,

    /// Main language used
    pub primary_language: Option<String>,

    /// Framework or stack (e.g., "React", "Django", "Rails")
    pub framework: Option<String>,

    /// Project-specific coding conventions
    pub conventions: HashMap<String, String>,

    /// Known dependencies
    pub dependencies: Vec<String>,

    /// Tags for categorization
    pub tags: Vec<String>,
}

impl ProjectContext {
    /// Create context for a new project
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            project_name: Some(name.into()),
            ..Default::default()
        }
    }

    /// Set the root path
    pub fn with_root(mut self, path: impl Into<String>) -> Self {
        self.root_path = Some(path.into());
        self
    }

    /// Set the primary language
    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.primary_language = Some(lang.into());
        self
    }

    /// Set the framework
    pub fn with_framework(mut self, framework: impl Into<String>) -> Self {
        self.framework = Some(framework.into());
        self
    }

    /// Add a convention
    pub fn with_convention(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.conventions.insert(key.into(), value.into());
        self
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    /// Detect context from a project directory
    pub fn detect_from_path(path: &std::path::Path) -> Self {
        let mut ctx = Self::default();

        // Detect project name
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            ctx.project_name = Some(name.to_string());
        }

        ctx.root_path = path.to_str().map(|s| s.to_string());

        // Detect language/framework from files
        let rust_toml = path.join("Cargo.toml");
        let python_req = path.join("requirements.txt");
        let js_package = path.join("package.json");

        if rust_toml.exists() {
            ctx.primary_language = Some("rust".to_string());
            ctx.dependencies.push("cargo".to_string());
        }

        if python_req.exists() {
            ctx.primary_language = Some("python".to_string());
        }

        if js_package.exists() {
            ctx.primary_language = Some("javascript".to_string());
            ctx.dependencies.push("npm".to_string());

            // Try to detect framework
            if let Ok(content) = std::fs::read_to_string(&js_package) {
                if content.contains("\"react\"") {
                    ctx.framework = Some("react".to_string());
                } else if content.contains("\"vue\"") {
                    ctx.framework = Some("vue".to_string());
                } else if content.contains("\"express\"") {
                    ctx.framework = Some("express".to_string());
                }
            }
        }

        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let ctx = ProjectContext::new("my-project")
            .with_language("rust")
            .with_framework("actix-web")
            .with_convention("error_handling", "result_types");

        assert_eq!(ctx.project_name, Some("my-project".to_string()));
        assert_eq!(ctx.primary_language, Some("rust".to_string()));
        assert_eq!(ctx.framework, Some("actix-web".to_string()));
    }
}
