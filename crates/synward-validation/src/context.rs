//! Validation Context — Shared state during validation

use std::path::PathBuf;
use std::collections::HashMap;

/// Context passed through validation layers.
#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
    /// File being validated.
    pub file_path: Option<PathBuf>,
    /// Source code being validated.
    pub source: String,
    /// Language of the source.
    pub language: String,
    /// Custom metadata.
    pub metadata: HashMap<String, String>,
}

impl ValidationContext {
    /// Create a new context for a file.
    pub fn for_file(path: impl Into<PathBuf>, source: String, language: String) -> Self {
        Self {
            file_path: Some(path.into()),
            source,
            language,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the context.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ValidationContext::for_file("test.rs", "fn main() {}".into(), "rust".into());
        assert_eq!(ctx.language, "rust");
        assert!(ctx.file_path.is_some());
    }
}
