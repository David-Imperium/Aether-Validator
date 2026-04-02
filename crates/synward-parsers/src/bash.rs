//! Bash/Shell Parser
//!
//! Parses Bash shell scripts using tree-sitter-bash.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// Bash shell script parser.
pub struct BashParser;

impl BashParser {
    /// Create a new Bash parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BashParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for BashParser {
    fn language(&self) -> &str {
        "bash"
    }
    
    fn extensions(&self) -> &[&str] {
        &["sh", "bash", "zsh", "ksh"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::bash(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse Bash source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
