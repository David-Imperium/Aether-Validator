//! Markdown Parser
//!
//! Parses Markdown documents using tree-sitter-md (CommonMark compatible).

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// Markdown document parser.
pub struct MarkdownParser;

impl MarkdownParser {
    /// Create a new Markdown parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for MarkdownParser {
    fn language(&self) -> &str {
        "markdown"
    }
    
    fn extensions(&self) -> &[&str] {
        &["md", "markdown", "mdown", "mkd"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::markdown(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse Markdown source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
