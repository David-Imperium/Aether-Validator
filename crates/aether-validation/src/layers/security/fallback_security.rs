//! Fallback Security Layer — Language-agnostic security checks
//!
//! This layer provides regex-based security checks for unsupported languages.
//! When tree-sitter parsing fails or language is unknown, this layer still
//! detects critical security vulnerabilities.

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity, deduplicate_violations};

/// Fallback security layer for unsupported languages.
///
/// Uses language-agnostic regex patterns to detect:
/// - Hardcoded credentials and secrets
/// - SQL injection patterns
/// - Command injection patterns
/// - Code injection patterns
/// - Path traversal patterns
pub struct FallbackSecurityLayer {
    patterns: Vec<SecurityPattern>,
}

/// A security pattern for regex-based detection.
#[derive(Debug, Clone)]
struct SecurityPattern {
    /// Regex pattern to match
    pattern: String,
    /// Rule ID (SECxxx)
    id: String,
    /// Human-readable message
    message: String,
    /// Severity level
    severity: Severity,
    /// Fix suggestion
    suggestion: Option<String>,
}

impl FallbackSecurityLayer {
    /// Create a new fallback security layer.
    pub fn new() -> Self {
        Self {
            patterns: Self::security_patterns(),
        }
    }

    fn security_patterns() -> Vec<SecurityPattern> {
        vec![
            // === HARDCODED SECRETS ===
            SecurityPattern {
                pattern: r#"password\s*=\s*["']"#.into(),
                id: "SEC001".into(),
                message: "HARDCODED PASSWORD: Credentials must not be hardcoded".into(),
                severity: Severity::Error,
                suggestion: Some("Use environment variables or secret management".into()),
            },
            SecurityPattern {
                pattern: r#"api_key\s*=\s*["']"#.into(),
                id: "SEC002".into(),
                message: "HARDCODED API KEY: Secrets must not be in source code".into(),
                severity: Severity::Error,
                suggestion: Some("Use environment variables or secret management".into()),
            },
            SecurityPattern {
                pattern: r#"secret_key\s*=\s*["']"#.into(),
                id: "SEC003".into(),
                message: "HARDCODED SECRET KEY: Use environment variables".into(),
                severity: Severity::Error,
                suggestion: Some("Store secrets in environment variables or vault".into()),
            },
            SecurityPattern {
                pattern: r#"token\s*=\s*["']"#.into(),
                id: "SEC004".into(),
                message: "HARDCODED TOKEN: Tokens must not be in source code".into(),
                severity: Severity::Error,
                suggestion: Some("Use environment variables or secret management".into()),
            },
            SecurityPattern {
                pattern: r#"private_key\s*=\s*["']"#.into(),
                id: "SEC005".into(),
                message: "HARDCODED PRIVATE KEY: Private keys must be secured".into(),
                severity: Severity::Error,
                suggestion: Some("Use secure key storage (HSM, vault, etc.)".into()),
            },
            SecurityPattern {
                pattern: "BEGIN (RSA )?PRIVATE KEY".into(),
                id: "SEC006".into(),
                message: "EMBEDDED PRIVATE KEY: Remove embedded private keys".into(),
                severity: Severity::Error,
                suggestion: Some("Use secure key storage instead of embedding".into()),
            },
            SecurityPattern {
                pattern: r#"ADMIN_PASSWORD\s*=\s*["']"#.into(),
                id: "SEC001".into(),
                message: "HARDCODED ADMIN PASSWORD: Credentials must not be hardcoded".into(),
                severity: Severity::Error,
                suggestion: Some("Use environment variables or secret management".into()),
            },
            SecurityPattern {
                pattern: r#"API_KEY\s*=\s*["']"#.into(),
                id: "SEC002".into(),
                message: "HARDCODED API KEY: Secrets must not be in source code".into(),
                severity: Severity::Error,
                suggestion: Some("Use environment variables or secret management".into()),
            },

            // === SQL INJECTION ===
            SecurityPattern {
                pattern: r#"SELECT\s+\*\s+FROM.*\$\{"#.into(),
                id: "SEC010".into(),
                message: "SQL INJECTION: String interpolation in SQL query".into(),
                severity: Severity::Error,
                suggestion: Some("Use parameterized queries instead".into()),
            },
            SecurityPattern {
                pattern: r#"SELECT\s+\*\s+FROM.*#\{"#.into(),
                id: "SEC010".into(),
                message: "SQL INJECTION: String interpolation in SQL query".into(),
                severity: Severity::Error,
                suggestion: Some("Use parameterized queries instead".into()),
            },
            SecurityPattern {
                pattern: r#"INSERT\s+INTO.*\$\{"#.into(),
                id: "SEC010".into(),
                message: "SQL INJECTION: String interpolation in SQL query".into(),
                severity: Severity::Error,
                suggestion: Some("Use parameterized queries instead".into()),
            },
            SecurityPattern {
                pattern: r#"DELETE\s+FROM.*\$\{"#.into(),
                id: "SEC010".into(),
                message: "SQL INJECTION: String interpolation in SQL query".into(),
                severity: Severity::Error,
                suggestion: Some("Use parameterized queries instead".into()),
            },
            SecurityPattern {
                pattern: r#"UPDATE\s+.*SET.*\$\{"#.into(),
                id: "SEC010".into(),
                message: "SQL INJECTION: String interpolation in SQL query".into(),
                severity: Severity::Error,
                suggestion: Some("Use parameterized queries instead".into()),
            },

            // === COMMAND INJECTION ===
            SecurityPattern {
                pattern: r#"Runtime\.getRuntime\(\)\.exec\("#.into(),
                id: "SEC020".into(),
                message: "COMMAND INJECTION: Runtime.exec() with potential user input".into(),
                severity: Severity::Error,
                suggestion: Some("Validate and sanitize user input before execution".into()),
            },
            SecurityPattern {
                pattern: r#"system\s*\(\s*["']ls\s*\+"#.into(),
                id: "SEC020".into(),
                message: "COMMAND INJECTION: system() with string concatenation".into(),
                severity: Severity::Error,
                suggestion: Some("Use array-based command execution with validated input".into()),
            },
            SecurityPattern {
                pattern: r#"`.*\$\{.*\}.*`"#.into(),
                id: "SEC021".into(),
                message: "SHELL INJECTION: Backtick command with interpolation".into(),
                severity: Severity::Error,
                suggestion: Some("Avoid shell command interpolation".into()),
            },
            SecurityPattern {
                pattern: r#"subprocess.*shell\s*=\s*True"#.into(),
                id: "SEC180".into(),
                message: "SHELL INJECTION: subprocess with shell=True".into(),
                severity: Severity::Error,
                suggestion: Some("Use shell=False and pass arguments as list".into()),
            },
            SecurityPattern {
                pattern: r#"os\.system\s*\("#.into(),
                id: "SEC181".into(),
                message: "COMMAND INJECTION: os.system() call".into(),
                severity: Severity::Error,
                suggestion: Some("Use subprocess.run() with shell=False".into()),
            },

            // === CODE INJECTION ===
            SecurityPattern {
                pattern: r#"eval\s*\(\s*["']"#.into(),
                id: "SEC090".into(),
                message: "CODE INJECTION: eval() with string argument".into(),
                severity: Severity::Error,
                suggestion: Some("Avoid eval() - use safer alternatives".into()),
            },
            SecurityPattern {
                pattern: r#"exec\s*\(\s*["']"#.into(),
                id: "SEC091".into(),
                message: "CODE INJECTION: exec() with string argument".into(),
                severity: Severity::Error,
                suggestion: Some("Avoid exec() - use safer alternatives".into()),
            },
            SecurityPattern {
                pattern: r#"new\s+Function\s*\("#.into(),
                id: "SEC092".into(),
                message: "CODE INJECTION: dynamic Function creation".into(),
                severity: Severity::Error,
                suggestion: Some("Avoid dynamic code generation".into()),
            },

            // === PATH TRAVERSAL ===
            SecurityPattern {
                pattern: r#"\.\./"#.into(),
                id: "SEC030".into(),
                message: "PATH TRAVERSAL: Relative path with parent directory".into(),
                severity: Severity::Warning,
                suggestion: Some("Validate and sanitize file paths".into()),
            },
            SecurityPattern {
                pattern: r#"\.\.\\"#.into(),
                id: "SEC030".into(),
                message: "PATH TRAVERSAL: Relative path with parent directory".into(),
                severity: Severity::Warning,
                suggestion: Some("Validate and sanitize file paths".into()),
            },

            // === TIMING ATTACK ===
            SecurityPattern {
                pattern: r#"if\s+\w+\s*==\s*\w+_PASSWORD"#.into(),
                id: "SEC080".into(),
                message: "TIMING ATTACK: Non-constant-time password comparison".into(),
                severity: Severity::Warning,
                suggestion: Some("Use constant-time comparison for secrets".into()),
            },

            // === WEAK RANDOM ===
            SecurityPattern {
                pattern: r#"Math\.random\s*\(\s*\)"#.into(),
                id: "SEC110".into(),
                message: "INSECURE RANDOM: Math.random() is not cryptographically secure".into(),
                severity: Severity::Warning,
                suggestion: Some("Use crypto.getRandomValues() for security".into()),
            },
            SecurityPattern {
                pattern: r#"rand\s*\(\s*\)"#.into(),
                id: "SEC110".into(),
                message: "INSECURE RANDOM: rand() is not cryptographically secure".into(),
                severity: Severity::Warning,
                suggestion: Some("Use secure random for security purposes".into()),
            },
        ]
    }

    /// Check if a line matches any security pattern.
    fn check_line(&self, line: &str, line_num: usize) -> Vec<Violation> {
        let mut violations = Vec::new();

        for pattern in &self.patterns {
            if let Ok(re) = regex::Regex::new(&pattern.pattern) {
                if re.is_match(line) {
                    let violation = match pattern.severity {
                        Severity::Critical => Violation::critical(&pattern.id, &pattern.message),
                        Severity::Error => Violation::error(&pattern.id, &pattern.message),
                        Severity::Warning => Violation::warning(&pattern.id, &pattern.message),
                        Severity::Info => Violation::info(&pattern.id, &pattern.message),
                        Severity::Hint => Violation::info(&pattern.id, &pattern.message),
                    };

                    let violation = violation.at(line_num, 1);

                    let violation = if let Some(suggestion) = &pattern.suggestion {
                        violation.suggest(suggestion)
                    } else {
                        violation
                    };

                    violations.push(violation);
                }
            }
        }

        violations
    }
}

impl Default for FallbackSecurityLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for FallbackSecurityLayer {
    fn name(&self) -> &str {
        "fallback_security"
    }

    fn priority(&self) -> u8 {
        35 // Same as SecurityLayer
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let mut violations = Vec::new();
        let source = &ctx.source;

        for (line_num, line) in source.lines().enumerate() {
            let line_violations = self.check_line(line, line_num + 1);
            violations.extend(line_violations);
        }

        // Add file path to violations
        let violations: Vec<Violation> = violations
            .into_iter()
            .map(|v| {
                if let Some(ref file) = ctx.file_path {
                    v.in_file(file)
                } else {
                    v
                }
            })
            .collect();

        let violations = deduplicate_violations(violations);

        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_context(source: &str) -> ValidationContext {
        ValidationContext::for_file(
            &PathBuf::from("test.kt"),
            source.to_string(),
            "unknown".into(),
        )
    }

    #[tokio::test]
    async fn test_hardcoded_password() {
        let layer = FallbackSecurityLayer::new();
        let ctx = make_context(r#"val password = "secret123""#);
        let result = layer.validate(&ctx).await;

        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.id == "SEC001"));
    }

    #[tokio::test]
    async fn test_sql_injection() {
        let layer = FallbackSecurityLayer::new();
        let ctx = make_context(r#"val query = "SELECT * FROM users WHERE id = ${userId}""#);
        let result = layer.validate(&ctx).await;

        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.id == "SEC010"));
    }

    #[tokio::test]
    async fn test_command_injection() {
        let layer = FallbackSecurityLayer::new();
        let ctx = make_context(r#"Runtime.getRuntime().exec("ls " + userInput)"#);
        let result = layer.validate(&ctx).await;

        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.id == "SEC020"));
    }

    #[tokio::test]
    async fn test_no_violations() {
        let layer = FallbackSecurityLayer::new();
        let ctx = make_context(r#"val name = "John""#);
        let result = layer.validate(&ctx).await;

        assert!(result.passed);
        assert!(result.violations.is_empty());
    }
}
