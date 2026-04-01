//! End-to-end pipeline test: Code → CPG → Feature Extraction → Tensor → Classification → Result.
//!
//! Tests the full neural pipeline WITHOUT requiring real trained models.
//! Uses stub .burnpack files to prove the wiring is complete.

use aether_intelligence::memory::{
    CodePropertyGraph, CPGEdgeType, CPGNode, CPGNodeType,
};
use aether_neural::{
    AetherNeural, CpgFeatureExtractor, CodeReasoner, Classification, ClassificationCategory,
    ConfidenceLevel, ConfidenceThresholds, DriftPredictor, DriftPrediction,
    DriftSeverity, ExperienceMeta, NeuralConfig, NeuralOrchestrator, PatternMatch,
    PatternMemory,
};
use std::io::Write;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Build a realistic CPG from a small Rust function.
///
/// Models: `fn process(data: &Vec<u8>) -> Result<i32, Error> { let x = data[0]; if x > 0 { x } else { 0 } }`
fn build_realistic_cpg() -> CodePropertyGraph {
    let mut cpg = CodePropertyGraph::new();

    // function: process
    cpg.addNode(CPGNode {
        id: 0,
        name: "process".into(),
        node_type: CPGNodeType::Function,
        file_path: "demo.rs".into(),
        start_line: 1,
        end_line: 5,
        scope_depth: 0,
        data_type: Some("fn(&Vec<u8>) -> Result<i32, Error>".into()),
        features: vec![],
    });

    // parameter: data
    cpg.addNode(CPGNode {
        id: 1,
        name: "data".into(),
        node_type: CPGNodeType::Parameter,
        file_path: "demo.rs".into(),
        start_line: 1,
        end_line: 1,
        scope_depth: 0,
        data_type: Some("&Vec<u8>".into()),
        features: vec![],
    });

    // local variable: x
    cpg.addNode(CPGNode {
        id: 2,
        name: "x".into(),
        node_type: CPGNodeType::Variable,
        file_path: "demo.rs".into(),
        start_line: 3,
        end_line: 3,
        scope_depth: 1,
        data_type: Some("u8".into()),
        features: vec![],
    });

    // index access: data[0]
    cpg.addNode(CPGNode {
        id: 3,
        name: "data[0]".into(),
        node_type: CPGNodeType::Variable,
        file_path: "demo.rs".into(),
        start_line: 3,
        end_line: 3,
        scope_depth: 1,
        data_type: Some("u8".into()),
        features: vec![],
    });

    // comparison: x > 0
    cpg.addNode(CPGNode {
        id: 4,
        name: ">".into(),
        node_type: CPGNodeType::BinaryOp,
        file_path: "demo.rs".into(),
        start_line: 4,
        end_line: 4,
        scope_depth: 1,
        data_type: Some("bool".into()),
        features: vec![],
    });

    // if block (Branch node)
    cpg.addNode(CPGNode {
        id: 5,
        name: "if".into(),
        node_type: CPGNodeType::Branch,
        file_path: "demo.rs".into(),
        start_line: 4,
        end_line: 5,
        scope_depth: 1,
        data_type: None,
        features: vec![],
    });

    // literal: 0
    cpg.addNode(CPGNode {
        id: 6,
        name: "0".into(),
        node_type: CPGNodeType::Literal,
        file_path: "demo.rs".into(),
        start_line: 4,
        end_line: 4,
        scope_depth: 1,
        data_type: Some("i32".into()),
        features: vec![],
    });

    // return: x
    cpg.addNode(CPGNode {
        id: 7,
        name: "return_x".into(),
        node_type: CPGNodeType::Return,
        file_path: "demo.rs".into(),
        start_line: 4,
        end_line: 4,
        scope_depth: 2,
        data_type: Some("i32".into()),
        features: vec![],
    });

    // return: 0 (else branch)
    cpg.addNode(CPGNode {
        id: 8,
        name: "return_0".into(),
        node_type: CPGNodeType::Return,
        file_path: "demo.rs".into(),
        start_line: 5,
        end_line: 5,
        scope_depth: 2,
        data_type: Some("i32".into()),
        features: vec![],
    });

    // ── AST edges ────────────────────────────────────────────────────────
    cpg.addEdge(0, 1, CPGEdgeType::AstChild);
    cpg.addEdge(0, 2, CPGEdgeType::AstChild);
    cpg.addEdge(0, 5, CPGEdgeType::AstChild);
    cpg.addEdge(2, 3, CPGEdgeType::AstChild);
    cpg.addEdge(5, 4, CPGEdgeType::AstChild);
    cpg.addEdge(4, 6, CPGEdgeType::AstChild);
    cpg.addEdge(5, 7, CPGEdgeType::AstChild);
    cpg.addEdge(5, 8, CPGEdgeType::AstChild);

    // ── CFG edges ────────────────────────────────────────────────────────
    cpg.addEdge(2, 4, CPGEdgeType::CfgNext);
    cpg.addEdge(4, 7, CPGEdgeType::CfgBranchTrue);
    cpg.addEdge(4, 8, CPGEdgeType::CfgBranchFalse);

    // ── DFG edges ────────────────────────────────────────────────────────
    cpg.addEdge(1, 3, CPGEdgeType::DfgDefUse);
    cpg.addEdge(3, 2, CPGEdgeType::DfgDefUse);
    cpg.addEdge(2, 4, CPGEdgeType::DfgDefUse);
    cpg.addEdge(2, 7, CPGEdgeType::DfgDefUse);

    cpg.markEntryPoint(0);

    cpg
}

/// Build a pathological CPG with common anti-patterns (for classification testing).
fn build_problematic_cpg() -> CodePropertyGraph {
    let mut cpg = CodePropertyGraph::new();

    // function with deeply nested control flow
    cpg.addNode(CPGNode {
        id: 0,
        name: "bad_handler".into(),
        node_type: CPGNodeType::Function,
        file_path: "bad.rs".into(),
        start_line: 1,
        end_line: 20,
        scope_depth: 0,
        data_type: Some("fn() -> Result<(), Error>".into()),
        features: vec![],
    });

    // bare unwrap
    cpg.addNode(CPGNode {
        id: 1,
        name: "unwrap".into(),
        node_type: CPGNodeType::Call,
        file_path: "bad.rs".into(),
        start_line: 5,
        end_line: 5,
        scope_depth: 1,
        data_type: None,
        features: vec![],
    });

    // nested if (Branch)
    cpg.addNode(CPGNode {
        id: 2,
        name: "if1".into(),
        node_type: CPGNodeType::Branch,
        file_path: "bad.rs".into(),
        start_line: 7,
        end_line: 10,
        scope_depth: 1,
        data_type: None,
        features: vec![],
    });

    cpg.addNode(CPGNode {
        id: 3,
        name: "if2".into(),
        node_type: CPGNodeType::Branch,
        file_path: "bad.rs".into(),
        start_line: 8,
        end_line: 9,
        scope_depth: 2,
        data_type: None,
        features: vec![],
    });

    cpg.addNode(CPGNode {
        id: 4,
        name: "if3".into(),
        node_type: CPGNodeType::Branch,
        file_path: "bad.rs".into(),
        start_line: 8,
        end_line: 9,
        scope_depth: 3,
        data_type: None,
        features: vec![],
    });

    cpg.addNode(CPGNode {
        id: 5,
        name: "panic!".into(),
        node_type: CPGNodeType::Call,
        file_path: "bad.rs".into(),
        start_line: 15,
        end_line: 15,
        scope_depth: 1,
        data_type: Some("! (never)".into()),
        features: vec![],
    });

    // edges
    cpg.addEdge(0, 1, CPGEdgeType::AstChild);
    cpg.addEdge(0, 2, CPGEdgeType::AstChild);
    cpg.addEdge(0, 5, CPGEdgeType::AstChild);
    cpg.addEdge(2, 3, CPGEdgeType::AstChild);
    cpg.addEdge(3, 4, CPGEdgeType::AstChild);
    cpg.addEdge(1, 2, CPGEdgeType::CfgNext);
    cpg.addEdge(2, 5, CPGEdgeType::CfgNext);

    cpg.markEntryPoint(0);

    cpg
}

/// Create a temporary directory with stub .burnpack files for model loading.
fn create_stub_models_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("aether_neural_e2e_test");
    let _ = std::fs::create_dir_all(&dir);

    for name in &["code_reasoner", "pattern_memory", "drift_predictor"] {
        let path = dir.join(format!("{}.burnpack", name));
        if !path.exists() {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(b"stub").unwrap();
        }
    }

    dir
}

// ── Stage 1: CPG → Feature Extraction ───────────────────────────────────────

#[test]
fn test_stage1_cpg_to_features() {
    let cpg = build_realistic_cpg();
    let extractor = CpgFeatureExtractor::new(500, 2000);
    let tensor = extractor.extract_from_cpg(&cpg).unwrap();

    // Basic shape checks
    assert_eq!(tensor.num_nodes, 9);
    assert!(tensor.num_edges >= 12); // 8 AST + 3 CFG + 4 DFG
    assert_eq!(tensor.feature_dim, 34);
    assert_eq!(tensor.node_features.len(), 9 * 34);
    assert_eq!(tensor.entry_mask.len(), 9);

    // Entry point
    assert_eq!(tensor.entry_mask[0], 1.0);
    for i in 1..9 {
        assert_eq!(tensor.entry_mask[i], 0.0);
    }

    // Validation passes
    assert!(tensor.validate().is_ok());
}

#[test]
fn test_stage1_feature_vector_correctness() {
    let cpg = build_realistic_cpg();
    let extractor = CpgFeatureExtractor::new(500, 2000);
    let tensor = extractor.extract_from_cpg(&cpg).unwrap();

    // Node 0 (Function): one-hot at index of Function encoding
    let fn_enc = CPGNodeType::Function.encoding();
    assert_eq!(tensor.node_features[fn_enc], 1.0, "Function node should have one-hot at its encoding");

    // Node 2 (Variable): scope_depth = 1, normalized
    let var_offset = 2 * 34;
    assert_eq!(tensor.node_features[var_offset + 18], 1.0 / 20.0, "scope_depth=1 should be 1/20");

    // Node 6 (Literal): data type = i32 (primitive)
    let lit_offset = 6 * 34;
    assert_eq!(tensor.node_features[lit_offset + 23], 1.0, "i32 should be Primitive category");

    // Node 1 (Parameter): data type = &Vec<u8> (complex)
    let param_offset = 1 * 34;
    assert_eq!(tensor.node_features[param_offset + 24], 1.0, "Vec<u8> should be Complex category");

    // Node 3 (Variable): data type = u8 → Primitive
    let id_offset = 3 * 34;
    assert_eq!(tensor.node_features[id_offset + 23], 1.0, "u8 should be Primitive category");
}

// ── Stage 2: Problematic CPG → Higher complexity features ───────────────────

#[test]
fn test_stage2_problematic_cpg_features() {
    let cpg = build_problematic_cpg();
    let extractor = CpgFeatureExtractor::new(500, 2000);
    let tensor = extractor.extract_from_cpg(&cpg).unwrap();

    assert_eq!(tensor.num_nodes, 6);

    // Node 0 (bad_handler) has 3 AST children → degree >= 3
    let handler_offset = 0 * 34;
    let degree_centrality = tensor.node_features[handler_offset + 30];
    assert!(
        degree_centrality > 0.0,
        "Entry function should have non-zero degree centrality"
    );

    // Node 2 (if1) has AST child + CFG edges → out-degree >= 1
    let if_offset = 2 * 34;
    let out_degree = tensor.node_features[if_offset + 32];
    assert!(out_degree > 0.0, "Control flow node should have non-zero out-degree");
}

// ── Stage 3: Feature Extraction → Tensor Validation ─────────────────────────

#[test]
fn test_stage3_truncation_large_graph() {
    let cpg = build_realistic_cpg();
    let extractor = CpgFeatureExtractor::new(3, 5); // Very small limits
    let tensor = extractor.extract_from_cpg(&cpg).unwrap();

    assert_eq!(tensor.num_nodes, 3, "Should truncate to max_nodes");
    assert!(tensor.num_edges <= 5, "Should truncate to max_edges");
    assert_eq!(tensor.node_features.len(), 3 * 34);
    assert!(tensor.validate().is_ok(), "Truncated tensor should still be valid");
}

// ── Stage 4: Classification + Orchestration (without real model) ─────────────

#[test]
fn test_stage4_orchestration_with_classifications() {
    let cpg = build_problematic_cpg();
    let extractor = CpgFeatureExtractor::new(500, 2000);
    let _tensor = extractor.extract_from_cpg(&cpg).unwrap();

    let orchestrator = NeuralOrchestrator::default();

    // Simulate what the CodeReasoner would produce (when trained)
    let classifications = vec![
        Classification {
            category: ClassificationCategory::UnhandledError,
            confidence: 0.92,
            attention_nodes: vec![(1, 0.8), (5, 0.6)], // unwrap and panic!
            description: "Bare unwrap and panic! in error handling path".into(),
        },
        Classification {
            category: ClassificationCategory::CodeSmell,
            confidence: 0.85,
            attention_nodes: vec![(2, 0.7), (3, 0.5), (4, 0.4)],
            description: "3 levels of nested if statements".into(),
        },
    ];

    let routing = orchestrator.route(&classifications, &[], None);

    assert!(routing.confidence() > 0.85, "High-confidence classifications should yield high routing confidence");
    assert!(!routing.should_defer(), "Should not defer with high confidence");
    assert_eq!(ConfidenceLevel::from_score(routing.confidence(), &ConfidenceThresholds::default()), ConfidenceLevel::High);
}

#[test]
fn test_stage4_low_confidence_defers_to_symbolic() {
    let orchestrator = NeuralOrchestrator::default();

    let classifications = vec![
        Classification {
            category: ClassificationCategory::Clean,
            confidence: 0.3,
            attention_nodes: vec![],
            description: "Uncertain classification".into(),
        },
    ];

    let routing = orchestrator.route(&classifications, &[], None);

    assert!(routing.should_defer(), "Low confidence should defer to symbolic");
    assert!(routing.confidence() < 0.5);
}

#[test]
fn test_stage4_empty_classifications_defers() {
    let orchestrator = NeuralOrchestrator::default();
    let routing = orchestrator.route(&[], &[], None);

    assert!(routing.should_defer(), "Empty classifications should defer");
    assert!(routing.confidence() < 0.5);
}

#[test]
fn test_stage4_mixed_confidence_hybrid_routing() {
    let orchestrator = NeuralOrchestrator::default();

    let classifications = vec![
        Classification {
            category: ClassificationCategory::UnhandledError,
            confidence: 0.88,
            attention_nodes: vec![(0, 0.5)],
            description: "Possible error path".into(),
        },
        Classification {
            category: ClassificationCategory::Clean,
            confidence: 0.45,
            attention_nodes: vec![],
            description: "Also looks fine?".into(),
        },
    ];

    let routing = orchestrator.route(&classifications, &[], None);

    let level = ConfidenceLevel::from_score(routing.confidence(), &ConfidenceThresholds::default());
    assert!(
        matches!(level, ConfidenceLevel::High | ConfidenceLevel::Moderate),
        "Mixed signals should yield at least moderate confidence"
    );
}

// ── Stage 5: Pattern Memory + Classification fusion ─────────────────────────

#[test]
fn test_stage5_pattern_memory_similar_matches_boost_confidence() {
    let orchestrator = NeuralOrchestrator::default();

    let classifications = vec![Classification {
        category: ClassificationCategory::UnhandledError,
        confidence: 0.75,
        attention_nodes: vec![(1, 0.9)],
        description: "unwrap call".into(),
    }];

    let similar = vec![PatternMatch {
        experience_id: "exp-001".into(),
        similarity: 0.95,
        category: "UnhandledError".into(),
        description: "Nearly identical pattern: bare unwrap in error handler".into(),
        source_file: Some("legacy/handler.rs".into()),
    }];

    let routing = orchestrator.route(&classifications, &similar, None);

    // Pattern memory with high similarity should boost routing confidence
    assert!(
        routing.confidence() >= 0.75,
        "Pattern memory match should not reduce confidence below base classification"
    );
}

// ── Stage 6: Drift Prediction integration ───────────────────────────────────

#[test]
fn test_stage6_drift_warning_affects_routing() {
    let orchestrator = NeuralOrchestrator::default();

    let classifications = vec![Classification {
        category: ClassificationCategory::Clean,
        confidence: 0.85,
        attention_nodes: vec![],
        description: "Code looks clean".into(),
    }];

    let drift = DriftPrediction {
        probability: 0.7,
        severity: DriftSeverity::Medium,
        timeframe_commits: Some(10),
        affected_components: vec!["demo.rs".into()],
        explanation: "Rapid structural changes detected in recent commits".into(),
    };

    let routing = orchestrator.route(&classifications, &[], Some(&drift));

    // Drift warning should influence the routing explanation
    assert!(
        routing.explanation().is_some(),
        "Drift warning should produce an explanation"
    );
}

// ── Stage 7: Full AetherNeural load + analyze (with stubs) ──────────────────

#[test]
fn test_stage7_full_pipeline_with_stub_models() {
    let models_dir = create_stub_models_dir();

    // Load all three networks (stubs)
    let code_reasoner = CodeReasoner::load(&models_dir).expect("CodeReasoner should load from stub");
    let pattern_memory = PatternMemory::load(&models_dir).expect("PatternMemory should load from stub");
    let drift_predictor = DriftPredictor::load(&models_dir).expect("DriftPredictor should load from stub");

    assert!(code_reasoner.is_loaded());
    assert!(pattern_memory.is_loaded());
    assert!(drift_predictor.is_loaded());

    // Build CPG → features
    let cpg = build_realistic_cpg();
    let extractor = CpgFeatureExtractor::new(500, 2000);
    let tensor = extractor.extract_from_cpg(&cpg).unwrap();
    assert!(tensor.validate().is_ok());

    // Run CodeReasoner (placeholder inference — returns empty)
    let classifications = code_reasoner.classify(&tensor).expect("classify should not error");
    assert!(classifications.is_empty(), "Stub model returns empty classifications");

    // Run PatternMemory (placeholder)
    let patterns = pattern_memory.find_similar(&tensor).expect("find_similar should not error");
    assert!(patterns.is_empty(), "Stub model returns empty patterns");

    // Run DriftPredictor (placeholder)
    let drift = drift_predictor.predict(&[tensor.clone()]).expect("predict should not error");
    assert_eq!(drift.severity, DriftSeverity::None, "Stub model returns None severity");

    // Orchestrate the (empty) results
    let orchestrator = NeuralOrchestrator::default();
    let routing = orchestrator.route(&classifications, &patterns, None);

    assert!(routing.should_defer(), "Empty results from stub models should defer to symbolic");
}

// ── Edge Cases ──────────────────────────────────────────────────────────────

#[test]
fn test_empty_cpg() {
    let cpg = CodePropertyGraph::new();
    let extractor = CpgFeatureExtractor::new(500, 2000);
    let tensor = extractor.extract_from_cpg(&cpg).unwrap();

    assert_eq!(tensor.num_nodes, 0);
    assert_eq!(tensor.num_edges, 0);
    assert!(tensor.validate().is_ok());
}

#[test]
fn test_single_node_cpg() {
    let mut cpg = CodePropertyGraph::new();
    cpg.addNode(CPGNode {
        id: 0,
        name: "main".into(),
        node_type: CPGNodeType::Function,
        file_path: "minimal.rs".into(),
        start_line: 1,
        end_line: 1,
        scope_depth: 0,
        data_type: Some("()".into()),
        features: vec![],
    });
    cpg.markEntryPoint(0);

    let extractor = CpgFeatureExtractor::new(500, 2000);
    let tensor = extractor.extract_from_cpg(&cpg).unwrap();

    assert_eq!(tensor.num_nodes, 1);
    assert_eq!(tensor.num_edges, 0);
    assert_eq!(tensor.node_features.len(), 34);
    assert_eq!(tensor.entry_mask[0], 1.0);
    assert!(tensor.validate().is_ok());
}

#[test]
fn test_disconnected_nodes() {
    let mut cpg = CodePropertyGraph::new();

    cpg.addNode(CPGNode {
        id: 0,
        name: "fn_a".into(),
        node_type: CPGNodeType::Function,
        file_path: "test.rs".into(),
        start_line: 1,
        end_line: 5,
        scope_depth: 0,
        data_type: Some("()".into()),
        features: vec![],
    });

    cpg.addNode(CPGNode {
        id: 1,
        name: "fn_b".into(),
        node_type: CPGNodeType::Function,
        file_path: "test.rs".into(),
        start_line: 7,
        end_line: 10,
        scope_depth: 0,
        data_type: Some("()".into()),
        features: vec![],
    });

    // No edges — disconnected graph
    cpg.markEntryPoint(0);

    let extractor = CpgFeatureExtractor::new(500, 2000);
    let tensor = extractor.extract_from_cpg(&cpg).unwrap();

    assert_eq!(tensor.num_nodes, 2);
    assert_eq!(tensor.num_edges, 0);
    // All degree features should be 0 for disconnected nodes
    for node_idx in 0..2 {
        let offset = node_idx * 34;
        assert_eq!(tensor.node_features[offset + 30], 0.0, "Degree centrality should be 0");
        assert_eq!(tensor.node_features[offset + 31], 0.0, "In-degree should be 0");
        assert_eq!(tensor.node_features[offset + 32], 0.0, "Out-degree should be 0");
    }
    assert!(tensor.validate().is_ok());
}

#[test]
fn test_all_edge_types_represented() {
    let mut cpg = CodePropertyGraph::new();

    // Create nodes for all edge types
    for i in 0..8 {
        cpg.addNode(CPGNode {
            id: i,
            name: format!("node_{}", i),
            node_type: CPGNodeType::Function,
            file_path: "test.rs".into(),
            start_line: i + 1,
            end_line: i + 5,
            scope_depth: 0,
            data_type: None,
            features: vec![],
        });
    }

    // Add all 8 edge types
    cpg.addEdge(0, 1, CPGEdgeType::AstChild);
    cpg.addEdge(0, 2, CPGEdgeType::CfgNext);
    cpg.addEdge(1, 3, CPGEdgeType::CfgBranchTrue);
    cpg.addEdge(1, 4, CPGEdgeType::CfgBranchFalse);
    cpg.addEdge(2, 5, CPGEdgeType::CfgLoopBack);
    cpg.addEdge(3, 6, CPGEdgeType::DfgDefUse);
    cpg.addEdge(6, 7, CPGEdgeType::Call);
    cpg.addEdge(7, 0, CPGEdgeType::CallerOf);

    let extractor = CpgFeatureExtractor::new(500, 2000);
    let tensor = extractor.extract_from_cpg(&cpg).unwrap();

    // All 8 edges should be present with correct type encodings
    assert_eq!(tensor.num_edges, 8);

    let expected_types = vec![
        CPGEdgeType::AstChild,
        CPGEdgeType::CfgNext,
        CPGEdgeType::CfgBranchTrue,
        CPGEdgeType::CfgBranchFalse,
        CPGEdgeType::CfgLoopBack,
        CPGEdgeType::DfgDefUse,
        CPGEdgeType::Call,
        CPGEdgeType::CallerOf,
    ];

    for (i, edge_type) in expected_types.iter().enumerate() {
        assert_eq!(tensor.edge_types[i], edge_type.encoding(), "Edge {} type mismatch", i);
    }

    // Verify src/dst pairs
    assert_eq!(tensor.edge_src[0], 0);
    assert_eq!(tensor.edge_dst[0], 1);
    assert_eq!(tensor.edge_src[7], 7);
    assert_eq!(tensor.edge_dst[7], 0);
}

// ── Stage 8: Pattern Memory — Store / Recall / Persist lifecycle ─────────────

/// Create a unique temp models dir per test to avoid cross-test contamination.
fn create_isolated_models_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "aether_neural_pm_test_{}",
        std::thread::current().name().unwrap_or("unknown").replace("::", "_")
    ));
    let _ = std::fs::create_dir_all(&dir);

    for name in &["code_reasoner", "pattern_memory", "drift_predictor"] {
        let path = dir.join(format!("{}.burnpack", name));
        if !path.exists() {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(b"stub").unwrap();
        }
    }

    dir
}

fn make_test_meta(id: &str, category: &str) -> ExperienceMeta {
    ExperienceMeta {
        experience_id: id.into(),
        category: category.into(),
        description: format!("test pattern {id}"),
        source_file: Some(format!("src/{id}.rs")),
        stored_at: 1_700_000_000,
    }
}

/// Test that AetherNeural loads with Pattern Memory available and reports size 0.
#[test]
fn test_stage8_neural_loads_with_pattern_memory() {
    let models_dir = create_isolated_models_dir();
    let config = NeuralConfig {
        models_dir: models_dir.clone(),
        ..NeuralConfig::default()
    };

    let neural = AetherNeural::load(config).expect("AetherNeural should load");
    assert_eq!(neural.pattern_memory_size(), 0, "Freshly loaded memory should be empty");

    let status = neural.status();
    assert!(status.pattern_memory_loaded, "PatternMemory burnpack exists → should report loaded");
}

/// Test store + search lifecycle through AetherNeural API.
#[test]
fn test_stage8_store_and_search_single_experience() {
    let models_dir = create_isolated_models_dir();
    let config = NeuralConfig {
        models_dir,
        ..NeuralConfig::default()
    };

    let mut neural = AetherNeural::load(config).expect("load");

    // Store an embedding (256-dim, matching EMBED_DIM)
    let embedding: Vec<f32> = (0..256).map(|i| (i as f32) / 256.0).collect();
    let meta = make_test_meta("exp-single", "UnhandledError");

    neural.store_experience(&embedding, meta).expect("store should succeed");
    assert_eq!(neural.pattern_memory_size(), 1);

    // Search with identical embedding → should find the stored pattern with high similarity
    let results = neural.search_memory(&embedding, 1).expect("search should succeed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].experience_id, "exp-single");
    assert!(results[0].similarity > 0.99, "Self-search should yield ~1.0 similarity");
}

/// Test batch store through AetherNeural API.
#[test]
fn test_stage8_batch_store_and_search() {
    let models_dir = create_isolated_models_dir();
    let config = NeuralConfig {
        models_dir,
        ..NeuralConfig::default()
    };

    let mut neural = AetherNeural::load(config).expect("load");

    // Create 5 distinct embeddings
    let mut embeddings = Vec::new();
    let metas: Vec<ExperienceMeta> = (0..5)
        .map(|i| {
            let emb: Vec<f32> = (0..256)
                .map(|j| if j == i { 1.0 } else { 0.0 })
                .collect();
            embeddings.extend_from_slice(&emb);
            make_test_meta(&format!("exp-batch-{i}"), "CodeSmell")
        })
        .collect();

    neural.store_experience_batch(&embeddings, metas).expect("batch store should succeed");
    assert_eq!(neural.pattern_memory_size(), 5);

    // Search for the pattern with feature at index 2
    let query: Vec<f32> = (0..256).map(|j| if j == 2 { 1.0 } else { 0.0 }).collect();
    let results = neural.search_memory(&query, 3).expect("search should succeed");

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].experience_id, "exp-batch-2", "Top match should be exact match");
    assert!(results[0].similarity > 0.99);
}

/// Test persistence roundtrip: store → persist → reload → verify.
#[test]
fn test_stage8_persist_and_reload() {
    let models_dir = create_isolated_models_dir();
    let config = NeuralConfig {
        models_dir: models_dir.clone(),
        ..NeuralConfig::default()
    };

    // Phase 1: store and persist
    {
        let mut neural = AetherNeural::load(config.clone()).expect("load");
        let emb: Vec<f32> = (0..256).map(|i| 1.0).collect();
        neural.store_experience(&emb, make_test_meta("exp-persist", "Clean")).expect("store");
        neural.persist_pattern_memory().expect("persist");
    }

    // Phase 2: reload from same models_dir — hopfield_state.bin should exist
    let config2 = NeuralConfig {
        models_dir,
        ..NeuralConfig::default()
    };
    let neural2 = AetherNeural::load(config2).expect("reload");
    assert_eq!(neural2.pattern_memory_size(), 1, "Reloaded memory should have 1 pattern");

    let query: Vec<f32> = (0..256).map(|i| 1.0).collect();
    let results = neural2.search_memory(&query, 1).expect("search after reload");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].experience_id, "exp-persist");
    assert!(results[0].similarity > 0.99, "Reloaded pattern should match perfectly");
}

/// Test search on empty store returns empty results.
#[test]
fn test_stage8_search_empty_store() {
    let models_dir = create_isolated_models_dir();
    let config = NeuralConfig {
        models_dir,
        ..NeuralConfig::default()
    };

    let neural = AetherNeural::load(config).expect("load");
    let query: Vec<f32> = vec![0.0; 256];
    let results = neural.search_memory(&query, 5).expect("search on empty should not error");
    assert!(results.is_empty());
}

/// Test wrong-dimension embedding is rejected (assertion in HopfieldStore).
#[test]
#[should_panic(expected = "embedding dimension mismatch")]
fn test_stage8_wrong_dimension_panics() {
    let models_dir = create_isolated_models_dir();
    let config = NeuralConfig {
        models_dir,
        ..NeuralConfig::default()
    };

    let mut neural = AetherNeural::load(config).expect("load");
    let wrong_emb: Vec<f32> = vec![0.0; 128]; // 128 instead of 256
    let _ = neural.store_experience(&wrong_emb, make_test_meta("bad", "x"));
}

/// Test Pattern Memory search boosts orchestration confidence.
#[test]
fn test_stage8_pattern_memory_boosts_orchestration() {
    let cpg = build_problematic_cpg();
    let extractor = CpgFeatureExtractor::new(500, 2000);
    let _tensor = extractor.extract_from_cpg(&cpg).unwrap();

    let orchestrator = NeuralOrchestrator::default();

    let classifications = vec![Classification {
        category: ClassificationCategory::UnhandledError,
        confidence: 0.70,
        attention_nodes: vec![(1, 0.8)],
        description: "Bare unwrap in handler".into(),
    }];

    // Without pattern memory
    let route_no_pm = orchestrator.route(&classifications, &[], None);

    // With pattern memory confirming the classification
    let similar = vec![PatternMatch {
        experience_id: "exp-boost".into(),
        similarity: 0.92,
        category: "UnhandledError".into(),
        description: "Identical unwrap pattern in error path".into(),
        source_file: Some("legacy/handler.rs".into()),
    }];

    let route_with_pm = orchestrator.route(&classifications, &similar, None);

    assert!(
        route_with_pm.confidence() >= route_no_pm.confidence(),
        "Pattern memory confirmation should not reduce confidence"
    );
}

/// Test FIFO eviction at capacity boundary.
#[test]
fn test_stage8_fifo_eviction_through_neural() {
    let mut pm = PatternMemory::with_capacity(256, 3);

    for i in 0..5 {
        let emb: Vec<f32> = (0..256).map(|j| if j == i { 1.0 } else { 0.0 }).collect();
        pm.store_embedding(&emb, make_test_meta(&format!("fifo-{i}"), "test"));
    }

    // Capacity is 3, so first 2 should be evicted
    assert_eq!(pm.num_stored(), 3);

    // Search for pattern 0 → should NOT be found (evicted)
    let query_0: Vec<f32> = (0..256).map(|j| if j == 0 { 1.0 } else { 0.0 }).collect();
    let results = pm.search(&query_0, 1);
    assert!(results.is_empty() || results[0].experience_id != "fifo-0", "Oldest pattern should be evicted");

    // Search for pattern 2 → should be found (survived eviction)
    let query_2: Vec<f32> = (0..256).map(|j| if j == 2 { 1.0 } else { 0.0 }).collect();
    let results = pm.search(&query_2, 1);
    assert_eq!(results.len(), 1, "Pattern 2 should still exist");
    assert_eq!(results[0].experience_id, "fifo-2");

    // Search for latest pattern 4 → should be found
    let query_4: Vec<f32> = (0..256).map(|j| if j == 4 { 1.0 } else { 0.0 }).collect();
    let results = pm.search(&query_4, 1);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].experience_id, "fifo-4");
}
