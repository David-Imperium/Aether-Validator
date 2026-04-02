//! C Parser
//!
//! Parses C source code using tree-sitter-c.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// C source code parser.
pub struct CParser;

impl CParser {
    /// Create a new C parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for CParser {
    fn language(&self) -> &str {
        "c"
    }
    
    fn extensions(&self) -> &[&str] {
        &["c", "h"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::c(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse C source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
