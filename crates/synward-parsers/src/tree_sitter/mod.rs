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
    
    /// Get C language.
    pub fn c() -> Language {
        tree_sitter_c::LANGUAGE.into()
    }
    
    /// Get GLSL language.
    pub fn glsl() -> Language {
        tree_sitter_glsl::LANGUAGE_GLSL.into()
    }
    
    /// Get CSS language.
    pub fn css() -> Language {
        tree_sitter_css::LANGUAGE.into()
    }
    
    /// Get HTML language.
    pub fn html() -> Language {
        tree_sitter_html::LANGUAGE.into()
    }
    
    /// Get JSON language.
    pub fn json() -> Language {
        tree_sitter_json::LANGUAGE.into()
    }
    
    /// Get YAML language.
    pub fn yaml() -> Language {
        tree_sitter_yaml::LANGUAGE.into()
    }

    /// Get TOML language (via tree-sitter-toml-ng).
    pub fn toml() -> Language {
        tree_sitter_toml_ng::LANGUAGE.into()
    }

    /// Get CMake language.
    pub fn cmake() -> Language {
        tree_sitter_cmake::LANGUAGE.into()
    }

    /// Get CUDA language.
    pub fn cuda() -> Language {
        tree_sitter_cuda::LANGUAGE.into()
    }

    /// Get SQL language (via tree-sitter-sequel).
    pub fn sql() -> Language {
        tree_sitter_sequel::LANGUAGE.into()
    }

    /// Get GraphQL language.
    pub fn graphql() -> Language {
        tree_sitter_graphql::LANGUAGE.into()
    }

    /// Get Markdown language.
    pub fn markdown() -> Language {
        tree_sitter_md::LANGUAGE.into()
    }

    /// Get Bash/Shell language.
    pub fn bash() -> Language {
        tree_sitter_bash::LANGUAGE.into()
    }
}
