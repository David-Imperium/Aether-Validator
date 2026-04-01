//! Scope Extractor trait — Language-specific scope extraction

use crate::scope::ScopeTree;

/// Result of scope extraction.
#[derive(Debug)]
pub struct ExtractionResult {
    /// The built scope tree
    pub tree: ScopeTree,
    /// Next available scope ID
    pub next_scope_id: usize,
    /// Extraction statistics
    pub stats: ExtractionStats,
}

/// Statistics about the extraction.
#[derive(Debug, Clone, Default)]
pub struct ExtractionStats {
    /// Number of scopes found
    pub scope_count: usize,
    /// Number of symbols defined
    pub symbol_count: usize,
    /// Number of references found
    pub reference_count: usize,
    /// Number of unresolved references
    pub unresolved_count: usize,
}

/// Trait for language-specific scope extractors.
///
/// Each language must implement this to provide scope analysis.
/// The extractor uses tree-sitter queries to find:
/// - Scope boundaries (functions, classes, blocks)
/// - Symbol definitions (variables, functions, parameters)
/// - Symbol references (usages)
pub trait ScopeExtractor: Send + Sync {
    /// Get the language name.
    fn language(&self) -> &str;

    /// Extract scopes from source code.
    ///
    /// This is the main entry point. It should:
    /// 1. Parse the source with tree-sitter
    /// 2. Discover all scope boundaries
    /// 3. Extract symbol definitions in each scope
    /// 4. Collect all references
    /// 5. Resolve references to definitions
    fn extract(&self, source: &str) -> ExtractionResult;

    /// Get tree-sitter queries for scope boundaries.
    ///
    /// Returns a list of S-expression queries that match scope nodes.
    /// Example for JavaScript:
    /// ```ignore
    /// [
    ///     "(function_declaration) @scope",      // Named functions
    ///     "(arrow_function) @scope",            // Arrow functions
    ///     "(method_definition) @scope",         // Methods
    ///     "(class_declaration) @scope",         // Classes
    ///     "(block) @scope",                     // Blocks
    /// ]
    /// ```
    fn scope_queries(&self) -> Vec<&str>;

    /// Get tree-sitter queries for symbol definitions.
    ///
    /// Returns queries that match symbol definitions.
    /// Example for JavaScript:
    /// ```ignore
    /// [
    ///     "(variable_declarator name: (identifier) @def)",
    ///     "(function_declaration name: (identifier) @def)",
    ///     "(parameter name: (identifier) @def)",
    /// ]
    /// ```
    fn definition_queries(&self) -> Vec<&str>;

    /// Get tree-sitter queries for symbol references.
    ///
    /// Returns queries that match symbol usages.
    /// Example for JavaScript:
    /// ```ignore
    /// [
    ///     "(identifier) @ref",
    /// ]
    /// ```
    fn reference_queries(&self) -> Vec<&str>;

    /// Check if a node should be excluded from references.
    ///
    /// Some nodes that look like references are actually definitions
    /// or keywords. Override this to filter them out.
    fn is_reference_excluded(&self, _node_text: &str, _node_type: &str) -> bool {
        false
    }

    /// Map a tree-sitter node type to a ScopeKind.
    fn scope_kind_for_node(&self, node_type: &str) -> ScopeKind {
        match node_type {
            "function_declaration" | "function_definition" | "arrow_function" | "method_definition" => ScopeKind::Function,
            "class_declaration" | "class_definition" | "struct_item" | "enum_item" => ScopeKind::Class,
            "block" | "compound_statement" => ScopeKind::Block,
            "for_statement" | "for_expression" | "while_statement" | "loop_expression" => ScopeKind::Loop,
            "closure" | "lambda" | "lambda_expression" => ScopeKind::Closure,
            "module" | "namespace" | "mod_item" => ScopeKind::Namespace,
            _ => ScopeKind::Block,
        }
    }
}

use crate::scope::tree::ScopeKind;

/// Base scope extractor with common functionality.
pub struct BaseExtractor {
    language_name: String,
}

impl BaseExtractor {
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language_name: language.into(),
        }
    }
}

impl ScopeExtractor for BaseExtractor {
    fn language(&self) -> &str {
        &self.language_name
    }

    fn extract(&self, source: &str) -> ExtractionResult {
        // Base implementation returns just root scope
        let tree = ScopeTree::new(source.len());
        ExtractionResult {
            tree,
            next_scope_id: 1,
            stats: ExtractionStats::default(),
        }
    }

    fn scope_queries(&self) -> Vec<&str> {
        Vec::new()
    }

    fn definition_queries(&self) -> Vec<&str> {
        Vec::new()
    }

    fn reference_queries(&self) -> Vec<&str> {
        Vec::new()
    }
}
