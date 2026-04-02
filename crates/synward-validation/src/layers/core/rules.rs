//! Pattern Rules for Logic Validation
//!
//! Defines PatternRule, ContextCheck and precision_rules() for the logic layer.

use crate::violation::Severity;

/// A pattern rule for logic validation.
#[derive(Debug, Clone)]
pub struct PatternRule {
    pub pattern: String,
    pub id: String,
    pub message: String,
    pub severity: Severity,
    pub suggestion: Option<String>,
    /// Context requirements for severity adjustment
    pub context_check: ContextCheck,
    /// Languages this rule applies to (None = all languages)
    pub languages: Option<Vec<String>>,
}

/// Context checks for smarter validation
#[derive(Debug, Clone, Copy)]
pub enum ContextCheck {
    /// Always report at base severity
    Always,
    /// Only report if in a loop (for clone patterns)
    InLoop,
    /// Only report if not preceded by safety check (for unwrap)
    UncheckedUnwrap,
    /// Only report if no SAFETY comment nearby (for unsafe)
    NoSafetyComment,
    /// Only report if not in test context
    NotInTest,
}

/// Precision rules with context awareness.
pub fn precision_rules() -> Vec<PatternRule> {
    vec![
        // === PANIC PATTERNS - ERROR ===
        PatternRule {
            pattern: "panic!(".into(),
            id: "LOGIC001".into(),
            message: "panic!() will crash the application".into(),
            severity: Severity::Error,
            suggestion: Some("Return Result<T, E> and propagate errors".into()),
            context_check: ContextCheck::NotInTest,
            languages: None,
        },
        PatternRule {
            pattern: "todo!(".into(),
            id: "LOGIC002".into(),
            message: "todo!() will panic at runtime".into(),
            severity: Severity::Warning,
            suggestion: Some("Implement or mark as intentionally unimplemented".into()),
            context_check: ContextCheck::NotInTest,
            languages: None,
        },
        PatternRule {
            pattern: "unimplemented!(".into(),
            id: "LOGIC003".into(),
            message: "unimplemented!() will panic when called".into(),
            severity: Severity::Info,
            suggestion: Some("Complete implementation before production".into()),
            context_check: ContextCheck::NotInTest,
            languages: None,
        },

        // === UNWRAP PATTERNS - Context-aware ===
        PatternRule {
            pattern: ".unwrap()".into(),
            id: "LOGIC010".into(),
            message: ".unwrap() can panic - use ? or expect(\"context\")".into(),
            severity: Severity::Warning,
            suggestion: Some("Use ? operator, expect(\"reason\"), or pattern matching".into()),
            context_check: ContextCheck::UncheckedUnwrap,
            languages: None,
        },
        PatternRule {
            pattern: ".unwrap_err()".into(),
            id: "LOGIC011".into(),
            message: ".unwrap_err() can panic on Ok variant".into(),
            severity: Severity::Warning,
            suggestion: Some("Use match or if let for proper error handling".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },

        // === EXPECT PATTERNS - Context-aware ===
        PatternRule {
            pattern: ".expect(\"\")".into(),
            id: "LOGIC020".into(),
            message: ".expect(\"\") has no context - add descriptive message".into(),
            severity: Severity::Warning,
            suggestion: Some("Add message: .expect(\"failed to load config\")".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },
        PatternRule {
            pattern: ".expect(\"error\")".into(),
            id: "LOGIC021".into(),
            message: ".expect(\"error\") is too generic".into(),
            severity: Severity::Info,
            suggestion: Some("Add context: .expect(\"failed to parse user config\")".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },

        // === UNSAFE PATTERNS - Context-aware ===
        PatternRule {
            pattern: "unsafe {".into(),
            id: "LOGIC030".into(),
            message: "unsafe block without SAFETY comment".into(),
            severity: Severity::Warning,
            suggestion: Some("Add SAFETY comment explaining why this is safe".into()),
            context_check: ContextCheck::NoSafetyComment,
            languages: None,
        },
        PatternRule {
            pattern: "unsafe fn ".into(),
            id: "LOGIC031".into(),
            message: "unsafe function - document safety requirements".into(),
            severity: Severity::Info,
            suggestion: Some("Add # Safety section in doc comment".into()),
            context_check: ContextCheck::NoSafetyComment,
            languages: None,
        },

        // === BOOLEAN ANTIPATTERNS - Always report ===
        PatternRule {
            pattern: "== true".into(),
            id: "LOGIC040".into(),
            message: "Redundant comparison: use the boolean directly".into(),
            severity: Severity::Info,
            suggestion: Some("Replace 'x == true' with 'x'".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },
        PatternRule {
            pattern: "== false".into(),
            id: "LOGIC041".into(),
            message: "Redundant comparison: use ! operator".into(),
            severity: Severity::Info,
            suggestion: Some("Replace 'x == false' with '!x'".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },

        // === CODE QUALITY MARKERS ===
        PatternRule {
            pattern: "TODO".into(),
            id: "LOGIC050".into(),
            message: "TODO: unfinished code".into(),
            severity: Severity::Info,
            suggestion: Some("Complete implementation before merge".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },
        PatternRule {
            pattern: "FIXME".into(),
            id: "LOGIC051".into(),
            message: "FIXME: code needs fixing".into(),
            severity: Severity::Info,
            suggestion: Some("Address before production".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },

        // === CLONE - Context-aware ===
        // LOGIC060: clone() in general (Info)
        PatternRule {
            pattern: ".clone()".into(),
            id: "LOGIC060".into(),
            message: ".clone() - verify necessity".into(),
            severity: Severity::Info,
            suggestion: Some("Check if & reference works, or if Arc is needed".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },
        // LOGIC061: clone() in loop (Warning) - performance issue
        PatternRule {
            pattern: ".clone()".into(),
            id: "LOGIC061".into(),
            message: ".clone() in loop - consider moving outside or using reference".into(),
            severity: Severity::Warning,
            suggestion: Some("Clone outside loop or use & reference to avoid repeated allocations".into()),
            context_check: ContextCheck::InLoop,
            languages: None,
        },
        PatternRule {
            pattern: ".to_string()".into(),
            id: "LOGIC061".into(),
            message: ".to_string() allocates - verify necessity".into(),
            severity: Severity::Info,
            suggestion: Some("Consider &str or format! for temporary strings".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },

        // === COLLECTION HINTS ===
        PatternRule {
            pattern: "Vec::new()".into(),
            id: "LOGIC070".into(),
            message: "Vec::new() - consider with_capacity if size known".into(),
            severity: Severity::Hint,
            suggestion: Some("Use Vec::with_capacity(n) to avoid reallocations".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },
        PatternRule {
            pattern: "HashMap::new()".into(),
            id: "LOGIC071".into(),
            message: "HashMap::new() - consider with_capacity if size known".into(),
            severity: Severity::Hint,
            suggestion: Some("Use HashMap::with_capacity(n) to avoid rehashing".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },

        // === PYTHON-SPECIFIC PATTERNS (AI Hallucination Traps) ===
        // TRAP 3: await in loop (should use asyncio.gather)
        PatternRule {
            pattern: "await ".into(),
            id: "LOGIC080".into(),
            message: "await in loop - sequential execution, not concurrent".into(),
            severity: Severity::Warning,
            suggestion: Some("Use asyncio.gather() for concurrent execution".into()),
            context_check: ContextCheck::InLoop,
            languages: None,
        },
        // TRAP 10: sort key with .get() can return None
        PatternRule {
            pattern: "key=lambda".into(),
            id: "LOGIC081".into(),
            message: "sort key with .get() may return None - comparison fails".into(),
            severity: Severity::Warning,
            suggestion: Some("Use key=lambda x: x.get('field', default) or handle None".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },
        // TRAP 15: integer division when float expected (Python)
        // Pattern matches: a // b (Python floor division)
        // Does NOT match: // comment (Rust/JS comments have no space before //)
        PatternRule {
            pattern: "// ".into(),  // Python: a//b or a // b, but NOT //comment
            id: "LOGIC082".into(),
            message: "Integer division (//) - may lose precision for ratios".into(),
            severity: Severity::Warning,
            suggestion: Some("Use / for float division if precision matters".into()),
            context_check: ContextCheck::Always,
            languages: Some(vec!["python".into()]),
        },
        // TRAP 7: list comprehension with side effects (Python)
        // Pattern: [func(x) for x in items] where func has side effects
        PatternRule {
            pattern: "[send_".into(),
            id: "LOGIC083".into(),
            message: "List comprehension with side effects - result discarded".into(),
            severity: Severity::Warning,
            suggestion: Some("Use a for loop or save the result if return values matter".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },
        // TRAP 14: file opened without context manager (Python)
        PatternRule {
            pattern: "= open(".into(),
            id: "LOGIC084".into(),
            message: "File opened without 'with' - may leak file handle".into(),
            severity: Severity::Warning,
            suggestion: Some("Use: with open(path) as f:".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },

        // === BLIND SPOTS - Collection/Float Safety (from Delta Analysis) ===
        // Float NaN comparison - always returns false
        PatternRule {
            pattern: ".partial_cmp(".into(),
            id: "LOGIC090".into(),
            message: ".partial_cmp() returns None for NaN - comparison may silently fail".into(),
            severity: Severity::Warning,
            suggestion: Some("Handle None case or use ordered_float crate".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },
        // Empty collection .first().unwrap()
        PatternRule {
            pattern: ".first().unwrap()".into(),
            id: "LOGIC091".into(),
            message: ".first().unwrap() panics on empty collection".into(),
            severity: Severity::Warning,
            suggestion: Some("Use .first().copied().unwrap_or_default() or check .is_empty()".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },
        // Empty collection .last().unwrap()
        PatternRule {
            pattern: ".last().unwrap()".into(),
            id: "LOGIC092".into(),
            message: ".last().unwrap() panics on empty collection".into(),
            severity: Severity::Warning,
            suggestion: Some("Use .last().copied().unwrap_or_default() or check .is_empty()".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },
        // HashMap/Graph unchecked node access
        PatternRule {
            pattern: ".node_weight(".into(),
            id: "LOGIC093".into(),
            message: ".node_weight() returns Option - unwrap may panic on invalid index".into(),
            severity: Severity::Warning,
            suggestion: Some("Use if let Some(w) = graph.node_weight(idx)".into()),
            context_check: ContextCheck::Always,
            languages: None,
        },
    ]
}
