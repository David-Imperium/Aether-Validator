//! Lua Parser — Parse Lua source code (Love2D support)
//!
//! Simple regex-based parser for Lua. Can be upgraded to tree-sitter later.

use async_trait::async_trait;
use regex::Regex;

use crate::parser::Parser;
use crate::ast::{AST, ASTNode, NodeKind, Token, TokenKind, Span as ASTSpan};
use crate::error::ParseResult;

/// Parser for Lua source code (including Love2D).
#[allow(dead_code)] // Fields prepared for future: comment/string parsing
pub struct LuaParser {
    function_pattern: Regex,
    local_function_pattern: Regex,
    table_pattern: Regex,
    method_pattern: Regex,
    comment_pattern: Regex,
    string_pattern: Regex,
}

impl LuaParser {
    /// Create a new Lua parser.
    pub fn new() -> Self {
        Self {
            // function name(args)
            function_pattern: Regex::new(r"function\s+([a-zA-Z_][a-zA-Z0-9_\.:]*)\s*\(").expect("valid regex"),
            // local function name(args)
            local_function_pattern: Regex::new(r"local\s+function\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").expect("valid regex"),
            // local name = {...}
            table_pattern: Regex::new(r"local\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*\{").expect("valid regex"),
            // function obj:method(args)
            method_pattern: Regex::new(r"function\s+([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*)").expect("valid regex"),
            // -- comment
            comment_pattern: Regex::new(r"--[^\n]*").expect("valid regex"),
            // "string" or 'string' or [[string]]
            string_pattern: Regex::new(r#"["'][^"']*["']|//!\[\[[\s\S]*?\]\]"#).expect("valid regex"),
        }
    }
}

impl Default for LuaParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for LuaParser {
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let mut root = ASTNode::default();
        let mut tokens = Vec::new();
        let errors = Vec::new();

        // Track line numbers
        let lines: Vec<&str> = source.lines().collect();

        // Parse functions
        for (line_num, line) in lines.iter().enumerate() {
            // Skip comments
            if line.trim().starts_with("--") {
                continue;
            }

            // Check for local function
            if let Some(caps) = self.local_function_pattern.captures(line) {
                let name = caps.get(1).map(|m| m.as_str()).unwrap_or("anonymous");
                root.children.push(ASTNode {
                    kind: NodeKind::Function,
                    span: ASTSpan::new(line_num, 0),
                    children: vec![],
                });
                tokens.push(Token::new(
                    TokenKind::Identifier,
                    name.to_string(),
                    ASTSpan::new(line_num, 0),
                ));
            }
            // Check for regular function or method
            else if let Some(caps) = self.function_pattern.captures(line) {
                let full_name = caps.get(1).map(|m| m.as_str()).unwrap_or("anonymous");
                
                // Check if it's a method (obj:method)
                if let Some(method_caps) = self.method_pattern.captures(line) {
                    let obj = method_caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let method = method_caps.get(2).map(|m| m.as_str()).unwrap_or("");
                    tokens.push(Token::new(
                        TokenKind::Identifier,
                        format!("{}:{}", obj, method),
                        ASTSpan::new(line_num, 0),
                    ));
                } else {
                    tokens.push(Token::new(
                        TokenKind::Identifier,
                        full_name.to_string(),
                        ASTSpan::new(line_num, 0),
                    ));
                }
                
                root.children.push(ASTNode {
                    kind: NodeKind::Function,
                    span: ASTSpan::new(line_num, 0),
                    children: vec![],
                });
            }
            // Check for table definition
            else if let Some(caps) = self.table_pattern.captures(line) {
                let name = caps.get(1).map(|m| m.as_str()).unwrap_or("anonymous");
                root.children.push(ASTNode {
                    kind: NodeKind::Struct, // Lua tables are like structs
                    span: ASTSpan::new(line_num, 0),
                    children: vec![],
                });
                tokens.push(Token::new(
                    TokenKind::Identifier,
                    name.to_string(),
                    ASTSpan::new(line_num, 0),
                ));
            }
        }

        // Detect Love2D callbacks
        let love_callbacks = [
            "love.load", "love.update", "love.draw", "love.keypressed",
            "love.keyreleased", "love.mousepressed", "love.mousereleased",
            "love.mousemoved", "love.joystickadded", "love.joystickremoved",
        ];

        for callback in love_callbacks {
            if source.contains(&format!("function {}", callback)) {
                tokens.push(Token::new(
                    TokenKind::Identifier,
                    callback.to_string(),
                    ASTSpan::new(0, 0),
                ));
            }
        }

        // Detect require statements
        let require_pattern = Regex::new(r#"require\s*\(?["']([^"']+)["']\)?"#).expect("valid regex");
        for caps in require_pattern.captures_iter(source) {
            if let Some(module) = caps.get(1) {
                tokens.push(Token::new(
                    TokenKind::String,
                    format!("require:{}", module.as_str()),
                    ASTSpan::new(0, 0),
                ));
            }
        }

        Ok(AST {
            root,
            tokens,
            errors,
        })
    }

    fn language(&self) -> &str {
        "lua"
    }

    fn extensions(&self) -> &[&str] {
        &[".lua"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_function() {
        let source = r#"
function hello()
    print("Hello, World!")
end
"#;
        let parser = LuaParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert!(ast.root.children.iter().any(|n| n.kind == NodeKind::Function));
    }

    #[tokio::test]
    async fn test_parse_local_function() {
        let source = r#"
local function greet(name)
    return "Hello, " .. name
end
"#;
        let parser = LuaParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert!(ast.root.children.iter().any(|n| n.kind == NodeKind::Function));
    }

    #[tokio::test]
    async fn test_parse_love2d() {
        let source = r#"
function love.load()
    player = { x = 100, y = 100 }
end

function love.update(dt)
    player.x = player.x + 1
end

function love.draw()
    love.graphics.rectangle("fill", player.x, player.y, 50, 50)
end
"#;
        let parser = LuaParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert!(ast.tokens.iter().any(|t| t.text == "love.load"));
        assert!(ast.tokens.iter().any(|t| t.text == "love.update"));
        assert!(ast.tokens.iter().any(|t| t.text == "love.draw"));
    }

    #[tokio::test]
    async fn test_parse_table() {
        let source = r#"
local config = {
    width = 800,
    height = 600,
    title = "My Game"
}
"#;
        let parser = LuaParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert!(ast.root.children.iter().any(|n| n.kind == NodeKind::Struct));
    }

    #[tokio::test]
    async fn test_parse_method() {
        let source = r#"
function Player:move(dx, dy)
    self.x = self.x + dx
    self.y = self.y + dy
end
"#;
        let parser = LuaParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert!(ast.tokens.iter().any(|t| t.text.contains("Player:move")));
    }

    #[test]
    fn test_extensions() {
        let parser = LuaParser::new();
        assert!(parser.can_parse("main.lua"));
        assert!(parser.can_parse("conf.lua"));
        assert!(!parser.can_parse("main.rs"));
    }
}
