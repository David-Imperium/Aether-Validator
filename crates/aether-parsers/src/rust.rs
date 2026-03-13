//! Rust Parser — Parse Rust source code using syn

use async_trait::async_trait;
use syn::{parse_file, visit::Visit, Expr};

use crate::parser::Parser;
use crate::ast::{AST, ASTNode, NodeKind, Token, TokenKind, Span as ASTSpan};
use crate::error::{ParseError, ParseResult};

/// Parser for Rust source code.
pub struct RustParser;

impl RustParser {
    /// Create a new Rust parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for RustParser {
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        // Parse using syn
        let syntax = parse_file(source)
            .map_err(|e| ParseError::Syntax {
                line: 0, // proc_macro2 doesn't expose line info easily
                column: 0,
                message: e.to_string(),
            })?;

        // Convert to AST
        let mut visitor = ASTVisitor::new();
        visitor.visit_file(&syntax);

        Ok(visitor.into_ast())
    }

    fn language(&self) -> &str {
        "rust"
    }

    fn extensions(&self) -> &[&str] {
        &[".rs"]
    }
}

/// Visitor to convert syn AST to our AST.
struct ASTVisitor {
    root: ASTNode,
    tokens: Vec<Token>,
    errors: Vec<String>,
}

impl ASTVisitor {
    fn new() -> Self {
        Self {
            root: ASTNode::default(),
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn into_ast(self) -> AST {
        AST {
            root: self.root,
            tokens: self.tokens,
            errors: self.errors,
        }
    }

    fn add_token(&mut self, kind: TokenKind, text: String, span: ASTSpan) {
        self.tokens.push(Token::new(kind, text, span));
    }
}

impl<'a> Visit<'a> for ASTVisitor {
    fn visit_item_fn(&mut self, f: &'a syn::ItemFn) {
        let child = ASTNode {
            kind: NodeKind::Function,
            span: ASTSpan::new(0, 0), // Simplified span
            children: Vec::new(),
        };
        self.root.children.push(child);
        syn::visit::visit_item_fn(self, f);
    }

    fn visit_item_struct(&mut self, s: &'a syn::ItemStruct) {
        let child = ASTNode {
            kind: NodeKind::Struct,
            span: ASTSpan::new(0, 0),
            children: Vec::new(),
        };
        self.root.children.push(child);
        syn::visit::visit_item_struct(self, s);
    }

    fn visit_item_enum(&mut self, e: &'a syn::ItemEnum) {
        let child = ASTNode {
            kind: NodeKind::Enum,
            span: ASTSpan::new(0, 0),
            children: Vec::new(),
        };
        self.root.children.push(child);
        syn::visit::visit_item_enum(self, e);
    }

    fn visit_item_trait(&mut self, t: &'a syn::ItemTrait) {
        let child = ASTNode {
            kind: NodeKind::Trait,
            span: ASTSpan::new(0, 0),
            children: Vec::new(),
        };
        self.root.children.push(child);
        syn::visit::visit_item_trait(self, t);
    }

    fn visit_item_impl(&mut self, i: &'a syn::ItemImpl) {
        let child = ASTNode {
            kind: NodeKind::Impl,
            span: ASTSpan::new(0, 0),
            children: Vec::new(),
        };
        self.root.children.push(child);
        syn::visit::visit_item_impl(self, i);
    }

    fn visit_item_use(&mut self, u: &'a syn::ItemUse) {
        let child = ASTNode {
            kind: NodeKind::Use,
            span: ASTSpan::new(0, 0),
            children: Vec::new(),
        };
        self.root.children.push(child);
        syn::visit::visit_item_use(self, u);
    }

    fn visit_local(&mut self, l: &'a syn::Local) {
        let child = ASTNode {
            kind: NodeKind::Let,
            span: ASTSpan::new(0, 0),
            children: Vec::new(),
        };
        self.root.children.push(child);
        syn::visit::visit_local(self, l);
    }

    fn visit_expr(&mut self, e: &'a Expr) {
        // Extract tokens from expressions
        if let Expr::Path(p) = e {
            if let Some(ident) = p.path.get_ident() {
                self.add_token(
                    TokenKind::Identifier,
                    ident.to_string(),
                    ASTSpan::new(0, 0),
                );
            }
        }
        syn::visit::visit_expr(self, e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_simple_function() {
        let source = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        let parser = RustParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert!(ast.root.children.iter().any(|n| n.kind == NodeKind::Function));
    }

    #[tokio::test]
    async fn test_parse_struct() {
        let source = r#"
struct Point {
    x: f64,
    y: f64,
}
"#;
        let parser = RustParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert!(ast.root.children.iter().any(|n| n.kind == NodeKind::Struct));
    }

    #[tokio::test]
    async fn test_parse_invalid() {
        let source = "fn main( {";  // Invalid syntax
        let parser = RustParser::new();
        let result = parser.parse(source).await;
        
        assert!(result.is_err());
    }

    #[test]
    fn test_extensions() {
        let parser = RustParser::new();
        assert!(parser.can_parse("main.rs"));
        assert!(parser.can_parse("lib.rs"));
        assert!(!parser.can_parse("main.cpp"));
    }
}
