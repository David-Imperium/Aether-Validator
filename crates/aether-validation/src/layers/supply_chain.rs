//! Supply Chain Layer — Dependency and supply chain security (OWASP A03)
//!
//! This layer checks for supply chain vulnerabilities:
//! - Vulnerable dependencies
//! - Unverified package sources
//! - Deprecated packages
//! - License compliance

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::{Violation, Severity, Span};

/// Supply chain validation layer.
///
/// Checks for:
/// - Known vulnerable packages — ERROR
/// - Unpinned dependencies — WARNING
/// - Deprecated packages — WARNING
/// - Unverified sources — WARNING
/// - License violations — INFO
pub struct SupplyChainLayer {
    /// Known vulnerable packages (simplified database)
    vulnerable_packages: Vec<VulnerablePackage>,
}

#[derive(Debug, Clone)]
struct VulnerablePackage {
    name: String,
    ecosystem: String,
    min_version: String,
    max_version: String,
    cve: String,
    severity: Severity,
}

impl SupplyChainLayer {
    /// Create with default vulnerable package database.
    pub fn new() -> Self {
        Self {
            vulnerable_packages: Self::load_vulnerabilities(),
        }
    }
    
    /// Load known vulnerability database.
    fn load_vulnerabilities() -> Vec<VulnerablePackage> {
        // In production, this would load from an external database
        // For now, include some well-known vulnerable packages
        vec![
            // Log4j (Java)
            VulnerablePackage {
                name: "log4j".to_string(),
                ecosystem: "maven".to_string(),
                min_version: "2.0.0".to_string(),
                max_version: "2.14.1".to_string(),
                cve: "CVE-2021-44228".to_string(),
                severity: Severity::Error,
            },
            // event-stream (npm) - malicious package
            VulnerablePackage {
                name: "event-stream".to_string(),
                ecosystem: "npm".to_string(),
                min_version: "0.0.0".to_string(),
                max_version: "999.999.999".to_string(),
                cve: "MALICIOUS".to_string(),
                severity: Severity::Error,
            },
            // lodash (npm) - prototype pollution
            VulnerablePackage {
                name: "lodash".to_string(),
                ecosystem: "npm".to_string(),
                min_version: "0.0.0".to_string(),
                max_version: "4.17.20".to_string(),
                cve: "CVE-2021-23337".to_string(),
                severity: Severity::Warning,
            },
            // requests (Python) - deprecated
            VulnerablePackage {
                name: "requests".to_string(),
                ecosystem: "pypi".to_string(),
                min_version: "2.0.0".to_string(),
                max_version: "2.27.0".to_string(),
                cve: "DEPRECATED".to_string(),
                severity: Severity::Info,
            },
        ]
    }
    
    /// Parse Cargo.toml for dependencies.
    fn parse_cargo_toml(source: &str) -> Vec<(String, String)> {
        let mut deps = Vec::new();
        let mut in_deps = false;
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            if trimmed == "[dependencies]" {
                in_deps = true;
                continue;
            }
            
            if trimmed.starts_with('[') && trimmed != "[dependencies]" {
                in_deps = false;
                continue;
            }
            
            if in_deps && trimmed.contains('=') {
                let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let name = parts[0].trim().to_string();
                    let version = parts[1].trim()
                        .trim_matches('"')
                        .trim_start_matches(' ')
                        .split(' ')
                        .next()
                        .unwrap_or("*")
                        .to_string();
                    deps.push((name, version));
                }
            }
        }
        
        deps
    }
    
    /// Parse package.json for dependencies.
    fn parse_package_json(source: &str) -> Vec<(String, String)> {
        let mut deps = Vec::new();
        let mut in_deps = false;
        let mut in_dev_deps = false;
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            if trimmed.contains("\"dependencies\"") {
                in_deps = true;
                in_dev_deps = false;
                continue;
            }
            
            if trimmed.contains("\"devDependencies\"") {
                in_deps = false;
                in_dev_deps = true;
                continue;
            }
            
            if trimmed == "}" {
                in_deps = false;
                in_dev_deps = false;
                continue;
            }
            
            if (in_deps || in_dev_deps) && trimmed.contains(':') {
                // Parse "package": "version"
                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() == 2 {
                    let name = parts[0].trim().trim_matches('"').to_string();
                    let version = parts[1].trim()
                        .trim_matches(',')
                        .trim_matches('"')
                        .to_string();
                    deps.push((name, version));
                }
            }
        }
        
        deps
    }
    
    /// Parse requirements.txt for dependencies.
    fn parse_requirements_txt(source: &str) -> Vec<(String, String)> {
        let mut deps = Vec::new();
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            // Parse package==version or package>=version
            if let Some(sep_pos) = trimmed.find(|c| c == '=' || c == '>' || c == '<') {
                let name = trimmed[..sep_pos].trim().to_string();
                let version = trimmed[sep_pos..].trim_matches(|c| c == '=' || c == '>' || c == '<').to_string();
                deps.push((name, version));
            } else if !trimmed.is_empty() {
                // No version specified - unpinned
                deps.push((trimmed.to_string(), "*".to_string()));
            }
        }
        
        deps
    }
    
    /// Parse pom.xml for Maven dependencies.
    fn parse_pom_xml(source: &str) -> Vec<(String, String)> {
        let mut deps = Vec::new();
        let mut in_deps = false;
        let mut current_group = None;
        let mut current_artifact = None;
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            if trimmed == "<dependencies>" {
                in_deps = true;
                continue;
            }
            
            if trimmed == "</dependencies>" {
                in_deps = false;
                continue;
            }
            
            if in_deps {
                if trimmed.starts_with("<groupId>") {
                    current_group = Some(trimmed.trim_start_matches("<groupId>")
                        .trim_end_matches("</groupId>").to_string());
                }
                if trimmed.starts_with("<artifactId>") {
                    current_artifact = Some(trimmed.trim_start_matches("<artifactId>")
                        .trim_end_matches("</artifactId>").to_string());
                }
                if trimmed.starts_with("<version>") {
                    let version = trimmed.trim_start_matches("<version>")
                        .trim_end_matches("</version>").to_string();
                    
                    if let (Some(artifact), Some(group)) = (&current_artifact, &current_group) {
                        deps.push((format!("{}:{}", group, artifact), version));
                    }
                    current_group = None;
                    current_artifact = None;
                }
            }
        }
        
        deps
    }
    
    /// Check if a dependency is vulnerable.
    fn check_dependency(&self, name: &str, version: &str, ecosystem: &str, file: Option<std::path::PathBuf>) -> Option<Violation> {
        for vuln in &self.vulnerable_packages {
            if vuln.ecosystem == ecosystem && vuln.name == name {
                // Simplified version check (should use semver)
                if version == "*" || version.starts_with('^') || version.starts_with('~') {
                    // Unpinned - could be vulnerable
                    return Some(Violation {
                        id: "SUPP001".to_string(),
                        message: format!(
                            "UNPINNED DEPENDENCY: {}@{} - could resolve to vulnerable version",
                            name, version
                        ),
                        severity: Severity::Warning,
                        span: Some(Span { line: 1, column: 1 }),
                        file,
                        suggestion: Some("Pin exact version number".to_string()),
                    });
                }
                
                // Check if version is in vulnerable range
                if self.version_in_range(version, &vuln.min_version, &vuln.max_version) {
                    return Some(Violation {
                        id: "SUPP002".to_string(),
                        message: format!(
                            "VULNERABLE DEPENDENCY: {}@{} - {}",
                            name, version, vuln.cve
                        ),
                        severity: vuln.severity.clone(),
                        span: Some(Span { line: 1, column: 1 }),
                        file,
                        suggestion: Some(format!("Upgrade to version > {}", vuln.max_version)),
                    });
                }
            }
        }
        
        None
    }
    
    /// Check if version is in range (simplified).
    fn version_in_range(&self, version: &str, min: &str, max: &str) -> bool {
        // Very simplified - should use proper semver comparison
        version >= min && version <= max
    }
    
    /// Detect dependency file type and parse.
    fn detect_and_parse(&self, ctx: &ValidationContext) -> Vec<(String, String, String)> {
        let filename = ctx.file_path.as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        match filename {
            "Cargo.toml" => {
                Self::parse_cargo_toml(&ctx.source)
                    .into_iter()
                    .map(|(n, v)| (n, v, "cargo".to_string()))
                    .collect()
            }
            "package.json" => {
                Self::parse_package_json(&ctx.source)
                    .into_iter()
                    .map(|(n, v)| (n, v, "npm".to_string()))
                    .collect()
            }
            "requirements.txt" => {
                Self::parse_requirements_txt(&ctx.source)
                    .into_iter()
                    .map(|(n, v)| (n, v, "pypi".to_string()))
                    .collect()
            }
            "pom.xml" => {
                Self::parse_pom_xml(&ctx.source)
                    .into_iter()
                    .map(|(n, v)| (n, v, "maven".to_string()))
                    .collect()
            }
            _ => Vec::new(),
        }
    }
    
    fn check_violations(&self, ctx: &ValidationContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        
        // Only check dependency files
        let deps = self.detect_and_parse(ctx);
        let file = ctx.file_path.clone();
        
        for (name, version, ecosystem) in deps {
            if let Some(violation) = self.check_dependency(&name, &version, &ecosystem, file.clone()) {
                violations.push(violation);
            }
            
            // Check for unpinned dependencies
            if version == "*" || version.starts_with('^') || version.starts_with('~') || version.starts_with('>') {
                violations.push(Violation {
                    id: "SUPP003".to_string(),
                    message: format!(
                        "LOOSE VERSION CONSTRAINT: {}@{} - pin exact version for reproducibility",
                        name, version
                    ),
                    severity: Severity::Warning,
                    span: Some(Span { line: 1, column: 1 }),
                    file: file.clone(),
                    suggestion: Some("Use exact version: package@1.2.3".to_string()),
                });
            }
        }
        
        // Check for suspicious patterns in source files
        self.check_source_patterns(ctx, &mut violations);
        
        violations
    }
    
    /// Check for supply chain patterns in source code.
    fn check_source_patterns(&self, ctx: &ValidationContext, violations: &mut Vec<Violation>) {
        for (line_num, line) in ctx.source.lines().enumerate() {
            // Check for dynamic requires/imports
            if line.contains("require(") && line.contains("+") {
                violations.push(Violation {
                    id: "SUPP004".to_string(),
                    message: "DYNAMIC REQUIRE: Potential supply chain risk".to_string(),
                    severity: Severity::Warning,
                    span: Some(Span { line: line_num + 1, column: 1 }),
                    file: ctx.file_path.clone(),
                    suggestion: Some("Use static imports for security".to_string()),
                });
            }
            
            // Check for import from URL (Deno/some JS)
            if line.contains("import") && (line.contains("http://") || line.contains("https://")) {
                violations.push(Violation {
                    id: "SUPP005".to_string(),
                    message: "REMOTE IMPORT: Code from unverified URL".to_string(),
                    severity: Severity::Warning,
                    span: Some(Span { line: line_num + 1, column: 1 }),
                    file: ctx.file_path.clone(),
                    suggestion: Some("Use verified packages from registry".to_string()),
                });
            }
            
            // Check for curl | bash pattern in shell
            if (line.contains("curl") || line.contains("wget")) && line.contains("|") && line.contains("bash") {
                violations.push(Violation {
                    id: "SUPP006".to_string(),
                    message: "CURL | BASH: Executing unverified code from network".to_string(),
                    severity: Severity::Error,
                    span: Some(Span { line: line_num + 1, column: 1 }),
                    file: ctx.file_path.clone(),
                    suggestion: Some("Download, verify checksum, then execute".to_string()),
                });
            }
        }
    }
}

impl Default for SupplyChainLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for SupplyChainLayer {
    fn name(&self) -> &str {
        "supply_chain"
    }
    
    fn priority(&self) -> u8 {
        15 // Very early - supply chain is critical
    }
    
    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let violations = self.check_violations(ctx);
        LayerResult {
            passed: violations.iter().all(|v| v.severity != Severity::Error),
            violations,
            infos: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    fn make_ctx(filename: &str, source: &str) -> ValidationContext {
        ValidationContext {
            source: source.to_string(),
            file_path: Some(PathBuf::from(filename)),
            language: "unknown".to_string(),
            metadata: Default::default(),
        }
    }
    
    #[test]
    fn test_parse_cargo_toml() {
        let source = r#"
[dependencies]
serde = "1.0"
tokio = "1.20"
"#;
        let deps = SupplyChainLayer::parse_cargo_toml(source);
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|(n, _)| n == "serde"));
    }
    
    #[test]
    fn test_parse_package_json() {
        let source = r#"
{
  "dependencies": {
    "lodash": "4.17.21",
    "express": "^4.18.0"
  }
}
"#;
        let deps = SupplyChainLayer::parse_package_json(source);
        assert_eq!(deps.len(), 2);
    }
    
    #[tokio::test]
    async fn test_unpinned_warning() {
        let layer = SupplyChainLayer::new();
        let ctx = make_ctx("package.json", r#"
{
  "dependencies": {
    "lodash": "^4.17.0"
  }
}
"#);
        let result = layer.validate(&ctx).await;
        assert!(result.violations.iter().any(|v| v.id == "SUPP003"));
    }
}
