//! CPG Types — Node, Edge, and Graph data structures.
//!
//! The Code Property Graph combines AST + CFG + DFG into a single unified
//! representation suitable for GNN-based analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Node types — every AST construct we want to represent in the CPG
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CPGNodeType {
    Function,
    Method,
    Variable,
    Parameter,
    Return,
    Literal,
    BinaryOp,
    UnaryOp,
    Call,
    Branch,
    Loop,
    Struct,
    Enum,
    Trait,
    Impl,
    Field,
    Module,
    Block,
}

impl CPGNodeType {
    /// Numeric encoding for GNN feature vectors.
    /// Order matches the discriminant, used as a one-hot index.
    pub fn encoding(&self) -> usize {
        match self {
            Self::Function => 0,
            Self::Method => 1,
            Self::Variable => 2,
            Self::Parameter => 3,
            Self::Return => 4,
            Self::Literal => 5,
            Self::BinaryOp => 6,
            Self::UnaryOp => 7,
            Self::Call => 8,
            Self::Branch => 9,
            Self::Loop => 10,
            Self::Struct => 11,
            Self::Enum => 12,
            Self::Trait => 13,
            Self::Impl => 14,
            Self::Field => 15,
            Self::Module => 16,
            Self::Block => 17,
        }
    }

    /// Total number of node types (for one-hot vector sizing).
    pub const COUNT: usize = 18;
}

// ---------------------------------------------------------------------------
// Edge types — relationships between CPG nodes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CPGEdgeType {
    AstChild,
    CfgNext,
    CfgBranchTrue,
    CfgBranchFalse,
    CfgLoopBack,
    DfgDefUse,
    Call,
    CallerOf,
}

impl CPGEdgeType {
    /// Numeric encoding for GNN edge-type tensors.
    pub fn encoding(&self) -> usize {
        match self {
            Self::AstChild => 0,
            Self::CfgNext => 1,
            Self::CfgBranchTrue => 2,
            Self::CfgBranchFalse => 3,
            Self::CfgLoopBack => 4,
            Self::DfgDefUse => 5,
            Self::Call => 6,
            Self::CallerOf => 7,
        }
    }

    /// Total number of edge types.
    pub const COUNT: usize = 8;
}

// ---------------------------------------------------------------------------
// CPG Node
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPGNode {
    pub id: usize,
    pub name: String,
    pub node_type: CPGNodeType,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub scope_depth: u32,
    /// Inferred or annotated type (e.g. "i32", "String", "Option<T>").
    pub data_type: Option<String>,
    /// Numeric feature vector for GNN input. Placeholder zeros for now.
    pub features: Vec<f32>,
}

// ---------------------------------------------------------------------------
// CPG Edge
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPGEdge {
    pub source: usize,
    pub target: usize,
    pub edge_type: CPGEdgeType,
    pub weight: f32,
}

// ---------------------------------------------------------------------------
// Edge-index representation — ready for GNN consumption
// ---------------------------------------------------------------------------

/// Sparse representation of the graph for GNN input.
/// Each field is a parallel array: `src[i]` connects to `dst[i]` via `edge_types[i]`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgeIndex {
    pub src: Vec<usize>,
    pub dst: Vec<usize>,
    pub edge_types: Vec<usize>,
}

// ---------------------------------------------------------------------------
// Code Property Graph
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePropertyGraph {
    nodes: Vec<CPGNode>,
    edges: Vec<CPGEdge>,
    /// Maps node id → index in `nodes` vec.
    pub(crate) node_index: HashMap<usize, usize>,
    /// Entry points (main, public API handlers, etc.).
    entry_points: Vec<usize>,
}

impl Default for CodePropertyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl CodePropertyGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_index: HashMap::new(),
            entry_points: Vec::new(),
        }
    }

    /// Add a node. Returns its id.
    pub fn addNode(&mut self, node: CPGNode) -> usize {
        let id = node.id;
        let idx = self.nodes.len();
        self.node_index.insert(id, idx);
        self.nodes.push(node);
        id
    }

    /// Add a directed edge.
    pub fn addEdge(&mut self, source: usize, target: usize, edge_type: CPGEdgeType) {
        self.edges.push(CPGEdge {
            source,
            target,
            edge_type,
            weight: 1.0,
        });
    }

    /// Get a node reference by id.
    pub fn getNode(&self, id: usize) -> Option<&CPGNode> {
        self.node_index.get(&id).map(|&idx| &self.nodes[idx])
    }

    /// Iterate all nodes that have an edge of the given type targeting `node_id`.
    pub fn neighbors(&self, node_id: usize, edge_type: Option<CPGEdgeType>) -> Vec<&CPGNode> {
        self.edges
            .iter()
            .filter(|e| e.target == node_id && edge_type.map_or(true, |t| e.edge_type == t))
            .filter_map(|e| self.getNode(e.source))
            .collect()
    }

    /// Mark a node as an entry point.
    pub fn markEntryPoint(&mut self, node_id: usize) {
        if !self.entry_points.contains(&node_id) {
            self.entry_points.push(node_id);
        }
    }

    /// Convert to sparse edge-index tensors for GNN input.
    pub fn toEdgeIndex(&self) -> EdgeIndex {
        let mut idx = EdgeIndex::default();
        idx.src.reserve(self.edges.len());
        idx.dst.reserve(self.edges.len());
        idx.edge_types.reserve(self.edges.len());

        for edge in &self.edges {
            idx.src.push(edge.source);
            idx.dst.push(edge.target);
            idx.edge_types.push(edge.edge_type.encoding());
        }

        idx
    }

    /// Node count.
    pub fn nodeCount(&self) -> usize {
        self.nodes.len()
    }

    /// Edge count.
    pub fn edgeCount(&self) -> usize {
        self.edges.len()
    }

    /// Iterate all nodes.
    pub fn nodes(&self) -> &[CPGNode] {
        &self.nodes
    }

    /// Iterate all edges.
    pub fn edges(&self) -> &[CPGEdge] {
        &self.edges
    }

    /// Entry point ids.
    pub fn entryPoints(&self) -> &[usize] {
        &self.entry_points
    }

    /// Mutable access to nodes (needed for post-build feature computation).
    pub fn nodesMut(&mut self) -> &mut [CPGNode] {
        &mut self.nodes
    }

    /// Check if a node is an entry point.
    pub fn isEntryPoint(&self, node_id: usize) -> bool {
        self.entry_points.contains(&node_id)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_encoding() {
        assert_eq!(CPGNodeType::Function.encoding(), 0);
        assert_eq!(CPGNodeType::Block.encoding(), 17);
        assert_eq!(CPGNodeType::COUNT, 18);
    }

    #[test]
    fn test_edge_type_encoding() {
        assert_eq!(CPGEdgeType::AstChild.encoding(), 0);
        assert_eq!(CPGEdgeType::CallerOf.encoding(), 7);
        assert_eq!(CPGEdgeType::COUNT, 8);
    }

    #[test]
    fn test_cpg_add_and_query() {
        let mut cpg = CodePropertyGraph::new();

        let fn_node = CPGNode {
            id: 0,
            name: "main".into(),
            node_type: CPGNodeType::Function,
            file_path: "test.rs".into(),
            start_line: 1,
            end_line: 5,
            scope_depth: 0,
            data_type: None,
            features: vec![],
        };
        cpg.addNode(fn_node);

        let call_node = CPGNode {
            id: 1,
            name: "helper".into(),
            node_type: CPGNodeType::Call,
            file_path: "test.rs".into(),
            start_line: 3,
            end_line: 3,
            scope_depth: 1,
            data_type: None,
            features: vec![],
        };
        cpg.addNode(call_node);

        cpg.addEdge(0, 1, CPGEdgeType::AstChild);

        assert_eq!(cpg.nodeCount(), 2);
        assert!(cpg.getNode(0).is_some());
        assert_eq!(cpg.getNode(0).unwrap().name, "main");

        let nbrs = cpg.neighbors(1, Some(CPGEdgeType::AstChild));
        assert_eq!(nbrs.len(), 1);
        assert_eq!(nbrs[0].name, "main");
    }

    #[test]
    fn test_edge_index() {
        let mut cpg = CodePropertyGraph::new();
        cpg.addNode(CPGNode {
            id: 0, name: "a".into(), node_type: CPGNodeType::Function,
            file_path: "".into(), start_line: 0, end_line: 0,
            scope_depth: 0, data_type: None, features: vec![],
        });
        cpg.addNode(CPGNode {
            id: 1, name: "b".into(), node_type: CPGNodeType::Call,
            file_path: "".into(), start_line: 0, end_line: 0,
            scope_depth: 0, data_type: None, features: vec![],
        });
        cpg.addEdge(0, 1, CPGEdgeType::CfgNext);
        cpg.addEdge(1, 0, CPGEdgeType::CfgLoopBack);

        let ei = cpg.toEdgeIndex();
        assert_eq!(ei.src, vec![0, 1]);
        assert_eq!(ei.dst, vec![1, 0]);
        assert_eq!(ei.edge_types, vec![1, 4]); // CfgNext=1, CfgLoopBack=4
    }
}
