//! JavaScript Parser
//!
//! Parses JavaScript source code using tree-sitter-javascript.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// JavaScript source code parser.
pub struct JavaScriptParser;

impl JavaScriptParser {
    /// Create a new JavaScript parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for JavaScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for JavaScriptParser {
    fn language(&self) -> &str {
        "javascript"
    }
    
    fn extensions(&self) -> &[&str] {
        &["js", "jsx", "mjs", "cjs"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::javascript(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse JavaScript source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
