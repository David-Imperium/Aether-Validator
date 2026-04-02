//! CUDA Parser
//!
//! Parses CUDA GPU programming files using tree-sitter-cuda.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// CUDA GPU programming parser.
#[derive(Debug)]
pub struct CudaParser;

impl CudaParser {
    /// Create a new CUDA parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CudaParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for CudaParser {
    fn language(&self) -> &str {
        "cuda"
    }

    fn extensions(&self) -> &[&str] {
        &["cu", "cuh"]
    }

    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::cuda(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse CUDA source".to_string()))?;

        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
