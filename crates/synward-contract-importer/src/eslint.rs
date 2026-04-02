//! ESLint Rule Importer
//!
//! Imports ESLint rules from the official ESLint repository.
//! Source: https://github.com/eslint/eslint (lib/rules/index.js)
//! Parses the rules index to extract all ~270+ rules.

use crate::{ImportedContract, ContractSource, Severity, Importer};
use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use std::collections::HashSet;

pub struct ESLintImporter {
    client: Client,
}

impl ESLintImporter {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("synward-contract-importer/1.0")
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Parse ESLint rules from the lib/rules/index.js file
    /// Format: "rule-name": () => require("./rule-name"),
    fn parse_rules_index(&self, content: &str) -> Vec<ImportedContract> {
        let mut rules = Vec::new();
        let mut seen_rules: HashSet<String> = HashSet::new();

        // Pattern: "rule-name": or 'rule-name':
        let rule_pattern = Regex::new(r#"["']([a-z][a-z0-9-]*)["']\s*:"#).unwrap();

        for cap in rule_pattern.captures_iter(content) {
            let rule_name = cap[1].to_string();

            // Skip non-rule entries (like special keys)
            if seen_rules.contains(&rule_name) || rule_name.starts_with('_') {
                continue;
            }
            seen_rules.insert(rule_name.clone());

            // Determine domain and severity based on rule name prefix/conventions
            let (domain, severity) = Self::classify_rule(&rule_name);

            let pattern = Self::rule_to_pattern(&rule_name);
            let suggestion = Self::rule_to_suggestion(&rule_name);
            
            rules.push(ImportedContract {
                id: format!("ESLINT_{}", rule_name.to_uppercase().replace('-', "_")),
                source: ContractSource::ESLint,
                name: rule_name.clone(),
                domain,
                severity,
                description: format!("ESLint rule: {}", rule_name),
                pattern,
                suggestion,
                references: vec![
                    format!("https://eslint.org/docs/latest/rules/{}", rule_name)
                ],
                tags: vec!["javascript".into(), "eslint".into(), "typescript".into()],
            });
        }

        rules
    }

    /// Classify ESLint rule by name patterns
    fn classify_rule(rule_name: &str) -> (String, Severity) {
        // Security-related rules
        let security_rules = [
            "no-eval", "no-implied-eval", "no-new-func", "no-script-url",
            "no-extend-native", "no-proto", "no-unsafe-negation",
            "no-unsafe-optional-chaining", "no-unsafe-finally",
        ];

        // Correctness rules (potential bugs)
        let correctness_rules = [
            "no-cond-assign", "no-unreachable", "no-fallthrough", "no-dupe-keys",
            "no-dupe-args", "no-duplicate-case", "no-empty", "no-ex-assign",
            "no-func-assign", "no-obj-calls", "no-undef", "no-unreachable-loop",
            "no-const-assign", "no-debugger", "no-dupe-else-if", "no-self-assign",
            "no-unused-vars", "use-isnan", "valid-typeof", "no-compare-neg-zero",
            "no-constant-condition", "no-control-regex", "no-empty-character-class",
            "no-invalid-regexp", "no-irregular-whitespace", "no-loss-of-precision",
            "no-misleading-character-class", "no-regex-spaces", "no-unsafe-negation",
            "require-yield", "getter-return", "no-setter-return", "no-case-declarations",
            "no-class-assign", "no-delete-var", "no-empty-pattern", "no-global-assign",
            "no-import-assign", "no-inner-declarations", "no-new-native-nonconstructor",
            "no-nonoctal-decimal-escape", "no-octal", "no-octal-escape", "no-redeclare",
            "no-self-compare", "no-undef-init", "no-useless-backreference",
        ];

        // Style rules
        let style_rules = [
            "no-var", "prefer-const", "prefer-arrow-callback", "object-shorthand",
            "quote-props", "quotes", "semi", "comma-dangle", "indent",
            "no-multiple-empty-lines", "no-trailing-spaces", "eol-last",
            "arrow-parens", "arrow-spacing", "block-spacing", "brace-style",
            "camelcase", "capitalized-comments", "comma-spacing", "comma-style",
            "computed-property-spacing", "consistent-this", "dot-location",
            "dot-notation", "func-call-spacing", "func-name-matching", "func-names",
            "func-style", "id-blacklist", "id-denylist", "id-length", "id-match",
            "implicit-arrow-linebreak", "jsx-quotes", "key-spacing", "keyword-spacing",
            "line-comment-position", "linebreak-style", "lines-around-comment",
            "lines-between-class-members", "max-depth", "max-len", "max-lines",
            "max-lines-per-function", "max-nested-callbacks", "max-params",
            "max-statements", "max-statements-per-line", "multiline-comment-style",
            "multiline-ternary", "new-cap", "new-parens", "newline-per-chained-call",
            "no-array-constructor", "no-bitwise", "no-continue", "no-inline-comments",
            "no-lonely-if", "no-mixed-operators", "no-multi-spaces", "no-negated-condition",
            "no-nested-ternary", "no-new-object", "no-plusplus", "no-tabs",
            "no-ternary", "no-underscore-dangle", "no-whitespace-before-property",
            "nonblock-statement-body-position", "object-curly-newline",
            "object-curly-spacing", "object-property-newline", "one-var",
            "one-var-declaration-per-line", "operator-assignment", "operator-linebreak",
            "padded-blocks", "padding-line-between-statements", "prefer-exponentiation-operator",
            "prefer-named-capture-group", "prefer-numeric-literals", "prefer-object-has-own",
            "prefer-object-spread", "prefer-template", "sort-imports", "sort-keys",
            "sort-vars", "space-before-blocks", "space-before-function-paren",
            "space-in-parens", "space-infix-ops", "space-unary-ops", "spaced-comment",
            "switch-colon-spacing", "template-curly-spacing", "template-tag-spacing",
            "unicode-bom", "wrap-iife", "wrap-regex", "yield-star-spacing", "yoda",
        ];

        // Best practices
        let best_practice_rules = [
            "array-callback-return", "block-scoped-var", "class-methods-use-this",
            "complexity", "consistent-return", "curly", "default-case",
            "default-case-last", "default-param-last", "dot-notation",
            "eqeqeq", "for-direction", "func-style", "guard-for-in",
            "max-classes-per-file", "no-alert", "no-caller", "no-constructor-return",
            "no-else-return", "no-empty-function", "no-eq-null", "no-eval",
            "no-extend-native", "no-extra-bind", "no-extra-label", "no-floating-decimal",
            "no-implicit-coercion", "no-implicit-globals", "no-implied-eval",
            "no-invalid-this", "no-iterator", "no-labels", "no-lone-blocks",
            "no-loop-func", "no-magic-numbers", "no-multi-assign", "no-multi-str",
            "no-new", "no-new-func", "no-new-wrappers", "no-param-reassign",
            "no-proto", "no-restricted-exports", "no-restricted-globals",
            "no-restricted-imports", "no-restricted-properties", "no-restricted-syntax",
            "no-return-assign", "no-return-await", "no-script-url", "no-sequences",
            "no-throw-literal", "no-unmodified-loop-condition", "no-unused-expressions",
            "no-unused-labels", "no-useless-assignment", "no-useless-call",
            "no-useless-catch", "no-useless-concat", "no-useless-constructor",
            "no-useless-escape", "no-useless-rename", "no-useless-return", "no-void",
            "no-warning-comments", "prefer-promise-reject-errors", "prefer-regex-literals",
            "prefer-rest-params", "prefer-spread", "radix", "require-await",
            "require-unicode-regexp", "strict", "symbol-description",
        ];

        // Performance rules
        let performance_rules = [
            "no-await-in-loop", "no-async-promise-executor", "require-atomic-updates",
        ];

        if security_rules.contains(&rule_name) {
            return ("security".into(), Severity::Critical);
        }
        if correctness_rules.contains(&rule_name) {
            return ("correctness".into(), Severity::Error);
        }
        if style_rules.contains(&rule_name) {
            return ("style".into(), Severity::Warning);
        }
        if best_practice_rules.contains(&rule_name) {
            return ("best-practices".into(), Severity::Warning);
        }
        if performance_rules.contains(&rule_name) {
            return ("performance".into(), Severity::Warning);
        }

        // Default classification based on name patterns
        if rule_name.starts_with("no-") {
            ("restriction".into(), Severity::Info)
        } else if rule_name.starts_with("prefer-") {
            ("style".into(), Severity::Info)
        } else {
            ("miscellaneous".into(), Severity::Info)
        }
    }

    /// Convert rule name to a regex pattern that matches the code
    fn rule_to_pattern(rule_name: &str) -> Option<String> {
        // Map common rule names to actual regex patterns
        let patterns: &[(&str, &str)] = &[
            // Security
            ("no-eval", r"eval\s*\("),
            ("no-implied-eval", r#"setTimeout\s*\([^)]*['"]|setInterval\s*\([^)]*['"]"#),
            ("no-new-func", r"new\s+Function\s*\("),
            ("no-script-url", r"javascript\s*:"),
            ("no-proto", r"__proto__"),
            ("no-extend-native", r"Object\.prototype\.\w+\s*="),
            ("no-unsafe-negation", r"!\s*\["),
            ("no-unsafe-optional-chaining", r"\?\.\s*\["),
            ("no-unsafe-finally", r"finally\s*\{[^}]*\b(throw|return|break|continue)\b"),
            
            // Correctness
            ("no-cond-assign", r"if\s*\([^)]*=[^=]"),
            ("no-unreachable", r"return[^;]*;[^}]*\w"),
            ("no-fallthrough", r"case[^:]+:[^}]*\w"),
            ("no-dupe-keys", r"\{[^}]*\b(\w+)\s*:.*\b\1\s*:"),
            ("no-empty", r"if\s*\([^)]*\)\s*\{\s*\}"),
            ("no-ex-assign", r"catch\s*\(\s*(\w+)\s*\)[^}]*\1\s*="),
            ("no-func-assign", r"function\s+(\w+)[^}]*\1\s*="),
            ("no-obj-calls", r"Math\s*\(|JSON\s*\("),
            ("no-undef", r"\b\w+\s*[^\w\s]"), // Generic - needs context
            ("no-unused-vars", r"(let|const|var)\s+(\w+)[^;]*;"),
            ("no-const-assign", r"const\s+(\w+)[^}]*\1\s*="),
            ("no-debugger", r"debugger\b"),
            ("no-self-assign", r"(\w+)\s*=\s*\1\b"),
            ("use-isnan", r"\w+\s*(==|===|!=|!==)\s*NaN"),
            ("valid-typeof", r#"typeof\s+\w+\s*(==|===|!=|!==)\s*['"]"#),
            ("no-compare-neg-zero", r#"(==|===)\s*-0|-0\s*(==|===)"#),
            ("getter-return", r#"get\s+\w+\s*\(\s*\)[^}]*\{[^}]*\}"#),
            ("no-case-declarations", r"case[^:]+:\s*(let|const)"),
            
            // Style
            ("no-var", r"\bvar\s+"),
            ("prefer-const", r"let\s+(\w+)\s*=[^;]*;[^}]*\1\s*="),
            ("prefer-arrow-callback", r"function\s*\([^)]*\)\s*\{"),
            ("object-shorthand", r"\w+\s*:\s*\w+"),
            ("semi", r"[^;\s]\s*\n"),
            ("quotes", r#"['"]"#),
            ("indent", r"^\s*"),
            ("comma-dangle", r",\s*[\]\}]"),
            ("arrow-parens", r"=>\s*\{"),
            ("camelcase", r"[a-z][A-Z]"),
            ("eqeqeq", r"\s(==|!=)\s"),
            ("dot-notation", r#"\['[a-zA-Z_][a-zA-Z0-9_]*'\]"#),
            ("curly", r"if\s*\([^)]*\)\s*[^{]"),
            ("brace-style", r"\}\s*else"),
            ("no-multiple-empty-lines", r"\n\s*\n\s*\n"),
            ("no-trailing-spaces", r"[ \t]+\n"),
            
            // Best practices
            ("array-callback-return", r"\.map\s*\([^)]*\)[^;]*;"),
            ("block-scoped-var", r"var\s+\w+[^;]*\{[^}]*\w+"),
            ("consistent-return", r"function[^}]*return[^}]*return"),
            ("default-case", r"switch[^}]*\}"),
            ("eqeqeq", r"==|!="),
            ("guard-for-in", r"for\s*\(\s*\w+\s+in\s+"),
            ("no-alert", r"alert\s*\(|confirm\s*\(|prompt\s*\("),
            ("no-caller", r"arguments\.callee|arguments\.caller"),
            ("no-else-return", r"else\s*\{\s*return"),
            ("no-empty-function", r"function\s*\([^)]*\)\s*\{\s*\}"),
            ("no-eq-null", r"==\s*null|null\s*==|!=\s*null|null\s*!="),
            ("no-implicit-coercion", r"!!\w+|String\(|Number\(|Boolean\("),
            ("no-implicit-globals", r"^\s*(let|const|var)\s+\w+[^=]*="),
            ("no-invalid-this", r"this\s*\."), // Context dependent
            ("no-iterator", r"__iterator__"),
            ("no-labels", r"^\s*\w+\s*:"),
            ("no-lone-blocks", r"\{\s*\}"),
            ("no-loop-func", r"for[^{]*\{[^}]*function"),
            ("no-magic-numbers", r"\b\d{2,}\b"),
            ("no-multi-assign", r"(\w+)\s*=\s*\w+\s*=\s*"),
            ("no-new", r"new\s+\w+\(\s*\)[^.;]"),
            ("no-new-wrappers", r"new\s+(String|Number|Boolean)\s*\("),
            ("no-param-reassign", r"function[^{]*\((\w+)\)[^}]*\1\s*="),
            ("no-restricted-globals", r"\b\w+\b"), // Generic
            ("no-return-assign", r"return\s+\w+\s*="),
            ("no-return-await", r"return\s+await"),
            ("no-sequences", r",\s*,"),
            ("no-throw-literal", r#"throw\s+['"]|throw\s+\d"#),
            ("no-unused-expressions", r"\w+\s*;"),
            ("no-useless-concat", r#"['"]\s*\+\s*['"]"#),
            ("no-useless-constructor", r"constructor\s*\(\s*\)\s*\{\s*\}"),
            ("no-void", r"void\s+"),
            ("prefer-promise-reject-errors", r"Promise\.reject\s*\([^E]"),
            ("prefer-rest-params", r"arguments"),
            ("prefer-spread", r"\.apply\s*\("),
            ("radix", r"parseInt\s*\([^,)]+\)"),
            ("require-await", r"async\s+function[^}]*\{[^}]*\}"),
            
            // Performance
            ("no-await-in-loop", r"for[^{]*\{[^}]*await"),
            ("no-async-promise-executor", r"new\s+Promise\s*\(\s*async"),
        ];
        
        // Find matching pattern
        for (rule, pattern) in patterns {
            if rule_name == *rule {
                return Some(pattern.to_string());
            }
        }
        
        // Generate pattern from rule name heuristics
        if rule_name.starts_with("no-") {
            // Extract what's being forbidden
            let forbidden = rule_name.strip_prefix("no-").unwrap_or("");
            if !forbidden.is_empty() {
                // Convert kebab-case to potential code pattern
                let _pattern_str = forbidden.replace('-', " ");
                return Some(forbidden.to_string());
            }
        }
        if rule_name.starts_with("prefer-") {
            let preferred = rule_name.strip_prefix("prefer-").unwrap_or("");
            if !preferred.is_empty() {
                return None; // Prefer rules are usually about style, not patterns
            }
        }
        
        None
    }

    /// Generate suggestion text for a rule
    fn rule_to_suggestion(rule_name: &str) -> Option<String> {
        let suggestions: &[(&str, &str)] = &[
            ("no-eval", "Avoid eval() - use safer alternatives like JSON.parse or Function constructor"),
            ("no-implied-eval", "Pass function reference to setTimeout/setInterval, not a string"),
            ("no-new-func", "Avoid dynamic code generation - use static functions"),
            ("no-script-url", "Avoid javascript: URLs - use event handlers instead"),
            ("no-var", "Use let or const for block scoping"),
            ("eqeqeq", "Use === for strict equality comparison"),
            ("prefer-const", "Use const for variables that don't get reassigned"),
            ("prefer-arrow-callback", "Use arrow functions for callbacks"),
            ("no-debugger", "Remove debugger statement before committing"),
            ("no-console", "Remove console statements before committing"),
            ("no-alert", "Use proper UI notifications instead of alerts"),
            ("semi", "Add semicolon at the end of statements"),
            ("quotes", "Use consistent quote style throughout the codebase"),
            ("indent", "Use consistent indentation"),
            ("camelcase", "Use camelCase for variable and function names"),
            ("no-unused-vars", "Remove unused variables or prefix with underscore if intentional"),
            ("no-param-reassign", "Avoid reassigning function parameters - create local copies"),
            ("no-return-await", "Remove unnecessary await in return statement"),
            ("require-await", "Add await inside async function or remove async keyword"),
            ("no-async-promise-executor", "Return a Promise instead of using async executor"),
        ];
        
        for (rule, suggestion) in suggestions {
            if rule_name == *rule {
                return Some(suggestion.to_string());
            }
        }
        
        None
    }

    /// Built-in ESLint rules as fallback (network failure)
    fn builtin_rules() -> Vec<ImportedContract> {
        vec![
            ImportedContract {
                id: "ESLINT_NO_COND_ASSIGN".into(),
                source: ContractSource::ESLint,
                name: "no-cond-assign".into(),
                domain: "correctness".into(),
                severity: Severity::Error,
                description: "Disallow assignment in conditional expressions".into(),
                pattern: Some(r"if\s*\([^)]*=[^=]".into()),
                suggestion: Some("Use == or === for comparison, not =".into()),
                references: vec!["https://eslint.org/docs/latest/rules/no-cond-assign".into()],
                tags: vec!["javascript".into(), "bug-risk".into()],
            },
            ImportedContract {
                id: "ESLINT_NO_UNREACHABLE".into(),
                source: ContractSource::ESLint,
                name: "no-unreachable".into(),
                domain: "correctness".into(),
                severity: Severity::Error,
                description: "Disallow unreachable code".into(),
                pattern: Some("return".into()),
                suggestion: Some("Remove code after return/throw/break".into()),
                references: vec!["https://eslint.org/docs/latest/rules/no-unreachable".into()],
                tags: vec!["javascript".into(), "dead-code".into()],
            },
            ImportedContract {
                id: "ESLINT_NO_EVAL".into(),
                source: ContractSource::ESLint,
                name: "no-eval".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Disallow eval() - code injection risk".into(),
                pattern: Some("eval(".into()),
                suggestion: Some("Avoid eval() - use safer alternatives".into()),
                references: vec!["https://eslint.org/docs/latest/rules/no-eval".into()],
                tags: vec!["javascript".into(), "security".into(), "injection".into()],
            },
            ImportedContract {
                id: "ESLINT_NO_IMPLICIT_EVAL".into(),
                source: ContractSource::ESLint,
                name: "no-implied-eval".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Disallow implied eval() via setTimeout/setInterval".into(),
                pattern: Some("setTimeout(".into()),
                suggestion: Some("Pass function reference, not string".into()),
                references: vec!["https://eslint.org/docs/latest/rules/no-implied-eval".into()],
                tags: vec!["javascript".into(), "security".into(), "injection".into()],
            },
            ImportedContract {
                id: "ESLINT_NO_VAR".into(),
                source: ContractSource::ESLint,
                name: "no-var".into(),
                domain: "style".into(),
                severity: Severity::Warning,
                description: "Require let/const instead of var".into(),
                pattern: Some("var ".into()),
                suggestion: Some("Use let or const for block scoping".into()),
                references: vec!["https://eslint.org/docs/latest/rules/no-var".into()],
                tags: vec!["javascript".into(), "modern".into()],
            },
            ImportedContract {
                id: "ESLINT_EQEQEQ".into(),
                source: ContractSource::ESLint,
                name: "eqeqeq".into(),
                domain: "correctness".into(),
                severity: Severity::Warning,
                description: "Require === instead of ==".into(),
                pattern: Some(" == ".into()),
                suggestion: Some("Use === for strict equality".into()),
                references: vec!["https://eslint.org/docs/latest/rules/eqeqeq".into()],
                tags: vec!["javascript".into(), "bug-risk".into()],
            },
            ImportedContract {
                id: "ESLINT_NO_NEW_FUNC".into(),
                source: ContractSource::ESLint,
                name: "no-new-func".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Disallow new Function() - similar to eval()".into(),
                pattern: Some("new Function(".into()),
                suggestion: Some("Avoid dynamic code generation".into()),
                references: vec!["https://eslint.org/docs/latest/rules/no-new-func".into()],
                tags: vec!["javascript".into(), "security".into(), "injection".into()],
            },
        ]
    }
}

impl Default for ESLintImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Importer for ESLintImporter {
    fn source(&self) -> ContractSource {
        ContractSource::ESLint
    }

    fn source_url(&self) -> Option<&str> {
        Some("https://eslint.org/docs/latest/rules/")
    }

    async fn import(&self) -> Result<Vec<ImportedContract>> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 500;

        // Try fetching from GitHub raw content
        let url = "https://raw.githubusercontent.com/eslint/eslint/main/lib/rules/index.js";

        for attempt in 0..MAX_RETRIES {
            match self.client
                .get(url)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    match response.text().await {
                        Ok(content) => {
                            let rules = self.parse_rules_index(&content);
                            if !rules.is_empty() {
                                eprintln!(
                                    "[INFO] ESLintImporter: Successfully imported {} rules from GitHub",
                                    rules.len()
                                );
                                return Ok(rules);
                            }
                            eprintln!(
                                "[WARN] ESLintImporter: Content parsed but no rules found, attempt {}/{}",
                                attempt + 1, MAX_RETRIES
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "[WARN] ESLintImporter: Failed to read response: {}, attempt {}/{}",
                                e, attempt + 1, MAX_RETRIES
                            );
                        }
                    }
                }
                Ok(response) => {
                    eprintln!(
                        "[WARN] ESLintImporter: HTTP error {}: attempt {}/{}",
                        response.status(), attempt + 1, MAX_RETRIES
                    );
                }
                Err(e) => {
                    eprintln!(
                        "[WARN] ESLintImporter: Network error: {}, attempt {}/{}",
                        e, attempt + 1, MAX_RETRIES
                    );
                }
            }

            if attempt + 1 < MAX_RETRIES {
                tokio::time::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS)).await;
            }
        }

        eprintln!("[WARN] ESLintImporter: All network attempts failed, falling back to built-in rules");
        Ok(Self::builtin_rules())
    }
}
