//! Java Parser
//!
//! Parses Java source code using tree-sitter-java.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// Java source code parser.
pub struct JavaParser;

impl JavaParser {
    /// Create a new Java parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for JavaParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for JavaParser {
    fn language(&self) -> &str {
        "java"
    }
    
    fn extensions(&self) -> &[&str] {
        &["java"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::java(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse Java source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
