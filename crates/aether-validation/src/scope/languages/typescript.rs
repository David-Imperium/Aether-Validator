//! TypeScript Scope Extractor
#![allow(clippy::cognitive_complexity, clippy::too_many_lines)] // Tree-sitter parsing is inherently complex
//!
//! Extends JavaScript extractor with TypeScript-specific features:
//! - Type annotations
//! - Interfaces
//! - Enums
//! - Type aliases
//! - Generic parameters

use crate::scope::{
    ScopeExtractor, ExtractionResult, ExtractionStats,
    ScopeTree, ScopeNode, ScopeKind,
    Symbol, SymbolKind, Reference, ReferenceKind,
};
use crate::scope::extractor::BaseExtractor;
use crate::scope::languages::javascript::JavaScriptScopeExtractor;

/// TypeScript-specific scope extractor.
pub struct TypeScriptScopeExtractor {
    base: BaseExtractor,
    js: JavaScriptScopeExtractor,
}

impl TypeScriptScopeExtractor {
    pub fn new() -> Self {
        Self {
            base: BaseExtractor::new("typescript"),
            js: JavaScriptScopeExtractor::new(),
        }
    }
}

impl Default for TypeScriptScopeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeExtractor for TypeScriptScopeExtractor {
    fn language(&self) -> &str {
        self.base.language()
    }

    fn extract(&self, source: &str) -> ExtractionResult {
        use tree_sitter::Parser;

        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .expect("Failed to set TypeScript language");

        let tree = parser.parse(source, None);
        let tree = match tree {
            Some(t) => t,
            None => return BaseExtractor::new("typescript").extract(source),
        };
        let root_node = tree.root_node();

        let mut scope_tree = ScopeTree::new(source.len());
        let mut next_id = 1;
        let mut stats = ExtractionStats::default();

        // Process tree with TS-specific handling
        self.process_node(root_node, source, 0, &mut scope_tree, &mut next_id, &mut stats);

        ExtractionResult {
            tree: scope_tree,
            next_scope_id: next_id,
            stats,
        }
    }

    fn scope_queries(&self) -> Vec<&str> {
        let mut queries = self.js.scope_queries();
        queries.extend(vec![
            "(interface_declaration) @scope",
            "(enum_declaration) @scope",
            "(namespace_export_declaration) @scope",
            "(module) @scope",
        ]);
        queries
    }

    fn definition_queries(&self) -> Vec<&str> {
        let mut queries = self.js.definition_queries();
        queries.extend(vec![
            "(interface_declaration name: (type_identifier) @def)",
            "(enum_declaration name: (identifier) @def)",
            "(type_alias_declaration name: (type_identifier) @def)",
            "(type_parameter) @tparam",
            "(property_signature name: (property_identifier) @def)",
            "(enum_assignment name: (identifier) @def)",
        ]);
        queries
    }

    fn reference_queries(&self) -> Vec<&str> {
        // TypeScript uses both identifier and type_identifier
        vec![
            "(identifier) @ref",
            "(type_identifier) @ref",
        ]
    }

    fn is_reference_excluded(&self, node_text: &str, node_type: &str) -> bool {
        // TypeScript keywords
        matches!(node_text,
            "interface" | "type" | "enum" | "namespace" | "module" |
            "public" | "private" | "protected" | "readonly" | "abstract" |
            "implements" | "extends" | "declare" | "keyof" | "infer" |
            "never" | "unknown" | "any" | "void" | "null" | "undefined"
        ) || self.js.is_reference_excluded(node_text, node_type)
    }

    fn scope_kind_for_node(&self, node_type: &str) -> ScopeKind {
        match node_type {
            "interface_declaration" => ScopeKind::Class,
            "enum_declaration" => ScopeKind::Class,
            "namespace_export_declaration" | "module" => ScopeKind::Namespace,
            _ => self.js.scope_kind_for_node(node_type),
        }
    }
}

impl TypeScriptScopeExtractor {
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
            "interface_declaration" | "enum_declaration" => Some(ScopeKind::Class),
            "namespace_export_declaration" | "module" => Some(ScopeKind::Namespace),
            "program" => Some(ScopeKind::Module),
            // Delegate to JS for common cases
            "function_declaration" | "function_expression" | "arrow_function" | "method_definition" => Some(ScopeKind::Function),
            "class_declaration" | "class_expression" => Some(ScopeKind::Class),
            "for_statement" | "for_in_statement" | "for_of_statement" => Some(ScopeKind::Loop),
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
                // TypeScript-specific
                "interface_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                        let start = name_node.start_position();
                        let end = name_node.end_position();
                        scope.add_symbol(Symbol::new(name, SymbolKind::Trait, scope_id, (start.row, end.row)));
                        stats.symbol_count += 1;
                    }
                }
                "enum_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                        let start = name_node.start_position();
                        let end = name_node.end_position();
                        scope.add_symbol(Symbol::new(name, SymbolKind::Enum, scope_id, (start.row, end.row)));
                        stats.symbol_count += 1;
                    }
                    // Extract enum members
                    self.extract_enum_members(child, source, scope_id, scope, stats);
                }
                "type_alias_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                        let start = name_node.start_position();
                        let end = name_node.end_position();
                        scope.add_symbol(Symbol::new(name, SymbolKind::TypeAlias, scope_id, (start.row, end.row)));
                        stats.symbol_count += 1;
                    }
                }
                "type_parameters" => {
                    self.extract_type_parameters(child, source, scope_id, scope, stats);
                }
                // Delegate to JS patterns
                "lexical_declaration" | "variable_declaration" => {
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

    fn extract_type_parameters(
        &self,
        node: tree_sitter::Node,
        source: &str,
        scope_id: usize,
        scope: &mut ScopeNode,
        stats: &mut ExtractionStats,
    ) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "type_parameter" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                    let start = name_node.start_position();
                    let end = name_node.end_position();
                    scope.add_symbol(Symbol::new(name, SymbolKind::TypeAlias, scope_id, (start.row, end.row)));
                    stats.symbol_count += 1;
                }
            }
        }
    }

    fn extract_enum_members(
        &self,
        node: tree_sitter::Node,
        source: &str,
        scope_id: usize,
        scope: &mut ScopeNode,
        stats: &mut ExtractionStats,
    ) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "enum_body" {
                let mut body_cursor = child.walk();
                for member in child.children(&mut body_cursor) {
                    if member.kind() == "enum_assignment" || member.kind() == "property_identifier" {
                        let name = member.utf8_text(source.as_bytes()).unwrap_or("");
                        let start = member.start_position();
                        let end = member.end_position();
                        scope.add_symbol(Symbol::new(name, SymbolKind::Variant, scope_id, (start.row, end.row)));
                        stats.symbol_count += 1;
                    }
                }
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
            if current.kind() == "identifier" || current.kind() == "shorthand_property_identifier_pattern" {
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
                "required_parameter" | "optional_parameter" | "rest_parameter" => {
                    if let Some(pattern) = child.child_by_field_name("pattern") {
                        if pattern.kind() == "identifier" {
                            let name = pattern.utf8_text(source.as_bytes()).unwrap_or("");
                            let start = pattern.start_position();
                            let end = pattern.end_position();
                            scope.add_symbol(Symbol::new(name, SymbolKind::Parameter, scope_id, (start.row, end.row)));
                            stats.symbol_count += 1;
                        }
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
        let mut cursor = node.walk();
        let mut stack = vec![node];

        while let Some(current) = stack.pop() {
            match current.kind() {
                "identifier" | "type_identifier" => {
                    // Check if definition site
                    let parent = current.parent();
                    let is_definition = parent.is_some_and(|p| {
                        let def_fields = ["name", "pattern", "alias"];
                        def_fields.iter().any(|&f| {
                            p.child_by_field_name(f).is_some_and(|n| n.id() == current.id())
                        })
                    });

                    if !is_definition {
                        let name = current.utf8_text(source.as_bytes()).unwrap_or("");
                        if !self.is_reference_excluded(name, "") {
                            let start = current.start_position();
                            let end = current.end_position();
                            scope.add_reference(Reference::new(
                                name,
                                if current.kind() == "type_identifier" {
                                    ReferenceKind::TypeUse
                                } else {
                                    ReferenceKind::Read
                                },
                                (start.row, end.row),
                                scope_id
                            ));
                            stats.reference_count += 1;
                        }
                    }
                }
                _ => {}
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
    fn test_ts_extractor_creation() {
        let extractor = TypeScriptScopeExtractor::new();
        assert_eq!(extractor.language(), "typescript");
    }

    #[test]
    fn test_ts_extract_simple() {
        let extractor = TypeScriptScopeExtractor::new();
        let source = r#"
function foo(x: number): number {
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
