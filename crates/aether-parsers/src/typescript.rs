//! TypeScript Parser
//!
//! Parses TypeScript source code using tree-sitter-typescript.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// TypeScript source code parser.
pub struct TypeScriptParser;

impl TypeScriptParser {
    /// Create a new TypeScript parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypeScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for TypeScriptParser {
    fn language(&self) -> &str {
        "typescript"
    }
    
    fn extensions(&self) -> &[&str] {
        &["ts", "tsx", "mts", "cts"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::typescript(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse TypeScript source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
