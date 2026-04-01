//! Aether Parsers — Parser implementations for multiple languages
//!
//! This crate provides:
//! - Parser trait abstraction
//! - Parser registry for language dispatch
//! - Implementations for Rust, Python, JavaScript, TypeScript, C++, Go, Java, Lua
//! - Extended: C, GLSL, CSS, HTML, JSON, YAML, TOML, CMake, CUDA
//! - New: SQL, GraphQL, Markdown, Dockerfile, Bash
//! - Private: Prism (David's systems language)
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
// Extended parsers
pub mod c;
pub mod glsl;
pub mod css;
pub mod html;
pub mod json;
pub mod yaml;
pub mod toml;
pub mod cmake;
pub mod cuda;
// New parsers
pub mod sql;
pub mod graphql;
pub mod markdown;
pub mod bash;
pub mod notebook;
// Private parsers (David only)
pub mod prism;

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
// Extended parser exports
pub use c::CParser;
pub use glsl::GlslParser;
pub use css::CssParser;
pub use html::HtmlParser;
pub use json::JsonParser;
pub use yaml::YamlParser;
pub use toml::TomlParser;
pub use cmake::CmakeParser;
pub use cuda::CudaParser;
// New parser exports
pub use sql::SqlParser;
pub use graphql::GraphQLParser;
pub use markdown::MarkdownParser;
pub use bash::BashParser;
pub use notebook::NotebookParser;
// Private parser exports
pub use prism::PrismParser;
