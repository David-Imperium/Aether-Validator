//! Style Layer — Naming conventions and formatting checks

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::Violation;

/// Style validation layer.
///
/// Checks for:
/// - Naming conventions (snake_case, PascalCase, etc.)
/// - Line length limits
/// - Function length limits
/// - Documentation presence
/// - Magic numbers
pub struct StyleLayer {
    /// Maximum line length
    max_line_length: usize,
    /// Maximum function length
    max_function_lines: usize,
    /// Require documentation on public items
    require_docs: bool,
}

impl StyleLayer {
    /// Create a new style layer with default settings.
    pub fn new() -> Self {
        Self {
            max_line_length: 120,
            max_function_lines: 50,
            require_docs: true,
        }
    }

    /// Create a style layer with custom settings.
    pub fn with_settings(max_line_length: usize, max_function_lines: usize, require_docs: bool) -> Self {
        Self {
            max_line_length,
            max_function_lines,
            require_docs,
        }
    }
}

impl Default for StyleLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for StyleLayer {
    fn name(&self) -> &str {
        "style"
    }

    fn priority(&self) -> u8 {
        50 // Last layer (after architecture)
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let mut violations = Vec::new();
        let source = &ctx.source;

        // Check naming conventions
        check_naming_conventions(source, &mut violations);

        // Check line length
        check_line_length(source, self.max_line_length, &mut violations);

        // Check function length
        check_function_length(source, self.max_function_lines, &mut violations);

        // Check documentation
        if self.require_docs {
            check_documentation(source, &mut violations);
        }

        // Check magic numbers
        check_magic_numbers(source, &mut violations);

        // Check whitespace issues
        check_whitespace(source, &mut violations);

        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }
}

fn check_naming_conventions(source: &str, violations: &mut Vec<Violation>) {
    let lines: Vec<&str> = source.lines().collect();
    
    for line in lines.iter() {
        // Check function naming (Rust: snake_case)
        if line.trim().starts_with("fn ") {
            if let Some(name) = extract_function_name(line) {
                if !is_snake_case(&name) && !name.starts_with('_') {
                    violations.push(Violation::warning(
                        "STYLE001",
                        format!("Function '{}' should use snake_case", name),
                    ).suggest(format!("Rename to '{}'", to_snake_case(&name))));
                }
            }
        }
        
        // Check struct/enum naming (Rust: PascalCase)
        if line.trim().starts_with("struct ") || line.trim().starts_with("enum ") {
            if let Some(name) = extract_type_name(line) {
                if !is_pascal_case(&name) {
                    violations.push(Violation::warning(
                        "STYLE002",
                        format!("Type '{}' should use PascalCase", name),
                    ).suggest(format!("Rename to '{}'", to_pascal_case(&name))));
                }
            }
        }
        
        // Check constant naming (Rust: SCREAMING_SNAKE_CASE)
        if line.trim().starts_with("const ") || line.trim().starts_with("static ") {
            if let Some(name) = extract_const_name(line) {
                if !is_screaming_snake_case(&name) {
                    violations.push(Violation::info(
                        "STYLE003",
                        format!("Constant '{}' should use SCREAMING_SNAKE_CASE", name),
                    ).suggest(format!("Rename to '{}'", to_screaming_snake_case(&name))));
                }
            }
        }
    }
}

fn check_line_length(source: &str, max_length: usize, violations: &mut Vec<Violation>) {
    for (i, line) in source.lines().enumerate() {
        if line.len() > max_length {
            violations.push(Violation::info(
                "STYLE004",
                format!("Line {} exceeds {} characters ({} chars)", i + 1, max_length, line.len()),
            ).suggest("Break line into multiple lines"));
        }
    }
}

fn check_function_length(source: &str, max_lines: usize, violations: &mut Vec<Violation>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut fn_start: Option<usize> = None;
    let mut brace_count = 0;
    let mut fn_name = String::new();
    
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("fn ") || line.trim().starts_with("pub fn ") || line.trim().starts_with("async fn ") {
            fn_name = extract_function_name(line).unwrap_or_default();
            fn_start = Some(i);
            brace_count = 0;
        }
        
        if fn_start.is_some() {
            brace_count += line.matches('{').count() as i32;
            brace_count -= line.matches('}').count() as i32;
            
            if brace_count == 0 && i > fn_start.unwrap_or(0) {
                let fn_length = i - fn_start.unwrap_or(0);
                if fn_length > max_lines {
                    violations.push(Violation::warning(
                        "STYLE005",
                        format!("Function '{}' is {} lines (max {})", fn_name, fn_length, max_lines),
                    ).suggest("Extract parts into helper functions"));
                }
                fn_start = None;
            }
        }
    }
}

fn check_documentation(source: &str, violations: &mut Vec<Violation>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut expecting_doc = false;
    let mut item_name = String::new();
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Check for public items that need documentation
        if trimmed.starts_with("pub fn ") 
            || trimmed.starts_with("pub struct ") 
            || trimmed.starts_with("pub enum ")
            || trimmed.starts_with("pub trait ") {
            expecting_doc = true;
            item_name = extract_item_name(trimmed);
        }
        
        // Check if documentation precedes the item
        if expecting_doc && i > 0 {
            let prev_line = lines[i - 1].trim();
            if !prev_line.starts_with("///") 
                && !prev_line.starts_with("/**") 
                && !prev_line.starts_with("//")
                && !prev_line.starts_with("#[") {
                violations.push(Violation::info(
                    "STYLE006",
                    format!("Public item '{}' missing documentation", item_name),
                ).suggest("Add /// documentation comments"));
            }
        }
        
        expecting_doc = false;
    }
}

fn check_magic_numbers(source: &str, violations: &mut Vec<Violation>) {
    let lines: Vec<&str> = source.lines().collect();
    
    for (i, line) in lines.iter().enumerate() {
        // Skip comments and strings
        if line.trim().starts_with("//") || line.trim().starts_with("///") {
            continue;
        }
        
        // Simple heuristic: look for standalone numbers that aren't 0, 1, or common values
        // This is very basic; real implementation would use AST
        let trimmed = line.trim();
        
        // Skip test functions and common patterns
        if trimmed.contains("test") || trimmed.contains("assert") {
            continue;
        }
        
        // Look for numeric literals in comparisons or assignments
        if let Some(num) = extract_magic_number(trimmed) {
            violations.push(Violation::info(
                "STYLE007",
                format!("Magic number {} on line {} - consider using a named constant", num, i + 1),
            ).suggest(format!("Define as const MAGIC_NUMBER: u32 = {};", num)));
        }
    }
}

fn check_whitespace(source: &str, violations: &mut Vec<Violation>) {
    let lines: Vec<&str> = source.lines().collect();
    
    for (i, line) in lines.iter().enumerate() {
        // Check for trailing whitespace
        if line.ends_with(' ') || line.ends_with('\t') {
            violations.push(Violation::info(
                "STYLE008",
                format!("Trailing whitespace on line {}", i + 1),
            ).suggest("Remove trailing whitespace"));
        }
        
        // Check for tabs (if preferring spaces)
        if line.contains('\t') && !line.trim().starts_with("//") {
            violations.push(Violation::info(
                "STYLE009",
                format!("Tab character on line {} - prefer spaces", i + 1),
            ).suggest("Replace tabs with spaces (typically 4)"));
        }
    }
    
    // Check for multiple blank lines
    let mut blank_count = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            blank_count += 1;
        } else {
            if blank_count > 2 {
                violations.push(Violation::info(
                    "STYLE010",
                    format!("Multiple blank lines before line {}", i + 1),
                ).suggest("Use at most 1 blank line"));
            }
            blank_count = 0;
        }
    }
}

// Helper functions

fn extract_function_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let start = trimmed.find("fn ")? + 3;
    let rest = &trimmed[start..];
    let end = rest.find(['(', '<', '{']).unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

fn extract_type_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let keyword = if trimmed.starts_with("struct ") { "struct " } else { "enum " };
    let start = trimmed.find(keyword)? + keyword.len();
    let rest = &trimmed[start..];
    let end = rest.find(['{', '<', '(', ':']).unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

fn extract_const_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let keyword = if trimmed.starts_with("const ") { "const " } else { "static " };
    let start = trimmed.find(keyword)? + keyword.len();
    let rest = &trimmed[start..];
    let end = rest.find(':').unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

fn extract_item_name(line: &str) -> String {
    if let Some(name) = extract_function_name(line) {
        return name;
    }
    if let Some(name) = extract_type_name(line) {
        return name;
    }
    "unknown".to_string()
}

fn extract_magic_number(line: &str) -> Option<String> {
    // Very basic heuristic: look for numeric literals in assignments
    // This will have false positives; real implementation needs AST
    if line.contains(" = ") || line.contains(" == ") || line.contains(" != ") {
        // Look for numbers that aren't 0, 1, 2
        let parts: Vec<&str> = line.split_whitespace().collect();
        for part in parts {
            if let Ok(num) = part.trim_matches(',').trim_matches(';').parse::<i32>() {
                if !(-1..=2).contains(&num) {
                    return Some(num.to_string());
                }
            }
        }
    }
    None
}

fn is_snake_case(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
}

fn is_pascal_case(s: &str) -> bool {
    s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
        && s.chars().all(|c| c.is_alphanumeric())
        && !s.contains('_')
}

fn is_screaming_snake_case(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn to_screaming_snake_case(s: &str) -> String {
    s.to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_snake_case_function() {
        let source = r#"
fn my_function() {}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = StyleLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_non_snake_case_function() {
        let source = r#"
fn myFunction() {}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = StyleLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.violations.iter().any(|v| v.id == "STYLE001"));
    }

    #[tokio::test]
    async fn test_pascal_case_struct() {
        let source = r#"
struct MyStruct {}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = StyleLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_non_pascal_case_struct() {
        let source = r#"
struct my_struct {}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = StyleLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.violations.iter().any(|v| v.id == "STYLE002"));
    }

    #[tokio::test]
    async fn test_long_line() {
        let source = "x".repeat(150);
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = StyleLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.violations.iter().any(|v| v.id == "STYLE004"));
    }

    #[tokio::test]
    async fn test_trailing_whitespace() {
        let source = "fn main() {}  \n";
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = StyleLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.violations.iter().any(|v| v.id == "STYLE008"));
    }
}
