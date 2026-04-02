//! JavaScript Scope Extractor
#![allow(clippy::cognitive_complexity, clippy::too_many_lines)] // Tree-sitter parsing is inherently complex

use crate::scope::{
    ScopeExtractor, ExtractionResult, ExtractionStats,
    ScopeTree, ScopeNode, ScopeKind,
    Symbol, SymbolKind, Reference,
};
use crate::scope::extractor::BaseExtractor;

/// JavaScript-specific scope extractor.
pub struct JavaScriptScopeExtractor {
    base: BaseExtractor,
}

impl JavaScriptScopeExtractor {
    pub fn new() -> Self {
        Self {
            base: BaseExtractor::new("javascript"),
        }
    }
}

impl Default for JavaScriptScopeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeExtractor for JavaScriptScopeExtractor {
    fn language(&self) -> &str {
        self.base.language()
    }

    fn extract(&self, source: &str) -> ExtractionResult {
        use tree_sitter::Parser;

        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_javascript::LANGUAGE.into())
            .expect("Failed to set JavaScript language");

        let tree = parser.parse(source, None);
        let tree = match tree {
            Some(t) => t,
            None => return BaseExtractor::new("javascript").extract(source),
        };
        let root_node = tree.root_node();

        let mut scope_tree = ScopeTree::new(source.len());
        let mut next_id = 1;
        let mut stats = ExtractionStats::default();

        // Process tree
        self.process_node(root_node, source, 0, &mut scope_tree, &mut next_id, &mut stats);

        ExtractionResult {
            tree: scope_tree,
            next_scope_id: next_id,
            stats,
        }
    }

    fn scope_queries(&self) -> Vec<&str> {
        vec![
            "(function_declaration) @scope",
            "(function_expression) @scope",
            "(arrow_function) @scope",
            "(method_definition) @scope",
            "(class_declaration) @scope",
            "(for_statement) @scope",
            "(for_in_statement) @scope",
            "(block_statement) @scope",
            "(catch_clause) @scope",
        ]
    }

    fn definition_queries(&self) -> Vec<&str> {
        vec![
            "(variable_declarator name: (identifier) @def)",
            "(function_declaration name: (identifier) @def)",
            "(class_declaration name: (identifier) @def)",
            "(formal_parameters (identifier) @param)",
            "(formal_parameters (assignment_pattern left: (identifier) @param))",
        ]
    }

    fn reference_queries(&self) -> Vec<&str> {
        vec![
            "(identifier) @ref",
        ]
    }

    fn is_reference_excluded(&self, node_text: &str, _node_type: &str) -> bool {
        // Keywords and built-ins
        matches!(node_text, 
            "function" | "const" | "let" | "var" | "class" | "if" | "else" |
            "for" | "while" | "do" | "switch" | "case" | "default" | "break" |
            "continue" | "return" | "throw" | "try" | "catch" | "finally" |
            "new" | "typeof" | "instanceof" | "in" | "of" | "async" | "await" |
            "import" | "export" | "from" | "as" | "yield"
        )
    }

    fn scope_kind_for_node(&self, node_type: &str) -> ScopeKind {
        match node_type {
            "function_declaration" | "function_expression" | "arrow_function" | "method_definition" => ScopeKind::Function,
            "class_declaration" => ScopeKind::Class,
            "for_statement" | "for_in_statement" | "for_of_statement" => ScopeKind::Loop,
            "catch_clause" => ScopeKind::Block,
            "block_statement" => ScopeKind::Block,
            _ => ScopeKind::Block,
        }
    }
}

impl JavaScriptScopeExtractor {
    fn process_node(
        &self,
        node: tree_sitter::Node,
        source: &str,
        parent_scope_id: usize,
        scope_tree: &mut ScopeTree,
        next_id: &mut usize,
        stats: &mut ExtractionStats,
    ) {
        let scope_kind = self.node_to_scope_kind(node.kind());

        if let Some(kind) = scope_kind {
            let start = node.start_position();
            let end = node.end_position();

            let scope_id = *next_id;
            *next_id += 1;
            stats.scope_count += 1;

            let mut scope = ScopeNode::new(scope_id, kind, Some(parent_scope_id), (start.row, end.row));

            // Extract symbols
            self.extract_symbols(node, source, scope_id, &mut scope, stats);

            // Extract references
            self.extract_references(node, source, scope_id, &mut scope, stats);

            // Add to parent
            if let Some(parent) = scope_tree.get_mut(parent_scope_id) {
                parent.children.push(scope_id);
            }

            // Process children
            self.process_children(node, source, scope_id, scope_tree, next_id, stats);
        } else {
            // No scope here, just process children
            self.process_children(node, source, parent_scope_id, scope_tree, next_id, stats);
        }
    }

    fn process_children(
        &self,
        node: tree_sitter::Node,
        source: &str,
        scope_id: usize,
        scope_tree: &mut ScopeTree,
        next_id: &mut usize,
        stats: &mut ExtractionStats,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.process_node(child, source, scope_id, scope_tree, next_id, stats);
        }
    }

    fn node_to_scope_kind(&self, kind: &str) -> Option<ScopeKind> {
        match kind {
            "function_declaration" | "function_expression" | "arrow_function" | "method_definition" => Some(ScopeKind::Function),
            "class_declaration" | "class_expression" => Some(ScopeKind::Class),
            "for_statement" | "for_in_statement" | "for_of_statement" | "while_statement" | "do_statement" => Some(ScopeKind::Loop),
            "catch_clause" => Some(ScopeKind::Block),
            "program" => Some(ScopeKind::Module),
            _ => None,
        }
    }

    fn extract_symbols(
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
                "variable_declaration" | "lexical_declaration" => {
                    self.extract_var_declarations(child, source, scope_id, scope, stats);
                }
                "function_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                        let start = name_node.start_position();
                        let end = name_node.end_position();
                        scope.add_symbol(Symbol::new(name, SymbolKind::Function, scope_id, (start.row, end.row)));
                        stats.symbol_count += 1;
                    }
                }
                "class_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                        let start = name_node.start_position();
                        let end = name_node.end_position();
                        scope.add_symbol(Symbol::new(name, SymbolKind::Class, scope_id, (start.row, end.row)));
                        stats.symbol_count += 1;
                    }
                }
                "formal_parameters" => {
                    self.extract_parameters(child, source, scope_id, scope, stats);
                }
                _ => {}
            }
        }
    }

    fn extract_var_declarations(
        &self,
        node: tree_sitter::Node,
        source: &str,
        scope_id: usize,
        scope: &mut ScopeNode,
        stats: &mut ExtractionStats,
    ) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    match name_node.kind() {
                        "identifier" => {
                            let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                            let start = name_node.start_position();
                            let end = name_node.end_position();
                            scope.add_symbol(Symbol::new(name, SymbolKind::Variable, scope_id, (start.row, end.row)));
                            stats.symbol_count += 1;
                        }
                        "object_pattern" | "array_pattern" => {
                            self.extract_destructuring(name_node, source, scope_id, scope, stats);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn extract_destructuring(
        &self,
        node: tree_sitter::Node,
        source: &str,
        scope_id: usize,
        scope: &mut ScopeNode,
        stats: &mut ExtractionStats,
    ) {
        let mut cursor = node.walk();
        let mut stack = vec![node];

        while let Some(current) = stack.pop() {
            if current.kind() == "identifier" {
                let name = current.utf8_text(source.as_bytes()).unwrap_or("");
                let start = current.start_position();
                let end = current.end_position();
                scope.add_symbol(Symbol::new(name, SymbolKind::Variable, scope_id, (start.row, end.row)));
                stats.symbol_count += 1;
            } else {
                for child in current.children(&mut cursor) {
                    stack.push(child);
                }
            }
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
            match child.kind() {
                "identifier" => {
                    let name = child.utf8_text(source.as_bytes()).unwrap_or("");
                    let start = child.start_position();
                    let end = child.end_position();
                    scope.add_symbol(Symbol::new(name, SymbolKind::Parameter, scope_id, (start.row, end.row)));
                    stats.symbol_count += 1;
                }
                "assignment_pattern" => {
                    if let Some(left) = child.child_by_field_name("left") {
                        if left.kind() == "identifier" {
                            let name = left.utf8_text(source.as_bytes()).unwrap_or("");
                            let start = left.start_position();
                            let end = left.end_position();
                            scope.add_symbol(Symbol::new(name, SymbolKind::Parameter, scope_id, (start.row, end.row)));
                            stats.symbol_count += 1;
                        }
                    }
                }
                "rest_parameter" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                        let start = name_node.start_position();
                        let end = name_node.end_position();
                        scope.add_symbol(Symbol::new(name, SymbolKind::Parameter, scope_id, (start.row, end.row)));
                        stats.symbol_count += 1;
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_references(
        &self,
        node: tree_sitter::Node,
        source: &str,
        scope_id: usize,
        scope: &mut ScopeNode,
        stats: &mut ExtractionStats,
    ) {
        // Find all identifier usages that aren't definitions
        let mut cursor = node.walk();
        let mut stack = vec![node];

        while let Some(current) = stack.pop() {
            if current.kind() == "identifier" {
                // Check if this is a definition site
                let parent = current.parent();
                let is_definition = parent.is_some_and(|p| {
                    matches!(p.kind(), "variable_declarator" | "formal_parameters" | "function_declaration" | "class_declaration")
                        && p.child_by_field_name("name").is_some_and(|n| n.id() == current.id())
                });

                if !is_definition {
                    let name = current.utf8_text(source.as_bytes()).unwrap_or("");
                    if !self.is_reference_excluded(name, "") {
                        let start = current.start_position();
                        let end = current.end_position();
                        scope.add_reference(Reference::read(name, (start.row, end.row), scope_id));
                        stats.reference_count += 1;
                    }
                }
            }

            for child in current.children(&mut cursor) {
                stack.push(child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_extractor_creation() {
        let extractor = JavaScriptScopeExtractor::new();
        assert_eq!(extractor.language(), "javascript");
    }

    #[test]
    fn test_js_extract_simple() {
        let extractor = JavaScriptScopeExtractor::new();
        let source = r#"
function foo(x) {
    let y = 1;
    let z = 2;
    return x + y;
}
"#;
        let result = extractor.extract(source);
        assert!(result.stats.symbol_count > 0, "Should have symbols");
        assert!(result.stats.reference_count > 0, "Should have references");
    }
}
