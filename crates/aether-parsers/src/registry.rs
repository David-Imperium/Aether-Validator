//! Parser Registry — Language dispatch for parsers

use std::collections::HashMap;
use std::sync::Arc;

use crate::parser::Parser;
use crate::error::{ParseError, ParseResult};

/// Registry for language-specific parsers.
///
/// The registry allows:
/// - Registering parsers by language
/// - Looking up parsers by file extension
/// - Automatic parser selection based on file type
pub struct ParserRegistry {
    parsers: HashMap<String, Arc<dyn Parser>>,
    extension_map: HashMap<String, String>,
}

impl ParserRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
            extension_map: HashMap::new(),
        }
    }

    /// Create a registry with all standard parsers registered.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(crate::rust::RustParser::new());
        registry.register(crate::python::PythonParser::new());
        registry.register(crate::javascript::JavaScriptParser::new());
        registry.register(crate::typescript::TypeScriptParser::new());
        registry.register(crate::cpp::CppParser::new());
        registry.register(crate::go::GoParser::new());
        registry.register(crate::java::JavaParser::new());
        registry.register(crate::lua::LuaParser::new());
        registry.register(crate::lex::LexParser::new());
        registry
    }

    /// Register a parser for a language.
    pub fn register(&mut self, parser: impl Parser + 'static) {
        let language = parser.language().to_string();
        let extensions: Vec<String> = parser.extensions()
            .iter()
            .map(|s| s.to_string())
            .collect();
        
        for ext in extensions {
            self.extension_map.insert(ext, language.clone());
        }
        
        self.parsers.insert(language, Arc::new(parser));
    }

    /// Get a parser by language name.
    pub fn get(&self, language: &str) -> Option<Arc<dyn Parser>> {
        self.parsers.get(language).cloned()
    }

    /// Get a parser for a file path.
    pub fn get_for_file(&self, path: &str) -> ParseResult<Arc<dyn Parser>> {
        let path_lower = path.to_lowercase();
        
        for (ext, language) in &self.extension_map {
            if path_lower.ends_with(ext) {
                return self.parsers.get(language)
                    .cloned()
                    .ok_or_else(|| ParseError::ParserNotFound(language.clone()));
            }
        }
        
        Err(ParseError::UnknownExtension(path.to_string()))
    }

    /// List all registered languages.
    pub fn languages(&self) -> Vec<&str> {
        self.parsers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AST;
    use async_trait::async_trait;

    struct RustParser;
    
    #[async_trait]
    impl Parser for RustParser {
        async fn parse(&self, _source: &str) -> ParseResult<AST> {
            Ok(AST::default())
        }
        
        fn language(&self) -> &str { "rust" }
        fn extensions(&self) -> &[&str] { &[".rs"] }
    }

    struct CppParser;
    
    #[async_trait]
    impl Parser for CppParser {
        async fn parse(&self, _source: &str) -> ParseResult<AST> {
            Ok(AST::default())
        }
        
        fn language(&self) -> &str { "cpp" }
        fn extensions(&self) -> &[&str] { &[".cpp", ".h", ".hpp"] }
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ParserRegistry::new();
        registry.register(RustParser);
        registry.register(CppParser);
        
        assert!(registry.get("rust").is_some());
        assert!(registry.get("cpp").is_some());
        assert!(registry.get("python").is_none());
    }

    #[test]
    fn test_registry_get_for_file() {
        let mut registry = ParserRegistry::new();
        registry.register(RustParser);
        registry.register(CppParser);
        
        assert!(registry.get_for_file("main.rs").is_ok());
        assert!(registry.get_for_file("main.cpp").is_ok());
        assert!(registry.get_for_file("main.py").is_err());
    }

    #[test]
    fn test_registry_with_defaults() {
        let registry = ParserRegistry::with_defaults();
        
        // Check all standard parsers are registered
        assert!(registry.get("rust").is_some());
        assert!(registry.get("python").is_some());
        assert!(registry.get("javascript").is_some());
        assert!(registry.get("typescript").is_some());
        assert!(registry.get("cpp").is_some());
        assert!(registry.get("go").is_some());
        assert!(registry.get("java").is_some());
        assert!(registry.get("lua").is_some());
        assert!(registry.get("lex").is_some());
    }

    #[test]
    fn test_registry_lex_file() {
        let registry = ParserRegistry::with_defaults();
        
        assert!(registry.get_for_file("game.lex").is_ok());
        assert!(registry.get_for_file("units.lex").is_ok());
        
        let parser = registry.get_for_file("game.lex").unwrap();
        assert_eq!(parser.language(), "lex");
    }
}
