//! Pylint Check Importer
//!
//! Imports Pylint checks from the official messages list.
//! Source: https://github.com/grovina/pylint-messages (pylint-messages.json)
//! Parses the JSON to extract all ~300+ message codes.

use crate::{ImportedContract, ContractSource, Severity, Importer};
use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;

/// Pylint message JSON format from grovina/pylint-messages
type PylintMessagesJson = HashMap<String, String>;

pub struct PylintImporter {
    client: Client,
}

impl PylintImporter {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("synward-contract-importer/1.0")
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Convert Pylint message name to a regex pattern that matches the code
    fn message_to_pattern(name: &str) -> Option<String> {
        // Map common Pylint messages to regex patterns
        let patterns = [
            // Security
            ("eval-used", r#"eval\s*\("#),
            ("exec-used", r#"exec\s*\("#),
            ("hardcoded-password", r#"password\s*=\s*['"]"#),
            ("hardcoded-sql-expression", r#"['"]\s*(SELECT|INSERT|UPDATE|DELETE|DROP)"#),
            ("shell-injection", r#"subprocess\.(call|run|Popen)\s*\([^)]*shell\s*=\s*True"#),
            ("pickle-insecure", r#"pickle\.(load|loads)\("#),
            ("marshal-insecure", r#"marshal\.(load|loads)\("#),
            ("tempfile-insecure", r#"tempfile\.mktemp"#),
            ("unsafe-load", r#"yaml\.(load|unsafe_load)\("#),
            
            // Correctness
            ("no-member", r#"\.\w+\s*\("#),  // Method call
            ("undefined-variable", r#"\b\w+\s*(?!=)"#),  // Variable used before definition
            ("not-callable", r#"\w+\s*\(\s*\)"#),  // Calling non-callable
            ("too-many-function-args", r#"\w+\s*\([^)]{50,}\)"#),  // Too many args
            ("unsubscriptable-object", r#"\w+\s*\[[^\]]+\]"#),  // Indexing non-subscriptable
            ("unsupported-membership-test", r#"\w+\s+(not\s+)?in\s+\w+"#),  // 'in' check
            ("not-an-iterable", r#"(for|in)\s+\w+"#),  // Iterating non-iterable
            
            // Style
            ("invalid-name", r#"\b[a-z]\s*="#),  // Single letter variable
            ("bad-whitespace", r#",\S|=\S|\S="#),  // Missing whitespace
            ("missing-docstring", r#"def\s+\w+\s*\([^)]*\)\s*:"#),  // Function without docstring
            ("empty-docstring", r#"\"\"\"\"\"\""#),  // Empty docstring
            ("trailing-whitespace", r#"[ \t]+$"#),  // Trailing whitespace
            ("line-too-long", r#".{120,}"#),  // Long line
            
            // Error handling
            ("broad-exception-caught", r#"except\s*:"#),  // Bare except
            ("broad-exception-raised", r#"raise\s+Exception"#),  // Generic raise
            ("raise-missing-from", r#"raise\s+\w+Error\s*\("#),  // Missing 'from'
            ("try-except-raise", r#"except\s+\w+:\s*raise\s+\w+"#),  // Re-raise same
            
            // Performance
            ("use-a-generator", r#"list\s*\(\s*\[#),  // list([...]) instead of [...]
            ("unnecessary-list-comprehension", r#"\[\s*\w+\s+for\s+"#),  // Simple comprehension
            ("consider-using-join", r#"\+\s*['"]"#),  // String concat in loop
            ("consider-using-dict-comprehension", r#"dict\s*\(\s*\["#),  // dict([...])
            
            // Complexity
            ("too-many-branches", r#"if\s+.*:\s*(elif|else)"#),  // Many branches
            ("too-many-statements", r#"def\s+\w+\s*\([^)]*\)\s*:[^}]{500,}"#),  // Long function
            ("too-many-locals", r#"def\s+\w+\s*\([^)]*\)"#),  // Many locals
            ("too-many-arguments", r#"def\s+\w+\s*\([^)]{50,}\)"#),  // Many args
            ("too-many-nested-blocks", r#"if\s+.*:\s*if\s+"#),  // Nested if
            ("too-few-public-methods", r#"class\s+\w+\s*\:"#),  // Class with few methods
            ("too-many-public-methods", r#"class\s+\w+\s*\:"#),  // Class with many methods
            
            // Type hints
            ("missing-type-doc", r#"def\s+\w+\s*\([^)]*\)\s*->\s*\w+"#),  // Missing return type
            ("missing-param-doc", r#"def\s+\w+\s*\("#),  // Missing param type
            ("missing-return-type-doc", r#"def\s+\w+\s*\([^)]*\)\s*:"#),  // No return annotation
            ("useless-type-doc", r#"type:\s*ignore"#),  // Useless type: ignore
            
            // Best practices
            ("unused-argument", r#"def\s+\w+\s*\([^)]*\)"#),  // Unused param
            ("unused-import", r#"^import\s+\w+"#),  // Unused import
            ("unused-variable", r#"^\s*\w+\s*="#),  // Unused variable
            ("redefined-outer-name", r#"^\s*\w+\s*="#),  // Shadowing
            ("redefined-builtin", r#"def\s+\w+\s*\("#),  // Redefining builtin
            ("global-variable-not-assigned", r#"global\s+\w+"#),  // Global read-only
            ("global-statement", r#"global\s+\w+"#),  // Global keyword
            ("deprecated-method", r#"\.\w+\s*\("#),  // Deprecated method
            
            // Format
            ("bad-indentation", r#"^\t"#),  // Tab indentation
            ("mixed-indentation", r#"^[ \t]+"#),  // Mixed spaces/tabs
            ("superfluous-parens", r#"\(\s*\w+\s*\)"#),  // Unnecessary parens
            ("unnecessary-semicolon", r#";\s*$"#),  // Unnecessary semicolon
            ("multiple-statements", r#";\s*\w+"#),  // Multiple statements on line
            
            // Import
            ("wrong-import-order", r#"^import\s+"#),  // Wrong import order
            ("wrong-import-position", r#"^import\s+"#),  // Import after code
            ("reimported", r#"^import\s+"#),  // Reimported module
            ("import-self", r#"^import\s+"#),  // Self-import
            ("cyclic-import", r#"^import\s+"#),  // Cyclic import
            
            // Design
            ("abstract-method", r#"def\s+\w+\s*\([^)]*\)\s*:\s*(pass|\.\.\.)"#),  // Not implemented
            ("interface-not-implemented", r#"class\s+\w+\s*\:"#),  // Missing interface impl
            ("attribute-defined-outside-init", r#"self\.\w+\s*="#),  // Attr outside __init__
            
            // Similarity
            ("duplicate-code", r#"[^}]{50,}"#),  // Duplicate code blocks
            
            // Exceptions
            ("raising-non-exception", r#"raise\s+"#),  // Raising non-exception
            ("notimplemented-raised", r#"raise\s+NotImplementedError"#),  // NotImplemented
            ("bad-format-string", r#"['"][^'"]*%[^'"]*['"]"#),  // Bad format string
            ("format-needs-mapping", r#"['"][^'"]*\{[^}]+\}[^'"]*['"]"#),  // Format without mapping
        ];

        for (msg_name, pattern) in patterns {
            if name == msg_name {
                return Some(pattern.to_string());
            }
        }

        // Try fuzzy matching for messages not in the exact list
        if name.contains("eval") || name.contains("exec") {
            return Some(r#"eval\s*\(|exec\s*\("#.to_string());
        }
        if name.contains("password") || name.contains("secret") || name.contains("token") {
            return Some(r#"password\s*=\s*['"]|secret\s*=\s*['"]|token\s*=\s*['"]"#.to_string());
        }
        if name.contains("sql") || name.contains("query") {
            return Some(r#"['"]\s*(SELECT|INSERT|UPDATE|DELETE|DROP)"#.to_string());
        }
        if name.contains("import") && name.contains("unused") {
            return Some(r#"^import\s+\w+|^from\s+\w+\s+import"#.to_string());
        }
        if name.contains("unused") {
            return Some(r#"^\s*\w+\s*="#.to_string());
        }
        if name.contains("todo") || name.contains("fixme") {
            return Some(r#"#?\s*(TODO|FIXME|XXX|HACK)"#.to_string());
        }
        if name.contains("deprecated") {
            return Some(r#"@deprecated|deprecated\s*"#.to_string());
        }

        None
    }

    /// Get suggestion for fixing a Pylint message
    fn message_to_suggestion(name: &str) -> Option<String> {
        let suggestions = [
            ("eval-used", "Avoid eval() - use ast.literal_eval() or safer alternatives"),
            ("exec-used", "Avoid exec() - use safer alternatives like subprocess"),
            ("hardcoded-password", "Use environment variables or secure vault for secrets"),
            ("shell-injection", "Use subprocess without shell=True and pass args as list"),
            ("pickle-insecure", "Use json or other safe serialization formats"),
            ("broad-exception-caught", "Catch specific exceptions instead of bare except"),
            ("broad-exception-raised", "Raise specific exception types"),
            ("missing-docstring", "Add a docstring describing the function/class purpose"),
            ("invalid-name", "Use descriptive names following PEP 8 naming conventions"),
            ("line-too-long", "Break long lines or use parentheses for implicit line continuation"),
            ("global-statement", "Avoid global state - use function parameters or class attributes"),
            ("too-many-arguments", "Refactor to use fewer arguments or use config objects"),
            ("too-many-branches", "Extract branches into separate functions or use polymorphism"),
            ("too-many-statements", "Break down large functions into smaller ones"),
            ("abstract-method", "Implement the abstract method or mark class as abstract"),
        ];

        for (msg_name, suggestion) in suggestions {
            if name == msg_name {
                return Some(suggestion.to_string());
            }
        }

        None
    }

    /// Parse Pylint messages from JSON
    /// Format: {"E1101": "no-member", "W0123": "eval-used", ...}
    fn parse_messages_json(&self, json: &str) -> Result<Vec<ImportedContract>> {
        let messages: PylintMessagesJson = serde_json::from_str(json)?;
        let mut contracts = Vec::new();

        for (code, name) in messages {
            let (domain, severity) = Self::classify_message(&code, &name);
            let pattern = Self::message_to_pattern(&name);
            let suggestion = Self::message_to_suggestion(&name);

            contracts.push(ImportedContract {
                id: format!("PYLINT_{}", code),
                source: ContractSource::Pylint,
                name: name.clone(),
                domain,
                severity,
                description: format!("Pylint message: {} ({})", name, code),
                pattern,
                suggestion,
                references: vec![
                    format!(
                        "https://pylint.readthedocs.io/en/latest/user_guide/messages/{}/{}.html",
                        Self::get_message_category(&code),
                        name
                    )
                ],
                tags: vec!["python".into(), "pylint".into()],
            });
        }

        // Sort by code for consistent output
        contracts.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(contracts)
    }

    /// Get message category from code prefix
    fn get_message_category(code: &str) -> &'static str {
        match code.chars().next() {
            Some('E') => "error",
            Some('W') => "warning",
            Some('C') => "convention",
            Some('R') => "refactor",
            Some('F') => "fatal",
            Some('I') => "info",
            _ => "warning",
        }
    }

    /// Classify Pylint message by code and name
    fn classify_message(code: &str, name: &str) -> (String, Severity) {
        // Security-related message names
        let security_names = [
            "eval-used", "exec-used", "shell-injection", "hardcoded-password",
            "hardcoded-sql-expression", "unsafe-load", "tempfile-insecure",
            "pickle-insecure", "marshal-insecure", "assert-on-string-literal",
        ];

        // Check for security keywords in name
        if security_names.iter().any(|s| name.contains(s)) {
            return ("security".into(), Severity::Critical);
        }

        // Classify by code prefix
        match code.chars().next() {
            Some('E') => {
                // Errors - correctness issues
                ("correctness".into(), Severity::Error)
            }
            Some('F') => {
                // Fatal - critical issues
                ("fatal".into(), Severity::Critical)
            }
            Some('W') => {
                // Warnings - potential issues
                if name.contains("dangerous") || name.contains("deprecated") {
                    ("correctness".into(), Severity::Warning)
                } else {
                    ("warning".into(), Severity::Warning)
                }
            }
            Some('C') => {
                // Convention - style issues
                ("style".into(), Severity::Info)
            }
            Some('R') => {
                // Refactor - code quality
                ("refactor".into(), Severity::Hint)
            }
            Some('I') => {
                // Info
                ("info".into(), Severity::Hint)
            }
            _ => {
                // Unknown
                ("miscellaneous".into(), Severity::Info)
            }
        }
    }

    /// Built-in Pylint checks as fallback (network failure)
    fn builtin_checks() -> Vec<ImportedContract> {
        vec![
            ImportedContract {
                id: "PYLINT_E1101".into(),
                source: ContractSource::Pylint,
                name: "no-member".into(),
                domain: "correctness".into(),
                severity: Severity::Error,
                description: "Accessing non-existent member".into(),
                pattern: Some(".".into()),
                suggestion: Some("Check that the attribute/method exists".into()),
                references: vec!["https://pylint.readthedocs.io/en/latest/user_guide/messages/error/no-member.html".into()],
                tags: vec!["python".into(), "attribute".into()],
            },
            ImportedContract {
                id: "PYLINT_W0123".into(),
                source: ContractSource::Pylint,
                name: "eval-used".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Use of eval() detected".into(),
                pattern: Some("eval(".into()),
                suggestion: Some("Use ast.literal_eval() for safe evaluation".into()),
                references: vec!["https://pylint.readthedocs.io/en/latest/user_guide/messages/warning/eval-used.html".into()],
                tags: vec!["python".into(), "security".into(), "injection".into()],
            },
            ImportedContract {
                id: "PYLINT_W0122".into(),
                source: ContractSource::Pylint,
                name: "exec-used".into(),
                domain: "security".into(),
                severity: Severity::Critical,
                description: "Use of exec() detected".into(),
                pattern: Some("exec(".into()),
                suggestion: Some("Avoid exec() - dangerous with user input".into()),
                references: vec!["https://pylint.readthedocs.io/en/latest/user_guide/messages/warning/exec-used.html".into()],
                tags: vec!["python".into(), "security".into(), "injection".into()],
            },
            ImportedContract {
                id: "PYLINT_W0611".into(),
                source: ContractSource::Pylint,
                name: "unused-import".into(),
                domain: "style".into(),
                severity: Severity::Warning,
                description: "Unused import".into(),
                pattern: Some("import ".into()),
                suggestion: Some("Remove unused imports".into()),
                references: vec!["https://pylint.readthedocs.io/en/latest/user_guide/messages/warning/unused-import.html".into()],
                tags: vec!["python".into(), "cleanup".into()],
            },
            ImportedContract {
                id: "PYLINT_W0612".into(),
                source: ContractSource::Pylint,
                name: "unused-variable".into(),
                domain: "style".into(),
                severity: Severity::Warning,
                description: "Unused variable".into(),
                pattern: None,
                suggestion: Some("Remove or use the variable".into()),
                references: vec!["https://pylint.readthedocs.io/en/latest/user_guide/messages/warning/unused-variable.html".into()],
                tags: vec!["python".into(), "cleanup".into()],
            },
            ImportedContract {
                id: "PYLINT_C0103".into(),
                source: ContractSource::Pylint,
                name: "invalid-name".into(),
                domain: "style".into(),
                severity: Severity::Info,
                description: "Name doesn't match naming convention".into(),
                pattern: None,
                suggestion: Some("Use snake_case for variables/functions, PascalCase for classes".into()),
                references: vec!["https://pylint.readthedocs.io/en/latest/user_guide/messages/convention/invalid-name.html".into()],
                tags: vec!["python".into(), "naming".into()],
            },
            ImportedContract {
                id: "PYLINT_R0913".into(),
                source: ContractSource::Pylint,
                name: "too-many-arguments".into(),
                domain: "design".into(),
                severity: Severity::Info,
                description: "Function has too many arguments (>5)".into(),
                pattern: None,
                suggestion: Some("Use config object or builder pattern".into()),
                references: vec!["https://pylint.readthedocs.io/en/latest/user_guide/messages/refactor/too-many-arguments.html".into()],
                tags: vec!["python".into(), "design".into()],
            },
            ImportedContract {
                id: "PYLINT_W0102".into(),
                source: ContractSource::Pylint,
                name: "dangerous-default-value".into(),
                domain: "correctness".into(),
                severity: Severity::Warning,
                description: "Mutable default argument".into(),
                pattern: Some("=[]".into()),
                suggestion: Some("Use None and create inside function".into()),
                references: vec!["https://pylint.readthedocs.io/en/latest/user_guide/messages/warning/dangerous-default-value.html".into()],
                tags: vec!["python".into(), "bug-risk".into(), "mutable-default".into()],
            },
        ]
    }
}

impl Default for PylintImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Importer for PylintImporter {
    fn source(&self) -> ContractSource {
        ContractSource::Pylint
    }

    fn source_url(&self) -> Option<&str> {
        Some("https://pylint.readthedocs.io/en/latest/user_guide/messages/messages_list.html")
    }

    async fn import(&self) -> Result<Vec<ImportedContract>> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 500;

        // Try fetching from GitHub - grovina/pylint-messages
        let url = "https://raw.githubusercontent.com/grovina/pylint-messages/master/pylint-messages.json";

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
                            match self.parse_messages_json(&content) {
                                Ok(messages) if !messages.is_empty() => {
                                    eprintln!(
                                        "[INFO] PylintImporter: Successfully imported {} messages from GitHub",
                                        messages.len()
                                    );
                                    return Ok(messages);
                                }
                                Ok(_) => {
                                    eprintln!(
                                        "[WARN] PylintImporter: JSON parsed but no messages found, attempt {}/{}",
                                        attempt + 1, MAX_RETRIES
                                    );
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[WARN] PylintImporter: Failed to parse JSON: {}, attempt {}/{}",
                                        e, attempt + 1, MAX_RETRIES
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[WARN] PylintImporter: Failed to read response: {}, attempt {}/{}",
                                e, attempt + 1, MAX_RETRIES
                            );
                        }
                    }
                }
                Ok(response) => {
                    eprintln!(
                        "[WARN] PylintImporter: HTTP error {}: attempt {}/{}",
                        response.status(), attempt + 1, MAX_RETRIES
                    );
                }
                Err(e) => {
                    eprintln!(
                        "[WARN] PylintImporter: Network error: {}, attempt {}/{}",
                        e, attempt + 1, MAX_RETRIES
                    );
                }
            }

            if attempt + 1 < MAX_RETRIES {
                tokio::time::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS)).await;
            }
        }

        eprintln!("[WARN] PylintImporter: All network attempts failed, falling back to built-in checks");
        Ok(Self::builtin_checks())
    }
}
