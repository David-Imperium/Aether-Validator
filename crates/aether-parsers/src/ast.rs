//! AST types — Abstract Syntax Tree representation

use std::fmt;

/// Abstract Syntax Tree.
#[derive(Debug, Clone, Default)]
pub struct AST {
    pub root: ASTNode,
    pub tokens: Vec<Token>,
    pub errors: Vec<String>,
}

impl AST {
    /// Create a new empty AST.
    pub fn new(root: ASTNode) -> Self {
        Self {
            root,
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Check if the AST has any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Node in the AST.
#[derive(Debug, Clone)]
pub struct ASTNode {
    pub kind: NodeKind,
    pub span: Span,
    pub children: Vec<ASTNode>,
}

impl Default for ASTNode {
    fn default() -> Self {
        Self {
            kind: NodeKind::Unknown,
            span: Span::default(),
            children: Vec::new(),
        }
    }
}

/// Kind of AST node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    // General programming constructs
    /// Root module
    Module,
    /// Function definition
    Function,
    /// Struct definition
    Struct,
    /// Enum definition
    Enum,
    /// Trait definition
    Trait,
    /// Impl block
    Impl,
    /// Use statement
    Use,
    /// Let binding
    Let,
    /// Expression
    Expr,
    /// Statement
    Stmt,
    
    // Lex language constructs
    /// Resource definition (Lex)
    Resource,
    /// Era definition (Lex)
    Era,
    /// Structure definition (Lex - game entity)
    LexStructure,
    /// Unit definition (Lex)
    Unit,
    /// Technology definition (Lex)
    Technology,
    /// Event definition (Lex)
    Event,
    /// Choice definition (Lex)
    Choice,
    /// Property definition (Lex)
    Property,
    /// Condition block (Lex)
    Condition,
    
    /// Unknown node
    Unknown,
}

/// Source span.
#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// Token from lexer.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, text: String, span: Span) -> Self {
        Self { kind, text, span }
    }
}

/// Kind of token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Keyword,
    String,
    Number,
    Operator,
    Punctuation,
    Whitespace,
    Comment,
    Unknown,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::Keyword => write!(f, "keyword"),
            TokenKind::String => write!(f, "string"),
            TokenKind::Number => write!(f, "number"),
            TokenKind::Operator => write!(f, "operator"),
            TokenKind::Punctuation => write!(f, "punctuation"),
            TokenKind::Whitespace => write!(f, "whitespace"),
            TokenKind::Comment => write!(f, "comment"),
            TokenKind::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_creation() {
        let ast = AST::default();
        assert!(!ast.has_errors());
    }

    #[test]
    fn test_span() {
        let span = Span::new(0, 10);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 10);
    }
}
