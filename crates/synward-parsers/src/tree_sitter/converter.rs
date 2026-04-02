//! Tree-sitter to AST Conversion
//!
//! Converts tree-sitter parse trees to our generic AST format.

use crate::ast::{AST, ASTNode, NodeKind, Token, TokenKind, Span};
use tree_sitter::{Node, Tree};

/// Convert a tree-sitter tree to our AST format.
pub struct TreeSitterConverter;

impl TreeSitterConverter {
    /// Convert a tree-sitter tree to AST.
    pub fn convert(tree: &Tree, source: &str) -> AST {
        let root = tree.root_node();
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        
        // Check for parse errors
        if root.has_error() {
            Self::collect_errors(&root, source, &mut errors);
        }
        
        let root_node = Self::convert_node(&root, source, &mut tokens);
        let mut ast = AST::new(root_node);
        ast.tokens = tokens;
        ast.errors = errors;
        ast
    }
    
    /// Convert a single node and its children.
    fn convert_node(
        node: &Node,
        source: &str,
        tokens: &mut Vec<Token>,
    ) -> ASTNode {
        let kind = Self::node_kind(node);
        let span = Self::node_span(node);
        
        // Process children first
        let mut children = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            children.push(Self::convert_node(&child, source, tokens));
        }
        
        // Add token for leaf nodes
        if children.is_empty() {
            let text = node.utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string();
            tokens.push(Token::new(
                Self::token_kind(node),
                text,
                span,
            ));
        }
        
        ASTNode {
            kind,
            span,
            children,
        }
    }
    
    /// Collect parse errors.
    fn collect_errors(node: &Node, source: &str, errors: &mut Vec<String>) {
        if node.is_error() {
            let text = node.utf8_text(source.as_bytes())
                .unwrap_or("<error>");
            errors.push(format!("Parse error at {:?}: {}", node.start_position(), text));
        }
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::collect_errors(&child, source, errors);
        }
    }
    
    /// Map tree-sitter node kind to our NodeKind.
    fn node_kind(node: &Node) -> NodeKind {
        match node.kind() {
            // Common
            "module" | "program" | "source_file" => NodeKind::Module,
            
            // Functions (all languages)
            "function_definition" | "function_declaration" | "method_definition" 
            | "function_item" | "arrow_function" | "function_expression" 
            | "func_literal" | "method_declaration" | "constructor_declaration" => NodeKind::Function,
            
            // Structs/Classes
            "class_definition" | "class_declaration" | "struct_item" 
            | "struct_definition" => NodeKind::Struct,
            
            // Enums
            "enum_definition" | "enum_declaration" | "enum_item" => NodeKind::Enum,
            
            // Traits/Interfaces
            "trait_definition" | "interface_declaration" | "trait_item" => NodeKind::Trait,
            
            // Impl blocks
            "impl_item" | "impl_block" => NodeKind::Impl,
            
            // Imports
            "use_statement" | "use_declaration" | "import_statement" | "import_declaration" => NodeKind::Use,
            
            // Variables/Let
            "let_declaration" | "variable_declaration" | "lexical_declaration" 
            | "const_item" | "field_declaration" | "variable_declarator" => NodeKind::Let,
            
            // Statements
            "if_statement" | "if_expression" | "if" => NodeKind::Stmt,
            "for_statement" | "for_expression" | "for_in_statement" | "for" => NodeKind::Stmt,
            "while_statement" | "while_expression" | "while" => NodeKind::Stmt,
            "return_statement" | "return_expression" | "return" => NodeKind::Stmt,
            "expression_statement" | "expression" | "expr" => NodeKind::Expr,
            
            // Fallback
            _ => NodeKind::Unknown,
        }
    }
    
    /// Get token kind for a node.
    fn token_kind(node: &Node) -> TokenKind {
        match node.kind() {
            // Identifiers
            "identifier" | "property_identifier" | "type_identifier" | "field_identifier" => TokenKind::Identifier,
            
            // Literals
            "string" | "string_literal" | "raw_string_literal" => TokenKind::String,
            "number" | "integer" | "float" | "number_literal" | "integer_literal" | "float_literal" => TokenKind::Number,
            
            // Booleans
            "true" | "false" | "boolean" | "bool_literal" => TokenKind::Keyword,
            
            // Comments
            "comment" | "line_comment" | "block_comment" => TokenKind::Comment,
            
            // Keywords
            "if" | "else" | "elif" | "match" | "switch" => TokenKind::Keyword,
            "for" | "while" | "do" | "loop" => TokenKind::Keyword,
            "return" | "break" | "continue" | "yield" => TokenKind::Keyword,
            "fn" | "func" | "function" | "def" | "fun" => TokenKind::Keyword,
            "class" | "struct" | "enum" | "interface" | "trait" | "type" => TokenKind::Keyword,
            "import" | "export" | "from" | "module" | "use" | "package" => TokenKind::Keyword,
            "let" | "const" | "var" | "val" | "mut" | "static" => TokenKind::Keyword,
            "pub" | "priv" | "private" | "public" | "protected" | "internal" => TokenKind::Keyword,
            
            // Operators
            "+" | "-" | "*" | "/" | "%" | "^" | "&" | "|" | "~"
            | "==" | "!=" | "<=" | ">=" | "<=>"
            | "&&" | "||" | "!" | "and" | "or" | "not"
            | "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "<<=" | ">>=" => TokenKind::Operator,
            
            // Comparison operators (must be before punctuation to avoid overlap with < and >)
            "<" | ">" => TokenKind::Operator,
            
            // Punctuation
            "(" | ")" | "[" | "]" | "{" | "}"
            | "." | "," | ";" | ":" | "::" | "->" | "=>" | ".." | "..." => TokenKind::Punctuation,
            
            // Fallback
            _ => TokenKind::Unknown,
        }
    }
    
    /// Get span for a node.
    fn node_span(node: &Node) -> Span {
        Span::new(
            node.start_byte(),
            node.end_byte(),
        )
    }
}
