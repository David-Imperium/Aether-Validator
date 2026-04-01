//! JSON Parser
//!
//! Parses JSON documents using tree-sitter-json.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// JSON document parser.
pub struct JsonParser;

impl JsonParser {
    /// Create a new JSON parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for JsonParser {
    fn language(&self) -> &str {
        "json"
    }
    
    fn extensions(&self) -> &[&str] {
        &["json"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::json(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse JSON source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
