//! CPG Builder — Constructs a CodePropertyGraph from source code.
//!
//! Pipeline:
//! 1. Parse source with tree-sitter → full AST
//! 2. Walk AST → create CPG nodes (functions, variables, expressions, etc.)
//! 3. Build CFG edges (sequential flow, branches, loops)
//! 4. Build DFG edges (variable def-use chains)
//! 5. Mark entry points

use super::types::*;
#[cfg(feature = "tree-sitter")]
use crate::tree_sitter_parser::Language;

// Dummy Language type for non-tree-sitter fallback compilation.
#[cfg(not(feature = "tree-sitter"))]
#[derive(Debug, Clone, Copy)]
enum Language {
    Rust,
}

// ---------------------------------------------------------------------------
// Node creation helpers
// ---------------------------------------------------------------------------

/// Maps tree-sitter kind strings to CPG node types.
fn kind_to_cpg_node_type(kind: &str) -> Option<CPGNodeType> {
    // Function-like
    if kind.contains("function") || kind == "method" || kind == "closure_expression" {
        return Some(CPGNodeType::Function);
    }
    // Struct/enum/trait declarations
    match kind {
        "struct_item" | "struct_expression" => Some(CPGNodeType::Struct),
        "enum_item" => Some(CPGNodeType::Enum),
        "trait_item" => Some(CPGNodeType::Trait),
        "impl_item" => Some(CPGNodeType::Impl),
        "field_identifier" | "field_declaration" => Some(CPGNodeType::Field),
        // Variables and parameters
        "identifier" => Some(CPGNodeType::Variable),
        "parameter" | "self_parameter" => Some(CPGNodeType::Parameter),
        "let_declaration" | "let_statement" => Some(CPGNodeType::Variable),
        // Expressions
        "binary_expression" => Some(CPGNodeType::BinaryOp),
        "unary_expression" | "negated_expression" => Some(CPGNodeType::UnaryOp),
        "call_expression" | "method_call_expression" | "macro_invocation" => Some(CPGNodeType::Call),
        "integer_literal" | "float_literal" | "string_literal" | "char_literal"
        | "true" | "false" | "nil" => Some(CPGNodeType::Literal),
        // Control flow
        "if_expression" | "match_expression" => Some(CPGNodeType::Branch),
        "for_expression" | "while_expression" | "loop_expression" => Some(CPGNodeType::Loop),
        "return_expression" => Some(CPGNodeType::Return),
        "block" | "statement_block" => Some(CPGNodeType::Block),
        // Module level
        "module" | "source_file" => Some(CPGNodeType::Module),
        // Assignment
        "assignment_expression" => Some(CPGNodeType::Variable),
        _ => None,
    }
}

/// Maps tree-sitter kind to CFG-relevant branch/loop types.
fn is_cfg_branch(kind: &str) -> bool {
    matches!(kind, "if_expression" | "match_expression")
}

fn is_cfg_loop(kind: &str) -> bool {
    matches!(kind, "for_expression" | "while_expression" | "loop_expression")
}

// ---------------------------------------------------------------------------
// Scope tracker for DFG (variable def-use chains)
// ---------------------------------------------------------------------------

/// Tracks variable definitions per scope for DFG construction.
struct ScopeStack {
    /// Each scope level maps variable name → node id where defined.
    scopes: Vec<std::collections::HashMap<String, usize>>,
}

impl ScopeStack {
    fn new() -> Self {
        Self {
            scopes: vec![std::collections::HashMap::new()],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(std::collections::HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a variable. Returns the previous definition node id if redefined.
    fn define(&mut self, name: &str, node_id: usize) -> Option<usize> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), node_id)
        } else {
            None
        }
    }

    /// Look up a variable's most recent definition across all scopes.
    fn lookup(&self, name: &str) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(&id) = scope.get(name) {
                return Some(id);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// CPG Builder
// ---------------------------------------------------------------------------

/// Builds a CodePropertyGraph from source code using tree-sitter.
///
/// The builder is feature-gated behind `feature = "tree-sitter"` and only
/// compiled when tree-sitter support is available.
pub struct CPGBuilder;

#[cfg(feature = "tree-sitter")]
use tree_sitter::{Node as TsNode, Parser, TreeCursor};

#[cfg(feature = "tree-sitter")]
impl CPGBuilder {
    /// Build a CPG from source code.
    ///
    /// `file_path` is used only for node metadata (not for reading files).
    pub fn build_from_source(
        source: &str,
        file_path: &str,
        language: Language,
    ) -> anyhow::Result<CodePropertyGraph> {
        let mut parser = Parser::new();
        parser.set_language(&Self::ts_language(language))?;

        let tree = parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("tree-sitter parse failed for {}", file_path))?;

        let mut cpg = CodePropertyGraph::new();
        let mut next_id: usize = 0;
        let mut scope_stack = ScopeStack::new();
        let lines: Vec<&str> = source.lines().collect();
        let max_lines = lines.len().max(1);

        let root = tree.root_node();

        // Walk the AST recursively to create nodes and AST edges.
        Self::walk_node(
            &root,
            &mut cpg,
            &mut next_id,
            &mut scope_stack,
            file_path,
            &lines,
            max_lines,
            0,
            None,
        );

        // Post-build: compute graph-level features (indices 26-33).
        Self::compute_graph_features(&mut cpg);

        Ok(cpg)
    }

    fn ts_language(language: Language) -> tree_sitter::Language {
        match language {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            #[cfg(feature = "tree-sitter-multi")]
            Language::Python => tree_sitter_python::LANGUAGE.into(),
            #[cfg(feature = "tree-sitter-multi")]
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            #[cfg(feature = "tree-sitter-multi")]
            Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            #[cfg(feature = "tree-sitter-multi")]
            Language::Go => tree_sitter_go::LANGUAGE.into(),
            #[cfg(feature = "tree-sitter-multi")]
            Language::Java => tree_sitter_java::LANGUAGE.into(),
            #[cfg(feature = "tree-sitter-multi")]
            Language::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            #[cfg(not(feature = "tree-sitter-multi"))]
            lang => unimplemented!("Language {:?} requires feature 'tree-sitter-multi'", lang),
        }
    }

    /// Recursively walk a tree-sitter node, creating CPG nodes and edges.
    ///
    /// Feature vector layout (34 dimensions):
    ///   [0..17]  — one-hot CPGNodeType
    ///   [18]     — scope_depth (normalized)
    ///   [19]     — local cyclomatic complexity (normalized)
    ///   [20..26] — walk-time features (computed during AST walk)
    ///   [26..34] — graph-level features (computed post-build, initialized to 0.0)
    fn walk_node(
        node: &TsNode,
        cpg: &mut CodePropertyGraph,
        next_id: &mut usize,
        scope_stack: &mut ScopeStack,
        file_path: &str,
        lines: &[&str],
        max_lines: usize,
        scope_depth: u32,
        parent_kind: Option<&str>,
    ) {
        let kind = node.kind();
        let start_row = node.start_position().row;
        let end_row = node.end_position().row;

        // Filter: only create nodes for interesting constructs.
        let node_type = match kind_to_cpg_node_type(kind) {
            Some(t) => t,
            None => {
                // Still recurse into children for interesting sub-nodes.
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    Self::walk_node(&child, cpg, next_id, scope_stack, file_path, lines, max_lines, scope_depth, Some(kind));
                }
                return;
            }
        };

        // --- Name and type inference ---
        let name = Self::extract_name(node, lines);
        let node_name = name.clone().unwrap_or_else(|| format!("<{}>", kind));
        let data_type = Self::infer_type(node, lines);

        // --- Build 34-dim feature vector ---
        // [0..17] one-hot CPGNodeType
        let mut features = vec![0.0f32; 34];
        features[node_type.encoding()] = 1.0;

        // [18] scope_depth normalized
        features[18] = scope_depth as f32 / 20.0;

        // [19] local cyclomatic complexity normalized
        features[19] = Self::local_complexity(node) as f32 / 10.0;

        // [20] lines_of_code normalized
        let loc = (end_row - start_row + 1) as f32;
        features[20] = loc / 100.0;

        // [21] child_count normalized
        let child_count = node.child_count() as f32;
        features[21] = child_count / 20.0;

        // [22] name_length normalized (log scale)
        let name_len = node_name.len() as f32;
        features[22] = (1.0 + name_len).ln() / 5.0;

        // [23] is_in_control_flow: 1.0 if parent is a branch or loop
        let in_cf = parent_kind.map_or(0.0, |pk| {
            if is_cfg_branch(pk) || is_cfg_loop(pk) { 1.0 } else { 0.0 }
        });
        features[23] = in_cf;

        // [24] has_block_body: 1.0 if node has a block child
        let has_block = Self::has_block_child(node);
        features[24] = if has_block { 1.0 } else { 0.0 };

        // [25] start_line_normalized
        features[25] = start_row as f32 / max_lines as f32;

        // [26..33] graph-level features — initialized to 0.0, filled by compute_graph_features()

        let id = *next_id;
        *next_id += 1;

        cpg.addNode(CPGNode {
            id,
            name: node_name,
            node_type,
            file_path: file_path.to_string(),
            start_line: start_row + 1,
            end_line: end_row + 1,
            scope_depth,
            data_type,
            features,
        });

        // Track variable definitions for DFG.
        if matches!(node_type, CPGNodeType::Parameter | CPGNodeType::Variable) {
            if let Some(ref var_name) = name {
                scope_stack.define(var_name, id);
            }
        }

        // Add DFG edge: variable use → its definition.
        if node_type == CPGNodeType::Variable {
            if let Some(ref var_name) = name {
                if let Some(def_id) = scope_stack.lookup(var_name) {
                    if def_id != id {
                        cpg.addEdge(def_id, id, CPGEdgeType::DfgDefUse);
                    }
                }
            }
        }

        // Recurse into children.
        scope_stack.push_scope();
        let mut cursor = node.walk();
        let mut prev_child_id: Option<usize> = None;

        for child in node.children(&mut cursor) {
            Self::walk_node(&child, cpg, next_id, scope_stack, file_path, lines, max_lines, scope_depth + 1, Some(kind));

            // AST edge: parent → child
            let child_node_id = *next_id - 1;
            cpg.addEdge(id, child_node_id, CPGEdgeType::AstChild);

            // CFG edge: sequential flow between siblings.
            if let Some(prev_id) = prev_child_id {
                cpg.addEdge(prev_id, child_node_id, CPGEdgeType::CfgNext);
            }

            // CFG branch edges.
            if is_cfg_branch(child.kind()) {
                Self::add_branch_edges(&child, cpg, child_node_id);
            }
            if is_cfg_loop(child.kind()) {
                Self::add_loop_edges(&child, cpg, child_node_id);
            }

            prev_child_id = Some(child_node_id);
        }

        scope_stack.pop_scope();

        // Mark entry points: top-level functions, pub fn, main.
        if node_type == CPGNodeType::Function && scope_depth == 0 {
            cpg.markEntryPoint(id);
        }
    }

    /// Extract a meaningful name from a tree-sitter node.
    fn extract_name(node: &TsNode, lines: &[&str]) -> Option<String> {
        // Try "name" child first (most function/struct definitions).
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier"
                || child.kind() == "field_identifier"
                || child.kind() == "type_identifier"
            {
                let start = child.start_byte();
                let end = child.end_byte();
                if let Some(line) = lines.get(child.start_position().row) {
                    if start <= line.len() && end <= line.len() {
                        return Some(line[start..end].to_string());
                    }
                }
            }
        }
        None
    }

    /// Infer a rough type annotation from the node's context.
    fn infer_type(_node: &TsNode, _lines: &[&str]) -> Option<String> {
        // Placeholder: full type inference requires scope-resolved type checker.
        // For now, return None. Will be implemented via heuristic patterns.
        None
    }

    /// Compute local cyclomatic complexity (branch count + 1).
    fn local_complexity(node: &TsNode) -> u32 {
        let mut complexity = 1u32;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "if_expression" | "match_expression"
                | "for_expression" | "while_expression" | "loop_expression"
                | "&&" | "||" => {
                    complexity += 1;
                }
                _ => {}
            }
        }
        complexity
    }

    /// Add CFG branch edges for an if/match node.
    fn add_branch_edges(node: &TsNode, cpg: &mut CodePropertyGraph, node_id: usize) {
        let mut cursor = node.walk();
        let mut i = 0usize;
        for child in node.children(&mut cursor) {
            match (child.kind(), i) {
                // First block-like child after condition → true branch
                ("block" | "statement_block", 0..=2) => {
                    let child_id = node_id + 1 + i;
                    cpg.addEdge(node_id, child_id, CPGEdgeType::CfgBranchTrue);
                }
                // "else" keyword followed by block → false branch
                ("else_clause", _) => {
                    let child_id = node_id + 1 + i;
                    cpg.addEdge(node_id, child_id, CPGEdgeType::CfgBranchFalse);
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// Add CFG loop-back edges for a loop node.
    fn add_loop_edges(node: &TsNode, cpg: &mut CodePropertyGraph, node_id: usize) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "block" || child.kind() == "statement_block" {
                let child_id = node_id + 1;
                cpg.addEdge(child_id, node_id, CPGEdgeType::CfgLoopBack);
                break;
            }
        }
    }

    /// Check if a tree-sitter node has a block/statement_block child.
    fn has_block_child(node: &TsNode) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "block" || child.kind() == "statement_block" {
                return true;
            }
        }
        false
    }

    /// Post-build: compute graph-level features for each node (indices 26-33).
    ///
    /// Requires the full graph to be built first so that edge statistics are available.
    fn compute_graph_features(cpg: &mut CodePropertyGraph) {
        if cpg.nodeCount() == 0 {
            return;
        }

        // Compute per-node degree counts.
        let mut in_degree: Vec<usize> = vec![0; cpg.nodeCount()];
        let mut out_degree: Vec<usize> = vec![0; cpg.nodeCount()];
        let mut has_cfg: Vec<bool> = vec![false; cpg.nodeCount()];
        let mut has_dfg: Vec<bool> = vec![false; cpg.nodeCount()];
        let mut has_call: Vec<bool> = vec![false; cpg.nodeCount()];

        for edge in cpg.edges() {
            let src_idx = *cpg.node_index.get(&edge.source).unwrap_or(&0);
            let dst_idx = *cpg.node_index.get(&edge.target).unwrap_or(&0);
            if src_idx < out_degree.len() {
                out_degree[src_idx] += 1;
            }
            if dst_idx < in_degree.len() {
                in_degree[dst_idx] += 1;
            }
            match edge.edge_type {
                CPGEdgeType::CfgNext | CPGEdgeType::CfgBranchTrue
                | CPGEdgeType::CfgBranchFalse | CPGEdgeType::CfgLoopBack => {
                    has_cfg[src_idx] = true;
                    has_cfg[dst_idx] = true;
                }
                CPGEdgeType::DfgDefUse => {
                    has_dfg[src_idx] = true;
                    has_dfg[dst_idx] = true;
                }
                CPGEdgeType::Call | CPGEdgeType::CallerOf => {
                    has_call[src_idx] = true;
                    has_call[dst_idx] = true;
                }
                CPGEdgeType::AstChild => {} // structural, not semantic
            }
        }

        // Max degree for normalization.
        let max_degree = in_degree.iter()
            .zip(out_degree.iter())
            .map(|(i, o)| i + o)
            .max()
            .unwrap_or(1)
            .max(1); // avoid division by zero

        // Write features [26..33] into each node's feature vector.
        for (idx, node) in cpg.nodesMut().iter_mut().enumerate() {
            let in_d = in_degree[idx] as f32;
            let out_d = out_degree[idx] as f32;
            let total_d = in_d + out_d;

            // [26] in_degree normalized
            node.features[26] = in_d / 20.0;
            // [27] out_degree normalized
            node.features[27] = out_d / 20.0;
            // [28] total_degree normalized
            node.features[28] = total_d / 40.0;
            // [29] has_cfg_edge
            node.features[29] = if has_cfg[idx] { 1.0 } else { 0.0 };
            // [30] has_dfg_edge
            node.features[30] = if has_dfg[idx] { 1.0 } else { 0.0 };
            // [31] has_call_edge
            node.features[31] = if has_call[idx] { 1.0 } else { 0.0 };
            // [32] is_entry_point
            node.features[32] = if cpg.isEntryPoint(node.id) { 1.0 } else { 0.0 };
            // [33] degree_centrality
            node.features[33] = total_d / max_degree as f32;
        }
    }
}

// ---------------------------------------------------------------------------
// Non-tree-sitter fallback — simple regex-based builder for small snippets
// ---------------------------------------------------------------------------

#[cfg(not(feature = "tree-sitter"))]
impl CPGBuilder {
    /// Minimal fallback: creates a CPG with a single module node.
    /// Requires tree-sitter feature for full CPG construction.
    pub fn build_from_source(
        _source: &str,
        file_path: &str,
        _language: Language,
    ) -> anyhow::Result<CodePropertyGraph> {
        let mut cpg = CodePropertyGraph::new();
        let mut features = vec![0.0f32; 34];
        features[CPGNodeType::Module.encoding()] = 1.0;

        cpg.addNode(CPGNode {
            id: 0,
            name: file_path.to_string(),
            node_type: CPGNodeType::Module,
            file_path: file_path.to_string(),
            start_line: 1,
            end_line: 1,
            scope_depth: 0,
            data_type: None,
            features,
        });
        Ok(cpg)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kind_mapping() {
        assert_eq!(kind_to_cpg_node_type("function_item"), Some(CPGNodeType::Function));
        assert_eq!(kind_to_cpg_node_type("binary_expression"), Some(CPGNodeType::BinaryOp));
        assert_eq!(kind_to_cpg_node_type("unknown_node"), None);
    }

    #[test]
    fn test_scope_tracking() {
        let mut scopes = ScopeStack::new();
        assert_eq!(scopes.lookup("x"), None);

        scopes.define("x", 42);
        assert_eq!(scopes.lookup("x"), Some(42));

        scopes.push_scope();
        scopes.define("y", 99);
        assert_eq!(scopes.lookup("y"), Some(99));
        // Outer scope still visible
        assert_eq!(scopes.lookup("x"), Some(42));

        scopes.pop_scope();
        assert_eq!(scopes.lookup("y"), None); // inner scope popped
        assert_eq!(scopes.lookup("x"), Some(42)); // outer scope still there
    }

    #[test]
    fn test_local_complexity() {
        // Simple node → complexity 1
        let source = "fn foo() {}";
        let mut parser = Parser::new();
        #[cfg(feature = "tree-sitter")]
        {
            parser.set_language(&tree_sitter_rust::LANGUAGE.into()).ok();
            if let Some(tree) = parser.parse(source, None) {
                let root = tree.root_node();
                let mut cursor = root.walk();
                if let Some(fn_node) = root.children(&mut cursor).next() {
                    assert_eq!(CPGBuilder::local_complexity(&fn_node), 1);
                }
            }
        }
    }

    #[test]
    fn test_feature_vector_dimensions() {
        // Verify the non-tree-sitter fallback produces 34-dim features.
        let result = CPGBuilder::build_from_source("fn main() {}", "test.rs", Language::Rust);
        assert!(result.is_ok());
        let cpg = result.unwrap();
        assert_eq!(cpg.nodeCount(), 1);
        assert_eq!(cpg.nodes()[0].features.len(), 34);
    }
}
