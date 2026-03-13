//! Architecture Layer — Dependency and layer boundary validation

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::Violation;

/// Architecture validation layer.
///
/// Checks for:
/// - Circular dependencies
/// - Layer boundary violations
/// - Forbidden imports
/// - Module coupling analysis
pub struct ArchitectureLayer {
    /// Forbidden import patterns
    forbidden_imports: Vec<ForbiddenImport>,
    /// Layer definitions for boundary checking
    layers: Vec<LayerDefinition>,
}

/// A forbidden import pattern.
#[derive(Debug, Clone)]
struct ForbiddenImport {
    from_layer: String,
    pattern: String,
    message: String,
}

/// A layer definition for boundary checking.
#[derive(Debug, Clone)]
struct LayerDefinition {
    name: String,
    allowed_deps: Vec<String>,
}

impl ArchitectureLayer {
    /// Create a new architecture layer with default rules.
    pub fn new() -> Self {
        Self {
            forbidden_imports: Self::default_forbidden_imports(),
            layers: Self::default_layers(),
        }
    }

    /// Create an architecture layer with custom rules.
    pub fn with_rules(
        forbidden_imports: Vec<(String, String, String)>,
        layers: Vec<(String, Vec<String>)>,
    ) -> Self {
        Self {
            forbidden_imports: forbidden_imports
                .into_iter()
                .map(|(from, pattern, msg)| ForbiddenImport {
                    from_layer: from,
                    pattern,
                    message: msg,
                })
                .collect(),
            layers: layers
                .into_iter()
                .map(|(name, deps)| LayerDefinition {
                    name,
                    allowed_deps: deps,
                })
                .collect(),
        }
    }

    fn default_forbidden_imports() -> Vec<ForbiddenImport> {
        vec![
            // UI should not import database directly
            ForbiddenImport {
                from_layer: "ui".into(),
                pattern: "sqlx::".into(),
                message: "UI layer should not access database directly".into(),
            },
            ForbiddenImport {
                from_layer: "ui".into(),
                pattern: "diesel::".into(),
                message: "UI layer should not access database directly".into(),
            },
            // Domain should not import external frameworks
            ForbiddenImport {
                from_layer: "domain".into(),
                pattern: "actix_web".into(),
                message: "Domain layer should not depend on web frameworks".into(),
            },
            ForbiddenImport {
                from_layer: "domain".into(),
                pattern: "rocket".into(),
                message: "Domain layer should not depend on web frameworks".into(),
            },
            // Test code in production
            ForbiddenImport {
                from_layer: "*".into(),
                pattern: "#[cfg(test)]".into(),
                message: "Test code should not be in production files".into(),
            },
        ]
    }

    fn default_layers() -> Vec<LayerDefinition> {
        vec![
            LayerDefinition {
                name: "ui".into(),
                allowed_deps: vec!["application".into(), "domain".into()],
            },
            LayerDefinition {
                name: "application".into(),
                allowed_deps: vec!["domain".into(), "infrastructure".into()],
            },
            LayerDefinition {
                name: "domain".into(),
                allowed_deps: vec![],
            },
            LayerDefinition {
                name: "infrastructure".into(),
                allowed_deps: vec!["domain".into()],
            },
        ]
    }

    /// Detect the layer from file path.
    #[allow(dead_code)]
    fn detect_layer(&self, ctx: &ValidationContext) -> Option<String> {
        let path = ctx.file_path.as_ref()?.to_str()?;
        
        // Common layer patterns
        if path.contains("/ui/") || path.contains("/presentation/") || path.contains("/handlers/") {
            return Some("ui".into());
        }
        if path.contains("/application/") || path.contains("/services/") || path.contains("/usecases/") {
            return Some("application".into());
        }
        if path.contains("/domain/") || path.contains("/entities/") || path.contains("/models/") {
            return Some("domain".into());
        }
        if path.contains("/infrastructure/") || path.contains("/persistence/") || path.contains("/db/") {
            return Some("infrastructure".into());
        }
        
        None
    }
}

impl Default for ArchitectureLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for ArchitectureLayer {
    fn name(&self) -> &str {
        "architecture"
    }

    fn priority(&self) -> u8 {
        40 // Fourth layer (after syntax, semantic, logic)
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let mut violations = Vec::new();
        let source = &ctx.source;

        // Check for circular dependencies
        check_circular_dependencies(source, &mut violations);

        // Check for forbidden imports
        check_forbidden_imports(source, ctx, &self.forbidden_imports, &mut violations);

        // Check for layer boundary violations
        check_layer_boundaries(source, ctx, &self.layers, &mut violations);

        // Check for excessive coupling
        check_coupling(source, &mut violations);

        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }
}

fn check_circular_dependencies(source: &str, violations: &mut Vec<Violation>) {
    // Extract module imports
    let imports = extract_imports(source);
    
    // Check for self-imports (importing from same module)
    for import in &imports {
        if let Some(module) = extract_current_module(source) {
            if import.starts_with(&module) && import.as_str() != module {
                violations.push(Violation::warning(
                    "ARCH001",
                    format!("Potential circular dependency: {} imports from same module tree", import),
                ).suggest("Consider restructuring to avoid circular imports"));
            }
        }
    }
    
    // Check for obvious cycles in use statements
    // A -> B, B -> A patterns (simplified)
    let mut import_set = std::collections::HashSet::new();
    for import in &imports {
        if import_set.contains(import) {
            violations.push(Violation::warning(
                "ARCH002",
                format!("Duplicate import: {}", import),
            ).suggest("Remove duplicate imports"));
        }
        import_set.insert(import.clone());
    }
}

fn check_forbidden_imports(
    source: &str,
    ctx: &ValidationContext,
    forbidden: &[ForbiddenImport],
    violations: &mut Vec<Violation>,
) {
    for rule in forbidden {
        if source.contains(&rule.pattern) {
            // Check if this rule applies to the current layer
            let applies = rule.from_layer == "*" 
                || ctx.file_path.as_ref()
                    .and_then(|p| p.to_str())
                    .map(|p| p.contains(&rule.from_layer))
                    .unwrap_or(false);
            
            if applies {
                violations.push(Violation::error(
                    "ARCH003",
                    &rule.message,
                ).suggest("Use dependency injection or a service layer instead"));
            }
        }
    }
}

fn check_layer_boundaries(
    source: &str,
    ctx: &ValidationContext,
    layers: &[LayerDefinition],
    violations: &mut Vec<Violation>,
) {
    // Try to detect current layer
    let current_layer = detect_file_layer(ctx);
    
    if let Some(ref current) = current_layer {
        // Find layer definition
        let layer_def = layers.iter().find(|l| &l.name == current);
        
        if let Some(def) = layer_def {
            let imports = extract_imports(source);
            
            for import in imports {
                // Check if import violates layer boundaries
                // For now, just warn about potentially problematic imports
                if is_external_crate(&import) && !def.allowed_deps.is_empty() {
                    // This is a simplified check; real implementation would map imports to layers
                }
            }
        }
    }
    
    // Check for common layer violations
    let imports = extract_imports(source);
    
    // Domain should not import infrastructure
    if current_layer.as_deref() == Some("domain") {
        for import in imports {
            if import.starts_with("infrastructure") 
                || import.contains("::db::") 
                || import.contains("::persistence::") {
                violations.push(Violation::error(
                    "ARCH004",
                    "Domain layer importing infrastructure",
                ).suggest("Domain should depend on abstractions, not implementations"));
            }
        }
    }
}

fn check_coupling(source: &str, violations: &mut Vec<Violation>) {
    let imports = extract_imports(source);
    
    // Warn about files with many imports (high coupling)
    if imports.len() > 20 {
        violations.push(Violation::warning(
            "ARCH005",
            format!("High coupling detected: {} imports", imports.len()),
        ).suggest("Consider splitting into smaller modules or using facade patterns"));
    }
    
    // Warn about importing everything from a module
    for line in source.lines() {
        if line.contains("use ") && line.contains("::*") {
            violations.push(Violation::info(
                "ARCH006",
                "Wildcard import may increase coupling",
            ).suggest("Import only needed items explicitly"));
        }
    }
}

fn extract_imports(source: &str) -> Vec<String> {
    let mut imports = Vec::new();
    
    for line in source.lines() {
        let trimmed = line.trim();
        
        // Rust use statements
        if trimmed.starts_with("use ") {
            let import = trimmed
                .strip_prefix("use ")
                .unwrap_or("")
                .trim_end_matches(';')
                .trim();
            imports.push(import.to_string());
        }
        
        // Python import statements
        if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
            imports.push(trimmed.to_string());
        }
        
        // JavaScript/TypeScript imports
        if trimmed.starts_with("import ") && trimmed.contains("from ") {
            imports.push(trimmed.to_string());
        }
    }
    
    imports
}

fn extract_current_module(source: &str) -> Option<String> {
    // Try to extract module declaration
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub mod ") || trimmed.starts_with("mod ") {
            let module = trimmed
                .split_whitespace()
                .nth(1)?
                .trim_end_matches(';')
                .trim_end_matches('{')
                .to_string();
            return Some(module);
        }
    }
    None
}

fn detect_file_layer(ctx: &ValidationContext) -> Option<String> {
    let path = ctx.file_path.as_ref()?.to_str()?;
    
    if path.contains("/ui/") || path.contains("/presentation/") || path.contains("/handlers/") {
        return Some("ui".into());
    }
    if path.contains("/application/") || path.contains("/services/") || path.contains("/usecases/") {
        return Some("application".into());
    }
    if path.contains("/domain/") || path.contains("/entities/") || path.contains("/models/") {
        return Some("domain".into());
    }
    if path.contains("/infrastructure/") || path.contains("/persistence/") || path.contains("/db/") {
        return Some("infrastructure".into());
    }
    
    None
}

fn is_external_crate(import: &str) -> bool {
    // Common external crates
    let external = [
        "std::", "core::",
        "serde", "tokio", "actix", "rocket", "axum",
        "sqlx", "diesel", "postgres",
        "reqwest", "hyper", "tower",
        "chrono", "uuid", "regex",
    ];
    
    external.iter().any(|&e| import.starts_with(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_violations() {
        let source = r#"
use std::collections::HashMap;

fn main() {
    let map = HashMap::new();
}
"#;
        let ctx = ValidationContext::for_file("src/main.rs", source.into(), "rust".into());
        let layer = ArchitectureLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_wildcard_import() {
        let source = r#"
use std::collections::*;
"#;
        let ctx = ValidationContext::for_file("src/main.rs", source.into(), "rust".into());
        let layer = ArchitectureLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(!result.violations.is_empty());
        assert!(result.violations.iter().any(|v| v.id == "ARCH006"));
    }

    #[tokio::test]
    async fn test_high_coupling() {
        let mut source = String::new();
        for i in 0..25 {
            source.push_str(&format!("use crate{}::Something;\n", i));
        }
        
        let ctx = ValidationContext::for_file("src/main.rs", source.into(), "rust".into());
        let layer = ArchitectureLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.violations.iter().any(|v| v.id == "ARCH005"));
    }

    #[tokio::test]
    async fn test_domain_importing_infrastructure() {
        let source = r#"
use infrastructure::Database;

fn main() {}
"#;
        let ctx = ValidationContext::for_file("src/domain/user.rs", source.into(), "rust".into());
        let layer = ArchitectureLayer::new();
        let result = layer.validate(&ctx).await;
        assert!(result.violations.iter().any(|v| v.id == "ARCH004"));
    }
}
