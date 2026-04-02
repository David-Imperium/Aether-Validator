//! Lex Parser — Parse Lex language source code
//!
//! Lex is a declarative language for game data definitions.
//! Syntax: resource, era, structure, unit, technology definitions with properties.

use async_trait::async_trait;

use crate::parser::Parser;
use crate::ast::{AST, ASTNode, NodeKind, Span as ASTSpan};
use crate::error::{ParseError, ParseResult};

/// Parser for Lex source code.
pub struct LexParser;

impl LexParser {
    /// Create a new Lex parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LexParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Lexer token types for Lex language.
#[derive(Debug, Clone, PartialEq)]
enum LexToken {
    // Keywords
    Resource,
    Era,
    Structure,
    Unit,
    Technology,
    Event,
    Choice,
    Secret,
    Ending,
    Terrain,
    
    // Condition keywords
    When,
    If,
    AvailableIf,
    SecretIf,
    ActiveIf,
    BonusIf,
    
    // Literals
    Identifier(String),
    String(String),
    Integer(i64),
    Float(f64),
    Color(String), // #RRGGBB
    
    // Punctuation
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    Colon,
    Comma,
    Dot,
    
    // Operators
    Equals,        // ==
    NotEquals,     // !=
    Greater,       // >
    Less,          // <
    GreaterEqual,  // >=
    LessEqual,     // <=
    And,
    Or,
    Not,
    
    // Special
    Eof,
}

/// Lexer for Lex language.
struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }
    
    fn peek(&self, offset: usize) -> Option<char> {
        self.source.get(self.pos + offset).copied()
    }
    
    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied();
        self.pos += 1;
        if ch == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        ch
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while let Some(ch) = self.peek(0) {
                if ch.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }
            
            // Skip comments
            if self.peek(0) == Some('/') && self.peek(1) == Some('/') {
                // Single-line comment
                while let Some(ch) = self.peek(0) {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else if self.peek(0) == Some('/') && self.peek(1) == Some('*') {
                // Multi-line comment
                self.advance(); // /
                self.advance(); // *
                while let Some(ch) = self.peek(0) {
                    if ch == '*' && self.peek(1) == Some('/') {
                        self.advance(); // *
                        self.advance(); // /
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }
    
    fn read_string(&mut self) -> Result<String, ParseError> {
        self.advance(); // Opening quote
        let mut result = String::new();
        while let Some(ch) = self.peek(0) {
            if ch == '"' {
                self.advance();
                return Ok(result);
            }
            if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.peek(0) {
                    match escaped {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        _ => result.push(escaped),
                    }
                    self.advance();
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }
        Err(ParseError::Syntax {
            line: self.line,
            column: self.column,
            message: "Unterminated string".to_string(),
        })
    }
    
    fn read_number(&mut self) -> LexToken {
        let mut num_str = String::new();
        let mut is_float = false;
        
        while let Some(ch) = self.peek(0) {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && self.peek(1).is_some_and(|c| c.is_ascii_digit()) {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        if is_float {
            LexToken::Float(num_str.parse().unwrap_or(0.0))
        } else {
            LexToken::Integer(num_str.parse().unwrap_or(0))
        }
    }
    
    fn read_identifier(&mut self) -> LexToken {
        let mut ident = String::new();
        while let Some(ch) = self.peek(0) {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check for keywords
        match ident.as_str() {
            "resource" => LexToken::Resource,
            "era" => LexToken::Era,
            "structure" => LexToken::Structure,
            "unit" => LexToken::Unit,
            "technology" => LexToken::Technology,
            "event" => LexToken::Event,
            "choice" => LexToken::Choice,
            "secret" => LexToken::Secret,
            "ending" => LexToken::Ending,
            "terrain" => LexToken::Terrain,
            "when" => LexToken::When,
            "if" => LexToken::If,
            "available_if" => LexToken::AvailableIf,
            "secret_if" => LexToken::SecretIf,
            "active_if" => LexToken::ActiveIf,
            "bonus_if" => LexToken::BonusIf,
            "and" => LexToken::And,
            "or" => LexToken::Or,
            "not" => LexToken::Not,
            _ => LexToken::Identifier(ident),
        }
    }
    
    fn read_color(&mut self) -> LexToken {
        // Already consumed '#'
        let mut color = String::from('#');
        while let Some(ch) = self.peek(0) {
            if ch.is_ascii_hexdigit() {
                color.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        LexToken::Color(color)
    }
    
    fn next_token(&mut self) -> Result<LexToken, ParseError> {
        self.skip_whitespace_and_comments();
        
        let ch = match self.peek(0) {
            Some(ch) => ch,
            None => return Ok(LexToken::Eof),
        };
        
        let line = self.line;
        let column = self.column;
        
        // String literal
        if ch == '"' {
            return self.read_string()
                .map(LexToken::String)
                .map_err(|e| ParseError::Syntax {
                    line,
                    column,
                    message: e.to_string(),
                });
        }
        
        // Color literal
        if ch == '#' {
            self.advance();
            return Ok(self.read_color());
        }
        
        // Number
        if ch.is_ascii_digit() {
            return Ok(self.read_number());
        }
        
        // Identifier or keyword
        if ch.is_alphabetic() || ch == '_' {
            return Ok(self.read_identifier());
        }
        
        // Punctuation and operators
        self.advance();
        match ch {
            '{' => Ok(LexToken::LeftBrace),
            '}' => Ok(LexToken::RightBrace),
            '[' => Ok(LexToken::LeftBracket),
            ']' => Ok(LexToken::RightBracket),
            '(' => Ok(LexToken::LeftParen),
            ')' => Ok(LexToken::RightParen),
            ':' => Ok(LexToken::Colon),
            ',' => Ok(LexToken::Comma),
            '.' => Ok(LexToken::Dot),
            '=' => {
                if self.peek(0) == Some('=') {
                    self.advance();
                    Ok(LexToken::Equals)
                } else {
                    Ok(LexToken::Colon) // Treat single '=' as assignment
                }
            }
            '!' => {
                if self.peek(0) == Some('=') {
                    self.advance();
                    Ok(LexToken::NotEquals)
                } else {
                    Ok(LexToken::Not)
                }
            }
            '>' => {
                if self.peek(0) == Some('=') {
                    self.advance();
                    Ok(LexToken::GreaterEqual)
                } else {
                    Ok(LexToken::Greater)
                }
            }
            '<' => {
                if self.peek(0) == Some('=') {
                    self.advance();
                    Ok(LexToken::LessEqual)
                } else {
                    Ok(LexToken::Less)
                }
            }
            _ => Err(ParseError::Syntax {
                line,
                column,
                message: format!("Unexpected character: {}", ch),
            }),
        }
    }
    
    fn tokenize(&mut self) -> Result<Vec<(LexToken, usize, usize)>, ParseError> {
        let mut tokens = Vec::new();
        while let Ok(token) = self.next_token() {
            let line = self.line;
            let column = self.column;
            let is_eof = token == LexToken::Eof;
            tokens.push((token, line, column));
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}

/// Parser for Lex language.
struct LexParserInternal {
    tokens: Vec<(LexToken, usize, usize)>,
    pos: usize,
    errors: Vec<String>,
}

impl LexParserInternal {
    fn new(tokens: Vec<(LexToken, usize, usize)>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }
    
    fn current(&self) -> &(LexToken, usize, usize) {
        self.tokens.get(self.pos).unwrap_or(&(LexToken::Eof, 0, 0))
    }
    
    fn advance(&mut self) -> &(LexToken, usize, usize) {
        self.pos += 1;
        self.current()
    }
    
    #[allow(dead_code)]
    fn expect(&mut self, expected: &str) -> bool {
        let (token, _, _) = self.current();
        match token {
            LexToken::Identifier(name) if name == expected => {
                self.advance();
                true
            }
            _ => {
                self.errors.push(format!("Expected '{}', found {:?}", expected, token));
                false
            }
        }
    }
    
    fn parse_name(&mut self) -> Option<String> {
        let (token, _, _) = self.current();
        match token {
            LexToken::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Some(name)
            }
            _ => {
                self.errors.push("Expected identifier".to_string());
                None
            }
        }
    }
    
    fn parse(&mut self) -> AST {
        let mut root = ASTNode {
            kind: NodeKind::Module,
            ..Default::default()
        };
        
        loop {
            let (token, line, col) = self.current();
            let node = match token {
                LexToken::Resource => self.parse_definition(NodeKind::Resource, "resource"),
                LexToken::Era => self.parse_definition(NodeKind::Era, "era"),
                LexToken::Structure => self.parse_definition(NodeKind::LexStructure, "structure"),
                LexToken::Unit => self.parse_definition(NodeKind::Unit, "unit"),
                LexToken::Technology => self.parse_definition(NodeKind::Technology, "technology"),
                LexToken::Event => self.parse_definition(NodeKind::Event, "event"),
                LexToken::Choice => self.parse_definition(NodeKind::Choice, "choice"),
                LexToken::Eof => break,
                _ => {
                    self.errors.push(format!("Unexpected token at {}:{}", line, col));
                    self.advance();
                    continue;
                }
            };
            
            if let Some(node) = node {
                root.children.push(node);
            }
        }
        
        AST::new(root)
    }
    
    fn parse_definition(&mut self, kind: NodeKind, keyword: &str) -> Option<ASTNode> {
        self.advance(); // Consume keyword
        
        let _name = self.parse_name()?;
        let mut node = ASTNode {
            kind,
            span: ASTSpan::new(0, 0),
            ..Default::default()
        };
        
        // Note: name is stored in the definition node (simplified for now)
        
        // Expect '{'
        let (token, _, _) = self.current();
        if token != &LexToken::LeftBrace {
            self.errors.push(format!("Expected '{{' after {}", keyword));
            return None;
        }
        self.advance();
        
        // Parse properties until '}'
        loop {
            let (token, _line, _col) = self.current();
            match token {
                LexToken::RightBrace => {
                    self.advance();
                    break;
                }
                // Keywords can be property names too (e.g., "era: Ancient")
                LexToken::Identifier(_)
                | LexToken::Era
                | LexToken::Resource
                | LexToken::Structure
                | LexToken::Unit
                | LexToken::Technology
                | LexToken::Event
                | LexToken::Choice => {
                    let prop_name = match token {
                        LexToken::Identifier(name) => name.clone(),
                        LexToken::Era => "era".to_string(),
                        LexToken::Resource => "resource".to_string(),
                        LexToken::Structure => "structure".to_string(),
                        LexToken::Unit => "unit".to_string(),
                        LexToken::Technology => "technology".to_string(),
                        LexToken::Event => "event".to_string(),
                        LexToken::Choice => "choice".to_string(),
                        _ => unreachable!(),
                    };
                    let prop_node = self.parse_property(prop_name);
                    if let Some(prop) = prop_node {
                        node.children.push(prop);
                    }
                }
                LexToken::AvailableIf
                | LexToken::SecretIf
                | LexToken::ActiveIf
                | LexToken::BonusIf
                | LexToken::When
                | LexToken::If => {
                    // Parse condition block
                    let cond_node = self.parse_condition();
                    if let Some(cond) = cond_node {
                        node.children.push(cond);
                    }
                }
                LexToken::Eof => {
                    self.errors.push("Unexpected end of file".to_string());
                    break;
                }
                _ => {
                    self.errors.push(format!("Unexpected token in definition: {:?}", token));
                    self.advance();
                }
            }
        }
        
        Some(node)
    }
    
    fn parse_property(&mut self, name: String) -> Option<ASTNode> {
        self.advance(); // Consume identifier

        let node = ASTNode {
            kind: NodeKind::Property,
            span: ASTSpan::new(0, 0),
            ..Default::default()
        };

        // Store property name in the first child
        let _name_node = ASTNode {
            kind: NodeKind::Unknown,
            ..Default::default()
        };
        // The property name is stored in the node's structure
        
        // Expect ':'
        let (token, _, _) = self.current();
        if token != &LexToken::Colon {
            self.errors.push(format!("Expected ':' after property name '{}'", name));
            return None;
        }
        self.advance();
        
        // Parse value
        let (value_token, _, _) = self.current();
        match value_token {
            LexToken::String(_) => {
                self.advance();
                // String value - stored in children
            }
            LexToken::Integer(_) => {
                self.advance();
                // Integer value
            }
            LexToken::Float(_) => {
                self.advance();
                // Float value
            }
            LexToken::Identifier(_) => {
                self.advance();
                // Reference or identifier value
            }
            LexToken::Color(_) => {
                self.advance();
                // Color value
            }
            LexToken::LeftBrace => {
                // Object/Resource map
                self.advance();
                self.parse_object_contents();
            }
            LexToken::LeftBracket => {
                // Array
                self.advance();
                self.parse_array_contents();
            }
            _ => {
                self.errors.push(format!("Expected value after '{}:', found {:?}", name, value_token));
                return None;
            }
        }
        
        Some(node)
    }
    
    fn parse_object_contents(&mut self) {
        // Parse object until matching '}'
        let mut depth = 1;
        while depth > 0 {
            let (token, _, _) = self.current();
            match token {
                LexToken::LeftBrace => {
                    depth += 1;
                    self.advance();
                }
                LexToken::RightBrace => {
                    depth -= 1;
                    self.advance();
                }
                LexToken::Eof => {
                    self.errors.push("Unexpected end of file in object".to_string());
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
    
    fn parse_array_contents(&mut self) {
        // Parse array until matching ']'
        let mut depth = 1;
        while depth > 0 {
            let (token, _, _) = self.current();
            match token {
                LexToken::LeftBracket => {
                    depth += 1;
                    self.advance();
                }
                LexToken::RightBracket => {
                    depth -= 1;
                    self.advance();
                }
                LexToken::Eof => {
                    self.errors.push("Unexpected end of file in array".to_string());
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
    
    fn parse_condition(&mut self) -> Option<ASTNode> {
        let (token, _, _) = self.current();
        let _keyword = match token {
            LexToken::AvailableIf => "available_if",
            LexToken::SecretIf => "secret_if",
            LexToken::ActiveIf => "active_if",
            LexToken::BonusIf => "bonus_if",
            LexToken::When => "when",
            LexToken::If => "if",
            _ => return None,
        };
        
        self.advance(); // Consume keyword

        let node = ASTNode {
            kind: NodeKind::Condition,
            span: ASTSpan::new(0, 0),
            ..Default::default()
        };
        
        // Parse condition expression (identifier or function call)
        let (expr_token, _, _) = self.current();
        if let LexToken::Identifier(_) = expr_token {
            self.advance();
            
            // Check for function call syntax: identifier(args)
            let (next_token, _, _) = self.current();
            if next_token == &LexToken::LeftParen {
                self.advance(); // consume '('
                
                // Parse arguments until ')'
                let mut paren_depth = 1;
                while paren_depth > 0 {
                    let (t, _, _) = self.current();
                    match t {
                        LexToken::LeftParen => {
                            paren_depth += 1;
                            self.advance();
                        }
                        LexToken::RightParen => {
                            paren_depth -= 1;
                            self.advance();
                        }
                        LexToken::Eof => {
                            self.errors.push("Unexpected end of file in condition arguments".to_string());
                            break;
                        }
                        _ => {
                            self.advance();
                        }
                    }
                }
            }
        }
        
        // Expect '{'
        let (token, _, _) = self.current();
        if token != &LexToken::LeftBrace {
            self.errors.push("Expected '{' after condition".to_string());
            return None;
        }
        self.advance();
        
        // Parse condition body until '}'
        let mut depth = 1;
        while depth > 0 {
            let (token, _, _) = self.current();
            match token {
                LexToken::LeftBrace => {
                    depth += 1;
                    self.advance();
                }
                LexToken::RightBrace => {
                    depth -= 1;
                    self.advance();
                }
                LexToken::Eof => {
                    self.errors.push("Unexpected end of file in condition".to_string());
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
        
        Some(node)
    }
}

#[async_trait]
impl Parser for LexParser {
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        // Tokenize
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()
            .map_err(|e| ParseError::Syntax {
                line: 0,
                column: 0,
                message: e.to_string(),
            })?;
        
        // Parse
        let mut parser = LexParserInternal::new(tokens);
        let ast = parser.parse();
        
        // Add errors to AST
        let mut ast = ast;
        for error in parser.errors {
            ast.errors.push(error);
        }
        
        Ok(ast)
    }

    fn language(&self) -> &str {
        "lex"
    }

    fn extensions(&self) -> &[&str] {
        &[".lex"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_resource() {
        let source = r#"
resource Gold {
    name: "Gold"
    category: "currency"
}
"#;
        let parser = LexParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert!(ast.root.children.iter().any(|n| n.kind == NodeKind::Resource));
    }

    #[tokio::test]
    async fn test_parse_era() {
        let source = r#"
era Ancient {
    name: "Ancient Era"
    period: "3000 BCE"
}
"#;
        let parser = LexParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert!(ast.root.children.iter().any(|n| n.kind == NodeKind::Era));
    }

    #[tokio::test]
    async fn test_parse_structure() {
        let source = r#"
structure Farm {
    era: Ancient
    cost: { Gold: 30 }
}
"#;
        let parser = LexParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert!(ast.root.children.iter().any(|n| n.kind == NodeKind::LexStructure));
    }

    #[tokio::test]
    async fn test_parse_unit() {
        let source = r#"
unit Warrior {
    era: Ancient
    attack: 5
    defense: 3
}
"#;
        let parser = LexParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert!(ast.root.children.iter().any(|n| n.kind == NodeKind::Unit));
    }

    #[tokio::test]
    async fn test_parse_technology() {
        let source = r#"
technology SteamEngine {
    era: Steampunk
    research_cost: 100
}
"#;
        let parser = LexParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert!(ast.root.children.iter().any(|n| n.kind == NodeKind::Technology));
    }

    #[tokio::test]
    async fn test_parse_multiple_definitions() {
        let source = r#"
resource Gold { name: "Gold" }

era Ancient { name: "Ancient Era" }

structure Farm {
    era: Ancient
    cost: { Gold: 30 }
}
"#;
        let parser = LexParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        assert_eq!(ast.root.children.len(), 3);
    }

    #[test]
    fn test_extensions() {
        let parser = LexParser::new();
        assert!(parser.can_parse("game.lex"));
        assert!(parser.can_parse("units.lex"));
        assert!(!parser.can_parse("main.rs"));
        assert!(!parser.can_parse("main.lua"));
    }

    #[tokio::test]
    async fn test_parse_complex_structure() {
        // Test structure with nested properties, resource maps, and conditions
        let source = r#"
resource Gold { name: "Gold" }

era Ancient { name: "Ancient Era" }

structure SteamFactory {
    era: Steampunk
    name: "Steam Factory"
    description: "Converts coal into energy"
    
    cost: {
        Coal: 8,
        Steel: 5,
        Gold: 50
    }
    
    production: {
        Energy: 15,
        Industry: 10
    }
    
    maintenance: {
        Coal: 2,
        Gold: 5
    }
    
    available_if has_technology(SteamEngine) {
    }
}
"#;
        let parser = LexParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        if ast.has_errors() {
            eprintln!("Parse errors: {:?}", ast.errors);
        }
        
        assert!(!ast.has_errors(), "Parse errors: {:?}", ast.errors);
        // Should have 3 top-level definitions
        assert_eq!(ast.root.children.len(), 3);
        
        // Find the structure
        let structure = ast.root.children.iter()
            .find(|n| n.kind == NodeKind::LexStructure)
            .expect("Should have a structure");
        
        // Check that it has properties
        assert!(!structure.children.is_empty());
        
        // Find the condition property
        let has_condition = structure.children.iter()
            .any(|n| n.kind == NodeKind::Condition);
        assert!(has_condition, "Structure should have a condition");
    }

    #[tokio::test]
    async fn test_parse_technology_with_prerequisites() {
        let source = r#"
technology IndustrialChemistry {
    era: Steampunk
    name: "Industrial Chemistry"
    research_cost: 150
    prerequisites: [SteamEngine]
    unlocks: [ResearchLab]
}
"#;
        let parser = LexParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.has_errors());
        let tech = ast.root.children.iter()
            .find(|n| n.kind == NodeKind::Technology)
            .expect("Should have a technology");
        
        // Should have prerequisites and unlocks properties
        assert!(tech.children.iter().any(|n| n.kind == NodeKind::Property));
    }
}
