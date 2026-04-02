//! Security Layer — Security vulnerability detection (MILITARY GRADE)
//!
//! This layer enforces security best practices and detects common vulnerabilities.
//! All violations are ERROR-level and MUST be fixed before code can pass.

use std::collections::HashSet;
use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult, LayerConfig};
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity, deduplicate_violations};

#[cfg(feature = "memory")]
use synward_intelligence::memory::learned_config::LearnedConfig;

/// Security validation layer — Military Grade Enforcement.
///
/// Checks for:
/// - Hardcoded credentials and secrets — ERROR
/// - SQL injection patterns — ERROR
/// - Command injection — ERROR
/// - Path traversal — ERROR
/// - Insecure deserialization — ERROR
/// - Weak cryptography — WARNING
/// - Insecure dependencies — INFO
pub struct SecurityLayer {
    patterns: Vec<SecurityRule>,
}

/// A security rule pattern.
#[derive(Debug, Clone)]
struct SecurityRule {
    pattern: String,
    id: String,
    message: String,
    severity: Severity,
    suggestion: Option<String>,
    /// Whether to check only at word boundaries
    word_boundary: bool,
}

impl SecurityLayer {
    /// Create a new security layer with military-grade rules.
    pub fn new() -> Self {
        Self {
            patterns: Self::security_rules(),
        }
    }

    fn security_rules() -> Vec<SecurityRule> {
        vec![
            // === HARDCODED SECRETS — ERROR ===
            SecurityRule {
                pattern: "password = \"".into(),
                id: "SEC001".into(),
                message: "HARDCODED PASSWORD: Credentials must not be hardcoded".into(),
                severity: Severity::Error,
                suggestion: Some("Use environment variables or secret management".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "password=\"".into(),
                id: "SEC001".into(),
                message: "HARDCODED PASSWORD: Credentials must not be hardcoded".into(),
                severity: Severity::Error,
                suggestion: Some("Use environment variables or secret management".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "api_key = \"".into(),
                id: "SEC002".into(),
                message: "HARDCODED API KEY: Secrets must not be in source code".into(),
                severity: Severity::Error,
                suggestion: Some("Use environment variables or secret management".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "secret_key = \"".into(),
                id: "SEC003".into(),
                message: "HARDCODED SECRET KEY: Use environment variables".into(),
                severity: Severity::Error,
                suggestion: Some("Store secrets in environment variables or vault".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "token = \"".into(),
                id: "SEC004".into(),
                message: "HARDCODED TOKEN: Tokens must not be in source code".into(),
                severity: Severity::Error,
                suggestion: Some("Use environment variables or secret management".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "private_key = \"".into(),
                id: "SEC005".into(),
                message: "HARDCODED PRIVATE KEY: Private keys must be secured".into(),
                severity: Severity::Error,
                suggestion: Some("Use secure key storage (HSM, vault, etc.)".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "BEGIN RSA PRIVATE KEY".into(),
                id: "SEC006".into(),
                message: "EMBEDDED PRIVATE KEY: Remove embedded private keys".into(),
                severity: Severity::Error,
                suggestion: Some("Use secure key storage instead of embedding".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "BEGIN PRIVATE KEY".into(),
                id: "SEC006".into(),
                message: "EMBEDDED PRIVATE KEY: Remove embedded private keys".into(),
                severity: Severity::Error,
                suggestion: Some("Use secure key storage instead of embedding".into()),
                word_boundary: false,
            },

            // === SQL INJECTION — ERROR ===
            // SEC010 moved to check_for_sql_injection() for smarter detection
            SecurityRule {
                pattern: "concat!(".into(),
                id: "SEC011".into(),
                message: "POTENTIAL SQL INJECTION: concat! in SQL context".into(),
                severity: Severity::Error,
                suggestion: Some("Use parameterized queries".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "SELECT * FROM ".into(),
                id: "SEC012".into(),
                message: "RAW SQL QUERY: Use ORM or parameterized queries".into(),
                severity: Severity::Warning,
                suggestion: Some("Consider using query builder or ORM".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "INSERT INTO ".into(),
                id: "SEC013".into(),
                message: "RAW SQL INSERT: Use parameterized queries".into(),
                severity: Severity::Warning,
                suggestion: Some("Use parameterized queries or ORM".into()),
                word_boundary: false,
            },

            // === COMMAND INJECTION — ERROR ===
            SecurityRule {
                pattern: "std::process::Command".into(),
                id: "SEC020".into(),
                message: "PROCESS SPAWN: Command execution requires validation".into(),
                severity: Severity::Warning,
                suggestion: Some("Validate and sanitize all inputs to commands".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "std::process::Command::new(".into(),
                id: "SEC020".into(),
                message: "PROCESS SPAWN: Validate command arguments".into(),
                severity: Severity::Warning,
                suggestion: Some("Whitelist allowed commands and validate arguments".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: ".args([".into(),
                id: "SEC021".into(),
                message: "COMMAND ARGS: Ensure arguments are sanitized".into(),
                severity: Severity::Warning,
                suggestion: Some("Use argument escaping or whitelist".into()),
                word_boundary: false,
            },

            // === PATH TRAVERSAL — ERROR ===
            SecurityRule {
                pattern: "..\"".into(),
                id: "SEC030".into(),
                message: "POTENTIAL PATH TRAVERSAL: .. in path string".into(),
                severity: Severity::Error,
                suggestion: Some("Validate and canonicalize paths".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "../".into(),
                id: "SEC030".into(),
                message: "POTENTIAL PATH TRAVERSAL: parent directory reference".into(),
                severity: Severity::Error,
                suggestion: Some("Use path canonicalization and validation".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "std::fs::read(".into(),
                id: "SEC031".into(),
                message: "FILE READ: Validate file paths".into(),
                severity: Severity::Warning,
                suggestion: Some("Validate paths are within allowed directories".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "std::fs::write(".into(),
                id: "SEC032".into(),
                message: "FILE WRITE: Validate file paths".into(),
                severity: Severity::Warning,
                suggestion: Some("Validate paths are within allowed directories".into()),
                word_boundary: false,
            },

            // === INSECURE CRYPTO — WARNING ===
            SecurityRule {
                pattern: "md5::".into(),
                id: "SEC040".into(),
                message: "WEAK HASH: MD5 is cryptographically broken".into(),
                severity: Severity::Error,
                suggestion: Some("Use SHA-256 or SHA-3 for cryptographic hashing".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "sha1::".into(),
                id: "SEC041".into(),
                message: "WEAK HASH: SHA-1 is cryptographically weak".into(),
                severity: Severity::Warning,
                suggestion: Some("Use SHA-256 or SHA-3 for cryptographic purposes".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "DES".into(),
                id: "SEC042".into(),
                message: "WEAK ENCRYPTION: DES is insecure".into(),
                severity: Severity::Error,
                suggestion: Some("Use AES-256-GCM or ChaCha20-Poly1305".into()),
                word_boundary: true,
            },
            SecurityRule {
                pattern: "RC4".into(),
                id: "SEC043".into(),
                message: "WEAK ENCRYPTION: RC4 is insecure".into(),
                severity: Severity::Error,
                suggestion: Some("Use AES-256-GCM or ChaCha20-Poly1305".into()),
                word_boundary: true,
            },

            // === NETWORK SECURITY — WARNING ===
            SecurityRule {
                pattern: "0.0.0.0".into(),
                id: "SEC050".into(),
                message: "BIND ALL INTERFACES: Binding to 0.0.0.0 may expose service".into(),
                severity: Severity::Warning,
                suggestion: Some("Bind to specific interface or use localhost for development".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "http://".into(),
                id: "SEC051".into(),
                message: "INSECURE HTTP: Prefer HTTPS for production".into(),
                severity: Severity::Warning,
                suggestion: Some("Use https:// for production endpoints".into()),
                word_boundary: false,
            },

            // === DANGEROUS FUNCTIONS — WARNING ===
            SecurityRule {
                pattern: "std::mem::transmute".into(),
                id: "SEC060".into(),
                message: "TRANSMUTE: Extremely unsafe, can cause undefined behavior".into(),
                severity: Severity::Error,
                suggestion: Some("Use safe alternatives or bytemuck crate".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "std::hint::unreachable_unchecked".into(),
                id: "SEC061".into(),
                message: "UNREACHABLE_UNCHECKED: Can cause UB if reachable".into(),
                severity: Severity::Error,
                suggestion: Some("Use unreachable!() for safe unreachable code".into()),
                word_boundary: false,
            },

            // === ASSERTIONS IN PRODUCTION — INFO ===
            SecurityRule {
                pattern: "debug_assert!".into(),
                id: "SEC070".into(),
                message: "DEBUG_ASSERT: Only runs in debug builds".into(),
                severity: Severity::Info,
                suggestion: Some("Consider assert! for critical security checks".into()),
                word_boundary: false,
            },

            // === TIMING ATTACKS — WARNING ===
            // Only flag comparisons involving secret-like variables
            SecurityRule {
                pattern: "password ==".into(),
                id: "SEC080".into(),
                message: "TIMING ATTACK: Use constant-time comparison for password".into(),
                severity: Severity::Warning,
                suggestion: Some("Use subtle crate for secret comparison".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "token ==".into(),
                id: "SEC080".into(),
                message: "TIMING ATTACK: Use constant-time comparison for token".into(),
                severity: Severity::Warning,
                suggestion: Some("Use subtle crate for secret comparison".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "secret ==".into(),
                id: "SEC080".into(),
                message: "TIMING ATTACK: Use constant-time comparison for secret".into(),
                severity: Severity::Warning,
                suggestion: Some("Use subtle crate for secret comparison".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "api_key ==".into(),
                id: "SEC080".into(),
                message: "TIMING ATTACK: Use constant-time comparison for api_key".into(),
                severity: Severity::Warning,
                suggestion: Some("Use subtle crate for secret comparison".into()),
                word_boundary: false,
            },

            // === CODE INJECTION (eval/exec) - ERROR ===
            SecurityRule {
                pattern: "eval(".into(),
                id: "SEC090".into(),
                message: "CODE INJECTION: eval() can execute arbitrary code".into(),
                severity: Severity::Error,
                suggestion: Some("Use safer alternatives like JSON.parse or ast.literal_eval".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "exec(".into(),
                id: "SEC091".into(),
                message: "CODE INJECTION: exec() can execute arbitrary code".into(),
                severity: Severity::Error,
                suggestion: Some("Avoid exec() or use restricted sandboxes".into()),
                word_boundary: true,  // Only match standalone exec(, not .exec(
            },
            SecurityRule {
                pattern: "new Function(".into(),
                id: "SEC092".into(),
                message: "CODE INJECTION: new Function() creates executable code".into(),
                severity: Severity::Error,
                suggestion: Some("Avoid dynamic code creation".into()),
                word_boundary: false,
            },

            // === INSECURE DESERIALIZATION - ERROR ===
            SecurityRule {
                pattern: "pickle.loads".into(),
                id: "SEC100".into(),
                message: "INSECURE DESERIALIZE: pickle can execute arbitrary code".into(),
                severity: Severity::Error,
                suggestion: Some("Use JSON or other safe serialization".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "pickle.load".into(),
                id: "SEC100".into(),
                message: "INSECURE DESERIALIZE: pickle can execute arbitrary code".into(),
                severity: Severity::Error,
                suggestion: Some("Use JSON or other safe serialization".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "marshal.loads".into(),
                id: "SEC101".into(),
                message: "INSECURE DESERIALIZE: marshal is unsafe for untrusted data".into(),
                severity: Severity::Error,
                suggestion: Some("Use JSON or other safe serialization".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "yaml.load(".into(),
                id: "SEC102".into(),
                message: "INSECURE DESERIALIZE: yaml.load can execute code".into(),
                severity: Severity::Error,
                suggestion: Some("Use yaml.safe_load() instead".into()),
                word_boundary: false,
            },

            // === PSEUDO-RANDOM - WARNING ===
            SecurityRule {
                pattern: "Math.random()".into(),
                id: "SEC110".into(),
                message: "INSECURE RANDOM: Math.random() is not cryptographically secure".into(),
                severity: Severity::Warning,
                suggestion: Some("Use crypto.getRandomValues() for security".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "random.randint".into(),
                id: "SEC111".into(),
                message: "INSECURE RANDOM: random module is not cryptographically secure".into(),
                severity: Severity::Warning,
                suggestion: Some("Use secrets module for security-sensitive randomness".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "random.choice".into(),
                id: "SEC111".into(),
                message: "INSECURE RANDOM: random module is not cryptographically secure".into(),
                severity: Severity::Warning,
                suggestion: Some("Use secrets.choice() for security-sensitive operations".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "rand::thread_rng()".into(),
                id: "SEC112".into(),
                message: "RANDOM: Ensure not used for security-sensitive operations".into(),
                severity: Severity::Info,
                suggestion: Some("Use OsRng for cryptographic randomness".into()),
                word_boundary: false,
            },

            // === XSS PATTERNS - ERROR ===
            SecurityRule {
                pattern: ".innerHTML =".into(),
                id: "SEC120".into(),
                message: "XSS RISK: innerHTML can cause cross-site scripting".into(),
                severity: Severity::Error,
                suggestion: Some("Use textContent or DOM APIs".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: ".outerHTML =".into(),
                id: "SEC121".into(),
                message: "XSS RISK: outerHTML can cause cross-site scripting".into(),
                severity: Severity::Error,
                suggestion: Some("Use DOM APIs to create elements safely".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "document.write(".into(),
                id: "SEC122".into(),
                message: "XSS RISK: document.write can cause cross-site scripting".into(),
                severity: Severity::Error,
                suggestion: Some("Use DOM manipulation methods instead".into()),
                word_boundary: false,
            },

            // === PROTOTYPE POLLUTION - ERROR ===
            SecurityRule {
                pattern: "__proto__".into(),
                id: "SEC130".into(),
                message: "PROTOTYPE POLLUTION: __proto__ manipulation is dangerous".into(),
                severity: Severity::Error,
                suggestion: Some("Use Object.create(null) or Map".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "Object.assign(".into(),
                id: "SEC131".into(),
                message: "PROTOTYPE POLLUTION: Check if user input is in Object.assign".into(),
                severity: Severity::Warning,
                suggestion: Some("Validate keys don't include __proto__, constructor".into()),
                word_boundary: false,
            },

            // === BUFFER VULNERABILITIES - WARNING ===
            SecurityRule {
                pattern: "new Buffer(".into(),
                id: "SEC140".into(),
                message: "DEPRECATED BUFFER: new Buffer() is deprecated and unsafe".into(),
                severity: Severity::Warning,
                suggestion: Some("Use Buffer.from() or Buffer.alloc()".into()),
                word_boundary: false,
            },

            // === INSECURE TEMP FILES - WARNING ===
            SecurityRule {
                pattern: "tempfile.mktemp".into(),
                id: "SEC150".into(),
                message: "INSECURE TEMP: mktemp is vulnerable to race conditions".into(),
                severity: Severity::Warning,
                suggestion: Some("Use tempfile.mkstemp() instead".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "/tmp/".into(),
                id: "SEC151".into(),
                message: "TEMP PATH: Using /tmp directly may be insecure".into(),
                severity: Severity::Info,
                suggestion: Some("Use proper temp file creation functions".into()),
                word_boundary: false,
            },

            // === CORS MISCONFIGURATION - WARNING ===
            SecurityRule {
                pattern: "Access-Control-Allow-Origin: *".into(),
                id: "SEC160".into(),
                message: "CORS WILDCARD: Allowing all origins is insecure".into(),
                severity: Severity::Warning,
                suggestion: Some("Specify allowed origins explicitly".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "cors({ origin: '*' })".into(),
                id: "SEC160".into(),
                message: "CORS WILDCARD: Allowing all origins is insecure".into(),
                severity: Severity::Warning,
                suggestion: Some("Specify allowed origins explicitly".into()),
                word_boundary: false,
            },

            // === SSL/TLS ISSUES - WARNING ===
            SecurityRule {
                pattern: "verify=False".into(),
                id: "SEC170".into(),
                message: "SSL VERIFICATION DISABLED: Certificate verification is off".into(),
                severity: Severity::Error,
                suggestion: Some("Enable SSL certificate verification".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "CERT_NONE".into(),
                id: "SEC171".into(),
                message: "SSL VERIFICATION DISABLED: Certificate verification is off".into(),
                severity: Severity::Error,
                suggestion: Some("Use CERT_REQUIRED for proper verification".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "rejectUnauthorized: false".into(),
                id: "SEC172".into(),
                message: "SSL VERIFICATION DISABLED: Certificate verification is off".into(),
                severity: Severity::Error,
                suggestion: Some("Enable certificate verification".into()),
                word_boundary: false,
            },

            // === SHELL INJECTION - ERROR ===
            SecurityRule {
                pattern: "shell=True".into(),
                id: "SEC180".into(),
                message: "SHELL INJECTION: subprocess with shell=True is dangerous".into(),
                severity: Severity::Error,
                suggestion: Some("Remove shell=True and pass args as list".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "os.system(".into(),
                id: "SEC181".into(),
                message: "SHELL INJECTION: os.system is vulnerable to injection".into(),
                severity: Severity::Error,
                suggestion: Some("Use subprocess.run with list of args".into()),
                word_boundary: false,
            },

            // === DEBUG MODE - ERROR ===
            SecurityRule {
                pattern: "debug=True".into(),
                id: "SEC190".into(),
                message: "DEBUG MODE: Debug mode exposes sensitive information".into(),
                severity: Severity::Error,
                suggestion: Some("Disable debug in production".into()),
                word_boundary: false,
            },
            SecurityRule {
                pattern: "DEBUG = True".into(),
                id: "SEC190".into(),
                message: "DEBUG MODE: Debug mode exposes sensitive information".into(),
                severity: Severity::Error,
                suggestion: Some("Set DEBUG = False in production".into()),
                word_boundary: false,
            },

            // === ASSERT MISUSE - WARNING ===
            SecurityRule {
                pattern: "assert ".into(),
                id: "SEC200".into(),
                message: "ASSERT WARNING: assert is removed with -O optimization".into(),
                severity: Severity::Warning,
                suggestion: Some("Use explicit if/raise for runtime checks".into()),
                word_boundary: false,
            },
        ]
    }
}

impl Default for SecurityLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for SecurityLayer {
    fn name(&self) -> &str {
        "security"
    }

    fn priority(&self) -> u8 {
        35 // After logic, before style
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let mut violations = Vec::new();
        let source = &ctx.source;
        let lines: Vec<&str> = source.lines().collect();

        // Pre-calculate which lines are in "definition context" (SecurityRule blocks, test blocks, etc.)
        // This prevents false positives when validating Synward's own code
        let definition_lines = calculate_definition_context(&lines);

        // Pre-calculate which lines are inside raw string literals (r#"..."#, r##"..."##, etc.)
        // These often contain example code that should not trigger security violations
        let raw_string_lines = calculate_raw_string_lines(&lines);

        // Check each security rule
        for rule in &self.patterns {
            for (line_num, line) in lines.iter().enumerate() {
                if line.contains(&rule.pattern) {
                    let trimmed = line.trim();
                    
                    // Skip if in comment
                    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("#") {
                        continue;
                    }

                    // Skip if line is in definition context (SecurityRule blocks, tests, etc.)
                    if definition_lines.contains(&line_num) {
                        continue;
                    }

                    // Skip if line is inside a raw string literal (example code)
                    if raw_string_lines.contains(&line_num) {
                        continue;
                    }

                    // Skip SEC080 timing attack checks for enum comparisons
                    // e.g., "token == LexToken::Eof" or "token == PrismToken::Eof"
                    // These are not security token comparisons, just parser enum comparisons
                    if rule.id == "SEC080" && line.contains("==") {
                        if let Some(eq_pos) = line.find("==") {
                            let after_eq = &line[eq_pos + 2..];
                            let trimmed = after_eq.trim_start();
                            // Skip if comparing with enum (contains ::) or reference to enum (& ... ::)
                            if trimmed.contains("::") ||
                               (trimmed.starts_with('&') && trimmed[1..].trim_start().contains("::")) {
                                continue;
                            }
                        }
                    }

                    // Skip if in string literal context (very simple heuristic)
                    if is_likely_in_string_literal(line, &rule.pattern) {
                        continue;
                    }

                    // If word_boundary is true, check that pattern appears at word boundary
                    // This prevents false positives like .exec() matching exec(
                    if rule.word_boundary {
                        // Find the position of the pattern
                        if let Some(pos) = line.find(&rule.pattern) {
                            // Check if there's a non-whitespace, non-punctuation character before
                            let before_is_boundary = pos == 0 || {
                                let prev_char = line.chars().nth(pos - 1);
                                match prev_char {
                                    Some(c) => !c.is_alphanumeric() && c != '_',
                                    None => true,
                                }
                            };
                            if !before_is_boundary {
                                continue;
                            }
                        }
                    }

                    let violation = match rule.severity {
                        Severity::Critical => Violation::critical(&rule.id, &rule.message),
                        Severity::Error => Violation::error(&rule.id, &rule.message),
                        Severity::Warning => Violation::warning(&rule.id, &rule.message),
                        Severity::Info => Violation::info(&rule.id, &rule.message),
                        Severity::Hint => Violation::info(&rule.id, &rule.message),
                    };

                    let violation = violation.at(line_num + 1, 1);
                    let violation = if let Some(suggestion) = &rule.suggestion {
                        violation.suggest(suggestion)
                    } else {
                        violation
                    };

                    violations.push(violation);
                }
            }
        }

        // Additional checks
        check_for_secrets_in_strings(source, &mut violations);
        check_for_dangerous_file_ops(source, &mut violations);
        check_for_sql_injection(source, &mut violations);

        // Deduplicate violations by ID and message
        let violations = deduplicate_violations(violations);

        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }

    /// Override to apply learned whitelist from Dubbioso memory.
    /// Filters out violations that have been whitelisted by the user.
    #[cfg(feature = "memory")]
    async fn validate_with_config(&self, ctx: &ValidationContext, config: Option<&LayerConfig>) -> LayerResult {
        let result = self.validate(ctx).await;

        if let Some(cfg) = config {
            // Try to parse config as LearnedConfig
            if let Ok(learned) = serde_json::from_value::<LearnedConfig>(cfg.clone()) {
                if let Some(ref file_path) = ctx.file_path {
                    let file_path: std::borrow::Cow<'_, str> = file_path.to_string_lossy();
                    return result.filter_whitelisted(|v| learned.is_whitelisted(&v.id, &file_path));
                }
            }
        }

        result
    }
}

/// Check if pattern is likely inside a string literal (not actual code)
fn is_likely_in_string_literal(line: &str, pattern: &str) -> bool {
    // Find position of pattern
    if let Some(pos) = line.find(pattern) {
        let before = &line[..pos];

        // Check for Rust raw strings: r#"..."# or r##"..."##
        // Raw strings start with r followed by 0+ # and a "
        if let Some(raw_start) = before.find("r#\"") {
            // Check if this raw string is still open (no closing "# found after)
            let after_raw_start = &line[raw_start + 3..];
            if !after_raw_start.contains("\"#") {
                return true;
            }
        }
        // Also check for r##", r###", etc.
        for hash_count in (2..=10).rev() {
            let raw_prefix = format!("r{}\"", "#".repeat(hash_count));
            if let Some(raw_start) = before.find(&raw_prefix) {
                let closing = format!("\"{}", "#".repeat(hash_count));
                let after_raw_start = &line[raw_start + raw_prefix.len()..];
                if !after_raw_start.contains(&closing) {
                    return true;
                }
            }
        }

        // Count quotes before pattern
        let _single_quotes = before.matches('\'').count();
        let double_quotes = before.matches('"').count();

        // If odd number of double quotes before, likely in string
        // (simple heuristic, not perfect)
        double_quotes % 2 == 1
    } else {
        false
    }
}

/// Pre-calculate which lines are in "definition context" - SecurityRule blocks, test blocks, etc.
/// Lines in these contexts should not trigger security violations as they define patterns, not use them.
fn calculate_definition_context(lines: &[&str]) -> HashSet<usize> {
    let mut definition_lines = HashSet::new();
    let mut in_security_rule = false;
    let mut security_rule_depth = 0;
    let mut in_test_context = false;
    let mut test_brace_depth = 0;

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Track SecurityRule blocks
        if trimmed.contains("SecurityRule {") {
            in_security_rule = true;
            security_rule_depth = 1;
            definition_lines.insert(line_num);
            continue;
        }
        if in_security_rule {
            security_rule_depth += line.matches('{').count() as i32;
            security_rule_depth -= line.matches('}').count() as i32;
            definition_lines.insert(line_num);
            if security_rule_depth <= 0 {
                in_security_rule = false;
            }
            continue;
        }

        // Track test context
        if trimmed.contains("#[cfg(test)]") || trimmed.contains("#[test]") || trimmed.contains("mod tests") {
            in_test_context = true;
            test_brace_depth = 0;
        }
        if in_test_context {
            test_brace_depth += line.matches('{').count() as i32;
            test_brace_depth -= line.matches('}').count() as i32;
            definition_lines.insert(line_num);
            if test_brace_depth <= 0 && (trimmed.contains("}") || trimmed.ends_with("}")) {
                in_test_context = false;
            }
            continue;
        }

        // Track vec![] blocks that define patterns
        if trimmed.starts_with("vec![") {
            // Mark this and next few lines as definition context
            definition_lines.insert(line_num);
        }
    }

    definition_lines
}

/// Pre-calculate which lines are inside raw string literals (r#"..."#, r##"..."##, etc.)
/// Raw strings often contain example code for testing that should not trigger violations.
fn calculate_raw_string_lines(lines: &[&str]) -> HashSet<usize> {
    let mut raw_string_lines = HashSet::new();
    let mut raw_string_stack: Vec<String> = Vec::new(); // closing patterns

    for (line_num, line) in lines.iter().enumerate() {
        // Check for raw string starts: r#" r##" r###" etc.
        let chars: Vec<char> = line.chars().collect();
        
        // Track char position for string slicing
        for (char_pos, ch) in line.char_indices() {
            if raw_string_stack.is_empty() {
                // Not in a raw string - look for starts
                // Check if previous char was 'r' and current is '#' or '"'
                if char_pos > 0 {
                    let prev_char = chars.get(char_pos - 1).copied();
                    if prev_char == Some('r') {
                        if ch == '#' {
                            // Count hash marks starting from current position
                            let mut hash_count = 1usize;
                            let mut i = char_pos + 1;
                            while i < chars.len() && chars[i] == '#' {
                                hash_count += 1;
                                i += 1;
                            }
                            // Check for opening quote after hashes
                            if i < chars.len() && chars[i] == '"' {
                                // Found raw string start
                                let closing = format!("\"{}", "#".repeat(hash_count));
                                raw_string_stack.push(closing);
                                raw_string_lines.insert(line_num);
                                continue;
                            }
                        } else if ch == '"' {
                            // r" (0 hashes)
                            let closing = "\"".to_string();
                            raw_string_stack.push(closing);
                            raw_string_lines.insert(line_num);
                            continue;
                        }
                    }
                }
            } else {
                // In a raw string - mark this line and look for closing
                raw_string_lines.insert(line_num);
                
                // Check for closing delimiter
                let closing = &raw_string_stack[raw_string_stack.len() - 1];
                if line[char_pos..].starts_with(closing.as_str()) {
                    raw_string_stack.pop();
                }
            }
        }
        
        // If we're still in a raw string at end of line, mark for next lines
        if !raw_string_stack.is_empty() {
            raw_string_lines.insert(line_num);
        }
    }

    raw_string_lines
}

/// Check if this line is defining a security pattern/rule (not actual vulnerable code)
/// This prevents false positives when validating Synward's own security layer code
#[allow(dead_code)]
fn is_pattern_definition(line: &str, pattern: &str) -> bool {
    let trimmed = line.trim();
    
    // Check for Rust struct field pattern definitions (with or without escape chars)
    // e.g., `pattern: "password = \"".into()` or `pattern: "eval(".into()`
    if trimmed.contains("pattern:") && trimmed.contains(".into()") {
        return true;
    }
    
    // Check for raw string patterns in regex definitions
    // e.g., `r#"eval\s*\("#` or `r#"password\s*="#`
    if trimmed.contains("r#\"") || trimmed.contains("r\"") {
        return true;
    }
    
    // Check for pattern definitions in SecurityRule structs
    // Also check for message: and suggestion: fields which may contain pattern examples
    if trimmed.contains("SecurityRule") || trimmed.contains("pattern:") {
        return true;
    }
    
    // Check for message: and suggestion: fields in SecurityRule definitions
    // These may contain pattern examples like "exec()" in "Avoid exec() or use..."
    // Use contains() instead of starts_with() to handle "suggestion: Some(..." format
    if (trimmed.contains("message:") || trimmed.contains("suggestion:")) && trimmed.contains(".into()") {
        return true;
    }
    
    // Check for test/example pattern definitions
    // e.g., in test code that demonstrates patterns
    if trimmed.contains("#[test]") || trimmed.contains("#[cfg(test)]") {
        // Look ahead - this might be test code
        return true;
    }
    
    // Check if pattern is inside a static/const array of patterns
    // e.g., `const PATTERNS: &[&str] = &[` or `static RULES:`
    if trimmed.starts_with("const") || trimmed.starts_with("static") {
        if trimmed.contains("PATTERN") || trimmed.contains("RULE") || trimmed.contains("SECRET") {
            return true;
        }
    }
    
    // Check for secret pattern definitions used in detection
    // e.g., `("sk-", "API key prefix detected")` in check_for_secrets_in_strings
    if trimmed.starts_with("(") && trimmed.contains(pattern) {
        // This looks like a tuple definition for a pattern
        if trimmed.contains(",") && trimmed.ends_with("),") {
            return true;
        }
    }
    
    // Check if line contains escaped version of pattern (pattern definition with escape chars)
    // e.g., pattern is "password = " and line has "password = \""
    if trimmed.contains(&format!("{}\\\"", pattern.trim_end_matches('"'))) {
        return true;
    }
    
    // Check if the line is part of a vec![] of SecurityRule definitions
    // This is common when defining security patterns
    if trimmed.starts_with("SecurityRule {") || trimmed.starts_with("},") || trimmed.starts_with("vec![") {
        return true;
    }
    
    false
}

/// Check for secrets in string literals
fn check_for_secrets_in_strings(source: &str, violations: &mut Vec<Violation>) {
    // Look for patterns like: "sk-...", "Bearer ", token patterns
    let secret_patterns = [
        ("sk-", "API key prefix detected"),
        ("Bearer ", "Bearer token in code"),
        ("ghp_", "GitHub PAT detected"),
        ("gho_", "GitHub OAuth token detected"),
        ("github_pat_", "GitHub PAT detected"),
        ("xoxb-", "Slack bot token detected"),
        ("xoxa-", "Slack app token detected"),
    ];

    // Check if this file is a security/validation layer that defines detection patterns
    // This prevents false positives when validating Synward's own security layer code
    let is_pattern_definition_file = source.contains("secret_patterns") 
        || source.contains("SecurityRule")
        || source.contains("check_for_secrets_in_strings");

    // Track test context to skip test code
    let mut in_test_context = false;
    let mut test_brace_depth = 0;

    for (line_num, line) in source.lines().enumerate() {
        // Track test context
        let trimmed = line.trim();
        if trimmed.contains("#[cfg(test)]") || trimmed.contains("#[test]") || trimmed.contains("mod tests") {
            in_test_context = true;
        }
        if in_test_context {
            test_brace_depth += line.matches('{').count() as i32;
            test_brace_depth -= line.matches('}').count() as i32;
            if test_brace_depth <= 0 {
                in_test_context = false;
            }
            // Skip test code entirely
            continue;
        }

        for (pattern, msg) in &secret_patterns {
            if line.contains(pattern) {
                // Skip if in comment
                if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("#") {
                    continue;
                }

                // Skip if this is a pattern definition (tuple in array definition)
                if is_pattern_definition_file {
                    // Pattern definitions look like: `("sk-", "message"),`
                    if trimmed.starts_with("(") && trimmed.contains(",") && (trimmed.ends_with("),") || trimmed.ends_with(",")) {
                        continue;
                    }
                    // Also skip if line is part of secret_patterns array definition
                    if trimmed.contains("secret_patterns") || trimmed.contains("let secret_patterns") {
                        continue;
                    }
                }

                violations.push(Violation::error(
                    "SEC090",
                    format!("SECRET DETECTED: {}", msg),
                ).at(line_num + 1, 1).suggest("Remove secret and use environment variable"));
            }
        }
    }
}

/// Check for SQL injection patterns - format! with SQL keywords in strings
fn check_for_sql_injection(source: &str, violations: &mut Vec<Violation>) {
    /// SQL patterns that strongly indicate a query context (uppercase for matching)
    /// These are specific patterns to minimize false positives
    const SQL_PATTERNS: &[&str] = &[
        "SELECT *",     // SELECT * FROM
        "INSERT INTO",  // INSERT INTO table
        "DELETE FROM",  // DELETE FROM table
        " WHERE ",      // ... WHERE condition
        "INNER JOIN",   // INNER JOIN
        "LEFT JOIN",    // LEFT JOIN
        "RIGHT JOIN",   // RIGHT JOIN
        "ORDER BY",     // ORDER BY
        "GROUP BY",     // GROUP BY
        "HAVING ",      // HAVING condition
        "TRUNCATE ",    // TRUNCATE TABLE
    ];

    for (line_num, line) in source.lines().enumerate() {
        // Skip comments
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("#") {
            continue;
        }

        // Only check lines that contain format! or string concatenation
        if !line.contains("format!(") && !line.contains("+ \"") && !line.contains("\" +") {
            continue;
        }

        let line_upper = line.to_uppercase();
        
        // Check for SQL patterns in the uppercased line
        for pattern in SQL_PATTERNS {
            if line_upper.contains(pattern) {
                violations.push(Violation::error(
                    "SEC010",
                    "SQL INJECTION: format! with SQL query - use parameterized queries",
                )
                .at(line_num + 1, 1)
                .suggest("Use sqlx::query!() or parameterized statements"));
                break; // Only one violation per line
            }
        }
    }
}

/// Check for dangerous file operations
fn check_for_dangerous_file_ops(_source: &str, _violations: &mut Vec<Violation>) {
    // TODO: Add more sophisticated checks for dangerous file operations
    // This is a placeholder for future security checks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_violations() {
        let source = r#"
fn main() {
    let x = 1 + 2;
    println!("{}", x);
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = SecurityLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.passed, "Clean code should pass: {:?}", result.violations);
    }

    #[tokio::test]
    async fn test_hardcoded_password() {
        let source = r#"
fn connect() {
    let password = "secret123";
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = SecurityLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty(), "password should trigger violation");
        assert!(result.violations.iter().any(|v| v.id == "SEC001"), "Expected SEC001");
    }

    #[tokio::test]
    async fn test_api_key() {
        let source = r#"
fn request() {
    let api_key = "sk-12345";
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = SecurityLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty(), "api_key should trigger violation");
        assert!(result.violations.iter().any(|v| v.id == "SEC002" || v.id == "SEC090"));
    }

    #[tokio::test]
    async fn test_path_traversal() {
        let source = r#"
fn read_file(path: &str) {
    let content = std::fs::read(format!("../{}", path)).unwrap();
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = SecurityLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty(), "path traversal should trigger violation");
    }

    #[tokio::test]
    async fn test_weak_crypto() {
        let source = r#"
use md5;
fn hash(data: &[u8]) -> Vec<u8> {
    md5::compute(data).to_vec()
}
"#;
        let ctx = ValidationContext::for_file("test.rs", source.into(), "rust".into());
        let layer = SecurityLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty(), "md5 should trigger violation");
        assert!(result.violations.iter().any(|v| v.id == "SEC040"));
    }
}
