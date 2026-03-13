//! Tree-sitter Integration
//!
//! Provides AST parsing for multiple languages using tree-sitter grammars.

mod converter;

pub use converter::TreeSitterConverter;

use tree_sitter::{Parser, Language, Tree};

/// Parse source code using tree-sitter.
pub fn parse_source(
    language: Language,
    source: &str,
) -> Option<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;
    parser.parse(source, None)
}

/// Language-specific parsers.
pub mod languages {
    use tree_sitter::Language;
    
    /// Get Python language.
    pub fn python() -> Language {
        tree_sitter_python::LANGUAGE.into()
    }
    
    /// Get JavaScript language.
    pub fn javascript() -> Language {
        tree_sitter_javascript::LANGUAGE.into()
    }
    
    /// Get TypeScript language.
    pub fn typescript() -> Language {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
    }
    
    /// Get TypeScript (TSX) language.
    pub fn tsx() -> Language {
        tree_sitter_typescript::LANGUAGE_TSX.into()
    }
    
    /// Get C++ language.
    pub fn cpp() -> Language {
        tree_sitter_cpp::LANGUAGE.into()
    }
    
    /// Get Go language.
    pub fn go() -> Language {
        tree_sitter_go::LANGUAGE.into()
    }
    
    /// Get Java language.
    pub fn java() -> Language {
        tree_sitter_java::LANGUAGE.into()
    }
}
