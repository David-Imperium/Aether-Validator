//! Aether Parsers — Parser implementations for multiple languages
//!
//! This crate provides:
//! - Parser trait abstraction
//! - Parser registry for language dispatch
//! - Implementations for Rust, Python, JavaScript, TypeScript, C++, Go, Java, Lua
//! - AST pattern matching utilities
//! - Tree-sitter integration for multi-language support

mod parser;
mod registry;
mod ast;
mod ast_matcher;
mod error;
pub mod tree_sitter;

pub use parser::Parser;
pub use registry::ParserRegistry;
pub use ast::{AST, ASTNode, Token, NodeKind, Span, TokenKind};
pub use ast_matcher::{ASTMatcher, NodePattern, ASTQuery, ASTMatch};
pub use error::{ParseError, ParseResult};
pub use tree_sitter::{TreeSitterConverter, parse_source};

// Language-specific parsers
pub mod rust;
pub mod python;
pub mod javascript;
pub mod typescript;
pub mod cpp;
pub mod go;
pub mod java;
pub mod lua;
pub mod lex;

// Re-export parsers for convenience
pub use rust::RustParser;
pub use python::PythonParser;
pub use javascript::JavaScriptParser;
pub use typescript::TypeScriptParser;
pub use cpp::CppParser;
pub use go::GoParser;
pub use java::JavaParser;
pub use lua::LuaParser;
pub use lex::LexParser;
