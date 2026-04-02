//! Prism Parser — Parse Prism language source code (PRIVATE)
//!
//! Prism is a systems programming language similar to Odin/Jai.
//! This parser is PRIVATE and only for David's use.
//!
//! Syntax features:
//! - `package name.submodule` - Package declaration
//! - `import "module"` - Import statements
//! - `Name :: type` - Constant/declaration syntax
//! - `proc() -> type` - Procedure definitions
//! - `struct`, `enum`, `union` - Type definitions
//! - `#directive` - Compiler directives (#export, #c_layout, etc.)
//! - `$Type: typeid` - Compile-time parameters

use async_trait::async_trait;

use crate::parser::Parser;
use crate::ast::{AST, ASTNode, NodeKind, Span};
use crate::error::{ParseError, ParseResult};

/// Parser for Prism source code.
pub struct PrismParser;

impl PrismParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PrismParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Prism token types.
#[derive(Debug, Clone, PartialEq)]
enum PrismToken {
    // Keywords
    Package,
    Import,
    Proc,
    Struct,
    Enum,
    Union,
    Const,
    Var,
    Let,
    Defer,
    Return,
    If,
    Else,
    When,
    For,
    While,
    Switch,
    Case,
    Default,
    Break,
    Continue,
    Fallthrough,
    In,
    NotIn,
    As,
    Cast,
    Transmute,
    Sizeof,
    Alignof,
    Typeid,
    Distinct,
    Opaque,
    
    // Types
    Int, Int8, Int16, Int32, Int64,
    UInt, UInt8, UInt16, UInt32, UInt64,
    Float16, Float32, Float64,
    Bool, String, Rune, Any, Type,
    CInt, CUInt, CFloat, CDouble, CChar, CShort, CLong,
    
    // Literals
    Identifier(String),
    StringLit(String),
    RawStringLit(String),
    CharLit(char),
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),
    Nil,
    
    // Punctuation
    LeftBrace, RightBrace,     // { }
    LeftBracket, RightBracket, // [ ]
    LeftParen, RightParen,     // ( )
    Colon, DoubleColon,        // : ::
    Comma, Dot, DoubleDot,     // , . ..
    Arrow, FatArrow,           // -> =>
    Hash, At,                  // # @
    Semicolon, Backslash,      // ; \
    Question, Bang,            // ? !
    
    // Operators
    Plus, Minus, Star, Slash,      // + - * /
    Percent, DoubleStar,           // % **
    Ampersand, Pipe, Caret, Tilde, // & | ^ ~
    DoubleAmp, DoublePipe,         // && ||
    DoubleLess, DoubleGreater,     // << >>
    TripleLess, TripleGreater,     // <<< >>>
    
    // Assignment
    Equals,       // =
    PlusEq,       // +=
    MinusEq,      // -=
    StarEq,       // *=
    SlashEq,      // /=
    PercentEq,    // %=
    AmpEq,        // &=
    PipeEq,       // |=
    CaretEq,      // ^=
    DoubleLessEq, // <<=
    DoubleGreatEq,// >>=
    
    // Comparison
    DoubleEq,    // ==
    NotEq,       // !=
    Less,        // <
    Greater,     // >
    LessEq,      // <=
    GreatEq,     // >=
    
    // Special
    Dollar,      // $ (compile-time param)
    Ellipsis,    // ... (varargs)
    Eof,
}

/// Lexer for Prism language.
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

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.source.get(self.pos).copied();
        self.pos += 1;
        if let Some('\n') = c {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                }
                '/' if self.peek_next() == Some('/') => {
                    // Line comment
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                '/' if self.peek_next() == Some('*') => {
                    // Block comment
                    self.advance(); // /
                    self.advance(); // *
                    while let Some(c) = self.peek() {
                        if c == '*' && self.peek_next() == Some('/') {
                            self.advance();
                            self.advance();
                            break;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn read_string(&mut self, delimiter: char) -> Result<String, ParseError> {
        let mut s = String::new();
        self.advance(); // consume opening quote
        
        loop {
            match self.peek() {
                None => {
                    return Err(ParseError::Syntax {
                        line: self.line,
                        column: self.column,
                        message: "Unterminated string literal".to_string(),
                    });
                }
                Some(c) if c == delimiter => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('r') => s.push('\r'),
                        Some('\\') => s.push('\\'),
                        Some('"') => s.push('"'),
                        Some('\'') => s.push('\''),
                        Some('0') => s.push('\0'),
                        Some(c) => s.push(c),
                        None => {}
                    }
                    self.advance();
                }
                Some(c) => {
                    s.push(c);
                    self.advance();
                }
            }
        }
        
        Ok(s)
    }

    fn read_identifier(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        s
    }

    fn read_number(&mut self) -> Result<PrismToken, ParseError> {
        let mut s = String::new();
        let mut is_float = false;
        let mut is_hex = false;
        
        // Check for hex prefix
        if self.peek() == Some('0') && self.peek_next() == Some('x') {
            is_hex = true;
            s.push(self.advance().unwrap());
            s.push(self.advance().unwrap());
        }
        
        while let Some(c) = self.peek() {
            match c {
                '0'..='9' => {
                    s.push(c);
                    self.advance();
                }
                '.' if !is_float && !is_hex => {
                    if self.peek_next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        is_float = true;
                        s.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                'a'..='f' | 'A'..='F' if is_hex => {
                    s.push(c);
                    self.advance();
                }
                'e' | 'E' if !is_hex => {
                    is_float = true;
                    s.push(c);
                    self.advance();
                    if self.peek() == Some('+') || self.peek() == Some('-') {
                        s.push(self.advance().unwrap());
                    }
                }
                'f' | 'F' if !is_hex => {
                    // Float suffix (e.g., 3.14f)
                    is_float = true;
                    s.push(c);
                    self.advance();
                    break;
                }
                'u' | 'U' if !is_float => {
                    // Unsigned suffix
                    s.push(c);
                    self.advance();
                    // Check for size suffix (u8, u16, u32, u64)
                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            s.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    break;
                }
                'i' | 'I' if !is_float => {
                    // Signed size suffix (i8, i16, i32, i64)
                    s.push(c);
                    self.advance();
                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            s.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    break;
                }
                _ => break,
            }
        }
        
        if is_float {
            let value = s.replace(['f', 'F'], "")
                .parse::<f64>()
                .map_err(|e| ParseError::Syntax {
                    line: self.line,
                    column: self.column,
                    message: format!("Invalid float literal: {}", e),
                })?;
            Ok(PrismToken::FloatLit(value))
        } else if is_hex {
            let value = i64::from_str_radix(&s[2..], 16)
                .map_err(|e| ParseError::Syntax {
                    line: self.line,
                    column: self.column,
                    message: format!("Invalid hex literal: {}", e),
                })?;
            Ok(PrismToken::IntLit(value))
        } else {
            let value = s.parse::<i64>()
                .map_err(|e| ParseError::Syntax {
                    line: self.line,
                    column: self.column,
                    message: format!("Invalid integer literal: {}", e),
                })?;
            Ok(PrismToken::IntLit(value))
        }
    }

    fn next_token(&mut self) -> Result<PrismToken, ParseError> {
        self.skip_whitespace();
        
        let line = self.line;
        let col = self.column;
        
        match self.peek() {
            None => Ok(PrismToken::Eof),
            
            // String literals
            Some('"') => {
                let s = self.read_string('"')?;
                Ok(PrismToken::StringLit(s))
            }
            Some('\'') => {
                let s = self.read_string('\'')?;
                if s.len() != 1 {
                    return Err(ParseError::Syntax {
                        line,
                        column: col,
                        message: "Character literal must have exactly one character".to_string(),
                    });
                }
                // Safe: checked len == 1 above, but use expect for clarity
                let ch = s.chars().next().expect("char literal should have exactly one char after len check");
                Ok(PrismToken::CharLit(ch))
            }
            
            // Raw string literals (backticks)
            Some('`') => {
                let mut s = String::new();
                self.advance();
                while let Some(c) = self.peek() {
                    if c == '`' {
                        self.advance();
                        break;
                    }
                    s.push(c);
                    self.advance();
                }
                Ok(PrismToken::RawStringLit(s))
            }
            
            // Identifiers and keywords
            Some(c) if c.is_alphabetic() || c == '_' => {
                let ident = self.read_identifier();
                Ok(match ident.as_str() {
                    "package" => PrismToken::Package,
                    "import" => PrismToken::Import,
                    "proc" => PrismToken::Proc,
                    "struct" => PrismToken::Struct,
                    "enum" => PrismToken::Enum,
                    "union" => PrismToken::Union,
                    "const" => PrismToken::Const,
                    "var" => PrismToken::Var,
                    "let" => PrismToken::Let,
                    "defer" => PrismToken::Defer,
                    "return" => PrismToken::Return,
                    "if" => PrismToken::If,
                    "else" => PrismToken::Else,
                    "when" => PrismToken::When,
                    "for" => PrismToken::For,
                    "while" => PrismToken::While,
                    "switch" => PrismToken::Switch,
                    "case" => PrismToken::Case,
                    "default" => PrismToken::Default,
                    "break" => PrismToken::Break,
                    "continue" => PrismToken::Continue,
                    "fallthrough" => PrismToken::Fallthrough,
                    "in" => PrismToken::In,
                    "not_in" => PrismToken::NotIn,
                    "as" => PrismToken::As,
                    "cast" => PrismToken::Cast,
                    "transmute" => PrismToken::Transmute,
                    "sizeof" => PrismToken::Sizeof,
                    "alignof" => PrismToken::Alignof,
                    "typeid" => PrismToken::Typeid,
                    "distinct" => PrismToken::Distinct,
                    "opaque" => PrismToken::Opaque,
                    "int" => PrismToken::Int,
                    "int8" => PrismToken::Int8,
                    "int16" => PrismToken::Int16,
                    "int32" => PrismToken::Int32,
                    "int64" => PrismToken::Int64,
                    "uint" => PrismToken::UInt,
                    "uint8" => PrismToken::UInt8,
                    "uint16" => PrismToken::UInt16,
                    "uint32" => PrismToken::UInt32,
                    "uint64" => PrismToken::UInt64,
                    "float16" => PrismToken::Float16,
                    "float32" => PrismToken::Float32,
                    "float64" => PrismToken::Float64,
                    "bool" => PrismToken::Bool,
                    "string" => PrismToken::String,
                    "rune" => PrismToken::Rune,
                    "any" => PrismToken::Any,
                    "type" => PrismToken::Type,
                    "c_int" => PrismToken::CInt,
                    "c_uint" => PrismToken::CUInt,
                    "c_float" => PrismToken::CFloat,
                    "c_double" => PrismToken::CDouble,
                    "c_char" => PrismToken::CChar,
                    "c_short" => PrismToken::CShort,
                    "c_long" => PrismToken::CLong,
                    "true" => PrismToken::BoolLit(true),
                    "false" => PrismToken::BoolLit(false),
                    "nil" => PrismToken::Nil,
                    _ => PrismToken::Identifier(ident),
                })
            }
            
            // Numbers
            Some(c) if c.is_ascii_digit() => {
                self.read_number()
            }
            
            // Operators and punctuation
            Some('{') => { self.advance(); Ok(PrismToken::LeftBrace) }
            Some('}') => { self.advance(); Ok(PrismToken::RightBrace) }
            Some('[') => { self.advance(); Ok(PrismToken::LeftBracket) }
            Some(']') => { self.advance(); Ok(PrismToken::RightBracket) }
            Some('(') => { self.advance(); Ok(PrismToken::LeftParen) }
            Some(')') => { self.advance(); Ok(PrismToken::RightParen) }
            Some(',') => { self.advance(); Ok(PrismToken::Comma) }
            Some(';') => { self.advance(); Ok(PrismToken::Semicolon) }
            Some('\\') => { self.advance(); Ok(PrismToken::Backslash) }
            Some('?') => { self.advance(); Ok(PrismToken::Question) }
            Some('@') => { self.advance(); Ok(PrismToken::At) }
            Some('$') => { self.advance(); Ok(PrismToken::Dollar) }
            
            Some(':') => {
                self.advance();
                if self.peek() == Some(':') {
                    self.advance();
                    Ok(PrismToken::DoubleColon)
                } else {
                    Ok(PrismToken::Colon)
                }
            }
            Some('.') => {
                self.advance();
                if self.peek() == Some('.') {
                    self.advance();
                    if self.peek() == Some('.') {
                        self.advance();
                        Ok(PrismToken::Ellipsis)
                    } else {
                        Ok(PrismToken::DoubleDot)
                    }
                } else {
                    Ok(PrismToken::Dot)
                }
            }
            Some('-') => {
                self.advance();
                match self.peek() {
                    Some('>') => { self.advance(); Ok(PrismToken::Arrow) }
                    Some('=') => { self.advance(); Ok(PrismToken::MinusEq) }
                    _ => Ok(PrismToken::Minus)
                }
            }
            Some('=') => {
                self.advance();
                match self.peek() {
                    Some('=') => { self.advance(); Ok(PrismToken::DoubleEq) }
                    Some('>') => { self.advance(); Ok(PrismToken::FatArrow) }
                    _ => Ok(PrismToken::Equals)
                }
            }
            Some('+') => {
                self.advance();
                match self.peek() {
                    Some('=') => { self.advance(); Ok(PrismToken::PlusEq) }
                    _ => Ok(PrismToken::Plus)
                }
            }
            Some('*') => {
                self.advance();
                match self.peek() {
                    Some('=') => { self.advance(); Ok(PrismToken::StarEq) }
                    Some('*') => { self.advance(); Ok(PrismToken::DoubleStar) }
                    _ => Ok(PrismToken::Star)
                }
            }
            Some('/') => {
                self.advance();
                match self.peek() {
                    Some('=') => { self.advance(); Ok(PrismToken::SlashEq) }
                    _ => Ok(PrismToken::Slash)
                }
            }
            Some('%') => {
                self.advance();
                match self.peek() {
                    Some('=') => { self.advance(); Ok(PrismToken::PercentEq) }
                    _ => Ok(PrismToken::Percent)
                }
            }
            Some('&') => {
                self.advance();
                match self.peek() {
                    Some('&') => { self.advance(); Ok(PrismToken::DoubleAmp) }
                    Some('=') => { self.advance(); Ok(PrismToken::AmpEq) }
                    _ => Ok(PrismToken::Ampersand)
                }
            }
            Some('|') => {
                self.advance();
                match self.peek() {
                    Some('|') => { self.advance(); Ok(PrismToken::DoublePipe) }
                    Some('=') => { self.advance(); Ok(PrismToken::PipeEq) }
                    _ => Ok(PrismToken::Pipe)
                }
            }
            Some('^') => {
                self.advance();
                match self.peek() {
                    Some('=') => { self.advance(); Ok(PrismToken::CaretEq) }
                    _ => Ok(PrismToken::Caret)
                }
            }
            Some('~') => { self.advance(); Ok(PrismToken::Tilde) }
            Some('!') => {
                self.advance();
                match self.peek() {
                    Some('=') => { self.advance(); Ok(PrismToken::NotEq) }
                    _ => Ok(PrismToken::Bang)
                }
            }
            Some('<') => {
                self.advance();
                match self.peek() {
                    Some('<') => {
                        self.advance();
                        match self.peek() {
                            Some('<') => { self.advance(); Ok(PrismToken::TripleLess) }
                            Some('=') => { self.advance(); Ok(PrismToken::DoubleLessEq) }
                            _ => Ok(PrismToken::DoubleLess)
                        }
                    }
                    Some('=') => { self.advance(); Ok(PrismToken::LessEq) }
                    _ => Ok(PrismToken::Less)
                }
            }
            Some('>') => {
                self.advance();
                match self.peek() {
                    Some('>') => {
                        self.advance();
                        match self.peek() {
                            Some('>') => { self.advance(); Ok(PrismToken::TripleGreater) }
                            Some('=') => { self.advance(); Ok(PrismToken::DoubleGreatEq) }
                            _ => Ok(PrismToken::DoubleGreater)
                        }
                    }
                    Some('=') => { self.advance(); Ok(PrismToken::GreatEq) }
                    _ => Ok(PrismToken::Greater)
                }
            }
            Some('#') => { self.advance(); Ok(PrismToken::Hash) }
            
            Some(c) => {
                Err(ParseError::Syntax {
                    line,
                    column: col,
                    message: format!("Unexpected character: '{}'", c),
                })
            }
        }
    }

    fn tokenize(&mut self) -> Result<Vec<(PrismToken, usize, usize)>, ParseError> {
        let mut tokens = Vec::new();
        
        loop {
            let line = self.line;
            let col = self.column;
            let token = self.next_token()?;
            
            if token == PrismToken::Eof {
                tokens.push((token, line, col));
                break;
            }
            
            tokens.push((token, line, col));
        }
        
        Ok(tokens)
    }
}

/// Parser for Prism AST.
struct PrismASTParser {
    tokens: Vec<(PrismToken, usize, usize)>,
    pos: usize,
    errors: Vec<String>,
}

impl PrismASTParser {
    fn new(tokens: Vec<(PrismToken, usize, usize)>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> &PrismToken {
        self.tokens.get(self.pos).map(|(t, _, _)| t).unwrap_or(&PrismToken::Eof)
    }

    fn advance(&mut self) -> PrismToken {
        let (token, _, _) = self.tokens.get(self.pos)
            .cloned()
            .unwrap_or((PrismToken::Eof, 0, 0));
        self.pos += 1;
        token
    }

    #[allow(dead_code)]
    fn expect(&mut self, expected: PrismToken) -> Result<(), ParseError> {
        let (token, line, col) = self.tokens.get(self.pos)
            .cloned()
            .unwrap_or((PrismToken::Eof, 0, 0));
        
        if std::mem::discriminant(&token) == std::mem::discriminant(&expected) {
            self.pos += 1;
            Ok(())
        } else {
            Err(ParseError::Syntax {
                line,
                column: col,
                message: format!("Expected {:?}, found {:?}", expected, token),
            })
        }
    }

    fn parse(&mut self) -> ParseResult<AST> {
        let mut root = ASTNode::default();
        let tokens = Vec::new();
        
        // Parse package declaration
        if let PrismToken::Package = self.peek() {
            if let Some(node) = self.parse_package()? {
                root.children.push(node);
            }
        }
        
        // Parse imports
        while let PrismToken::Import = self.peek() {
            if let Some(node) = self.parse_import()? {
                root.children.push(node);
            }
        }
        
        // Parse top-level declarations
        loop {
            match self.peek() {
                PrismToken::Eof => break,
                PrismToken::Identifier(_) => {
                    // Declaration: Name :: ...
                    if let Some(node) = self.parse_declaration()? {
                        root.children.push(node);
                    }
                }
                _ => {
                    // Skip unexpected tokens
                    self.advance();
                }
            }
        }
        
        Ok(AST {
            root,
            tokens,
            errors: self.errors.clone(),
        })
    }

    fn parse_package(&mut self) -> ParseResult<Option<ASTNode>> {
        self.advance(); // consume 'package'
        
        let mut name = String::new();
        
        // Package name (e.g., "neural.ffi")
        while let PrismToken::Identifier(ident) = self.peek() {
            if !name.is_empty() {
                name.push('.');
            }
            name.push_str(ident);
            self.advance();
            
            // Check for dot separator
            if let PrismToken::Dot = self.peek() {
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(Some(ASTNode {
            kind: NodeKind::Module,
            span: Span::new(0, 0),
            children: vec![],
        }))
    }

    fn parse_import(&mut self) -> ParseResult<Option<ASTNode>> {
        self.advance(); // consume 'import'
        
        // Import path (string or identifier)
        match self.peek() {
            PrismToken::StringLit(_) | PrismToken::Identifier(_) => {
                self.advance();
            }
            _ => {}
        }
        
        Ok(Some(ASTNode {
            kind: NodeKind::Use,
            span: Span::new(0, 0),
            children: vec![],
        }))
    }

    fn parse_declaration(&mut self) -> ParseResult<Option<ASTNode>> {
        // Get name
        let name = match self.peek() {
            PrismToken::Identifier(ident) => {
                let s = ident.clone();
                self.advance();
                s
            }
            _ => return Ok(None),
        };
        
        // Expect ::
        match self.peek() {
            PrismToken::DoubleColon => {
                self.advance();
            }
            _ => return Ok(None),
        }
        
        // Determine declaration type
        let kind = match self.peek() {
            PrismToken::Proc => {
                self.advance();
                self.parse_proc(&name)?;
                NodeKind::Function
            }
            PrismToken::Struct => {
                self.advance();
                self.parse_struct(&name)?;
                NodeKind::Struct
            }
            PrismToken::Enum => {
                self.advance();
                self.parse_enum(&name)?;
                NodeKind::Enum
            }
            PrismToken::Union => {
                self.advance();
                self.parse_union(&name)?;
                NodeKind::Struct
            }
            _ => {
                // Constant or variable
                self.parse_const_or_var(&name)?;
                NodeKind::Let
            }
        };
        
        Ok(Some(ASTNode {
            kind,
            span: Span::new(0, 0),
            children: vec![],
        }))
    }

    fn parse_proc(&mut self, _name: &str) -> ParseResult<()> {
        // Expect (
        if let PrismToken::LeftParen = self.peek() {
            self.advance();
            // Parse parameters
            self.parse_params()?;
        }
        
        // Optional return type
        if let PrismToken::Arrow = self.peek() {
            self.advance();
            self.parse_type()?;
        }
        
        // Optional #export, #c_layout, etc.
        while let PrismToken::Hash = self.peek() {
            self.advance();
            if let PrismToken::Identifier(_) = self.peek() {
                self.advance();
            }
        }
        
        // Body
        if let PrismToken::LeftBrace = self.peek() {
            self.advance();
            self.parse_block()?;
        }
        
        Ok(())
    }

    fn parse_params(&mut self) -> ParseResult<()> {
        // Skip to closing paren
        let mut depth = 1;
        while depth > 0 {
            match self.peek() {
                PrismToken::LeftParen => { self.advance(); depth += 1; }
                PrismToken::RightParen => { self.advance(); depth -= 1; }
                PrismToken::Eof => break,
                _ => { self.advance(); }
            }
        }
        Ok(())
    }

    fn parse_type(&mut self) -> ParseResult<()> {
        // Skip type tokens
        loop {
            match self.peek() {
                PrismToken::Identifier(_)
                | PrismToken::Int | PrismToken::Int8 | PrismToken::Int16
                | PrismToken::Int32 | PrismToken::Int64
                | PrismToken::UInt | PrismToken::UInt8 | PrismToken::UInt16
                | PrismToken::UInt32 | PrismToken::UInt64
                | PrismToken::Float16 | PrismToken::Float32 | PrismToken::Float64
                | PrismToken::Bool | PrismToken::String | PrismToken::Any
                | PrismToken::CInt | PrismToken::CUInt | PrismToken::CFloat
                | PrismToken::CDouble | PrismToken::CChar | PrismToken::CShort
                | PrismToken::CLong => {
                    self.advance();
                }
                PrismToken::Star => { self.advance(); } // pointer
                PrismToken::LeftBracket => {
                    self.advance();
                    // Skip array type
                    let mut depth = 1;
                    while depth > 0 {
                        match self.peek() {
                            PrismToken::LeftBracket => { self.advance(); depth += 1; }
                            PrismToken::RightBracket => { self.advance(); depth -= 1; }
                            PrismToken::Eof => break,
                            _ => { self.advance(); }
                        }
                    }
                }
                PrismToken::Dollar => {
                    // Compile-time parameter
                    self.advance();
                    if let PrismToken::Identifier(_) = self.peek() {
                        self.advance();
                    }
                    if let PrismToken::Colon = self.peek() {
                        self.advance();
                        if let PrismToken::Typeid = self.peek() {
                            self.advance();
                        }
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn parse_block(&mut self) -> ParseResult<()> {
        let mut depth = 1;
        while depth > 0 {
            match self.peek() {
                PrismToken::LeftBrace => { self.advance(); depth += 1; }
                PrismToken::RightBrace => { self.advance(); depth -= 1; }
                PrismToken::Eof => break,
                _ => { self.advance(); }
            }
        }
        Ok(())
    }

    fn parse_struct(&mut self, _name: &str) -> ParseResult<()> {
        // Optional #c_layout, etc.
        while let PrismToken::Hash = self.peek() {
            self.advance();
            if let PrismToken::Identifier(_) = self.peek() {
                self.advance();
            }
        }
        
        // Expect {
        if let PrismToken::LeftBrace = self.peek() {
            self.advance();
            self.parse_block()?;
        }
        Ok(())
    }

    fn parse_enum(&mut self, _name: &str) -> ParseResult<()> {
        // Optional base type
        self.parse_type()?;
        
        // Expect {
        if let PrismToken::LeftBrace = self.peek() {
            self.advance();
            self.parse_block()?;
        }
        Ok(())
    }

    fn parse_union(&mut self, _name: &str) -> ParseResult<()> {
        // Expect {
        if let PrismToken::LeftBrace = self.peek() {
            self.advance();
            self.parse_block()?;
        }
        Ok(())
    }

    fn parse_const_or_var(&mut self, _name: &str) -> ParseResult<()> {
        // Could be: Type = value, or just Type
        self.parse_type()?;
        
        if let PrismToken::Equals = self.peek() {
            self.advance();
            self.parse_expr()?;
        }
        Ok(())
    }

    fn parse_expr(&mut self) -> ParseResult<()> {
        // Skip expression tokens
        let mut depth = 0;
        loop {
            match self.peek() {
                PrismToken::LeftParen | PrismToken::LeftBrace | PrismToken::LeftBracket => {
                    self.advance();
                    depth += 1;
                }
                PrismToken::RightParen | PrismToken::RightBrace | PrismToken::RightBracket => {
                    if depth == 0 {
                        break;
                    }
                    self.advance();
                    depth -= 1;
                }
                PrismToken::Comma | PrismToken::Semicolon => {
                    if depth == 0 {
                        break;
                    }
                    self.advance();
                }
                PrismToken::Eof => break,
                _ => { self.advance(); }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Parser for PrismParser {
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        // Tokenize
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        
        // Parse
        let mut parser = PrismASTParser::new(tokens);
        parser.parse()
    }

    fn language(&self) -> &str {
        "prism"
    }

    fn extensions(&self) -> &[&str] {
        &[".prism"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_package() {
        let source = r#"
            package neural.ffi
            
            import "device"
            import "core"
            
            NeuralBackend :: enum c_int {
                None = 0,
                DLSS = 1,
            }
            
            test :: proc() -> c_int {
                return 0
            }
        "#;
        
        let parser = PrismParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        // Should have: package, imports, enum, proc
        assert!(!ast.root.children.is_empty());
    }

    #[tokio::test]
    async fn test_parse_struct() {
        let source = r#"
            NeuralState :: struct {
                initialized: bool,
                backend: NeuralBackend,
            }
        "#;
        
        let parser = PrismParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.root.children.is_empty());
    }

    #[tokio::test]
    async fn test_parse_proc() {
        let source = r#"
            init :: proc(width: c_uint, height: c_uint) -> c_int #export {
                return 0
            }
        "#;
        
        let parser = PrismParser::new();
        let ast = parser.parse(source).await.unwrap();
        
        assert!(!ast.root.children.is_empty());
    }
}
