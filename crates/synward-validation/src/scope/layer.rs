//! Scope Analysis Layer — ValidationLayer implementation
#![allow(clippy::cognitive_complexity)] // Orchestration layer with multiple branches

use async_trait::async_trait;
use crate::layer::{ValidationLayer, LayerResult};
use crate::context::ValidationContext;
use crate::violation::Violation;
use crate::scope::{ScopeExtractor, ScopeTree, SymbolKind};
use std::collections::HashMap;
use crate::scope::languages::{PythonScopeExtractor, JavaScriptScopeExtractor, TypeScriptScopeExtractor};

/// Scope analysis validation layer.
///
/// Uses tree-sitter based scope extraction to detect:
/// - Unused variables/imports
/// - Undefined references
/// - Variable shadowing
/// - Scope pollution
pub struct ScopeAnalysisLayer {
    /// Extractors per language
    extractors: HashMap<String, Box<dyn ScopeExtractor>>,
}

impl ScopeAnalysisLayer {
    /// Create a new scope analysis layer with default extractors.
    pub fn new() -> Self {
        let mut extractors: HashMap<String, Box<dyn ScopeExtractor>> = HashMap::new();
        
        // Register default extractors
        extractors.insert("python".to_string(), Box::new(PythonScopeExtractor::new()));
        extractors.insert("javascript".to_string(), Box::new(JavaScriptScopeExtractor::new()));
        extractors.insert("typescript".to_string(), Box::new(TypeScriptScopeExtractor::new()));
        extractors.insert("ts".to_string(), Box::new(TypeScriptScopeExtractor::new()));
        
        Self { extractors }
    }

    /// Register an extractor for a language.
    pub fn register(&mut self, language: impl Into<String>, extractor: Box<dyn ScopeExtractor>) {
        self.extractors.insert(language.into(), extractor);
    }

    /// Analyze scope and generate violations.
    fn analyze(&self, source: &str, language: &str) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Get extractor for language
        let extractor = match self.extractors.get(language) {
            Some(e) => e.as_ref(),
            None => return violations, // No extractor for this language
        };

        // Extract scope tree
        let result = extractor.extract(source);
        let tree = &result.tree;

        // Check for unused symbols
        self.check_unused_symbols(tree, &mut violations);

        // Check for undefined references
        self.check_undefined_references(tree, &mut violations);

        // Check for shadowing
        self.check_shadowing(tree, &mut violations);

        violations
    }

    /// Check for unused symbols.
    fn check_unused_symbols(&self, tree: &ScopeTree, violations: &mut Vec<Violation>) {
        tree.walk(|scope| {
            for symbol in &scope.symbols {
                // Skip exported symbols, parameters, and underscore-prefixed
                if symbol.is_exported
                    || symbol.kind == SymbolKind::Parameter
                    || symbol.name.starts_with('_')
                {
                    continue;
                }

                // Count references to this symbol
                let ref_count = self.count_references(tree, &symbol.name, symbol.scope_id);

                if ref_count == 0 {
                    violations.push(Violation::warning(
                        "SCOPE001",
                        format!("Unused {}: {}", symbol_kind_name(&symbol.kind), symbol.name),
                    ).suggest(format!("Remove or prefix with underscore: _{}", symbol.name)));
                }
            }
        });
    }

    /// Check for undefined references.
    fn check_undefined_references(&self, tree: &ScopeTree, violations: &mut Vec<Violation>) {
        tree.walk(|scope| {
            for reference in &scope.references {
                // Try to resolve the reference
                if tree.lookup(reference.scope_id, &reference.symbol_name).is_none() {
                    violations.push(Violation::error(
                        "SCOPE002",
                        format!("Undefined reference: {}", reference.symbol_name),
                    ).suggest("Check spelling or import the symbol"));
                }
            }
        });
    }

    /// Check for variable shadowing.
    fn check_shadowing(&self, tree: &ScopeTree, violations: &mut Vec<Violation>) {
        tree.walk(|scope| {
            for symbol in &scope.symbols {
                // Check if this symbol shadows one in an outer scope
                if let Some(parent_id) = scope.parent_id {
                    if let Some(_outer_symbol) = tree.lookup(parent_id, &symbol.name) {
                        violations.push(Violation::info(
                            "SCOPE003",
                            format!("Variable '{}' shadows outer scope", symbol.name),
                        ).suggest("Use a different name to avoid confusion"));
                    }
                }
            }
        });
    }

    /// Count references to a symbol.
    fn count_references(&self, tree: &ScopeTree, name: &str, definition_scope_id: usize) -> usize {
        let mut count = 0;
        tree.walk(|scope| {
            for reference in &scope.references {
                if reference.symbol_name == name {
                    // Check if this reference can see the definition
                    if self.can_see(scope.id, definition_scope_id, tree) {
                        count += 1;
                    }
                }
            }
        });
        count
    }

    /// Check if a scope can see a definition in another scope.
    fn can_see(&self, from_scope: usize, definition_scope: usize, tree: &ScopeTree) -> bool {
        let mut current = Some(from_scope);
        while let Some(id) = current {
            if id == definition_scope {
                return true;
            }
            current = tree.get(id).and_then(|s| s.parent_id);
        }
        false
    }
}

impl Default for ScopeAnalysisLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationLayer for ScopeAnalysisLayer {
    fn name(&self) -> &str {
        "scope"
    }

    fn priority(&self) -> u8 {
        18 // After AST (15), before semantic (20)
    }

    async fn validate(&self, ctx: &ValidationContext) -> LayerResult {
        let violations = self.analyze(&ctx.source, &ctx.language);
        if violations.is_empty() {
            LayerResult::pass()
        } else {
            LayerResult::fail(violations)
        }
    }
}

/// Get a human-readable name for a symbol kind.
fn symbol_kind_name(kind: &SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Variable => "variable",
        SymbolKind::Function => "function",
        SymbolKind::Class => "class",
        SymbolKind::Struct => "struct",
        SymbolKind::Enum => "enum",
        SymbolKind::Trait => "trait",
        SymbolKind::Module => "module",
        SymbolKind::Constant => "constant",
        SymbolKind::TypeAlias => "type alias",
        SymbolKind::Field => "field",
        SymbolKind::Variant => "variant",
        SymbolKind::Macro => "macro",
        SymbolKind::Import => "import",
        SymbolKind::Parameter => "parameter",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_layer_creation() {
        let layer = ScopeAnalysisLayer::new();
        assert_eq!(layer.name(), "scope");
        assert_eq!(layer.priority(), 18);
    }
}
