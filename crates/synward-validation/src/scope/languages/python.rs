//! Python Scope Extractor
#![allow(clippy::cognitive_complexity, clippy::too_many_lines)] // Tree-sitter parsing is inherently complex

use crate::scope::{
    ScopeExtractor, ExtractionResult, ExtractionStats,
    ScopeTree, ScopeNode, ScopeKind,
    Symbol, SymbolKind, Reference,
};
use crate::scope::extractor::BaseExtractor;

/// Python-specific scope extractor.
pub struct PythonScopeExtractor {
    base: BaseExtractor,
}

impl PythonScopeExtractor {
    pub fn new() -> Self {
        Self {
            base: BaseExtractor::new("python"),
        }
    }
    
    #[allow(dead_code)]
    fn node_to_scope_kind(&self, kind: &str) -> Option<ScopeKind> {
        match kind {
            "function_definition" => Some(ScopeKind::Function),
            "class_definition" => Some(ScopeKind::Class),
            "lambda" => Some(ScopeKind::Closure),
            "list_comprehension" | "dictionary_comprehension" | "generator_expression" => Some(ScopeKind::Block),
            "with_statement" | "for_statement" => Some(ScopeKind::Loop),
            "module" => Some(ScopeKind::Module),
            _ => None,
        }
    }
}

impl Default for PythonScopeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeExtractor for PythonScopeExtractor {
    fn language(&self) -> &str {
        self.base.language()
    }

    fn extract(&self, source: &str) -> ExtractionResult {
        use tree_sitter::Parser;

        // Parse with tree-sitter-python
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to set Python language");

        let tree = parser.parse(source, None);
        let tree = match tree {
            Some(t) => t,
            None => return BaseExtractor::new("python").extract(source),
        };
        let root_node = tree.root_node();

        // Build scope tree
        let mut scope_tree = ScopeTree::new(source.len());
        let mut stats = ExtractionStats::default();
        
        // Extract symbols/references at module level (scope 0)
        // ScopeTree::new() should create the root scope at index 0
        {
            let root = scope_tree.get_mut(0)
                .expect("ScopeTree::new() should create root scope at index 0");
            self.extract_from_node(root_node, source, 0, root, &mut stats);
        }

        ExtractionResult {
            tree: scope_tree,
            next_scope_id: 1,
            stats,
        }
    }

    fn scope_queries(&self) -> Vec<&str> {
        vec!["(function_definition) @scope", "(class_definition) @scope"]
    }

    fn definition_queries(&self) -> Vec<&str> {
        vec!["(identifier) @def"]
    }

    fn reference_queries(&self) -> Vec<&str> {
        vec!["(identifier) @ref"]
    }
}

impl PythonScopeExtractor {
    fn extract_from_node(
        &self,
        node: tree_sitter::Node,
        source: &str,
        scope_id: usize,
        scope: &mut ScopeNode,
        stats: &mut ExtractionStats,
    ) {
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_definition" => {
                    // Add function symbol
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                        let start = name_node.start_position();
                        let end = name_node.end_position();
                        scope.add_symbol(Symbol::new(name, SymbolKind::Function, scope_id, (start.row, end.row)));
                        stats.symbol_count += 1;
                    }
                    // Extract parameters as symbols
                    if let Some(params) = child.child_by_field_name("parameters") {
                        self.extract_parameters(params, source, scope_id, scope, stats);
                    }
                }
                "class_definition" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                        let start = name_node.start_position();
                        let end = name_node.end_position();
                        scope.add_symbol(Symbol::new(name, SymbolKind::Class, scope_id, (start.row, end.row)));
                        stats.symbol_count += 1;
                    }
                }
                "assignment" => {
                    if let Some(left) = child.child_by_field_name("left") {
                        self.extract_assignment_targets(left, source, scope_id, scope, stats);
                    }
                }
                "for_statement" => {
                    if let Some(left) = child.child_by_field_name("left") {
                        self.extract_assignment_targets(left, source, scope_id, scope, stats);
                    }
                }
                "identifier" => {
                    // Check if this is a reference (not a definition)
                    let parent = child.parent();
                    let is_definition = parent.is_some_and(|p| {
                        matches!(p.kind(), "function_definition" | "class_definition")
                            && child.kind() == "name"
                            || p.kind() == "parameters"
                            || p.kind() == "assignment" && (p.child_by_field_name("left") == Some(child))
                            || p.kind() == "for_statement" && (p.child_by_field_name("left") == Some(child))
                    });
                    
                    if !is_definition {
                        let name = child.utf8_text(source.as_bytes()).unwrap_or("");
                        if !name.starts_with("__") && !name.starts_with('_') {
                            let start = child.start_position();
                            let end = child.end_position();
                            scope.add_reference(Reference::read(name, (start.row, end.row), scope_id));
                            stats.reference_count += 1;
                        }
                    }
                }
                _ => {}
            }
            
            // Recurse into children
            self.extract_from_node(child, source, scope_id, scope, stats);
        }
    }

    fn extract_parameters(
        &self,
        node: tree_sitter::Node,
        source: &str,
        scope_id: usize,
        scope: &mut ScopeNode,
        stats: &mut ExtractionStats,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let name = child.utf8_text(source.as_bytes()).unwrap_or("");
                let start = child.start_position();
                let end = child.end_position();
                scope.add_symbol(Symbol::new(name, SymbolKind::Parameter, scope_id, (start.row, end.row)));
                stats.symbol_count += 1;
            } else if child.kind() == "default_parameter" || child.kind() == "typed_parameter" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                    let start = name_node.start_position();
                    let end = name_node.end_position();
                    scope.add_symbol(Symbol::new(name, SymbolKind::Parameter, scope_id, (start.row, end.row)));
                    stats.symbol_count += 1;
                }
            }
        }
    }

    fn extract_assignment_targets(
        &self,
        node: tree_sitter::Node,
        source: &str,
        scope_id: usize,
        scope: &mut ScopeNode,
        stats: &mut ExtractionStats,
    ) {
        match node.kind() {
            "identifier" => {
                let name = node.utf8_text(source.as_bytes()).unwrap_or("");
                let start = node.start_position();
                let end = node.end_position();
                scope.add_symbol(Symbol::new(name, SymbolKind::Variable, scope_id, (start.row, end.row)));
                stats.symbol_count += 1;
            }
            "pattern_list" | "tuple_pattern" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_assignment_targets(child, source, scope_id, scope, stats);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_extractor_creation() {
        let extractor = PythonScopeExtractor::new();
        assert_eq!(extractor.language(), "python");
    }
    
    #[test]
    fn test_python_extract_simple() {
        let extractor = PythonScopeExtractor::new();
        let source = r#"
def foo(x):
    y = 1
    return x + y
"#;
        let result = extractor.extract(source);
        assert!(result.stats.symbol_count > 0, "Should have symbols");
        assert!(result.stats.reference_count > 0, "Should have references");
    }
    
    #[test]
    fn test_scope_layer_detects_unused() {
        use crate::scope::layer::ScopeAnalysisLayer;
        use crate::layer::ValidationLayer;
        
        let layer = ScopeAnalysisLayer::new();
        let source = r#"
def foo():
    x = 1
    y = 2
    return x
"#;
        let ctx = crate::context::ValidationContext::for_file(
            "test.py".to_string(),
            source.to_string(),
            "python".to_string(),
        );
        
        // Run validation (need to use tokio runtime)
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(layer.validate(&ctx));
        
        // Should detect 'y' as unused
        assert!(!result.violations.is_empty(), "Should have violations for unused 'y'");
        
        // Debug: print violations
        for v in &result.violations {
            eprintln!("VIOLATION: {} - {}", v.id, v.message);
        }
    }
}
