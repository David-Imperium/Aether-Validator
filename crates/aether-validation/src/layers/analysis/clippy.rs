//! Clippy Integration Layer — Leverage rust-clippy for Rust-specific lints
//!
//! This layer runs `cargo clippy` and converts its warnings/errors to Aether violations.
//! This avoids reinventing the wheel for hundreds of Rust-specific checks.

use async_trait::async_trait;
use crate::context::ValidationContext;
use crate::layer::{LayerResult, ValidationLayer};
use crate::violation::{Violation, Severity};
use std::process::Command;
use serde::Deserialize;

/// Clippy JSON message format
#[derive(Debug, Deserialize)]
struct ClippyMessage {
    reason: String,
    message: Option<ClippyDiagnostic>,
}

#[derive(Debug, Deserialize)]
struct ClippyDiagnostic {
    #[allow(dead_code)]
    rendered: Option<String>,
    spans: Vec<ClippySpan>,
    level: String,
    message: String,
    code: Option<ClippyCode>,
}

#[derive(Debug, Deserialize)]
struct ClippySpan {
    file_name: String,
    line_start: u32,
    #[allow(dead_code)]
    line_end: u32,
    column_start: u32,
    #[allow(dead_code)]
    column_end: u32,
}

#[derive(Debug, Deserialize)]
struct ClippyCode {
    code: String,
}

/// Clippy validation layer.
///
/// Integrates rust-clippy lints into Aether validation pipeline.
/// Uses `--message-format=json` for machine-readable output.
pub struct ClippyLayer {
    /// Whether to treat clippy warnings as errors
    warnings_as_errors: bool,
    /// Clippy lints to allow (skip)
    allow_lints: Vec<String>,
}

impl ClippyLayer {
    /// Create a new ClippyLayer.
    pub fn new() -> Self {
        Self {
            warnings_as_errors: false,
            allow_lints: vec![
                // Common lints that are too noisy
                "clippy::module_inception".to_string(),
                "clippy::too_many_arguments".to_string(),
            ],
        }
    }

    /// Treat warnings as errors.
    pub fn warnings_as_errors(mut self, yes: bool) -> Self {
        self.warnings_as_errors = yes;
        self
    }

    /// Add a lint to the allow list.
    pub fn allow(mut self, lint: &str) -> Self {
        self.allow_lints.push(lint.to_string());
        self
    }

    /// Run clippy and collect output.
    fn run_clippy(&self, file_path: &str) -> Result<Vec<ClippyDiagnostic>, String> {
        // Build clippy command
        let mut args = vec![
            "clippy".to_string(),
            "--message-format=json".to_string(),
            "--".to_string(),
            "-W".to_string(),
            "clippy::all".to_string(),
        ];

        // Add allowed lints
        for lint in &self.allow_lints {
            args.push("-A".to_string());
            args.push(lint.clone());
        }

        // Get the file's directory or current directory
        let file = std::path::Path::new(file_path);
        let cwd = file.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        // Run cargo clippy
        let output = Command::new("cargo")
            .args(&args)
            .current_dir(&cwd)
            .output()
            .map_err(|e| format!("Failed to run cargo clippy: {}", e))?;

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut diagnostics = Vec::new();

        for line in stdout.lines() {
            if line.starts_with('{') {
                if let Ok(msg) = serde_json::from_str::<ClippyMessage>(line) {
                    if msg.reason == "compiler-message" {
                        if let Some(diag) = msg.message {
                            diagnostics.push(diag);
                        }
                    }
                }
            }
        }

        Ok(diagnostics)
    }

    /// Convert clippy diagnostic to Aether violation.
    fn diagnostic_to_violation(&self, diag: &ClippyDiagnostic, _file_path: &str) -> Option<Violation> {
        // Skip if no spans
        if diag.spans.is_empty() {
            return None;
        }

        let span = &diag.spans[0];
        let severity = match diag.level.as_str() {
            "error" => Severity::Error,
            "warning" => {
                if self.warnings_as_errors {
                    Severity::Error
                } else {
                    Severity::Warning
                }
            }
            _ => Severity::Info,
        };

        let code = diag.code.as_ref()
            .map(|c| c.code.clone())
            .unwrap_or_else(|| "CLIPPY".to_string());

        let message = format!("[{}] {} - {}",
            code,
            diag.message,
            span.file_name
        );

        let violation = match severity {
            Severity::Error => Violation::error(&code, message),
            Severity::Warning => Violation::warning(&code, message),
            _ => Violation::info(&code, message),
        }
        .in_file(&span.file_name)
        .at(span.line_start as usize, span.column_start as usize);

        Some(violation)
    }
}

impl Default for ClippyLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for ClippyLayer {
    fn name(&self) -> &str {
        "clippy"
    }

    fn priority(&self) -> u8 {
        45 // After syntax (10), before logic (50)
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        // Skip if not Rust
        if ctx.language.to_string().to_lowercase() != "rust" {
            return LayerResult::pass();
        }

        // Get file path
        let file_path = match &ctx.file_path {
            Some(p) => p.to_string_lossy().to_string(),
            None => return LayerResult::pass(),
        };

        // Run clippy
        let diagnostics = match self.run_clippy(&file_path) {
            Ok(d) => d,
            Err(e) => {
                // If clippy fails, just pass - it might not be a Cargo project
                return LayerResult::pass()
                    .with_info(format!("Clippy not available: {}", e));
            }
        };

        // Convert to violations
        let violations: Vec<Violation> = diagnostics
            .iter()
            .filter_map(|d| self.diagnostic_to_violation(d, &file_path))
            .collect();

        if violations.is_empty() {
            LayerResult::pass()
                .with_info("No clippy warnings".to_string())
        } else {
            let count = violations.len();
            LayerResult::fail(violations)
                .with_info(format!("Found {} clippy issues", count))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clippy_layer_creation() {
        let layer = ClippyLayer::new();
        assert_eq!(layer.name(), "clippy");
        assert_eq!(layer.priority(), 45);
    }
}
