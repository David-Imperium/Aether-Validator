//! SCSS Parser
//!
//! Parses SCSS stylesheets using tree-sitter-scss.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// SCSS stylesheet parser.
pub struct ScssParser;

impl ScssParser {
    /// Create a new SCSS parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ScssParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for ScssParser {
    fn language(&self) -> &str {
        "scss"
    }
    
    fn extensions(&self) -> &[&str] {
        &["scss", "sass"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::scss(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse SCSS source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
