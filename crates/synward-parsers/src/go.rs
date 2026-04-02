//! Go Parser
//!
//! Parses Go source code using tree-sitter-go.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// Go source code parser.
pub struct GoParser;

impl GoParser {
    /// Create a new Go parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for GoParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for GoParser {
    fn language(&self) -> &str {
        "go"
    }
    
    fn extensions(&self) -> &[&str] {
        &["go"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::go(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse Go source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
