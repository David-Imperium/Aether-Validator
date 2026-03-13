//! Parser trait — Abstraction for language-specific parsers

use async_trait::async_trait;
use crate::{AST, ParseResult};

/// Parser trait for language-specific parsing.
///
/// Each language supported by Aether must implement this trait
/// to provide:
/// - Parsing from source code to AST
/// - Language identification
/// - File extension matching
#[async_trait]
pub trait Parser: Send + Sync {
    /// Parse source code into an AST.
    async fn parse(&self, source: &str) -> ParseResult<AST>;
    
    /// Get the language name this parser handles.
    fn language(&self) -> &str;
    
    /// Get the file extensions this parser handles.
    fn extensions(&self) -> &[&str];
    
    /// Check if this parser can handle the given file.
    fn can_parse(&self, path: &str) -> bool {
        let path = path.to_lowercase();
        self.extensions().iter().any(|ext| path.ends_with(ext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockParser;
    
    #[async_trait]
    impl Parser for MockParser {
        async fn parse(&self, _source: &str) -> ParseResult<AST> {
            Ok(AST::default())
        }
        
        fn language(&self) -> &str {
            "mock"
        }
        
        fn extensions(&self) -> &[&str] {
            &[".mock"]
        }
    }

    #[tokio::test]
    async fn test_parser_can_parse() {
        let parser = MockParser;
        assert!(parser.can_parse("test.mock"));
        assert!(!parser.can_parse("test.rs"));
    }
}
