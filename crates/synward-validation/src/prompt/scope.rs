//! Scope Extraction
//!
//! Determines what parts of the codebase are affected.

use std::path::PathBuf;

/// Scope level (granularity).
/// Ordered from most specific (File) to most general (Project).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ScopeLevel {
    /// Unknown scope.
    Unknown,
    /// Single file.
    File,
    /// Single function/method.
    Function,
    /// Single class/struct.
    Class,
    /// Multiple related files.
    Module,
    /// Entire project.
    Project,
}

impl std::fmt::Display for ScopeLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Project => write!(f, "PROJECT"),
            Self::Module => write!(f, "MODULE"),
            Self::Class => write!(f, "CLASS"),
            Self::Function => write!(f, "FUNCTION"),
            Self::File => write!(f, "FILE"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Type of scope entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeEntityType {
    File,
    Function,
    Class,
    Module,
    Namespace,
    Variable,
}

/// A specific entity in scope.
#[derive(Debug, Clone)]
pub struct ScopeEntity {
    /// Entity type.
    pub entity_type: ScopeEntityType,
    /// Entity name.
    pub name: String,
    /// File path (if applicable).
    pub file: Option<PathBuf>,
    /// Line number (if applicable).
    pub line: Option<usize>,
    /// Confidence score.
    pub confidence: f32,
}

impl ScopeEntity {
    /// Get entity type as a string.
    pub fn entity_type_as_str(&self) -> &'static str {
        match self.entity_type {
            ScopeEntityType::File => "File",
            ScopeEntityType::Function => "Function",
            ScopeEntityType::Class => "Class",
            ScopeEntityType::Module => "Module",
            ScopeEntityType::Namespace => "Namespace",
            ScopeEntityType::Variable => "Variable",
        }
    }
}

/// Result of scope extraction.
#[derive(Debug, Clone)]
pub struct ScopeResult {
    /// Scope level.
    pub level: ScopeLevel,
    /// Identified entities.
    pub entities: Vec<ScopeEntity>,
    /// Whether scope is ambiguous.
    pub is_ambiguous: bool,
}

impl ScopeResult {
    /// Create an empty scope result.
    pub fn new() -> Self {
        Self {
            level: ScopeLevel::Unknown,
            entities: Vec::new(),
            is_ambiguous: false,
        }
    }

    /// Create a file scope.
    pub fn file(path: impl Into<PathBuf>) -> Self {
        let path_buf = path.into();
        Self {
            level: ScopeLevel::File,
            entities: vec![ScopeEntity {
                entity_type: ScopeEntityType::File,
                name: path_buf.to_string_lossy().to_string(),
                file: Some(path_buf),
                line: None,
                confidence: 1.0,
            }],
            is_ambiguous: false,
        }
    }

    /// Create a class scope.
    pub fn class(name: impl Into<String>, file: Option<PathBuf>) -> Self {
        Self {
            level: ScopeLevel::Class,
            entities: vec![ScopeEntity {
                entity_type: ScopeEntityType::Class,
                name: name.into(),
                file,
                line: None,
                confidence: 0.9,
            }],
            is_ambiguous: false,
        }
    }
}

impl Default for ScopeResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Extracts scope from prompts.
pub struct ScopeExtractor {
    /// Known file extensions.
    extensions: Vec<String>,
}

impl ScopeExtractor {
    /// Create a new extractor.
    pub fn new() -> Self {
        Self {
            extensions: vec![
                "rs".into(), "cpp".into(), "h".into(), "hpp".into(),
                "js".into(), "ts".into(), "py".into(), "go".into(),
            ],
        }
    }

    /// Extract scope from a prompt.
    pub fn extract(&self, prompt: &str) -> ScopeResult {
        let prompt_lower = prompt.to_lowercase();
        let mut entities = Vec::new();
        let mut level = ScopeLevel::Unknown;

        // Extract file references
        let file_entities = self.extract_files(prompt);
        if !file_entities.is_empty() {
            level = ScopeLevel::File;
            entities.extend(file_entities);
        }

        // Extract class references
        let class_entities = self.extract_classes(&prompt_lower);
        if !class_entities.is_empty() {
            level = level.max(ScopeLevel::Class);
            entities.extend(class_entities);
        }

        // Extract function references
        let func_entities = self.extract_functions(&prompt_lower);
        if !func_entities.is_empty() {
            level = level.max(ScopeLevel::Function);
            entities.extend(func_entities);
        }

        // Check for module-level keywords
        if self.is_module_level(&prompt_lower) {
            level = ScopeLevel::Module;
        }

        // Check for project-level keywords
        if self.is_project_level(&prompt_lower) {
            level = ScopeLevel::Project;
        }

        let is_ambiguous = entities.len() > 3 || 
            (entities.is_empty() && level == ScopeLevel::Unknown);

        ScopeResult {
            level,
            entities,
            is_ambiguous,
        }
    }

    /// Extract file references from prompt.
    fn extract_files(&self, prompt: &str) -> Vec<ScopeEntity> {
        let mut entities = Vec::new();
        
        // Look for file patterns like "file.rs" or "file.cpp"
        for ext in &self.extensions {
            let pattern = format!(".{}", ext);
            for (i, _) in prompt.match_indices(&pattern) {
                // Find start of filename
                let start = prompt[..i].rfind(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                    .map(|pos| pos + 1)
                    .unwrap_or(0);
                let filename = &prompt[start..i + ext.len() + 1];
                
                entities.push(ScopeEntity {
                    entity_type: ScopeEntityType::File,
                    name: filename.to_string(),
                    file: Some(PathBuf::from(filename)),
                    line: None,
                    confidence: 0.9,
                });
            }
        }

        entities
    }

    /// Extract class references from prompt.
    fn extract_classes(&self, prompt_lower: &str) -> Vec<ScopeEntity> {
        let mut entities = Vec::new();
        
        // Look for class keyword patterns
        let patterns = ["class ", "struct ", "the class", "the struct"];
        for pattern in patterns {
            if let Some(pos) = prompt_lower.find(pattern) {
                // Extract name after pattern
                let after = &prompt_lower[pos + pattern.len()..];
                if let Some(name) = after.split_whitespace().next() {
                    entities.push(ScopeEntity {
                        entity_type: ScopeEntityType::Class,
                        name: name.to_string(),
                        file: None,
                        line: None,
                        confidence: 0.7,
                    });
                }
            }
        }

        entities
    }

    /// Extract function references from prompt.
    fn extract_functions(&self, prompt_lower: &str) -> Vec<ScopeEntity> {
        let mut entities = Vec::new();
        
        // Look for function keyword patterns
        let patterns = ["function ", "method ", "the function", "the method"];
        for pattern in patterns {
            if let Some(pos) = prompt_lower.find(pattern) {
                let after = &prompt_lower[pos + pattern.len()..];
                if let Some(name) = after.split_whitespace().next() {
                    entities.push(ScopeEntity {
                        entity_type: ScopeEntityType::Function,
                        name: name.trim_end_matches(['(', ')']).to_string(),
                        file: None,
                        line: None,
                        confidence: 0.7,
                    });
                }
            }
        }

        entities
    }

    /// Check if prompt suggests module-level scope.
    fn is_module_level(&self, prompt_lower: &str) -> bool {
        let keywords = ["system", "module", "subsystem", "component", "package"];
        keywords.iter().any(|k| prompt_lower.contains(k))
    }

    /// Check if prompt suggests project-level scope.
    fn is_project_level(&self, prompt_lower: &str) -> bool {
        let keywords = ["project", "entire codebase", "all files", "upgrade"];
        keywords.iter().any(|k| prompt_lower.contains(k))
    }
}

impl Default for ScopeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_scope() {
        let extractor = ScopeExtractor::new();
        let result = extractor.extract("Fix the bug in enemy.rs");
        
        assert_eq!(result.level, ScopeLevel::File);
        assert!(!result.entities.is_empty());
        assert!(result.entities[0].name.contains("enemy.rs"));
    }

    #[test]
    fn test_extract_class_scope() {
        let extractor = ScopeExtractor::new();
        let result = extractor.extract("Add a method to the class Enemy");
        
        // Should detect class reference
        assert!(result.entities.iter().any(|e| e.name.contains("enemy")));
    }

    #[test]
    fn test_extract_function_scope() {
        let extractor = ScopeExtractor::new();
        let result = extractor.extract("Update the update function in player");
        
        // Should detect function reference
        assert!(!result.entities.is_empty() || result.level != ScopeLevel::Unknown);
    }

    #[test]
    fn test_extract_module_scope() {
        let extractor = ScopeExtractor::new();
        let result = extractor.extract("Refactor the AI system");
        
        assert_eq!(result.level, ScopeLevel::Module);
    }

    #[test]
    fn test_extract_project_scope() {
        let extractor = ScopeExtractor::new();
        let result = extractor.extract("Upgrade the project to C++20");
        
        assert_eq!(result.level, ScopeLevel::Project);
    }

    #[test]
    fn test_scope_level_ordering() {
        // Unknown < File < Function < Class < Module < Project
        assert!(ScopeLevel::Project > ScopeLevel::Module);
        assert!(ScopeLevel::Module > ScopeLevel::Class);
        assert!(ScopeLevel::Class > ScopeLevel::Function);
        assert!(ScopeLevel::Function > ScopeLevel::File);
        assert!(ScopeLevel::File > ScopeLevel::Unknown);
    }
}
