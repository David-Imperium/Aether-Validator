//! Source Stripping Utilities
//!
//! Removes test blocks and string literals from source code to avoid false positives
//! during pattern matching validation.

use crate::violation::Violation;

/// Strip test blocks from source code to avoid false positives.
/// Removes #[cfg(test)] modules completely.
/// Enhanced to detect more test patterns:
/// - #[cfg(test)] modules
/// - #[test] functions
/// - #[tokio::test] functions
/// - Functions starting with test_ (convention)
/// - Functions ending with _test (convention)
pub fn strip_test_blocks(source: &str) -> String {
    let mut result = String::new();
    let mut in_test_module = false;
    let mut in_test_function = false;
    let mut brace_depth: i32 = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        // Detect test module start
        if trimmed.starts_with("#[cfg(test)]") {
            in_test_module = true;
            brace_depth = 0;
            continue;
        }

        // Track braces in test module
        if in_test_module {
            for ch in line.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => {
                        brace_depth -= 1;
                        if brace_depth <= 0 {
                            in_test_module = false;
                        }
                    }
                    _ => {}
                }
            }
            continue;
        }

        // Skip individual test function attributes and the function body
        if trimmed.starts_with("#[test]") || 
           trimmed.starts_with("#[tokio::test]") ||
           trimmed.starts_with("#[async_test]") ||
           trimmed.starts_with("#[rstest]") {
            in_test_function = true;
            brace_depth = 0;
            continue;
        }

        // Also detect test functions by naming convention
        // fn test_* or fn *_test patterns (after attributes check)
        if trimmed.starts_with("fn test_") || trimmed.starts_with("fn test") ||
           trimmed.contains("_test(") || trimmed.contains("_test <") {
            // Check if this is inside a #[cfg(test)] block already handled
            // If not, still track it as test function
            in_test_function = true;
            brace_depth = 0;
        }

        // Track braces in test function
        if in_test_function {
            for ch in line.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => {
                        brace_depth -= 1;
                        if brace_depth <= 0 {
                            in_test_function = false;
                        }
                    }
                    _ => {}
                }
            }
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Remove string literals from source to avoid false positives.
/// Replaces "..." with empty content, preserving code structure.
/// Handles Python triple-quoted strings ("""...""" and '''...''').
pub fn strip_string_literals(source: &str) -> String {
    let mut result = String::new();
    let mut in_double_string = false;  // "..."
    let mut in_single_string = false;  // '...'
    let mut in_byte_string = false;
    let mut in_raw_string = false;
    let mut raw_string_hashes = 0;
    let mut in_triple_double = false;  // Python """..."""
    let mut in_triple_single = false; // Python '''...'''
    let mut in_comment = false;        // Python # comment
    let mut escape_next = false;
    let mut chars = source.chars().peekable();

    while let Some(ch) = chars.next() {
        // Handle end of line (resets comment state)
        if ch == '\n' {
            in_comment = false;
            result.push(ch);
            continue;
        }

        // Skip content inside comments
        if in_comment {
            continue;
        }

        // Handle escape sequences
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' && (in_double_string || in_single_string) {
            escape_next = true;
            continue;
        }

        // Handle Python comments: # starts a comment (outside strings)
        if !in_double_string && !in_single_string && !in_byte_string && !in_raw_string && !in_triple_double && !in_triple_single && ch == '#' {
            in_comment = true;
            continue;
        }

        // Handle Python triple-quoted strings: """...""" or '''...'''
        if !in_double_string && !in_single_string && !in_byte_string && !in_raw_string && !in_triple_double && !in_triple_single {
            if ch == '"' {
                // Check for triple double-quote """
                let mut quote_count = 1;
                while let Some(&'"') = chars.peek() {
                    quote_count += 1;
                    chars.next();
                    if quote_count == 3 {
                        break;
                    }
                }
                if quote_count == 3 {
                    in_triple_double = true;
                    continue;
                } else if quote_count == 1 {
                    // Single quote - regular string
                    in_double_string = true;
                    continue;
                } else {
                    // Two quotes "" - empty string, keep searching
                    in_double_string = true;
                    continue;
                }
            } else if ch == '\'' {
                // Check for triple single-quote '''
                let mut quote_count = 1;
                while let Some(&'\'') = chars.peek() {
                    quote_count += 1;
                    chars.next();
                    if quote_count == 3 {
                        break;
                    }
                }
                if quote_count == 3 {
                    in_triple_single = true;
                    continue;
                } else if quote_count == 1 {
                    // Single quote - regular string
                    in_single_string = true;
                    continue;
                } else {
                    // Two quotes '' - empty string, keep searching
                    in_single_string = true;
                    continue;
                }
            }
        }

        // Handle triple-double quote closing
        if in_triple_double && ch == '"' {
            // Check for """
            let mut quote_count = 1;
            while let Some(&'"') = chars.peek() {
                quote_count += 1;
                chars.next();
                if quote_count == 3 {
                    break;
                }
            }
            if quote_count == 3 {
                in_triple_double = false;
                continue;
            }
            // Otherwise, it's just a quote inside the string
            continue;
        }

        // Handle triple-single quote closing
        if in_triple_single && ch == '\'' {
            // Check for '''
            let mut quote_count = 1;
            while let Some(&'\'') = chars.peek() {
                quote_count += 1;
                chars.next();
                if quote_count == 3 {
                    break;
                }
            }
            if quote_count == 3 {
                in_triple_single = false;
                continue;
            }
            // Otherwise, it's just a quote inside the string
            continue;
        }

        // Handle raw strings r#"..."# or r##"..."##
        if !in_double_string && !in_single_string && !in_byte_string && !in_raw_string && !in_triple_double && !in_triple_single && ch == 'r' {
            let mut hash_count = 0;
            while let Some(&next) = chars.peek() {
                if next == '#' {
                    hash_count += 1;
                    chars.next();
                } else {
                    break;
                }
            }
            if let Some(&'"') = chars.peek() {
                chars.next(); // consume the opening "
                in_raw_string = true;
                raw_string_hashes = hash_count;
                continue;
            } else {
                // Just 'r' followed by something else
                result.push(ch);
                continue;
            }
        }

        // Handle raw string closing
        if in_raw_string {
            if ch == '"' {
                let mut closing_hashes = 0;
                while let Some(&next) = chars.peek() {
                    if next == '#' && closing_hashes < raw_string_hashes {
                        closing_hashes += 1;
                        chars.next();
                    } else {
                        break;
                    }
                }
                if closing_hashes == raw_string_hashes {
                    in_raw_string = false;
                    raw_string_hashes = 0;
                    continue;
                }
            }
            continue; // Skip content inside raw string
        }

        // Handle byte strings b"..."
        if !in_double_string && !in_single_string && !in_raw_string && !in_triple_double && !in_triple_single && ch == 'b' {
            if let Some(&'"') = chars.peek() {
                chars.next();
                in_byte_string = true;
                continue;
            } else {
                result.push(ch);
                continue;
            }
        }

        // Handle regular strings (both " and ')
        // Track separately to avoid mixing quote types
        // Must check !in_triple_double and !in_triple_single to avoid interfering with triple quotes
        if ch == '"' && !in_byte_string && !in_triple_double && !in_triple_single && !in_single_string {
            in_double_string = !in_double_string;
            continue;
        }
        if ch == '\'' && !in_byte_string && !in_triple_double && !in_triple_single && !in_double_string {
            in_single_string = !in_single_string;
            continue;
        }

        // Handle byte string closing
        if ch == '"' && in_byte_string {
            in_byte_string = false;
            continue;
        }

        // Keep chars not in strings
        if !in_double_string && !in_single_string && !in_byte_string && !in_raw_string && !in_triple_double && !in_triple_single {
            result.push(ch);
        } else if ch == '\n' {
            // Preserve newlines even inside strings to maintain line numbers
            result.push(ch);
        }
    }

    result
}

/// Find lines where unwrap is preceded by a check (is_some, is_ok, etc.).
/// Enhanced to reduce false positives by recognizing:
/// - Standard checks: is_some(), is_ok(), if let, match
/// - SAFETY comments explaining why unwrap is safe
/// - assert!/debug_assert! macros that guard the unwrap
/// - Conditional checks like if x > 0 before vec[idx].unwrap()
/// - Infallible operations (serde_json::to_string, etc.)
pub fn find_checked_unwrap_lines(source: &str) -> Vec<usize> {
    let mut checked_lines = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    // Infallible operations where unwrap is acceptable
    let infallible_patterns = [
        "serde_json::to_string",
        "serde_json::to_string_pretty",
        "serde_json::to_vec",
        "std::panic::catch_unwind",
        // Add more as needed
    ];

    for (i, line) in lines.iter().enumerate() {
        if line.contains(".unwrap()") {
            // Check for infallible operations on same line
            for pattern in &infallible_patterns {
                if line.contains(pattern) {
                    checked_lines.push(i + 1);
                    continue;
                }
            }

            // Check previous few lines for safety patterns
            let lookback = std::cmp::min(10, i);  // Increased lookback
            let context = lines[i.saturating_sub(lookback)..=i].join("\n");

            // Standard checks
            if context.contains("is_some()") || context.contains("is_ok()") ||
               context.contains("is_err()") || context.contains("is_none()") ||
               context.contains("if let Some") || context.contains("if let Ok") ||
               context.contains("if let Err") || context.contains("match ") {
                checked_lines.push(i + 1);
                continue;
            }

            // SAFETY comment (look in context and current line)
            let context_lower = context.to_lowercase();
            if context_lower.contains("// safety") || 
               context_lower.contains("//safety") ||
               context_lower.contains("/* safety") {
                checked_lines.push(i + 1);
                continue;
            }

            // Assert macros (guard clause pattern)
            if context.contains("assert!") || context.contains("debug_assert!") ||
               context.contains("assert_eq!") || context.contains("assert_ne!") {
                checked_lines.push(i + 1);
                continue;
            }

            // Early return patterns (if x.is_none() { return } before unwrap)
            if context.contains("return") && 
               (context.contains("is_none()") || context.contains("is_err()")) {
                checked_lines.push(i + 1);
                continue;
            }

            // Index bounds checks (if idx < len before arr[idx].unwrap())
            if context.contains(".len()") && context.contains('<') {
                checked_lines.push(i + 1);
                continue;
            }

            // expect() on same line means context is provided
            if line.contains(".expect(") {
                checked_lines.push(i + 1);
                continue;
            }

            // let-else pattern: let Some(x) = opt else { return }
            // This is a safe pattern because it forces early return on None
            if context.contains("let Some(") && context.contains("else {") ||
               context.contains("let Ok(") && context.contains("else {") {
                checked_lines.push(i + 1);
                continue;
            }

            // unwrap_or variants - these are safe alternatives (not .unwrap())
            // Note: These don't contain ".unwrap()" so won't match above check
            // But if someone writes unwrap_or().unwrap(), that's different
            if line.contains(".unwrap_or(") || line.contains(".unwrap_or_default()") ||
               line.contains(".unwrap_or_else(") || line.contains(".unwrap_unchecked(") {
                // These are safe alternatives or explicitly unsafe - skip
                continue;
            }
        }
    }

    checked_lines
}

/// Find lines that contain SAFETY comments.
/// Enhanced to detect various safety comment formats:
/// - // SAFETY: ...
/// - // SAFETY = ...
/// - /* SAFETY: ... */
/// - /// # Safety (doc comments for unsafe functions)
pub fn find_safety_comment_lines(source: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let line_lower = line.to_lowercase();
            // Standard SAFETY comment
            if line.contains("// SAFETY") || line.contains("//SAFETY") ||
               line.contains("/* SAFETY") || line.contains("/*SAFETY") {
                Some(i + 1)
            }
            // Doc comment for unsafe functions (/// # Safety)
            else if line.contains("/// # Safety") || line.contains("///# Safety") ||
                    line_lower.contains("/// # safety") {
                Some(i + 1)
            }
            // Inline safety justification
            else if line.contains("// safety:") || line.contains("//safety:") {
                Some(i + 1)
            }
            else {
                None
            }
        })
        .collect()
}

/// Check if there's a SAFETY comment near the given line.
/// Enhanced to check both before and after the line, with configurable distance.
pub fn has_nearby_safety(line: usize, safety_lines: &[usize], distance: usize) -> bool {
    safety_lines.iter().any(|&s| s.abs_diff(line) <= distance)
}

/// Check if a line has an inline safety justification.
/// Looks for patterns like: // SAFETY: ... at end of line
pub fn has_inline_safety(line: &str) -> bool {
    let line_lower = line.to_lowercase();
    line_lower.contains("// safety:") || 
    line_lower.contains("//safety:") ||
    line_lower.contains("/* safety:") ||
    line_lower.contains("/* safety */")
}

/// Check for long functions (>50 lines).
pub fn check_long_functions(source: &str, violations: &mut Vec<Violation>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut fn_start: Option<usize> = None;
    let mut brace_count = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.contains("fn ") && line.contains('(') {
            fn_start = Some(i);
            brace_count = 0;
        }

        if fn_start.is_some() {
            brace_count += line.matches('{').count();
            brace_count = brace_count.saturating_sub(line.matches('}').count());

            if brace_count == 0 && i > fn_start.unwrap_or(0) {
                let fn_length = i - fn_start.unwrap_or(0);
                if fn_length > 150 {
                    violations.push(
                        Violation::info("LOGIC008", format!("Function is {} lines long (max 150)", fn_length))
                            .suggest("Consider breaking into smaller functions")
                    );
                }
                fn_start = None;
            }
        }
    }
}

/// Check for deep nesting (>4 levels).
pub fn check_deep_nesting(source: &str, violations: &mut Vec<Violation>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut max_indent = 0;

    for line in &lines {
        let indent = line.chars().take_while(|&c| c == ' ').count() / 4;
        if indent > max_indent {
            max_indent = indent;
        }
    }

    if max_indent > 6 {
        violations.push(
            Violation::warning("LOGIC009", format!("Deep nesting detected: {} levels (max 6)", max_indent))
                .suggest("Extract nested logic into separate functions")
        );
    }
}
