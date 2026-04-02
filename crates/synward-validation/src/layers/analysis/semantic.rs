//! Semantic Layer — Type checking and scope analysis

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::{Violation, deduplicate_violations};

/// Semantic validation layer.
///
/// Checks for:
/// - Unused variables
/// - Type mismatches (basic)
/// - Scope issues
/// - Dead code detection
/// - Python-specific: method errors, wrong exception types
pub struct SemanticLayer;

impl SemanticLayer {
    /// Create a new semantic layer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SemanticLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for SemanticLayer {
    fn name(&self) -> &str {
        "semantic"
    }

    fn priority(&self) -> u8 {
        20 // Second layer
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let mut violations = Vec::new();
        let source = &ctx.source;
        let language = ctx.language.as_str();

        // Check for unused variables (pattern: let x = ...; with no later use)
        check_unused_variables(source, &mut violations);

        // Check for unreachable code (pattern: return ...; ... code after)
        check_unreachable_code(source, &mut violations);

        // Check for shadowing (pattern: let x = ...; let x = ...;)
        check_variable_shadowing(source, &mut violations);

        // Python-specific semantic checks
        if language == "python" {
            check_python_method_errors(source, &mut violations);
            check_python_wrong_exception(source, &mut violations);
            check_python_mutable_defaults(source, &mut violations);
            check_python_iterator_exhaustion(source, &mut violations);
            check_python_regex_anchoring(source, &mut violations);
            check_python_boolean_trap(source, &mut violations);
            check_python_on2_pattern(source, &mut violations);
        }

        let violations = deduplicate_violations(violations);
        
        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }
}

fn check_unused_variables(source: &str, violations: &mut Vec<Violation>) {
    // Simple heuristic: find let bindings that are never referenced
    // This is a basic check; real implementation would use AST
    let lines: Vec<&str> = source.lines().collect();
    let mut bindings = Vec::new();

    for line in &lines {
        if let Some(binding) = extract_let_binding(line) {
            bindings.push(binding);
        }
    }

    for binding in &bindings {
        let binding_used = source.matches(binding).count() > 1;
        if !binding_used && binding != "_" {
            violations.push(Violation::warning(
                "SEMANTIC001",
                format!("Unused variable: {}", binding),
            ).suggest(format!("Prefix with underscore: _{}", binding)));
        }
    }
}

fn check_unreachable_code(source: &str, violations: &mut Vec<Violation>) {
    // Check for code after return/panic/break/continue
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("return") || trimmed.starts_with("panic!") || trimmed.starts_with("break") {
            // Check if there's code after this line in the same block
            if i + 1 < lines.len() {
                let next_trimmed = lines[i + 1].trim();
                // Skip continuation lines (lines starting with operators or closing braces)
                if next_trimmed.is_empty() 
                    || next_trimmed.starts_with('}') 
                    || next_trimmed.starts_with("//")
                    || next_trimmed.starts_with("&&")
                    || next_trimmed.starts_with("||")
                    || next_trimmed.starts_with('|')
                    || next_trimmed.starts_with('&')
                    || next_trimmed.starts_with('+')
                    || next_trimmed.starts_with('-')
                    || next_trimmed.starts_with('*')
                    || next_trimmed.starts_with('/')
                    || next_trimmed.starts_with(',')
                    || next_trimmed.starts_with('.')
                    || next_trimmed.starts_with(':')
                {
                    continue;
                }
                // Skip if previous line doesn't end with semicolon (continuation)
                if !trimmed.ends_with(';') && !trimmed.ends_with('}') {
                    continue;
                }
                violations.push(Violation::info(
                    "SEMANTIC002",
                    "Potentially unreachable code after return/break",
                ).suggest("Remove unreachable code or restructure control flow"));
            }
        }
    }
}

fn check_variable_shadowing(source: &str, violations: &mut Vec<Violation>) {
    // Simple heuristic: find multiple let bindings with same name
    let lines: Vec<&str> = source.lines().collect();
    let mut seen_bindings = std::collections::HashSet::new();

    for line in &lines {
        if let Some(binding) = extract_let_binding(line) {
            if seen_bindings.contains(&binding) && binding != "_" {
                violations.push(Violation::info(
                    "SEMANTIC003",
                    format!("Variable shadowing: {}", binding),
                ).suggest("Consider using different names to avoid confusion"));
            }
            seen_bindings.insert(binding);
        }
    }
}

fn extract_let_binding(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("let ") {
        return None;
    }

    // Extract variable name after 'let' and before '=' or ':'
    let rest = trimmed.strip_prefix("let ")?.trim();
    let name_end = rest.find(['=', ':']).unwrap_or(rest.len());
    let name = rest[..name_end].trim();

    // Skip patterns like "mut x"
    let name = if name.starts_with("mut ") {
        name.strip_prefix("mut ")?.trim()
    } else {
        name
    };

    if name.is_empty() || name.starts_with('(') || name.starts_with('[') {
        return None;
    }

    Some(name.to_string())
}

// ============================================================================
// Python-Specific Semantic Checks (AI Hallucination Traps)
// ============================================================================

/// TRAP 2: Check for method calls on wrong types (e.g., str.append())
fn check_python_method_errors(source: &str, violations: &mut Vec<Violation>) {
    // Pattern: variable.method() where method doesn't exist on the inferred type
    // Common AI mistakes:
    // - str.append() - strings don't have append, use += or list
    // - str.push() - strings don't have push
    // - list.add() - lists have append, not add
    // - dict.append() - dicts don't have append

    let str_methods = ["append", "push", "add", "extend"];
    let _list_methods = ["add", "push"]; // lists don't have these
    let _dict_methods = ["append", "push", "add", "extend"]; // dicts don't have these

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Check for common method errors
        // Pattern: name.append("...") where name is likely a string
        for method in &str_methods {
            let pattern = format!(".{}(", method);
            if trimmed.contains(&pattern) {
                // Heuristic: if the variable name suggests a string
                // Check if the line context suggests string usage
                if is_likely_string_context(trimmed) {
                    violations.push(Violation::error(
                        "SEMANTIC010",
                        format!("str.{}() - strings don't have this method", method),
                    ).suggest(match *method {
                        "append" => "Use += for string concatenation or convert to list first",
                        "push" => "Use += for string concatenation",
                        "add" => "Strings don't have add() - use += or format!",
                        "extend" => "Strings don't have extend() - use += or join()",
                        _ => "Check the type - this method may not exist",
                    }));
                }
            }
        }

        // Check dict.update() return value misuse (TRAP 5)
        if trimmed.contains(".update(") && (trimmed.contains("return ") || trimmed.contains("=")) {
            // Pattern: x = dict.update(...) or return dict.update(...)
            if trimmed.contains("return ") || (trimmed.contains('=') && !trimmed.contains("==")) {
                violations.push(Violation::warning(
                    "SEMANTIC011",
                    "dict.update() returns None, not the updated dict",
                ).suggest("Update in-place, then return the dict separately"));
            }
        }
    }
}

/// Heuristic to detect if a line context suggests string variable
fn is_likely_string_context(line: &str) -> bool {
    // If the variable is being used in string operations
    line.contains("\"") || line.contains("'") || line.contains("f\"") || line.contains("f'")
        || line.contains("name") || line.contains("text") || line.contains("str")
        || line.contains("msg") || line.contains("message") || line.contains("path")
}

/// TRAP 4: Check for wrong exception types in except clauses
fn check_python_wrong_exception(source: &str, violations: &mut Vec<Violation>) {
    // Common AI mistakes:
    // - json.loads() raises JSONDecodeError, not ValueError
    // - int() raises ValueError, but float("inf") doesn't raise OverflowError
    // - open() raises FileNotFoundError, not IOError (Python 3)

    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check for json.loads with ValueError catch
        if trimmed.contains("json.loads") || trimmed.contains("json.load") {
            // Look for surrounding try/except block
            let lookback = std::cmp::min(5, i);
            let lookahead = std::cmp::min(5, lines.len() - i - 1);
            let context: String = lines[i.saturating_sub(lookback)..=i + lookahead].join("\n");

            if context.contains("except ValueError") {
                violations.push(Violation::error(
                    "SEMANTIC020",
                    "json.loads() raises json.JSONDecodeError, not ValueError",
                ).suggest("Use: except json.JSONDecodeError as e:"));
            }
        }

        // Check for datetime.strptime with wrong format codes (TRAP 6)
        if trimmed.contains("strptime") && trimmed.contains("%M") {
            // %M is MINUTE, %m is MONTH - common confusion
            // Check if this looks like a date format (has year, day separators)
            if trimmed.contains("%Y") || trimmed.contains("%d") || trimmed.contains("-") {
                violations.push(Violation::warning(
                    "SEMANTIC021",
                    "datetime format: %M is MINUTE, did you mean %m (month)?",
                ).suggest("Use %m for month, %M for minute"));
            }
        }

        // Check for int() with wrong exception
        if trimmed.contains("int(") {
            let context = get_surrounding_context(&lines, i, 3);
            if context.contains("except TypeError") {
                violations.push(Violation::warning(
                    "SEMANTIC022",
                    "int() raises ValueError for invalid strings, not TypeError",
                ).suggest("Use: except ValueError:"));
            }
        }
    }
}

/// TRAP 8: Check for mutable default arguments
fn check_python_mutable_defaults(source: &str, violations: &mut Vec<Violation>) {
    // Pattern: def func(x=[], y={}, z=set())
    // Classic Python bug - mutable defaults are shared between calls

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            // Check for mutable defaults in function signature
            if trimmed.contains("=[]") || trimmed.contains("= {}") || trimmed.contains("=set()") {
                violations.push(Violation::error(
                    "SEMANTIC030",
                    "Mutable default argument - shared between function calls",
                ).suggest("Use None as default and create mutable inside function"));
            }
        }
    }
}

/// TRAP 11: Check for iterator exhaustion patterns
fn check_python_iterator_exhaustion(source: &str, violations: &mut Vec<Violation>) {
    // Pattern: calling list() or next() multiple times on same iterator
    // iter(data) then list(iter) exhausts it

    let mut iter_vars: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Track iterator creation: x = iter(...)
        if let Some(pos) = trimmed.find("= iter(") {
            let var_name = trimmed[..pos].trim();
            iter_vars.insert(var_name.to_string(), line_num);
            continue;
        }

        // Check for multiple uses of same iterator
        for (var_name, &_created_line) in &iter_vars {
            // First use
            if trimmed.contains(&format!("list({})", var_name))
               || trimmed.contains(&format!("next({})", var_name)) {
                // Check if there's another use later
                // This is a heuristic - real implementation would use AST
                let remaining = source.lines().skip(line_num + 1).collect::<Vec<_>>();
                for future_line in &remaining {
                    if future_line.contains(&format!("list({})", var_name))
                       || future_line.contains(&format!("next({})", var_name)) {
                        violations.push(Violation::warning(
                            "SEMANTIC040",
                            format!("Iterator '{}' exhausted - second use will be empty", var_name),
                        ).suggest("Create new iterator or convert to list once"));
                        break;
                    }
                }
            }
        }
    }
}

/// TRAP 12: Check for regex patterns without anchors
fn check_python_regex_anchoring(source: &str, violations: &mut Vec<Violation>) {
    // Pattern: re.match(pattern, ...) where pattern lacks ^ or $
    // This allows partial matches that can be exploited

    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Look for re.match or re.search with a pattern variable or literal
        if trimmed.contains("re.match(") || trimmed.contains("re.search(") {
            // Check if there's a pattern string on this line or previous
            let lookback = std::cmp::min(2, i);
            let context: String = source.lines()
                .skip(i.saturating_sub(lookback))
                .take(lookback + 1)
                .collect::<Vec<_>>()
                .join("\n");

            // Check if pattern is a string literal without anchors
            if context.contains("r\"") || context.contains("r'") || context.contains("\"") || context.contains("'") {
                // Extract the pattern string
                if let Some(pattern_start) = context.find("r\"").or_else(|| context.find("r'")) {
                    let quote_char = if context.contains("r\"") { '"' } else { '\'' };
                    if let Some(pattern_end) = context[pattern_start + 2..].find(quote_char) {
                        let pattern = &context[pattern_start + 2..pattern_start + 2 + pattern_end];
                        // Check if pattern lacks anchors
                        if !pattern.starts_with('^') && !pattern.ends_with('$') {
                            // Only warn if pattern looks like it should be anchored (email, url, etc.)
                            if pattern.contains("@") || pattern.contains(".") || pattern.contains(":") {
                                violations.push(Violation::warning(
                                    "SEMANTIC041",
                                    "Regex pattern lacks anchors - may match partial strings",
                                ).suggest("Add ^ at start and $ at end to match entire string"));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// TRAP 9: Check for boolean logic traps with empty dict
fn check_python_boolean_trap(source: &str, violations: &mut Vec<Violation>) {
    // Pattern: if user and user.get(...) where empty dict {} passes first check
    // The bug is that {} is truthy, so empty dict passes the first condition

    for line in source.lines() {
        let trimmed = line.trim();

        // Pattern: if <var> and <var>.get(...)
        if trimmed.starts_with("if ") && trimmed.contains(" and ") && trimmed.contains(".get(") {
            // Extract the variable being checked
            let parts: Vec<&str> = trimmed.split(" and ").collect();
            if parts.len() >= 2 {
                let first_part = parts[0].strip_prefix("if ").unwrap_or(parts[0]).trim();
                let second_part = parts[1].trim();

                // Check if same variable is used in both parts
                if second_part.starts_with(first_part) || second_part.contains(&format!("{}.", first_part)) {
                    violations.push(Violation::warning(
                        "SEMANTIC042",
                        "Boolean check may pass for empty dict {} - .get() on empty dict returns None",
                    ).suggest("Check explicitly: if user is not None and user.get('field')"));
                }
            }
        }
    }
}

/// TRAP 13: Check for O(n²) patterns in loops
fn check_python_on2_pattern(source: &str, violations: &mut Vec<Violation>) {
    // Pattern: for item in items: if item in items[:i] or items[i+1:]
    // This is O(n²) because each 'in' check is O(n) inside a loop

    let lines: Vec<&str> = source.lines().collect();
    let mut in_loop = false;
    let mut loop_indent = 0;

    for line in lines.iter() {
        let trimmed = line.trim();

        // Track for loop context
        if trimmed.starts_with("for ") && trimmed.contains(" in ") {
            in_loop = true;
            // Calculate indentation of the for loop
            loop_indent = line.len() - line.trim_start().len();
        }

        // Check indentation to detect loop end (more indented = inside loop)
        if !trimmed.is_empty() {
            let current_indent = line.len() - line.trim_start().len();
            // If we're back to the same or less indentation than the for loop, we've exited
            if in_loop && current_indent <= loop_indent && !trimmed.starts_with("for ") {
                in_loop = false;
            }
        }

        // Check for 'if item in list[:i]' pattern inside a loop
        if in_loop && trimmed.starts_with("if ") && trimmed.contains(" in ") {
            // Check if it's checking membership in a slice
            if trimmed.contains("[:") || trimmed.contains("[i:") || trimmed.contains("[:-") {
                violations.push(Violation::warning(
                    "SEMANTIC043",
                    "O(n²) pattern - 'in' on slice inside loop",
                ).suggest("Use a set for O(1) lookups: seen = set()"));
            }
        }
    }
}

/// Helper: get surrounding context for a line
fn get_surrounding_context(lines: &[&str], center: usize, radius: usize) -> String {
    let start = center.saturating_sub(radius);
    let end = std::cmp::min(lines.len(), center + radius + 1);
    lines[start..end].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_violations() {
        let source = r#"
fn main() {
    let x = 1;
    println!("{}", x);
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = SemanticLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_unused_variable() {
        let source = r#"
fn main() {
    let unused = 1;
    println!("hello");
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = SemanticLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty());
    }
}
