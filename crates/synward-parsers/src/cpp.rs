//! C++ Parser
//!
//! Parses C++ source code using tree-sitter-cpp.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// C++ source code parser.
pub struct CppParser;

impl CppParser {
    /// Create a new C++ parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CppParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for CppParser {
    fn language(&self) -> &str {
        "cpp"
    }
    
    fn extensions(&self) -> &[&str] {
        &["cpp", "cc", "cxx", "hpp", "h", "hxx"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::cpp(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse C++ source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
