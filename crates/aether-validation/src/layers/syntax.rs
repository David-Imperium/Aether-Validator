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

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let mut violations = Vec::new();

        // Check for common syntax issues
        let source = &ctx.source;

        // Check for unmatched braces
        if let Some(violation) = check_unmatched_braces(source) {
            violations.push(violation);
        }

        // Check for unmatched parentheses
        if let Some(violation) = check_unmatched_parens(source) {
            violations.push(violation);
        }

        // Check for unmatched brackets
        if let Some(violation) = check_unmatched_brackets(source) {
            violations.push(violation);
        }

        // Check for unclosed strings
        if let Some(violation) = check_unclosed_strings(source) {
            violations.push(violation);
        }

        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }
}

fn check_unmatched_braces(source: &str) -> Option<Violation> {
    let mut depth = 0;
    for (i, c) in source.chars().enumerate() {
        match c {
            '{' => depth += 1,
            '}' => {
                if depth == 0 {
                    return Some(Violation::error(
                        "SYNTAX001",
                        format!("Unmatched closing brace at position {}", i),
                    ).suggest("Add opening brace before this"));
                }
                depth -= 1;
            }
            _ => {}
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

fn check_unmatched_parens(source: &str) -> Option<Violation> {
    let mut depth = 0;
    for (i, c) in source.chars().enumerate() {
        match c {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return Some(Violation::error(
                        "SYNTAX003",
                        format!("Unmatched closing parenthesis at position {}", i),
                    ).suggest("Add opening parenthesis before this"));
                }
                depth -= 1;
            }
            _ => {}
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

fn check_unmatched_brackets(source: &str) -> Option<Violation> {
    let mut depth = 0;
    for (i, c) in source.chars().enumerate() {
        match c {
            '[' => depth += 1,
            ']' => {
                if depth == 0 {
                    return Some(Violation::error(
                        "SYNTAX005",
                        format!("Unmatched closing bracket at position {}", i),
                    ).suggest("Add opening bracket before this"));
                }
                depth -= 1;
            }
            _ => {}
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

fn check_unclosed_strings(source: &str) -> Option<Violation> {
    let mut in_string = false;
    let mut in_char = false;
    let mut in_raw_string = false;
    let mut raw_hash_count = 0;
    let mut escape_next = false;
    let mut string_start = 0;
    let mut chars = source.chars().peekable();

    let mut i = 0;
    while let Some(c) = chars.next() {
        let pos = i;
        i += 1;

        // Handle escape sequences
        if escape_next {
            escape_next = false;
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

        // Skip content inside raw strings
        if in_raw_string {
            if c == '"' {
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
            }
            continue;
        }

        // Handle content inside regular strings
        if in_string {
            if c == '\\' {
                escape_next = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        // Handle raw strings: r#"..."# or r##"..."##
        if c == 'r' {
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

        // Handle char literals
        if c == '\'' {
            in_char = true;
            continue;
        }

        // Handle regular string start
        if c == '"' {
            in_string = true;
            string_start = pos;
        }
    }

    if in_string || in_raw_string {
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
