//! TOML Parser
//!
//! Parses TOML configuration files using tree-sitter-toml-ng.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// TOML configuration parser.
#[derive(Debug)]
pub struct TomlParser;

impl TomlParser {
    /// Create a new TOML parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TomlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for TomlParser {
    fn language(&self) -> &str {
        "toml"
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }

    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::toml(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse TOML source".to_string()))?;

        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
