//! YAML Parser
//!
//! Parses YAML documents using tree-sitter-yaml.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// YAML document parser.
pub struct YamlParser;

impl YamlParser {
    /// Create a new YAML parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for YamlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for YamlParser {
    fn language(&self) -> &str {
        "yaml"
    }
    
    fn extensions(&self) -> &[&str] {
        &["yaml", "yml"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::yaml(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse YAML source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
