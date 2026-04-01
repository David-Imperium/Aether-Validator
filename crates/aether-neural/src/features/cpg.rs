//! CPG Feature Extractor — converts Code Property Graphs to tensor inputs.
//!
//! Takes a CPG (from aether-intelligence) and produces the flat tensor
//! representation expected by the GNN models: node features, edge index,
//! edge types, and entry point masks.
//!
//! Node feature vector (34 dimensions):
//! - [0..18]: one-hot node type (18 CPGNodeType variants)
//! - [18]: scope_depth (normalized to 0.0 — 1.0)
//! - [19..22]: cyclomatic complexity bucket (one-hot: 0, 1-2, 3-5, 6+)
//! - [22]: is_entry_point (0.0 or 1.0)
//! - [23..25]: data type category (one-hot: primitive, complex, unknown)
//! - [25..30]: violations_count bucket (one-hot: 0, 1, 2-3, 4-5, 6+)
//! - [30]: degree centrality (normalized)
//! - [31]: in-degree (normalized)
//! - [32]: out-degree (normalized)
//! - [33]: betweenness centrality proxy (normalized)

use crate::error::Result;
use crate::inference::CpgTensorInput;
use aether_intelligence::memory::CodePropertyGraph;

/// Feature dimensionality for a single CPG node.
pub const FEATURE_DIM: usize = 34;

/// Maximum scope depth for normalization.
const MAX_SCOPE_DEPTH: f32 = 20.0;

/// Maximum degree for normalization.
const MAX_DEGREE: f32 = 50.0;

/// Extracts tensor features from a Code Property Graph.
pub struct CpgFeatureExtractor {
    /// Maximum nodes to include (truncates large graphs).
    max_nodes: usize,
    /// Maximum edges to include.
    max_edges: usize,
}

impl CpgFeatureExtractor {
    /// Create a new extractor with the given limits.
    pub fn new(max_nodes: usize, max_edges: usize) -> Self {
        Self { max_nodes, max_edges }
    }

    /// Extract tensor features from a CPG.
    ///
    /// The output is ready for GNN inference: flat arrays that can be
    /// converted to Burn tensors.
    pub fn extract_from_cpg(&self, cpg: &CodePropertyGraph) -> Result<CpgTensorInput> {
        let nodes = cpg.nodes();
        let edges = cpg.edges();

        // Truncate if necessary
        let node_count = nodes.len().min(self.max_nodes);
        let mut edge_count = 0;

        // Build degree maps
        let mut in_degree = vec![0usize; node_count];
        let mut out_degree = vec![0usize; node_count];
        let mut degree = vec![0usize; node_count];

        // Build old_id -> new_index mapping (0-based, contiguous)
        let mut id_map = std::collections::HashMap::new();
        for (idx, node) in nodes.iter().enumerate().take(node_count) {
            id_map.insert(node.id, idx);
        }

        // Count edges (only between nodes we're keeping)
        let mut edge_src = Vec::with_capacity(edges.len().min(self.max_edges));
        let mut edge_dst = Vec::with_capacity(edges.len().min(self.max_edges));
        let mut edge_types = Vec::with_capacity(edges.len().min(self.max_edges));

        for edge in edges {
            let src_idx = id_map.get(&edge.source).copied();
            let dst_idx = id_map.get(&edge.target).copied();

            if let (Some(s), Some(d)) = (src_idx, dst_idx) {
                if edge_count >= self.max_edges {
                    break;
                }
                edge_src.push(s);
                edge_dst.push(d);
                edge_types.push(edge.edge_type.encoding());
                out_degree[s] += 1;
                in_degree[d] += 1;
                degree[s] += 1;
                degree[d] += 1;
                edge_count += 1;
            }
        }

        // Build entry mask
        let entry_points: std::collections::HashSet<usize> = cpg
            .entryPoints()
            .iter()
            .filter_map(|id| id_map.get(id).copied())
            .collect();

        // Build node features
        let mut node_features = Vec::with_capacity(node_count * FEATURE_DIM);
        for (idx, node) in nodes.iter().enumerate().take(node_count) {
            let mut features = [0.0f32; FEATURE_DIM];

            // [0..18]: one-hot node type
            features[node.node_type.encoding()] = 1.0;

            // [18]: scope_depth (normalized)
            features[18] = (node.scope_depth as f32 / MAX_SCOPE_DEPTH).min(1.0);

            // [19..22]: cyclomatic complexity bucket (simplified: based on degree)
            // TODO: compute actual cyclomatic complexity from CFG edges
            let complexity = degree[idx];
            match complexity {
                0 => features[19] = 1.0,       // simple (0)
                1..=2 => features[20] = 1.0,   // low (1-2)
                3..=5 => features[21] = 1.0,   // moderate (3-5)
                _ => {}                          // high (6+) — default 0.0
            }

            // [22]: is_entry_point
            features[22] = if entry_points.contains(&idx) { 1.0 } else { 0.0 };

            // [23..25]: data type category (one-hot: primitive, complex, unknown)
            match classify_data_type(&node.data_type) {
                DataTypeCategory::Primitive => features[23] = 1.0,
                DataTypeCategory::Complex => features[24] = 1.0,
                DataTypeCategory::Unknown => features[25] = 1.0,
            }

            // [26..30]: violations_count bucket — always 0 for inference (no history)
            // features[26] = 1.0; // 0 violations (default)

            // [30]: degree centrality (normalized)
            features[30] = (degree[idx] as f32 / MAX_DEGREE).min(1.0);

            // [31]: in-degree (normalized)
            features[31] = (in_degree[idx] as f32 / MAX_DEGREE).min(1.0);

            // [32]: out-degree (normalized)
            features[32] = (out_degree[idx] as f32 / MAX_DEGREE).min(1.0);

            // [33]: betweenness proxy (normalized out-degree * entry proximity)
            let entry_proximity = if entry_points.contains(&idx) {
                1.0
            } else {
                0.0
            };
            features[33] = ((out_degree[idx] as f32 / MAX_DEGREE).min(1.0) * entry_proximity)
                .min(1.0);

            node_features.extend_from_slice(&features);
        }

        // Build entry mask
        let entry_mask: Vec<f32> = (0..node_count)
            .map(|i| if entry_points.contains(&i) { 1.0 } else { 0.0 })
            .collect();

        let input = CpgTensorInput {
            node_features,
            num_nodes: node_count,
            feature_dim: FEATURE_DIM,
            edge_src,
            edge_dst,
            edge_types,
            num_edges: edge_count,
            entry_mask,
        };

        input.validate()?;
        Ok(input)
    }

    /// Convenience method: extract from raw source code.
    ///
    /// Note: This requires the `tree-sitter` feature. Without it,
    /// returns an empty CPG tensor input.
    pub fn extract(&self, source: &str, language: &str) -> Result<CpgTensorInput> {
        // TODO: When tree-sitter feature is active, parse source → CPG → tensor.
        // For now, return an empty input to allow the pipeline to compile.
        tracing::debug!(
            "extract() called with language='{}', source_len={} — CPG extraction requires tree-sitter feature",
            language,
            source.len()
        );

        Ok(CpgTensorInput::empty())
    }
}

/// Data type classification for CPG node features.
enum DataTypeCategory {
    /// Primitive types: i32, f64, bool, char, u8, etc.
    Primitive,
    /// Complex types: String, Vec<T>, Option<T>, Result<T>, custom structs.
    Complex,
    /// Unknown or untyped.
    Unknown,
}

/// Classify a data type string into a category.
fn classify_data_type(data_type: &Option<String>) -> DataTypeCategory {
    match data_type {
        Some(dt) if dt.is_empty() => DataTypeCategory::Unknown,
        Some(dt) => {
            let dt_lower = dt.to_lowercase();
            // Primitives
            if matches!(
                dt_lower.as_str(),
                "i8" | "i16" | "i32" | "i64" | "i128"
                    | "u8" | "u16" | "u32" | "u64" | "u128"
                    | "f32" | "f64"
                    | "bool"
                    | "char"
                    | "()" | "str"
            ) {
                DataTypeCategory::Primitive
            } else {
                DataTypeCategory::Complex
            }
        }
        None => DataTypeCategory::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_intelligence::memory::{CPGNode, CPGEdgeType, CPGNodeType};

    fn make_test_cpg() -> CodePropertyGraph {
        let mut cpg = CodePropertyGraph::new();

        cpg.addNode(CPGNode {
            id: 0,
            name: "main".into(),
            node_type: CPGNodeType::Function,
            file_path: "test.rs".into(),
            start_line: 1,
            end_line: 10,
            scope_depth: 0,
            data_type: Some("()".into()),
            features: vec![],
        });

        cpg.addNode(CPGNode {
            id: 1,
            name: "result".into(),
            node_type: CPGNodeType::Variable,
            file_path: "test.rs".into(),
            start_line: 3,
            end_line: 3,
            scope_depth: 1,
            data_type: Some("Result<i32, Error>".into()),
            features: vec![],
        });

        cpg.addNode(CPGNode {
            id: 2,
            name: "handle".into(),
            node_type: CPGNodeType::Call,
            file_path: "test.rs".into(),
            start_line: 5,
            end_line: 5,
            scope_depth: 1,
            data_type: None,
            features: vec![],
        });

        cpg.addEdge(0, 1, CPGEdgeType::AstChild);
        cpg.addEdge(0, 2, CPGEdgeType::AstChild);
        cpg.addEdge(1, 2, CPGEdgeType::CfgNext);

        cpg.markEntryPoint(0);

        cpg
    }

    #[test]
    fn test_feature_dim() {
        assert_eq!(FEATURE_DIM, 34);
    }

    #[test]
    fn test_extract_from_cpg() {
        let cpg = make_test_cpg();
        let extractor = CpgFeatureExtractor::new(100, 200);
        let tensor = extractor.extract_from_cpg(&cpg).unwrap();

        assert_eq!(tensor.num_nodes, 3);
        assert_eq!(tensor.num_edges, 3);
        assert_eq!(tensor.feature_dim, 34);
        assert_eq!(tensor.node_features.len(), 3 * 34);
        assert!(tensor.validate().is_ok());
    }

    #[test]
    fn test_node_type_one_hot() {
        let cpg = make_test_cpg();
        let extractor = CpgFeatureExtractor::new(100, 200);
        let tensor = extractor.extract_from_cpg(&cpg).unwrap();

        // Node 0: Function → index 0 should be 1.0
        assert_eq!(tensor.node_features[0], 1.0);
        assert_eq!(tensor.node_features[1], 0.0); // not Method

        // Node 1: Variable → index 2 should be 1.0
        assert_eq!(tensor.node_features[34 + 2], 1.0);

        // Node 2: Call → index 8 should be 1.0
        assert_eq!(tensor.node_features[68 + 8], 1.0);
    }

    #[test]
    fn test_entry_point_mask() {
        let cpg = make_test_cpg();
        let extractor = CpgFeatureExtractor::new(100, 200);
        let tensor = extractor.extract_from_cpg(&cpg).unwrap();

        assert_eq!(tensor.entry_mask[0], 1.0); // main is entry point
        assert_eq!(tensor.entry_mask[1], 0.0); // result is not
        assert_eq!(tensor.entry_mask[2], 0.0); // handle is not
    }

    #[test]
    fn test_edge_types() {
        let cpg = make_test_cpg();
        let extractor = CpgFeatureExtractor::new(100, 200);
        let tensor = extractor.extract_from_cpg(&cpg).unwrap();

        // 2 AstChild (0) + 1 CfgNext (1)
        assert_eq!(tensor.edge_types, vec![0, 0, 1]);
    }

    #[test]
    fn test_data_type_classification() {
        assert!(matches!(
            classify_data_type(&Some("i32".into())),
            DataTypeCategory::Primitive
        ));
        assert!(matches!(
            classify_data_type(&Some("Result<i32, Error>".into())),
            DataTypeCategory::Complex
        ));
        assert!(matches!(
            classify_data_type(&None),
            DataTypeCategory::Unknown
        ));
    }

    #[test]
    fn test_truncation() {
        let cpg = make_test_cpg();
        let extractor = CpgFeatureExtractor::new(2, 200); // max 2 nodes
        let tensor = extractor.extract_from_cpg(&cpg).unwrap();

        assert_eq!(tensor.num_nodes, 2);
        assert_eq!(tensor.node_features.len(), 2 * 34);
    }

    #[test]
    fn test_empty_extraction() {
        let extractor = CpgFeatureExtractor::new(100, 200);
        let tensor = extractor.extract("", "rust").unwrap();

        assert_eq!(tensor.num_nodes, 0);
        assert_eq!(tensor.num_edges, 0);
    }
}
