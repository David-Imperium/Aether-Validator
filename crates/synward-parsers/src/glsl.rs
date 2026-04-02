//! GLSL Parser
//!
//! Parses GLSL shader code using tree-sitter-glsl.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// GLSL shader parser.
pub struct GlslParser;

impl GlslParser {
    /// Create a new GLSL parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for GlslParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for GlslParser {
    fn language(&self) -> &str {
        "glsl"
    }
    
    fn extensions(&self) -> &[&str] {
        &["frag", "vert", "comp", "tesc", "tese", "geom", "glsl"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::glsl(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse GLSL source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
