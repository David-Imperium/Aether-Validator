//! SQL Parser
//!
//! Parses SQL source code using tree-sitter-sequel (general SQL grammar).

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// SQL source code parser.
pub struct SqlParser;

impl SqlParser {
    /// Create a new SQL parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for SqlParser {
    fn language(&self) -> &str {
        "sql"
    }
    
    fn extensions(&self) -> &[&str] {
        &["sql", "ddl", "dml"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::sql(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse SQL source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
