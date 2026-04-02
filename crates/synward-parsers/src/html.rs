//! HTML Parser
//!
//! Parses HTML documents using tree-sitter-html.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// HTML document parser.
pub struct HtmlParser;

impl HtmlParser {
    /// Create a new HTML parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for HtmlParser {
    fn language(&self) -> &str {
        "html"
    }
    
    fn extensions(&self) -> &[&str] {
        &["html", "htm"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::html(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse HTML source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
