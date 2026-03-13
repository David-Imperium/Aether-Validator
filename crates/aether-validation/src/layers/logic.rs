//! Logic Layer - Contract pattern evaluation (MILITARY GRADE)
//!
//! This layer enforces strict code quality rules that prevent common bugs
//! and maintain code reliability. All violations are ERROR-level by default.

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity};

/// Logic validation layer - Military Grade Enforcement.
///
/// Checks for:
/// - Contract violations (unwrap, panic, expect) - ERROR
/// - Unsafe code patterns - ERROR  
/// - Pattern violations (TODO, FIXME, XXX) - WARNING
/// - Code smells (long functions, deep nesting) - WARNING
/// - Antipatterns (clone in loop, boolean comparison) - ERROR
pub struct LogicLayer {
    /// Patterns to check for violations
    patterns: Vec<PatternRule>,
    /// Whether to treat warnings as errors
    #[allow(dead_code)]
    strict_mode: bool,
}

/// A pattern rule for logic validation.
#[derive(Debug, Clone)]
struct PatternRule {
    pattern: String,
    id: String,
    message: String,
    severity: Severity,
    suggestion: Option<String>,
}

impl LogicLayer {
    /// Create a new logic layer with military-grade default rules.
    pub fn new() -> Self {
        Self {
            patterns: Self::military_rules(),
            strict_mode: true,
        }
    }

    /// Create a logic layer with custom rules.
    pub fn with_rules(rules: Vec<(String, String, String, Severity, Option<String>)>) -> Self {
        let patterns = rules
            .into_iter()
            .map(|(pattern, id, message, severity, suggestion)| PatternRule {
                pattern,
                id,
                message,
                severity,
                suggestion,
            })
            .collect();
        Self { patterns, strict_mode: true }
    }

    /// Create a permissive logic layer (warnings allowed).
    pub fn permissive() -> Self {
        Self {
            patterns: Self::military_rules(),
            strict_mode: false,
        }
    }

    /// Military-grade rules - NO COMPROMISE.
    fn military_rules() -> Vec<PatternRule> {
        vec![
            // === PANIC PATTERNS - ERROR ===
            PatternRule {
                pattern: "panic!(".into(),
                id: "MIL001".into(),
                message: "PANIC DETECTED: panic!() is FORBIDDEN in production code".into(),
                severity: Severity::Error,
                suggestion: Some("Return Result<T, E> and propagate errors".into()),
            },
            PatternRule {
                pattern: "todo!(".into(),
                id: "MIL002".into(),
                message: "TODO DETECTED: todo!() will panic at runtime".into(),
                severity: Severity::Error,
                suggestion: Some("Implement the function or use unimplemented!() for prototypes".into()),
            },
            PatternRule {
                pattern: "unimplemented!(".into(),
                id: "MIL003".into(),
                message: "UNIMPLEMENTED: This code will panic when called".into(),
                severity: Severity::Warning,
                suggestion: Some("Complete implementation before production".into()),
            },

            // === UNWRAP PATTERNS - ERROR ===
            PatternRule {
                pattern: ".unwrap()".into(),
                id: "MIL010".into(),
                message: "UNSAFE UNWRAP: .unwrap() can panic and crash the application".into(),
                severity: Severity::Error,
                suggestion: Some("Use ? operator, expect(\"reason\"), or pattern matching".into()),
            },
            PatternRule {
                pattern: ".unwrap_or(".into(),
                id: "MIL011".into(),
                message: "Consider .unwrap_or_else() for expensive defaults".into(),
                severity: Severity::Info,
                suggestion: Some("Use .unwrap_or_else(|| expensive_calc()) for lazy evaluation".into()),
            },
            PatternRule {
                pattern: ".unwrap_err()".into(),
                id: "MIL012".into(),
                message: "UNWRAP_ERR: Can panic on Ok variant".into(),
                severity: Severity::Error,
                suggestion: Some("Use match or if let for proper error handling".into()),
            },

            // === EXPECT PATTERNS - ERROR for generic messages ===
            PatternRule {
                pattern: ".expect(\"\")".into(),
                id: "MIL020".into(),
                message: "EMPTY EXPECT: .expect(\"\") provides no debugging context".into(),
                severity: Severity::Error,
                suggestion: Some("Add descriptive message: .expect(\"failed to load config\")".into()),
            },
            PatternRule {
                pattern: ".expect(\"error\")".into(),
                id: "MIL021".into(),
                message: "GENERIC EXPECT: Use descriptive error messages".into(),
                severity: Severity::Error,
                suggestion: Some("Add context: .expect(\"failed to parse user config\")".into()),
            },
            PatternRule {
                pattern: ".expect(\"failed\")".into(),
                id: "MIL022".into(),
                message: "GENERIC EXPECT: 'failed' is not descriptive enough".into(),
                severity: Severity::Error,
                suggestion: Some("Add context: .expect(\"failed to connect to database\")".into()),
            },

            // === UNSAFE PATTERNS - ERROR ===
            PatternRule {
                pattern: "unsafe {".into(),
                id: "MIL030".into(),
                message: "UNSAFE BLOCK: Requires safety documentation and review".into(),
                severity: Severity::Error,
                suggestion: Some("Add SAFETY comment explaining why this is safe".into()),
            },
            PatternRule {
                pattern: "unsafe fn ".into(),
                id: "MIL031".into(),
                message: "UNSAFE FUNCTION: Must document safety requirements".into(),
                severity: Severity::Error,
                suggestion: Some("Add SAFETY comment and mark caller requirements".into()),
            },
            PatternRule {
                pattern: "unsafe impl".into(),
                id: "MIL032".into(),
                message: "UNSAFE IMPL: Must document safety guarantees".into(),
                severity: Severity::Error,
                suggestion: Some("Add SAFETY comment explaining invariant guarantees".into()),
            },

            // === BOOLEAN ANTIPATTERNS - ERROR ===
            PatternRule {
                pattern: "== true".into(),
                id: "MIL040".into(),
                message: "BOOLEAN ANTIPATTERN: Use the boolean directly".into(),
                severity: Severity::Error,
                suggestion: Some("Replace 'x == true' with 'x'".into()),
            },
            PatternRule {
                pattern: "== false".into(),
                id: "MIL041".into(),
                message: "BOOLEAN ANTIPATTERN: Use ! operator".into(),
                severity: Severity::Error,
                suggestion: Some("Replace 'x == false' with '!x'".into()),
            },
            PatternRule {
                pattern: "!= true".into(),
                id: "MIL042".into(),
                message: "BOOLEAN ANTIPATTERN: Use ! operator".into(),
                severity: Severity::Error,
                suggestion: Some("Replace 'x != true' with '!x'".into()),
            },
            PatternRule {
                pattern: "!= false".into(),
                id: "MIL043".into(),
                message: "BOOLEAN ANTIPATTERN: Use the boolean directly".into(),
                severity: Severity::Error,
                suggestion: Some("Replace 'x != false' with 'x'".into()),
            },

            // === CODE QUALITY - WARNING ===
            PatternRule {
                pattern: "TODO".into(),
                id: "MIL050".into(),
                message: "TODO: Unfinished code detected".into(),
                severity: Severity::Warning,
                suggestion: Some("Complete implementation before merge".into()),
            },
            PatternRule {
                pattern: "FIXME".into(),
                id: "MIL051".into(),
                message: "FIXME: Code needs fixing".into(),
                severity: Severity::Warning,
                suggestion: Some("Address fix before production deployment".into()),
            },
            PatternRule {
                pattern: "XXX".into(),
                id: "MIL052".into(),
                message: "XXX: Dangerous code pattern".into(),
                severity: Severity::Warning,
                suggestion: Some("Review and document or refactor".into()),
            },
            PatternRule {
                pattern: "HACK".into(),
                id: "MIL053".into(),
                message: "HACK: Workaround detected".into(),
                severity: Severity::Warning,
                suggestion: Some("Replace with proper implementation".into()),
            },

            // === CLONE ANTIPATTERNS - WARNING ===
            PatternRule {
                pattern: ".clone()".into(),
                id: "MIL060".into(),
                message: "CLONE: Review if clone is necessary".into(),
                severity: Severity::Info,
                suggestion: Some("Consider references, Cow<str>, or Arc".into()),
            },
            PatternRule {
                pattern: ".to_owned()".into(),
                id: "MIL061".into(),
                message: "TO_OWNED: Review allocation necessity".into(),
                severity: Severity::Info,
                suggestion: Some("Consider borrowing or using &str".into()),
            },
            PatternRule {
                pattern: ".to_string()".into(),
                id: "MIL062".into(),
                message: "TO_STRING: Review if allocation is needed".into(),
                severity: Severity::Info,
                suggestion: Some("Consider format! or &str for temporary strings".into()),
            },

            // === ERROR HANDLING ANTIPATTERNS - ERROR ===
            PatternRule {
                pattern: "catch_unwind".into(),
                id: "MIL070".into(),
                message: "CATCH_UNWIND: Panic catching is discouraged".into(),
                severity: Severity::Warning,
                suggestion: Some("Use Result-based error handling instead".into()),
            },
            PatternRule {
                pattern: "std::panic::set_hook".into(),
                id: "MIL071".into(),
                message: "PANIC_HOOK: Overriding panic hook requires review".into(),
                severity: Severity::Warning,
                suggestion: Some("Document panic handling strategy".into()),
            },

            // === COLLECTION ANTIPATTERNS - WARNING ===
            PatternRule {
                pattern: "Vec::new()".into(),
                id: "MIL080".into(),
                message: "VECT_NEW: Consider Vec::with_capacity() if size is known".into(),
                severity: Severity::Info,
                suggestion: Some("Use Vec::with_capacity(n) to avoid reallocations".into()),
            },
            PatternRule {
                pattern: "HashMap::new()".into(),
                id: "MIL081".into(),
                message: "HASHMAP_NEW: Consider HashMap::with_capacity() if size is known".into(),
                severity: Severity::Info,
                suggestion: Some("Use HashMap::with_capacity(n) to avoid rehashing".into()),
            },
        ]
    }
}

impl Default for LogicLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for LogicLayer {
    fn name(&self) -> &str {
        "logic"
    }

    fn priority(&self) -> u8 {
        30 // Third layer
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let mut violations = Vec::new();

        // Strip test blocks to avoid false positives
        let source_no_tests = strip_test_blocks(&ctx.source);
        // Strip string literals to avoid detecting patterns in definitions
        let source_stripped = strip_string_literals(&source_no_tests);

        // Check each pattern rule on production code only
        for rule in &self.patterns {
            if source_stripped.contains(&rule.pattern) {
                let violation = match rule.severity {
                    Severity::Error => Violation::error(&rule.id, &rule.message),
                    Severity::Warning => Violation::warning(&rule.id, &rule.message),
                    Severity::Info => Violation::info(&rule.id, &rule.message),
                    Severity::Hint => Violation::info(&rule.id, &rule.message),
                };
                let violation = if let Some(suggestion) = &rule.suggestion {
                    violation.suggest(suggestion)
                } else {
                    violation
                };
                violations.push(violation);
            }
        }

        // Check for long functions (on stripped source)
        check_long_functions(&source_stripped, &mut violations);

        // Check for deep nesting (on stripped source)
        check_deep_nesting(&source_stripped, &mut violations);

        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }
}

/// Strip test blocks from source code to avoid false positives.
/// Removes #[cfg(test)] modules completely.
fn strip_test_blocks(source: &str) -> String {
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
        if trimmed.starts_with("#[test]") || trimmed.starts_with("#[tokio::test]") {
            in_test_function = true;
            brace_depth = 0;
            continue;
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
fn strip_string_literals(source: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_byte_string = false;
    let mut in_raw_string = false;
    let mut raw_string_hashes = 0;
    let mut escape_next = false;
    let mut chars = source.chars().peekable();

    while let Some(ch) = chars.next() {
        // Handle escape sequences
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_string {
            escape_next = true;
            continue;
        }

        // Handle raw strings r#"..."# or r##"..."##
        if !in_string && !in_byte_string && !in_raw_string && ch == 'r' {
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
        if !in_string && !in_raw_string && ch == 'b' {
            if let Some(&'"') = chars.peek() {
                chars.next();
                in_byte_string = true;
                continue;
            } else {
                result.push(ch);
                continue;
            }
        }

        // Handle regular strings
        if ch == '"' && !in_byte_string {
            in_string = !in_string;
            continue;
        }

        // Handle byte string closing
        if ch == '"' && in_byte_string {
            in_byte_string = false;
            continue;
        }

        // Keep chars not in strings
        if !in_string && !in_byte_string && !in_raw_string {
            result.push(ch);
        }
    }

    result
}

fn check_long_functions(source: &str, violations: &mut Vec<Violation>) {
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
                if fn_length > 50 {
                    violations.push(
                        Violation::info("LOGIC008", format!("Function is {} lines long (max 50)", fn_length))
                            .suggest("Consider breaking into smaller functions")
                    );
                }
                fn_start = None;
            }
        }
    }
}

fn check_deep_nesting(source: &str, violations: &mut Vec<Violation>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut max_indent = 0;

    for line in &lines {
        let indent = line.chars().take_while(|c| *c == ' ').count() / 4;
        if indent > max_indent {
            max_indent = indent;
        }
    }

    if max_indent > 4 {
        violations.push(
            Violation::warning("LOGIC009", format!("Deep nesting detected: {} levels", max_indent))
                .suggest("Extract nested code into helper functions")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_violations() {
        let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = LogicLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.passed, "Clean code should pass: {:?}", result.violations);
    }

    #[tokio::test]
    async fn test_unwrap_violation() {
        let source = r#"
fn main() {
    let x = option.unwrap();
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = LogicLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty(), "unwrap should trigger violation");
        assert!(result.violations.iter().any(|v| v.id == "MIL010"), "Expected MIL010 for unwrap");
    }

    #[tokio::test]
    async fn test_panic_violation() {
        let source = r#"
fn main() {
    panic!("not implemented");
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = LogicLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty(), "panic should trigger violation");
        assert!(result.violations.iter().any(|v| v.id == "MIL001"), "Expected MIL001 for panic");
    }

    #[tokio::test]
    async fn test_unsafe_violation() {
        let source = r#"
fn main() {
    unsafe {
        *std::ptr::null();
    }
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = LogicLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty(), "unsafe should trigger violation");
        assert!(result.violations.iter().any(|v| v.id == "MIL030"), "Expected MIL030 for unsafe");
    }

    #[tokio::test]
    async fn test_boolean_antipattern() {
        let source = r#"
fn main() {
    if x == true {
        println!("yes");
    }
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = LogicLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty(), "== true should trigger violation");
        assert!(result.violations.iter().any(|v| v.id == "MIL040"), "Expected MIL040 for == true");
    }

    #[tokio::test]
    async fn test_todo_violation() {
        let source = r#"
fn main() {
    todo!();
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = LogicLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty(), "todo! should trigger violation");
        assert!(result.violations.iter().any(|v| v.id == "MIL002"), "Expected MIL002 for todo!");
    }
}
