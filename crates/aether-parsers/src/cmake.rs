//! CMake Parser
//!
//! Parses CMake build files using tree-sitter-cmake.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// CMake build system parser.
#[derive(Debug)]
pub struct CmakeParser;

impl CmakeParser {
    /// Create a new CMake parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CmakeParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for CmakeParser {
    fn language(&self) -> &str {
        "cmake"
    }

    fn extensions(&self) -> &[&str] {
        &["cmake"]
    }

    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::cmake(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse CMake source".to_string()))?;

        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
