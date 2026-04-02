//! GraphQL Parser
//!
//! Parses GraphQL schema and query documents using tree-sitter-graphql.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// GraphQL source code parser.
pub struct GraphQLParser;

impl GraphQLParser {
    /// Create a new GraphQL parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for GraphQLParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for GraphQLParser {
    fn language(&self) -> &str {
        "graphql"
    }
    
    fn extensions(&self) -> &[&str] {
        &["graphql", "gql"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::graphql(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse GraphQL source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}
