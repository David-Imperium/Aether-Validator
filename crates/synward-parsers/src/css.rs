//! CSS Parser
//!
//! Parses CSS stylesheets using tree-sitter-css.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// CSS stylesheet parser.
pub struct CssParser;

impl CssParser {
    /// Create a new CSS parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CssParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for CssParser {
    fn language(&self) -> &str {
        "css"
    }
    
    fn extensions(&self) -> &[&str] {
        &["css"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::css(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse CSS source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
