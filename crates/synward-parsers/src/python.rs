//! Python Parser
//!
//! Parses Python source code using tree-sitter-python.

use crate::ast::AST;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

use async_trait::async_trait;

/// Python source code parser.
pub struct PythonParser;

impl PythonParser {
    /// Create a new Python parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PythonParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for PythonParser {
    fn language(&self) -> &str {
        "python"
    }
    
    fn extensions(&self) -> &[&str] {
        &["py", "pyw"]
    }
    
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::python(), source)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse Python source".to_string()))?;
        
        Ok(TreeSitterConverter::convert(&tree, source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_python_parse_simple() {
        let parser = PythonParser::new();
        let source = r#"
def hello():
    print("Hello, World!")

class MyClass:
    def __init__(self):
        self.value = 42
"#;
        let ast = parser.parse(source).await.unwrap();
        assert!(!ast.has_errors());
    }
    
    #[tokio::test]
    async fn test_python_parse_error() {
        let parser = PythonParser::new();
        let source = "def broken(";  // Incomplete function
        let ast = parser.parse(source).await.unwrap();
        assert!(ast.has_errors());
    }
}
