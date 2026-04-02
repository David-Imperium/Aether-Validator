//! ValidationService - Validates files and classifies errors as NEW vs PREX

use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Security: Base directory for path validation (prevents path traversal)
const ALLOWED_BASE_DIR: &str = "."; // Current working directory

/// Security: Canonicalize a path and verify it's inside the allowed directory
fn validate_path(path: &Path) -> Result<PathBuf, String> {
    // Get canonical base directory
    let base = std::fs::canonicalize(ALLOWED_BASE_DIR)
        .map_err(|e| format!("Failed to resolve base directory: {}", e))?;
    
    // Canonicalize the target path (resolves .., symlinks, etc.)
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("Failed to resolve path '{}': {}", path.display(), e))?;
    
    // Security: Verify path is inside allowed directory
    if !canonical.starts_with(&base) {
        return Err(format!(
            "Security: Path '{}' is outside allowed directory",
            path.display()
        ));
    }
    
    Ok(canonical)
}

/// Result of file validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub file: PathBuf,
    pub language: String,
    pub passed: bool,
    pub errors: Vec<ClassifiedError>,
    pub duration_ms: u64,
}

/// Error with classification (NEW or PREX)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedError {
    pub id: String,
    pub severity: String,
    pub message: String,
    pub line: usize,
    pub suggestion: Option<String>,
    pub classification: ErrorClass,
    pub file: PathBuf,
}

/// Error classification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorClass {
    New,         // Error in newly written code
    Preexisting, // Error in existing code
}

/// Validation severity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeverityLevel {
    Basic,    // Only critical errors
    Standard, // Errors + warnings
    Strict,   // All issues
}

impl Default for SeverityLevel {
    fn default() -> Self {
        Self::Standard
    }
}

/// Validation service using Synward validation crate directly
pub struct ValidationService {
    severity_level: SeverityLevel,
}

impl ValidationService {
    /// Create new validation service
    pub fn new(_synward_path: PathBuf, severity: SeverityLevel) -> Self {
        Self {
            severity_level: severity,
        }
    }

    /// Create with default settings
    pub fn default_service() -> Self {
        Self {
            severity_level: SeverityLevel::Standard,
        }
    }

    /// Validate a file using synward_validation crate
    pub async fn validate_file(&self, path: &Path) -> Result<ValidationResult, String> {
        let start = std::time::Instant::now();
        
        info!(?path, "Validating file");

        // Security: Validate and canonicalize path to prevent path traversal
        let safe_path = validate_path(path)?;

        // Detect language from extension
        let language = self.detect_language(&safe_path)?;

        // Read file content using validated path
        let content = std::fs::read_to_string(&safe_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Use validation crate directly
        let errors = self.validate_with_crate(&content, &language, path).await?;

        let passed = errors.iter().all(|e| e.severity != "error");

        Ok(ValidationResult {
            file: path.to_path_buf(),
            language,
            passed,
            errors,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Validate content using synward_validation crate
    async fn validate_with_crate(&self, content: &str, language: &str, path: &Path) -> Result<Vec<ClassifiedError>, String> {
        use synward_validation::{ValidationPipeline, ValidationContext, layers::{SyntaxLayer, ASTLayer, LogicLayer, SecurityLayer}};

        let pipeline = ValidationPipeline::new()
            .add_layer(SyntaxLayer::new())
            .add_layer(ASTLayer::new())
            .add_layer(LogicLayer::new())
            .add_layer(SecurityLayer::new());

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let ctx = ValidationContext::for_file(file_name, content.to_string(), language.to_string());
        let result = pipeline.execute(&ctx).await;

        // Get previous version for classification
        let previous_content = self.get_previous_version(path);

        let mut errors = Vec::new();
        for (_, layer_result) in &result.results {
            for v in &layer_result.violations {
                let line = v.span.map(|s| s.line).unwrap_or(0);
                
                // Determine if line is new or modified
                let is_new = self.is_line_new(&previous_content, content, line);

                errors.push(ClassifiedError {
                    id: v.id.clone(),
                    severity: format!("{:?}", v.severity).to_lowercase(),
                    message: v.message.clone(),
                    line,
                    suggestion: v.suggestion.clone(),
                    classification: if is_new { ErrorClass::New } else { ErrorClass::Preexisting },
                    file: path.to_path_buf(),
                });
            }
        }

        Ok(errors)
    }

    /// Check if a line is new or modified compared to previous version
    fn is_line_new(&self, previous: &Option<String>, current: &str, line: usize) -> bool {
        match previous {
            Some(prev) => {
                let prev_lines: Vec<&str> = prev.lines().collect();
                let curr_lines: Vec<&str> = current.lines().collect();
                
                // Line is beyond old file length or content differs
                line >= prev_lines.len() || 
                    prev_lines.get(line).map(|&l| l.trim()) != 
                    curr_lines.get(line).map(|&l| l.trim())
            }
            None => true, // New file - all errors are "new"
        }
    }

    /// Get previous version of file from git
    fn get_previous_version(&self, path: &Path) -> Option<String> {
        // Check if file is tracked by git
        let status = Command::new("git")
            .args(["status", "--porcelain", "--"])
            .arg(path)
            .output()
            .ok()?;

        let status_str = String::from_utf8_lossy(&status.stdout);
        
        // If file is untracked, it's new
        if status_str.starts_with("??") || status_str.starts_with(" A") {
            return None;
        }

        // Get previous version from HEAD
        let relative_path = path.strip_prefix(std::env::current_dir().ok()?)
            .unwrap_or(path);
        
        let output = Command::new("git")
            .args(["show", &format!("HEAD:{}", relative_path.display())])
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            None
        }
    }

    /// Detect language from file extension
    fn detect_language(&self, path: &Path) -> Result<String, String> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| "No file extension".to_string())?;

        match ext.to_lowercase().as_str() {
            // Core languages
            "rs" => Ok("rust".to_string()),
            "py" | "pyi" => Ok("python".to_string()),
            "js" | "jsx" | "mjs" | "cjs" => Ok("javascript".to_string()),
            "ts" | "tsx" | "mts" | "cts" => Ok("typescript".to_string()),
            "cpp" | "cc" | "cxx" => Ok("cpp".to_string()),
            "hpp" | "hxx" => Ok("cpp".to_string()),
            "c" => Ok("c".to_string()),
            "h" => Ok("c".to_string()),
            "go" => Ok("go".to_string()),
            "java" => Ok("java".to_string()),
            "lua" => Ok("lua".to_string()),
            "lex" => Ok("lex".to_string()),
            // New languages
            "glsl" | "frag" | "vert" | "comp" | "tesc" | "tese" | "geom" => Ok("glsl".to_string()),
            "css" => Ok("css".to_string()),
            "html" | "htm" => Ok("html".to_string()),
            "json" => Ok("json".to_string()),
            "yaml" | "yml" => Ok("yaml".to_string()),
            "toml" => Ok("toml".to_string()),
            "cmake" => Ok("cmake".to_string()),
            "cu" | "cuh" => Ok("cuda".to_string()),
            _ => Err(format!("Unknown extension: {}", ext)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language_rust() {
        let service = ValidationService::default_service();
        assert_eq!(service.detect_language(Path::new("test.rs")).unwrap(), "rust");
    }

    #[test]
    fn test_detect_language_cpp() {
        let service = ValidationService::default_service();
        assert_eq!(service.detect_language(Path::new("test.cpp")).unwrap(), "cpp");
        assert_eq!(service.detect_language(Path::new("test.hpp")).unwrap(), "cpp");
    }

    #[test]
    fn test_detect_language_unknown() {
        let service = ValidationService::default_service();
        assert!(service.detect_language(Path::new("test.xyz")).is_err());
    }
}
