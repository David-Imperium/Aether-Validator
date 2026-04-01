//! AutoFixService - Pattern-based automatic fixes with user confirmation

use std::path::Path;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Proposed fix for an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixProposal {
    pub error_id: String,
    pub description: String,
    pub line: usize,
    pub original: String,
    pub replacement: String,
    pub confidence: f32,
    pub file: String,
}

/// Fix pattern for automatic correction
#[derive(Debug, Clone)]
pub struct FixPattern {
    pub id: String,
    pub language: String,
    pub description: String,
    pub find: Option<String>,
    pub replace: Option<String>,
}

/// Auto-fix service with pattern-based corrections
pub struct AutoFixService {
    patterns: Vec<FixPattern>,
}

impl AutoFixService {
    /// Create new auto-fix service with built-in patterns
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // Rust patterns
                FixPattern {
                    id: "MIL010".to_string(),
                    language: "rust".to_string(),
                    description: "Replace .unwrap() with .expect()".to_string(),
                    find: Some(".unwrap()".to_string()),
                    replace: Some(".expect(\"TODO: provide context\")".to_string()),
                },
                FixPattern {
                    id: "MIL001".to_string(),
                    language: "rust".to_string(),
                    description: "Replace panic! with proper error handling".to_string(),
                    find: Some("panic!(".to_string()),
                    replace: Some("return Err(anyhow::anyhow!(".to_string()),
                },
                FixPattern {
                    id: "PRIV020".to_string(),
                    language: "rust".to_string(),
                    description: "Replace dbg! with proper logging".to_string(),
                    find: Some("dbg!(".to_string()),
                    replace: Some("log::debug!(".to_string()),
                },
                FixPattern {
                    id: "PRIV010".to_string(),
                    language: "rust".to_string(),
                    description: "Replace print! with log::info!".to_string(),
                    find: Some("print!(".to_string()),
                    replace: Some("log::info!(".to_string()),
                },
                FixPattern {
                    id: "PRIV011".to_string(),
                    language: "rust".to_string(),
                    description: "Replace println! with log::info!".to_string(),
                    find: Some("println!(".to_string()),
                    replace: Some("log::info!(".to_string()),
                },
                // C++ patterns
                FixPattern {
                    id: "CPP_SEC001".to_string(),
                    language: "cpp".to_string(),
                    description: "Replace NULL with nullptr".to_string(),
                    find: Some("= NULL".to_string()),
                    replace: Some("= nullptr".to_string()),
                },
                FixPattern {
                    id: "CPP_SEC002".to_string(),
                    language: "cpp".to_string(),
                    description: "Replace NULL with nullptr (pointer init)".to_string(),
                    find: Some("NULL;".to_string()),
                    replace: Some("nullptr;".to_string()),
                },
                FixPattern {
                    id: "CPP_SEC003".to_string(),
                    language: "cpp".to_string(),
                    description: "Add explicit comparison to nullptr".to_string(),
                    find: Some("if (".to_string()),
                    replace: Some("if (nullptr != ".to_string()),
                },
                // Python patterns
                FixPattern {
                    id: "SEC090".to_string(),
                    language: "python".to_string(),
                    description: "Replace eval() with safer alternative".to_string(),
                    find: Some("eval(".to_string()),
                    replace: Some("ast.literal_eval(".to_string()),
                },
                FixPattern {
                    id: "SEC100".to_string(),
                    language: "python".to_string(),
                    description: "Replace exec() with safer alternative".to_string(),
                    find: Some("exec(".to_string()),
                    replace: Some("# WARNING: exec() is dangerous\n# exec(".to_string()),
                },
                // JavaScript patterns
                FixPattern {
                    id: "SEC110".to_string(),
                    language: "javascript".to_string(),
                    description: "Replace == with ===".to_string(),
                    find: Some(" == ".to_string()),
                    replace: Some(" === ".to_string()),
                },
                FixPattern {
                    id: "SEC111".to_string(),
                    language: "javascript".to_string(),
                    description: "Replace != with !==".to_string(),
                    find: Some(" != ".to_string()),
                    replace: Some(" !== ".to_string()),
                },
                // Common patterns
                FixPattern {
                    id: "LOG050".to_string(),
                    language: "*".to_string(),
                    description: "Replace TODO with implementation".to_string(),
                    find: Some("TODO:".to_string()),
                    replace: Some("FIXME: implement".to_string()),
                },
            ],
        }
    }

    /// Generate fix proposal for an error
    pub fn propose_fix(&self, error_id: &str, language: &str, line: usize, content: &str, file: &str) -> Option<FixProposal> {
        // Find matching pattern
        let pattern = self.patterns.iter()
            .find(|p| {
                (p.id == error_id || p.id == "*" || error_id.contains(&p.id)) &&
                (p.language == language || p.language == "*")
            })?;

        let find = pattern.find.as_ref()?;
        let replace = pattern.replace.as_ref()?;

        // Get line content
        let line_content = content.lines().nth(line)?;
        
        // Check if pattern matches
        if !line_content.contains(find) {
            return None;
        }

        let original = line_content.to_string();
        let replacement = line_content.replace(find, replace);

        // Calculate confidence based on pattern specificity
        let confidence = if pattern.language == "*" { 0.6 } else { 0.85 };

        Some(FixProposal {
            error_id: error_id.to_string(),
            description: pattern.description.clone(),
            line,
            original,
            replacement,
            confidence,
            file: file.to_string(),
        })
    }

    /// Apply fix to file
    pub fn apply_fix(&self, file_path: &Path, fix: &FixProposal) -> Result<String, String> {
        info!(?file_path, line = fix.line, "Applying fix");

        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let lines: Vec<&str> = content.lines().collect();

        if fix.line >= lines.len() {
            return Err(format!("Line {} out of range (file has {} lines)", fix.line, lines.len()));
        }

        // Replace the line
        let mut new_lines = lines.clone();
        new_lines[fix.line] = &fix.replacement;

        // Join back, preserving newline style
        let new_content = new_lines.join("\n");

        // Write back
        std::fs::write(file_path, &new_content)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(new_content)
    }

    /// Generate diff preview
    pub fn preview_diff(&self, fix: &FixProposal) -> String {
        let mut diff = String::new();
        
        diff.push_str(&format!("@@ -{},1 +{},1 @@\n", fix.line + 1, fix.line + 1));
        diff.push_str(&format!("-{}\n", fix.original));
        diff.push_str(&format!("+{}\n", fix.replacement));
        
        diff
    }

    /// Get available patterns for a language
    pub fn get_patterns(&self, language: &str) -> Vec<&FixPattern> {
        self.patterns.iter()
            .filter(|p| p.language == language || p.language == "*")
            .collect()
    }
}

impl Default for AutoFixService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propose_fix_unwrap() {
        let service = AutoFixService::new();
        let content = "let x = option.unwrap();";
        
        let fix = service.propose_fix("MIL010", "rust", 0, content, "test.rs");
        
        assert!(fix.is_some());
        let fix = fix.unwrap();
        assert!(fix.replacement.contains("expect"));
        assert_eq!(fix.line, 0);
    }

    #[test]
    fn test_propose_fix_cpp_null() {
        let service = AutoFixService::new();
        let content = "int* ptr = NULL;";
        
        let fix = service.propose_fix("CPP_SEC001", "cpp", 0, content, "test.cpp");
        
        assert!(fix.is_some());
        let fix = fix.unwrap();
        assert!(fix.replacement.contains("nullptr"));
    }

    #[test]
    fn test_propose_fix_no_match() {
        let service = AutoFixService::new();
        let content = "let x = 5;";
        
        let fix = service.propose_fix("MIL010", "rust", 0, content, "test.rs");
        
        assert!(fix.is_none());
    }

    #[test]
    fn test_preview_diff() {
        let service = AutoFixService::new();
        let fix = FixProposal {
            error_id: "MIL010".to_string(),
            description: "Test fix".to_string(),
            line: 5,
            original: "let x = option.unwrap();".to_string(),
            replacement: "let x = option.expect(\"context\");".to_string(),
            confidence: 0.85,
            file: "test.rs".to_string(),
        };

        let diff = service.preview_diff(&fix);
        assert!(diff.contains("@@ -6,1 +6,1 @@"));
        assert!(diff.contains("-let x = option.unwrap();"));
        assert!(diff.contains("+let x = option.expect"));
    }
}
