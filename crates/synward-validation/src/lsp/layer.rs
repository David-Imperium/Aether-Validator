//! LSP Analysis Layer
//!
//! Validation layer that uses Language Server Protocol for deep semantic analysis.
//! Used on-demand when tree-sitter analysis is insufficient.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity};
use super::client::LspClient;
use super::types::LspDiagnostic;

/// LSP analysis validation layer.
pub struct LspAnalysisLayer {
    name: String,
    /// Cached LSP clients per language.
    #[allow(dead_code)]
    clients: Arc<Mutex<HashMap<String, LspClient>>>,
    /// Enabled languages for LSP analysis.
    enabled_languages: Vec<String>,
}

impl LspAnalysisLayer {
    /// Create a new LSP analysis layer.
    pub fn new() -> Self {
        Self {
            name: "lsp_analysis".to_string(),
            clients: Arc::new(Mutex::new(HashMap::new())),
            enabled_languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "typescript".to_string(),
                "javascript".to_string(),
                "go".to_string(),
                "c".to_string(),
                "cpp".to_string(),
            ],
        }
    }

    /// Create with custom enabled languages.
    pub fn with_languages(languages: Vec<String>) -> Self {
        Self {
            name: "lsp_analysis".to_string(),
            clients: Arc::new(Mutex::new(HashMap::new())),
            enabled_languages: languages,
        }
    }

    /// Check if LSP is available for a language.
    pub fn is_language_supported(&self, language: &str) -> bool {
        self.enabled_languages.contains(&language.to_string())
    }

    /// Get or create an LSP client for the language.
    #[allow(dead_code)]
    fn get_client(&self, language: &str, root_uri: &str) -> Result<(), String> {
        let mut clients = self.clients.lock().map_err(|_| "Lock error".to_string())?;

        if !clients.contains_key(language) {
            // Create LSP client using the for_language convenience method
            let mut client = LspClient::for_language(language)
                .ok_or_else(|| format!("No LSP server available for language: {}", language))?;
            client.initialize(root_uri).map_err(|e| e.message)?;
            clients.insert(language.to_string(), client);
        }

        // Client is now cached
        Ok(())
    }

    /// Convert LSP diagnostic to Synward violation.
    #[allow(dead_code)]
    fn diagnostic_to_violation(diagnostic: &LspDiagnostic) -> Violation {
        let severity = match diagnostic.severity {
            Some(super::types::LspDiagnosticSeverity::Error) => Severity::Error,
            Some(super::types::LspDiagnosticSeverity::Warning) => Severity::Warning,
            Some(super::types::LspDiagnosticSeverity::Information) => Severity::Info,
            Some(super::types::LspDiagnosticSeverity::Hint) => Severity::Hint,
            None => Severity::Warning,
        };

        let id = match &diagnostic.code {
            Some(code) => match code {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => format!("LSP{}", n),
                _ => "LSP_UNKNOWN".to_string(),
            },
            None => "LSP_DIAGNOSTIC".to_string(),
        };

        let _source = diagnostic.source.as_deref().unwrap_or("lsp");

        let mut violation = match severity {
            Severity::Critical => Violation::critical(&id, &diagnostic.message),
            Severity::Error => Violation::error(&id, &diagnostic.message),
            Severity::Warning => Violation::warning(&id, &diagnostic.message),
            Severity::Info => Violation::info(&id, &diagnostic.message),
            Severity::Hint => Violation::warning(&id, &diagnostic.message),
        };

        violation = violation.at(diagnostic.range.start.line as usize + 1, 
                                  diagnostic.range.start.character as usize + 1);
        violation
    }
}

impl Default for LspAnalysisLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for LspAnalysisLayer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        // Check if language is supported
        if !self.is_language_supported(&ctx.language) {
            return LayerResult {
                passed: true,
                violations: Vec::new(),
                infos: vec![format!("LSP analysis not available for language: {}", ctx.language)],
                whitelisted_count: 0,
            };
        }

        let violations = Vec::new();

        // Try to get LSP client
        // Note: Full implementation requires async LSP communication
        // This is a placeholder that returns no violations if LSP is not available
        
        let uri = ctx.file_path.as_ref()
            .map(|p| format!("file://{}", p.display()))
            .unwrap_or_else(|| "file://unknown".to_string());

        // In production, we would:
        // 1. Get/create LSP client for language
        // 2. Open the document
        // 3. Wait for publishDiagnostics notification
        // 4. Convert diagnostics to violations
        
        // For now, return a marker that LSP would be used
        let info = format!(
            "LSP analysis: Would analyze {} with {} server (not yet fully integrated)",
            uri, ctx.language
        );

        LayerResult {
            passed: true,
            violations,
            infos: vec![info],
            whitelisted_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let layer = LspAnalysisLayer::new();
        assert_eq!(layer.name(), "lsp_analysis");
        assert!(layer.is_language_supported("rust"));
        assert!(layer.is_language_supported("python"));
        assert!(!layer.is_language_supported("unknown"));
    }

    #[test]
    fn test_custom_languages() {
        let layer = LspAnalysisLayer::with_languages(vec!["rust".to_string()]);
        assert!(layer.is_language_supported("rust"));
        assert!(!layer.is_language_supported("python"));
    }
}
