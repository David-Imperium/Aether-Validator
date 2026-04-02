//! Syntax Layer — Parsing and syntax validation

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::Violation;

/// Syntax validation layer.
///
/// Checks for:
/// - Parsing errors
/// - Malformed code
/// - Invalid syntax
///
/// This is a CRITICAL layer - syntax errors stop the pipeline
/// because malformed code cannot be analyzed by other layers.
pub struct SyntaxLayer;

impl SyntaxLayer {
    /// Create a new syntax layer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SyntaxLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for SyntaxLayer {
    fn name(&self) -> &str {
        "syntax"
    }

    fn priority(&self) -> u8 {
        10 // First layer
    }

    /// Syntax layer is CRITICAL: stop pipeline on errors.
    /// Malformed code cannot be analyzed by semantic/logic layers.
    fn can_continue(&self, result: &LayerResult) -> bool {
        !result.has_errors()
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let mut violations = Vec::new();

        // Check for common syntax issues
        let source = &ctx.source;
        let language = &ctx.language;

        // Check for unmatched braces (language-aware)
        if let Some(violation) = check_unmatched_braces(source, language) {
            violations.push(violation);
        }

        // Check for unmatched parentheses (language-aware)
        if let Some(violation) = check_unmatched_parens(source, language) {
            violations.push(violation);
        }

        // Check for unmatched brackets (language-aware)
        if let Some(violation) = check_unmatched_brackets(source, language) {
            violations.push(violation);
        }

        // Check for unclosed strings (language-aware)
        if let Some(violation) = check_unclosed_strings(source, language) {
            violations.push(violation);
        }

        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }
}

/// State for tracking what region of code we're in (string, comment, etc.)
#[derive(Debug, Clone, Copy, PartialEq)]
enum SkipRegion {
    None,
    LineComment,
    BlockComment,
    String,
    Char,
    RawString(usize),    // Rust raw string with N hash delimiters
    RawByteString(usize), // Rust raw byte string with N hash delimiters
    ByteString,          // Rust byte string b"..."
    ByteChar,            // Rust byte char b'...'
    #[allow(dead_code)]
    TripleString(char),  // Python triple-quoted string with quote char
}

/// Skip state machine - handles all language-specific string/comment types.
/// Returns the new skip region after processing character at position `i`.
#[allow(clippy::cognitive_complexity)] // Tokenizer complexity is inherent to the task
fn update_skip_state(
    c: char,
    chars: &mut std::iter::Peekable<impl Iterator<Item = char>>,
    skip: SkipRegion,
    language: &str,
    i: &mut usize,
) -> SkipRegion {
    let is_rust = language == "rust";
    let is_python = language == "python";

    // Handle escape sequences in strings/chars
    if matches!(skip, SkipRegion::String | SkipRegion::Char | SkipRegion::ByteString | SkipRegion::ByteChar)
        && c == '\\' {
            chars.next();
            *i += 1; // Skip the escaped char
            return skip;
        }

    match skip {
        SkipRegion::None => {
            handle_none_skip_state(c, chars, is_rust, is_python, i)
        }

        SkipRegion::LineComment => {
            if c == '\n' {
                return SkipRegion::None;
            }
            SkipRegion::LineComment
        }

        SkipRegion::BlockComment => {
            if c == '*' {
                if let Some(&'/') = chars.peek() {
                    chars.next();
                    *i += 1;
                    return SkipRegion::None;
                }
            }
            SkipRegion::BlockComment
        }

        SkipRegion::String => {
            if c == '"' {
                return SkipRegion::None;
            }
            SkipRegion::String
        }

        SkipRegion::Char => {
            if c == '\'' {
                return SkipRegion::None;
            }
            SkipRegion::Char
        }

        SkipRegion::RawString(hash_count) => {
            if c == '"' {
                let mut closing_hashes = 0;
                while let Some(&next) = chars.peek() {
                    if next == '#' && closing_hashes < hash_count {
                        closing_hashes += 1;
                        chars.next();
                        *i += 1;
                    } else {
                        break;
                    }
                }
                if closing_hashes == hash_count {
                    return SkipRegion::None;
                }
            }
            SkipRegion::RawString(hash_count)
        }

        SkipRegion::RawByteString(hash_count) => {
            if c == '"' {
                let mut closing_hashes = 0;
                while let Some(&next) = chars.peek() {
                    if next == '#' && closing_hashes < hash_count {
                        closing_hashes += 1;
                        chars.next();
                        *i += 1;
                    } else {
                        break;
                    }
                }
                if closing_hashes == hash_count {
                    return SkipRegion::None;
                }
            }
            SkipRegion::RawByteString(hash_count)
        }

        SkipRegion::ByteString => {
            if c == '"' {
                return SkipRegion::None;
            }
            SkipRegion::ByteString
        }

        SkipRegion::ByteChar => {
            if c == '\'' {
                return SkipRegion::None;
            }
            SkipRegion::ByteChar
        }

        SkipRegion::TripleString(quote) => {
            // Python triple-quoted string
            if c == quote {
                // Check for triple closing
                let mut quote_count = 1;
                while let Some(&next) = chars.peek() {
                    if next == quote && quote_count < 3 {
                        quote_count += 1;
                        chars.next();
                        *i += 1;
                    } else {
                        break;
                    }
                }
                if quote_count == 3 {
                    return SkipRegion::None;
                }
            }
            SkipRegion::TripleString(quote)
        }
    }
}

/// Handler for SkipRegion::None case - checks for comment/string start
fn handle_none_skip_state(
    c: char,
    chars: &mut std::iter::Peekable<impl Iterator<Item = char>>,
    is_rust: bool,
    is_python: bool,
    i: &mut usize,
) -> SkipRegion {
    // Line comment start
    if c == '/' {
        if let Some(&next) = chars.peek() {
            if next == '/' {
                chars.next();
                *i += 1;
                return SkipRegion::LineComment;
            } else if next == '*' {
                chars.next();
                *i += 1;
                return SkipRegion::BlockComment;
            }
        }
    }

    // Python line comment
    if is_python && c == '#' {
        return SkipRegion::LineComment;
    }

    // Rust raw string: r#"..."# or r##"..."##
    if is_rust && c == 'r' {
        if let Some(region) = try_parse_rust_raw_string(chars, i) {
            return region;
        }
    }

    // Rust raw byte string: br#"..."#
    if is_rust && c == 'b' {
        if let Some(region) = try_parse_rust_byte_string(chars, i) {
            return region;
        }
    }

    // Regular string
    if c == '"' {
        return SkipRegion::String;
    }

    // Char literal or lifetime in Rust
    if c == '\'' {
        if is_rust && is_likely_lifetime(chars) {
            return SkipRegion::None;
        }
        return SkipRegion::Char;
    }

    SkipRegion::None
}

/// Try to parse Rust raw string (r#"..."#)
fn try_parse_rust_raw_string(
    chars: &mut std::iter::Peekable<impl Iterator<Item = char>>,
    i: &mut usize,
) -> Option<SkipRegion> {
    let hash_count = count_hash_prefixes(chars, i);
    if let Some(&'"') = chars.peek() {
        chars.next();
        *i += 1;
        return Some(SkipRegion::RawString(hash_count));
    }
    Some(SkipRegion::None)
}

/// Try to parse Rust byte string (b"...", b'...', br#"..."#)
fn try_parse_rust_byte_string(
    chars: &mut std::iter::Peekable<impl Iterator<Item = char>>,
    i: &mut usize,
) -> Option<SkipRegion> {
    if let Some(&'r') = chars.peek() {
        chars.next();
        *i += 1;
        let hash_count = count_hash_prefixes(chars, i);
        if let Some(&'"') = chars.peek() {
            chars.next();
            *i += 1;
            return Some(SkipRegion::RawByteString(hash_count));
        }
        return Some(SkipRegion::None);
    }
    if let Some(&'"') = chars.peek() {
        chars.next();
        *i += 1;
        return Some(SkipRegion::ByteString);
    }
    if let Some(&'\'') = chars.peek() {
        chars.next();
        *i += 1;
        return Some(SkipRegion::ByteChar);
    }
    Some(SkipRegion::None)
}

/// Count hash prefixes (for raw strings like r##"..."##)
fn count_hash_prefixes(
    chars: &mut std::iter::Peekable<impl Iterator<Item = char>>,
    i: &mut usize,
) -> usize {
    let mut hash_count = 0;
    while let Some(&next) = chars.peek() {
        if next == '#' {
            hash_count += 1;
            chars.next();
            *i += 1;
        } else {
            break;
        }
    }
    hash_count
}

/// Check if current position looks like a Rust lifetime (not char literal)
fn is_likely_lifetime(chars: &mut std::iter::Peekable<impl Iterator<Item = char>>) -> bool {
    if let Some(&next) = chars.peek() {
        next.is_alphabetic() || next == '_'
    } else {
        false
    }
}

fn check_unmatched_braces(source: &str, language: &str) -> Option<Violation> {
    let mut depth = 0;
    let mut skip = SkipRegion::None;
    let mut chars = source.chars().peekable();
    let mut i = 0;

    while let Some(c) = chars.next() {
        let pos = i;
        i += 1;

        // Update skip state (handles strings, comments, raw strings, etc.)
        skip = update_skip_state(c, &mut chars, skip, language, &mut i);

        // Only count braces if we're not in a skip region
        if skip == SkipRegion::None {
            match c {
                '{' => depth += 1,
                '}' => {
                    if depth == 0 {
                        return Some(Violation::error(
                            "SYNTAX001",
                            format!("Unmatched closing brace at position {}", pos),
                        ).suggest("Add opening brace before this"));
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
    }

    if depth > 0 {
        Some(Violation::warning(
            "SYNTAX002",
            format!("{} unmatched opening brace(s)", depth),
        ).suggest("Add closing brace(s)"))
    } else {
        None
    }
}

fn check_unmatched_parens(source: &str, language: &str) -> Option<Violation> {
    let mut depth = 0;
    let mut skip = SkipRegion::None;
    let mut chars = source.chars().peekable();
    let mut i = 0;

    while let Some(c) = chars.next() {
        let pos = i;
        i += 1;

        // Update skip state (handles strings, comments, raw strings, etc.)
        skip = update_skip_state(c, &mut chars, skip, language, &mut i);

        // Only count parens if we're not in a skip region
        if skip == SkipRegion::None {
            match c {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 {
                        return Some(Violation::error(
                            "SYNTAX003",
                            format!("Unmatched closing parenthesis at position {}", pos),
                        ).suggest("Add opening parenthesis before this"));
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
    }

    if depth > 0 {
        Some(Violation::warning(
            "SYNTAX004",
            format!("{} unmatched opening parenthesis", depth),
        ).suggest("Add closing parenthesis"))
    } else {
        None
    }
}

fn check_unmatched_brackets(source: &str, language: &str) -> Option<Violation> {
    let mut depth = 0;
    let mut skip = SkipRegion::None;
    let mut chars = source.chars().peekable();
    let mut i = 0;

    while let Some(c) = chars.next() {
        let pos = i;
        i += 1;

        // Update skip state (handles strings, comments, raw strings, etc.)
        skip = update_skip_state(c, &mut chars, skip, language, &mut i);

        // Only count brackets if we're not in a skip region
        if skip == SkipRegion::None {
            match c {
                '[' => depth += 1,
                ']' => {
                    if depth == 0 {
                        return Some(Violation::error(
                            "SYNTAX005",
                            format!("Unmatched closing bracket at position {}", pos),
                        ).suggest("Add opening bracket before this"));
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
    }

    if depth > 0 {
        Some(Violation::warning(
            "SYNTAX006",
            format!("{} unmatched opening bracket(s)", depth),
        ).suggest("Add closing bracket(s)"))
    } else {
        None
    }
}

fn check_unclosed_strings(source: &str, language: &str) -> Option<Violation> {
    let mut in_string = false;
    let mut in_char = false;
    let mut in_raw_string = false;
    let mut in_triple_string = false;
    let mut in_line_comment = false;
    let mut raw_hash_count = 0;
    let mut escape_next = false;
    let mut string_start = 0;
    let mut string_quote = '"'; // Track which quote char opened the string
    let mut chars = source.chars().peekable();

    let is_python = language == "python";
    let is_rust = language == "rust";

    let mut i = 0;
    while let Some(c) = chars.next() {
        let pos = i;
        i += 1;

        // Handle line comments: skip until newline
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
            }
            continue;
        }

        // Handle escape sequences
        if escape_next {
            escape_next = false;
            continue;
        }

        // Python line comment: # starts a comment (but not inside strings)
        if is_python && c == '#' && !in_string && !in_raw_string && !in_triple_string {
            in_line_comment = true;
            continue;
        }

        // Skip content inside char literals
        if in_char {
            if c == '\\' {
                escape_next = true;
            } else if c == '\'' {
                in_char = false;
            }
            continue;
        }

        // Skip content inside triple-quoted strings (Python)
        if in_triple_string && is_python {
            if c == string_quote {
                // Check for triple closing
                if let Some(&next1) = chars.peek() {
                    if next1 == string_quote {
                        chars.next();
                        i += 1;
                        if let Some(&next2) = chars.peek() {
                            if next2 == string_quote {
                                chars.next();
                                i += 1;
                                in_triple_string = false;
                            }
                        }
                    }
                }
            }
            continue;
        }

        // Skip content inside raw strings
        if in_raw_string {
            if is_rust && c == '"' {
                // Rust raw string: count closing hashes
                let mut closing_hashes = 0;
                while let Some(&next) = chars.peek() {
                    if next == '#' && closing_hashes < raw_hash_count {
                        closing_hashes += 1;
                        chars.next();
                        i += 1;
                    } else {
                        break;
                    }
                }
                if closing_hashes == raw_hash_count {
                    in_raw_string = false;
                    raw_hash_count = 0;
                }
            } else if is_python && c == string_quote {
                // Python raw string: simple quote close
                in_raw_string = false;
            }
            continue;
        }

        // Handle content inside regular strings
        if in_string {
            if c == '\\' {
                escape_next = true;
            } else if c == string_quote {
                in_string = false;
            }
            continue;
        }

        // Python-specific: check for triple-quoted strings (""" or ''')
        if is_python && (c == '"' || c == '\'') {
            let quote = c;
            let mut quote_count = 1;
            while let Some(&next) = chars.peek() {
                if next == quote {
                    quote_count += 1;
                    chars.next();
                    i += 1;
                } else {
                    break;
                }
            }
            if quote_count >= 3 {
                // Triple-quoted string
                in_triple_string = true;
                string_quote = quote;
                string_start = pos;
                continue;
            } else if quote_count == 1 {
                // Single quote - regular string
                in_string = true;
                string_quote = quote;
                string_start = pos;
                continue;
            }
            // quote_count == 2 is two adjacent strings, not special
            continue;
        }

        // Python-specific: check for prefix strings (r"", f"", b"", u"", rf"", fr"")
        if is_python && (c == 'r' || c == 'f' || c == 'b' || c == 'u') {
            let mut prefix = c.to_string();
            // Check for combined prefixes (rf, fr, rb, br, etc.)
            if let Some(&next) = chars.peek() {
                if next == 'r' || next == 'f' || next == 'b' {
                    prefix.push(next);
                    chars.next();
                    i += 1;
                }
            }
            // Check for quote after prefix
            if let Some(&quote) = chars.peek() {
                if quote == '"' || quote == '\'' {
                    chars.next();
                    i += 1;
                    // Check for triple quote
                    let mut quote_count = 1;
                    while let Some(&next) = chars.peek() {
                        if next == quote {
                            quote_count += 1;
                            chars.next();
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    if quote_count >= 3 {
                        in_triple_string = true;
                        string_quote = quote;
                    } else {
                        in_raw_string = true; // Python prefix strings are "raw" in terms of escape handling
                        string_quote = quote;
                    }
                    string_start = pos;
                    continue;
                }
            }
            // Not a prefix string, continue normal parsing
            continue;
        }

        // Rust-specific: raw strings r#"..."# or r##"..."##
        if is_rust && c == 'r' {
            let mut hash_count = 0;
            while let Some(&next) = chars.peek() {
                if next == '#' {
                    hash_count += 1;
                    chars.next();
                    i += 1;
                } else {
                    break;
                }
            }
            if let Some(&'"') = chars.peek() {
                chars.next(); // consume opening "
                i += 1;
                in_raw_string = true;
                raw_hash_count = hash_count;
                string_start = pos;
            }
            continue;
        }

        // Handle char literals (Rust)
        if is_rust && c == '\'' {
            in_char = true;
            continue;
        }

        // Handle regular string start (non-Python)
        if !is_python && c == '"' {
            in_string = true;
            string_quote = '"';
            string_start = pos;
        }
    }

    if in_string || in_raw_string || in_triple_string {
        Some(Violation::error(
            "SYNTAX007",
            format!("Unclosed string starting at position {}", string_start),
        ).suggest("Add closing quote"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid_syntax() {
        let ctx = ValidationContext::for_file("test.rs", "fn main() {}".into(), "rust".into());
        let layer = SyntaxLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_unmatched_brace() {
        let ctx = ValidationContext::for_file("test.rs", "fn main() {".into(), "rust".into());
        let layer = SyntaxLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.passed);
    }

    #[tokio::test]
    async fn test_unclosed_string() {
        let ctx = ValidationContext::for_file("test.rs", r#"let s = "hello"#.into(), "rust".into());
        let layer = SyntaxLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.passed);
    }
}
