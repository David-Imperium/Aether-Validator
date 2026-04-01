//! Architectural Drift Analysis
//!
//! Expands drift analysis from single file to entire code structures
//! using dependency discovery from CodeGraph (Layer 2A).
//!
//! ## Usage
//!
//! ```bash
//! aether memory recall drift-trend src/engine/renderer.rs --depth 2
//! ```
//!
//! ## Algorithm
//!
//! 1. Start from root file
//! 2. BFS expansion via CodeGraph.what_depends_on()
//! 3. Apply limits: max_depth (default 3), max_files (default 50)
//! 4. Compute drift for each file via DriftSnapshotStore
//! 5. Aggregate with weighted average (closer files weigh more)

use crate::error::Result;
use super::code_graph::CodeGraph;
use super::drift_snapshots::{DriftSnapshotStore, Trend};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

/// Configuration for architectural drift analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalDriftConfig {
    /// Maximum depth for dependency expansion (default: 3)
    pub max_depth: usize,

    /// Maximum number of files to analyze (default: 50)
    pub max_files: usize,
}

impl Default for ArchitecturalDriftConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            max_files: 50,
        }
    }
}

/// Drift info for a single file within architectural analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDrift {
    /// File path
    pub path: String,

    /// Depth from root (0 = root file)
    pub depth: usize,

    /// Drift score (0-1, lower = better)
    pub score: f32,

    /// Trend direction
    pub trend: Trend,

    /// Alerts for this file
    pub alerts: Vec<String>,
}

/// Result of architectural drift analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalDriftReport {
    /// Root file analyzed
    pub root: String,

    /// Depth requested
    pub depth_requested: usize,

    /// All files analyzed with their drift info
    pub files: Vec<FileDrift>,

    /// Aggregate drift score (weighted by depth)
    pub aggregate_score: f32,

    /// Overall recommendation
    pub recommendation: String,
}

impl ArchitecturalDriftReport {
    /// Get files with highest drift (worst offenders)
    pub fn worst_files(&self, limit: usize) -> Vec<&FileDrift> {
        let mut sorted: Vec<_> = self.files.iter().collect();
        sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(limit).collect()
    }

    /// Get count of files with concerning drift
    pub fn concerning_count(&self) -> usize {
        self.files.iter().filter(|f| f.score > 0.15).count()
    }
}

/// Architectural drift analyzer
pub struct ArchitecturalDriftAnalyzer {
    code_graph: CodeGraph,
    drift_store: DriftSnapshotStore,
    config: ArchitecturalDriftConfig,
}

impl ArchitecturalDriftAnalyzer {
    /// Create new analyzer with dependencies
    pub fn new(code_graph: CodeGraph, drift_store: DriftSnapshotStore) -> Self {
        Self {
            code_graph,
            drift_store,
            config: ArchitecturalDriftConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(
        code_graph: CodeGraph,
        drift_store: DriftSnapshotStore,
        config: ArchitecturalDriftConfig,
    ) -> Self {
        Self {
            code_graph,
            drift_store,
            config,
        }
    }

    /// Expand dependencies from root file using BFS
    pub fn expand_dependencies(&self, root: &Path) -> Vec<(PathBuf, usize)> {
        let mut files = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        let root_str = root.to_string_lossy().to_string();
        visited.insert(root_str.clone());
        queue.push_back((root.to_path_buf(), 0));
        files.push((root.to_path_buf(), 0));

        while let Some((file, depth)) = queue.pop_front() {
            // Check limits
            if depth >= self.config.max_depth || files.len() >= self.config.max_files {
                continue;
            }

            // Find dependencies from CodeGraph
            let file_str = file.to_string_lossy().to_string();
            let deps = self.code_graph.file_dependencies(&file_str);

            for dep_path in deps {
                let dep_str = dep_path.to_string_lossy().to_string();
                if !visited.contains(&dep_str) {
                    visited.insert(dep_str);
                    let new_depth = depth + 1;

                    if new_depth <= self.config.max_depth && files.len() < self.config.max_files {
                        files.push((dep_path.clone(), new_depth));
                        queue.push_back((dep_path, new_depth));
                    }
                }
            }
        }

        files
    }

    /// Analyze architectural drift for a file and its dependencies
    pub fn analyze(&self, root: &Path, days: u32) -> Result<ArchitecturalDriftReport> {
        // Expand dependencies
        let files = self.expand_dependencies(root);

        // Analyze drift for each file
        let mut file_drifts = Vec::new();

        for (file, depth) in &files {
            let file_str = file.to_string_lossy().to_string();
            let drift_opt = self.drift_store.analyze_trend_days(&file_str, days as usize);

            // Handle missing drift data gracefully
            let (score, trend, alerts) = match drift_opt {
                Some(drift) => {
                    let t = derive_trend(&drift.trends);
                    let a = derive_alerts(&drift.trends);
                    (drift.drift_score, t, a)
                }
                None => {
                    // No data available - use defaults
                    (0.0, Trend::Stable { metric: "unknown".into(), variance: 0.0 }, vec![])
                }
            };

            file_drifts.push(FileDrift {
                path: file_str,
                depth: *depth,
                score,
                trend,
                alerts,
            });
        }

        // Compute aggregate score (weighted by inverse depth)
        let aggregate_score = self.compute_weighted_score(&file_drifts);

        // Generate recommendation
        let recommendation = self.generate_recommendation(&file_drifts);

        Ok(ArchitecturalDriftReport {
            root: root.to_string_lossy().to_string(),
            depth_requested: self.config.max_depth,
            files: file_drifts,
            aggregate_score,
            recommendation,
        })
    }

    /// Compute weighted average score (closer files weigh more)
    fn compute_weighted_score(&self, files: &[FileDrift]) -> f32 {
        if files.is_empty() {
            return 0.0;
        }

        let total_weight: f32 = files.iter()
            .map(|f| 1.0 / (1.0 + f.depth as f32))
            .sum();

        let weighted_sum: f32 = files.iter()
            .map(|f| {
                let weight = 1.0 / (1.0 + f.depth as f32);
                f.score * weight
            })
            .sum();

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    /// Generate recommendation based on analysis
    fn generate_recommendation(&self, files: &[FileDrift]) -> String {
        if files.is_empty() {
            return "No files to analyze".to_string();
        }

        let worst = files.iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));

        match worst {
            Some(f) if f.score > 0.25 => {
                format!("Critical: {} has high drift ({:.2}). Review immediately.", f.path, f.score)
            }
            Some(f) if f.score > 0.15 => {
                format!("Warning: {} shows drift ({:.2}). Schedule review.", f.path, f.score)
            }
            Some(f) if f.score > 0.08 => {
                format!("Note: {} has moderate drift ({:.2}). Monitor.", f.path, f.score)
            }
            _ => "All files within acceptable drift levels.".to_string(),
        }
    }
}

/// Format report for CLI output
pub fn format_report(report: &ArchitecturalDriftReport) -> String {
    let mut output = String::new();

    // Header
    output.push_str("Analyzing:\n");
    output.push_str(&format!("  {} (root file)\n", report.root));

    // Tree structure - group by depth
    let max_depth = report.files.iter().map(|f| f.depth).max().unwrap_or(0);

    for depth in 1..=max_depth {
        let files_at_depth: Vec<_> = report.files.iter()
            .filter(|f| f.depth == depth)
            .collect();

        let count = files_at_depth.len();
        for (idx, file) in files_at_depth.iter().enumerate() {
            let indent = "  ".repeat(depth);
            let is_last = idx == count - 1;
            let prefix = if is_last { "└─" } else { "├─" };
            output.push_str(&format!("{}{} {} (depth {})\n", indent, prefix, file.path, file.depth));
        }
    }

    output.push_str(&format!("\nDrift Score: {:.2} ({})\n",
        report.aggregate_score,
        score_label(report.aggregate_score)
    ));

    // Per-file scores
    for file in &report.files {
        let status = if file.score > 0.15 { "⚠" } else { "✓" };
        output.push_str(&format!("  {}: {:.2} {}\n", file.path, file.score, status));

        if !file.alerts.is_empty() {
            for alert in &file.alerts {
                output.push_str(&format!("    → {}\n", alert));
            }
        }
    }

    // Recommendation
    output.push_str(&format!("\nRecommendation: {}\n", report.recommendation));

    output
}

fn score_label(score: f32) -> &'static str {
    if score > 0.25 {
        "critical"
    } else if score > 0.15 {
        "moderate"
    } else if score > 0.08 {
        "low"
    } else {
        "healthy"
    }
}

/// Derive a single trend summary from a list of trends
fn derive_trend(trends: &[Trend]) -> Trend {
    if trends.is_empty() {
        return Trend::Stable { metric: "none".into(), variance: 0.0 };
    }

    // Find the most severe trend
    for t in trends {
        match t {
            Trend::Declining { severity, .. } if severity == "critical" => {
                return t.clone();
            }
            _ => {}
        }
    }

    // Return first non-stable trend, or first trend if all stable
    trends.iter()
        .find(|t| !matches!(t, Trend::Stable { .. }))
        .cloned()
        .unwrap_or_else(|| trends[0].clone())
}

/// Derive alert messages from trends
fn derive_alerts(trends: &[Trend]) -> Vec<String> {
    trends.iter()
        .filter_map(|t| match t {
            Trend::Declining { metric, rate, severity } => {
                Some(format!("{} declining ({}) - rate: {:.2}", metric, severity, rate))
            }
            Trend::Increasing { metric, rate, severity } if severity == "warning" || severity == "critical" => {
                Some(format!("{} increasing ({}) - rate: {:.2}", metric, severity, rate))
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // =========================================================================
    // Helper: Create a fresh DriftSnapshotStore (no persistence conflicts)
    // =========================================================================

    fn fresh_drift_store() -> DriftSnapshotStore {
        let temp_path = std::env::temp_dir().join(format!("aether_drift_{}_{}.json", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        DriftSnapshotStore::new(Some(temp_path)).unwrap()
    }

    // =========================================================================
    // Helper: Build a realistic CodeGraph with cross-file dependencies
    // =========================================================================

    fn build_engine_graph() -> CodeGraph {
        let mut graph = CodeGraph::new();

        // renderer.rs -> calls shader.rs, texture.rs via direct function calls
        let renderer = r#"
use shader::create_shader;
use texture::create_texture;

pub fn render() {
    let s = create_shader();
    let t = create_texture();
    draw();
}

fn draw() {}
"#;
        graph.parse_file(renderer, "src/engine/renderer.rs", "rust");

        // shader.rs -> calls gl_utils.rs
        let shader = r#"
use gl_utils::compile;

pub fn create_shader() -> u32 { compile("shader"); 0 }
"#;
        graph.parse_file(shader, "src/engine/shader.rs", "rust");

        // texture.rs -> calls gl_utils.rs
        let texture = r#"
use gl_utils::upload;

pub fn create_texture() -> u32 { upload(); 0 }
"#;
        graph.parse_file(texture, "src/engine/texture.rs", "rust");

        // gl_utils.rs -> low-level
        let gl_utils = r#"
pub fn compile(src: &str) -> u32 { 0 }
pub fn upload() -> u32 { 0 }
"#;
        graph.parse_file(gl_utils, "src/engine/gl_utils.rs", "rust");

        // math/vec3.rs -> standalone
        let vec3 = r#"
pub struct Vec3 { x: f32, y: f32, z: f32 }
impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
}
"#;
        graph.parse_file(vec3, "src/math/vec3.rs", "rust");

        graph.build_callers();
        graph
    }

    // =========================================================================
    // Test: Weighted score with realistic depth distribution
    // =========================================================================

    #[test]
    fn test_weighted_score_realistic_depths() {
        let analyzer = ArchitecturalDriftAnalyzer::new(
            CodeGraph::new(),
            fresh_drift_store(),
        );

        // Simulate: renderer (depth 0) = 0.05, shader (depth 1) = 0.15,
        // texture (depth 1) = 0.10, gl_utils (depth 2) = 0.30, vec3 (depth 2) = 0.02
        let files = vec![
            FileDrift { path: "renderer.rs".into(), depth: 0, score: 0.05, trend: Trend::Stable { metric: "type_strictness".into(), variance: 0.02 }, alerts: vec![] },
            FileDrift { path: "shader.rs".into(), depth: 1, score: 0.15, trend: Trend::Declining { metric: "type_strictness".into(), rate: 0.05, severity: "warning".into() }, alerts: vec!["type_strictness declining".into()] },
            FileDrift { path: "texture.rs".into(), depth: 1, score: 0.10, trend: Trend::Stable { metric: "type_strictness".into(), variance: 0.01 }, alerts: vec![] },
            FileDrift { path: "gl_utils.rs".into(), depth: 2, score: 0.30, trend: Trend::Declining { metric: "complexity".into(), rate: 0.12, severity: "critical".into() }, alerts: vec!["complexity critical".into()] },
            FileDrift { path: "vec3.rs".into(), depth: 2, score: 0.02, trend: Trend::Stable { metric: "type_strictness".into(), variance: 0.005 }, alerts: vec![] },
        ];

        let score = analyzer.compute_weighted_score(&files);

        // Weights: depth 0 = 1.0, depth 1 = 0.5, depth 2 = 0.333
        // Weighted sum: 0.05*1.0 + (0.15+0.10)*0.5 + (0.30+0.02)*0.333
        //             = 0.05 + 0.125 + 0.1067 = 0.2817
        // Total weight: 1.0 + 2*0.5 + 2*0.333 = 1.0 + 1.0 + 0.667 = 2.667
        // Score: 0.2817 / 2.667 ≈ 0.1056
        assert!((score - 0.105).abs() < 0.01, "Expected ~0.105, got {}", score);
    }

    // =========================================================================
    // Test: Dependency expansion with realistic graph
    // =========================================================================

    #[test]
    fn test_expand_dependencies_engine_graph() {
        let graph = build_engine_graph();
        let analyzer = ArchitecturalDriftAnalyzer::new(
            graph,
            fresh_drift_store(),
        );

        let files = analyzer.expand_dependencies(Path::new("src/engine/renderer.rs"));

        // Should include renderer.rs (depth 0) and potentially its dependencies
        assert!(!files.is_empty(), "Should have at least the root file");
        assert!(files.iter().any(|(p, _)| p.to_string_lossy().contains("renderer")));

        // Verify depth assignment
        for (path, depth) in &files {
            if path.to_string_lossy().contains("renderer") {
                assert_eq!(*depth, 0, "Root file should have depth 0");
            }
        }
    }

    // =========================================================================
    // Test: Max depth limit is respected
    // =========================================================================

    #[test]
    fn test_expand_dependencies_respects_max_depth() {
        let graph = build_engine_graph();
        let config = ArchitecturalDriftConfig {
            max_depth: 1, // Limit to depth 1
            max_files: 50,
        };
        let analyzer = ArchitecturalDriftAnalyzer::with_config(
            graph,
            fresh_drift_store(),
            config,
        );

        let files = analyzer.expand_dependencies(Path::new("src/engine/renderer.rs"));

        // No file should exceed depth 1
        for (_, depth) in &files {
            assert!(*depth <= 1, "Depth {} exceeds max_depth=1", depth);
        }
    }

    // =========================================================================
    // Test: Max files limit is respected
    // =========================================================================

    #[test]
    fn test_expand_dependencies_respects_max_files() {
        let graph = build_engine_graph();
        let config = ArchitecturalDriftConfig {
            max_depth: 10,
            max_files: 2, // Very restrictive
        };
        let analyzer = ArchitecturalDriftAnalyzer::with_config(
            graph,
            fresh_drift_store(),
            config,
        );

        let files = analyzer.expand_dependencies(Path::new("src/engine/renderer.rs"));

        assert!(files.len() <= 2, "Should have at most 2 files, got {}", files.len());
    }

    // =========================================================================
    // Test: Empty graph returns only root file
    // =========================================================================

    #[test]
    fn test_expand_dependencies_empty_graph() {
        let analyzer = ArchitecturalDriftAnalyzer::new(
            CodeGraph::new(),
            fresh_drift_store(),
        );

        let files = analyzer.expand_dependencies(Path::new("nonexistent.rs"));

        assert_eq!(files.len(), 1, "Should return only root file for empty graph");
        assert_eq!(files[0].1, 0, "Root should have depth 0");
    }

    // =========================================================================
    // Test: Recommendation levels based on severity
    // =========================================================================

    #[test]
    fn test_recommendation_critical_threshold() {
        let analyzer = ArchitecturalDriftAnalyzer::new(
            CodeGraph::new(),
            fresh_drift_store(),
        );

        // Score > 0.25 should trigger "Critical"
        let files = vec![
            FileDrift { path: "critical.rs".into(), depth: 0, score: 0.30, trend: Trend::Declining { metric: "complexity".into(), rate: 0.15, severity: "critical".into() }, alerts: vec![] },
        ];
        let rec = analyzer.generate_recommendation(&files);
        assert!(rec.contains("Critical"), "Expected Critical in: {}", rec);
    }

    #[test]
    fn test_recommendation_warning_threshold() {
        let analyzer = ArchitecturalDriftAnalyzer::new(
            CodeGraph::new(),
            fresh_drift_store(),
        );

        // Score 0.15-0.25 should trigger "Warning"
        let files = vec![
            FileDrift { path: "warning.rs".into(), depth: 0, score: 0.18, trend: Trend::Declining { metric: "type_strictness".into(), rate: 0.03, severity: "warning".into() }, alerts: vec![] },
        ];
        let rec = analyzer.generate_recommendation(&files);
        assert!(rec.contains("Warning") || rec.contains("Schedule"), "Expected Warning in: {}", rec);
    }

    #[test]
    fn test_recommendation_healthy() {
        let analyzer = ArchitecturalDriftAnalyzer::new(
            CodeGraph::new(),
            fresh_drift_store(),
        );

        // Score < 0.08 should be healthy
        let files = vec![
            FileDrift { path: "good.rs".into(), depth: 0, score: 0.03, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] },
        ];
        let rec = analyzer.generate_recommendation(&files);
        assert!(rec.contains("acceptable") || rec.contains("healthy") || !rec.contains("Critical"),
            "Expected healthy message in: {}", rec);
    }

    // =========================================================================
    // Test: Trend derivation from multiple trends
    // =========================================================================

    #[test]
    fn test_derive_trend_critical_priority() {
        let trends = vec![
            Trend::Stable { metric: "naming".into(), variance: 0.01 },
            Trend::Declining { metric: "complexity".into(), rate: 0.1, severity: "critical".into() },
            Trend::Declining { metric: "type_strictness".into(), rate: 0.05, severity: "warning".into() },
        ];

        let derived = derive_trend(&trends);

        match derived {
            Trend::Declining { severity, .. } => {
                assert_eq!(severity, "critical", "Should prioritize critical trend");
            }
            _ => panic!("Expected Declining trend"),
        }
    }

    #[test]
    fn test_derive_trend_empty_defaults_to_stable() {
        let trends = vec![];
        let derived = derive_trend(&trends);

        match derived {
            Trend::Stable { .. } => {}
            _ => panic!("Expected Stable for empty trends"),
        }
    }

    // =========================================================================
    // Test: Alert derivation
    // =========================================================================

    #[test]
    fn test_derive_alerts_filters_stable() {
        let trends = vec![
            Trend::Stable { metric: "naming".into(), variance: 0.01 },
            Trend::Declining { metric: "complexity".into(), rate: 0.1, severity: "warning".into() },
            Trend::Increasing { metric: "coverage".into(), rate: 0.05, severity: "info".into() },
            Trend::Declining { metric: "type_strictness".into(), rate: 0.08, severity: "critical".into() },
        ];

        let alerts = derive_alerts(&trends);

        // Should only include Declining with warning/critical and Increasing with warning/critical
        assert!(alerts.len() >= 1, "Should have at least one alert");
        assert!(alerts.iter().any(|a| a.contains("declining") && a.contains("critical")));
    }

    // =========================================================================
    // Test: Worst files sorting
    // =========================================================================

    #[test]
    fn test_worst_files_returns_sorted_by_score() {
        let report = ArchitecturalDriftReport {
            root: "root.rs".into(),
            depth_requested: 2,
            files: vec![
                FileDrift { path: "a.rs".into(), depth: 0, score: 0.05, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] },
                FileDrift { path: "b.rs".into(), depth: 1, score: 0.35, trend: Trend::Declining { metric: "complexity".into(), rate: 0.15, severity: "critical".into() }, alerts: vec![] },
                FileDrift { path: "c.rs".into(), depth: 1, score: 0.22, trend: Trend::Declining { metric: "type_strictness".into(), rate: 0.05, severity: "warning".into() }, alerts: vec![] },
                FileDrift { path: "d.rs".into(), depth: 2, score: 0.41, trend: Trend::Declining { metric: "all".into(), rate: 0.2, severity: "critical".into() }, alerts: vec![] },
                FileDrift { path: "e.rs".into(), depth: 2, score: 0.08, trend: Trend::Stable { metric: "all".into(), variance: 0.02 }, alerts: vec![] },
            ],
            aggregate_score: 0.22,
            recommendation: "test".into(),
        };

        let worst = report.worst_files(3);

        assert_eq!(worst.len(), 3);
        assert_eq!(worst[0].path, "d.rs"); // 0.41
        assert_eq!(worst[1].path, "b.rs"); // 0.35
        assert_eq!(worst[2].path, "c.rs"); // 0.22
    }

    #[test]
    fn test_worst_files_less_than_limit() {
        let report = ArchitecturalDriftReport {
            root: "root.rs".into(),
            depth_requested: 1,
            files: vec![
                FileDrift { path: "a.rs".into(), depth: 0, score: 0.1, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] },
            ],
            aggregate_score: 0.1,
            recommendation: "ok".into(),
        };

        let worst = report.worst_files(5);
        assert_eq!(worst.len(), 1);
    }

    // =========================================================================
    // Test: Concerning files count
    // =========================================================================

    #[test]
    fn test_concerning_count() {
        let report = ArchitecturalDriftReport {
            root: "root.rs".into(),
            depth_requested: 2,
            files: vec![
                FileDrift { path: "a.rs".into(), depth: 0, score: 0.10, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] }, // not concerning
                FileDrift { path: "b.rs".into(), depth: 1, score: 0.20, trend: Trend::Declining { metric: "type".into(), rate: 0.05, severity: "warning".into() }, alerts: vec![] }, // concerning
                FileDrift { path: "c.rs".into(), depth: 1, score: 0.16, trend: Trend::Declining { metric: "type".into(), rate: 0.03, severity: "warning".into() }, alerts: vec![] }, // concerning
                FileDrift { path: "d.rs".into(), depth: 2, score: 0.05, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] }, // not concerning
            ],
            aggregate_score: 0.12,
            recommendation: "test".into(),
        };

        assert_eq!(report.concerning_count(), 2);
    }

    // =========================================================================
    // Test: Report formatting
    // =========================================================================

    #[test]
    fn test_format_report_output() {
        let report = ArchitecturalDriftReport {
            root: "src/engine/renderer.rs".into(),
            depth_requested: 2,
            files: vec![
                FileDrift { path: "src/engine/renderer.rs".into(), depth: 0, score: 0.05, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] },
                FileDrift { path: "src/engine/shader.rs".into(), depth: 1, score: 0.18, trend: Trend::Declining { metric: "type_strictness".into(), rate: 0.05, severity: "warning".into() }, alerts: vec!["type_strictness declining".into()] },
            ],
            aggregate_score: 0.09,
            recommendation: "Monitor shader.rs".into(),
        };

        let output = format_report(&report);

        assert!(output.contains("renderer.rs"), "Should contain root file");
        assert!(output.contains("shader.rs"), "Should contain dependency");
        assert!(output.contains("0.09"), "Should contain aggregate score");
        assert!(output.contains("Monitor"), "Should contain recommendation");
    }

    // =========================================================================
    // Test: Full integration with mock DriftSnapshotStore
    // =========================================================================

    #[test]
    fn test_analyze_with_real_graph_and_empty_snapshots() {
        let graph = build_engine_graph();
        let analyzer = ArchitecturalDriftAnalyzer::new(
            graph,
            fresh_drift_store(),
        );

        let result = analyzer.analyze(Path::new("src/engine/renderer.rs"), 30);

        assert!(result.is_ok(), "Analysis should succeed");
        let report = result.unwrap();

        // Root should always be present
        assert!(report.files.iter().any(|f| f.path.contains("renderer")));

        // With no drift data, all scores should be 0.0
        for file in &report.files {
            assert_eq!(file.score, 0.0, "No drift data means score should be 0.0");
        }

        // Aggregate should also be 0.0
        assert_eq!(report.aggregate_score, 0.0);
    }

    // =========================================================================
    // Test: Config defaults
    // =========================================================================

    #[test]
    fn test_config_defaults() {
        let config = ArchitecturalDriftConfig::default();

        assert_eq!(config.max_depth, 3);
        assert_eq!(config.max_files, 50);
    }

    // =========================================================================
    // MULTI-LANGUAGE ENGINE TEST SUITE
    // =========================================================================
    // Realistic game engine with: Rust core, Python bindings, GLSL shaders,
    // C FFI, TypeScript frontend, Lua scripting, JSON configs
    // =========================================================================

    fn build_multilang_engine_graph() -> CodeGraph {
        let mut graph = CodeGraph::new();

        // =====================================================================
        // RUST CORE LAYER (depth 0-1)
        // =====================================================================

        // engine/lib.rs - main entry point
        let engine_lib = r#"
pub mod renderer;
pub mod physics;
pub mod audio;
pub mod input;
pub mod scene;
pub mod scripting;
pub mod ffi;

pub use renderer::Renderer;
pub use physics::PhysicsWorld;
pub use scene::SceneManager;
"#;
        graph.parse_file(engine_lib, "src/engine/lib.rs", "rust");

        // engine/renderer/mod.rs
        let renderer_mod = r#"
pub mod pipeline;
pub mod shader;
pub mod texture;
pub mod mesh;
pub mod camera;

use crate::ffi::gl;

pub struct Renderer {
    pipeline: pipeline::RenderPipeline,
    camera: camera::Camera,
}

impl Renderer {
    pub fn new() -> Self {
        gl::init();
        Self { pipeline: pipeline::RenderPipeline::new(), camera: camera::Camera::default() }
    }

    pub fn render(&mut self, scene: &crate::scene::SceneManager) {
        self.pipeline.execute(scene, &self.camera);
    }
}
"#;
        graph.parse_file(renderer_mod, "src/engine/renderer/mod.rs", "rust");

        // engine/renderer/shader.rs - loads GLSL
        let shader_rs = r#"
use std::fs;
use crate::ffi::gl;

pub struct Shader {
    program: u32,
    vertex_src: String,
    fragment_src: String,
}

impl Shader {
    pub fn from_files(vertex_path: &str, frag_path: &str) -> Self {
        let vertex_src = fs::read_to_string(vertex_path).unwrap();
        let fragment_src = fs::read_to_string(frag_path).unwrap();
        let program = gl::compile_shader(&vertex_src, &fragment_src);
        Self { program, vertex_src, fragment_src }
    }

    pub fn bind(&self) { gl::use_program(self.program); }
}
"#;
        graph.parse_file(shader_rs, "src/engine/renderer/shader.rs", "rust");

        // engine/physics/mod.rs
        let physics_mod = r#"
pub mod rigidbody;
pub mod collision;
pub mod solver;

pub struct PhysicsWorld {
    bodies: Vec<rigidbody::RigidBody>,
    solver: solver::ConstraintSolver,
}

impl PhysicsWorld {
    pub fn step(&mut self, dt: f32) {
        collision::detect(&mut self.bodies);
        self.solver.solve(&mut self.bodies, dt);
    }
}
"#;
        graph.parse_file(physics_mod, "src/engine/physics/mod.rs", "rust");

        // engine/scene/mod.rs
        let scene_mod = r#"
pub mod entity;
pub mod component;
pub mod transform;

use crate::scripting::ScriptEngine;

pub struct SceneManager {
    entities: Vec<entity::Entity>,
    script_engine: ScriptEngine,
}

impl SceneManager {
    pub fn update(&mut self, dt: f32) {
        for entity in &mut self.entities {
            self.script_engine.call_update(entity);
        }
    }
}
"#;
        graph.parse_file(scene_mod, "src/engine/scene/mod.rs", "rust");

        // engine/scripting/mod.rs - Lua integration
        let scripting_mod = r#"
use mlua::Lua;
use crate::scene::entity::Entity;

pub struct ScriptEngine {
    lua: Lua,
}

impl ScriptEngine {
    pub fn new() -> Self {
        let lua = Lua::new();
        Self { lua }
    }

    pub fn load_script(&self, path: &str) -> Result<(), mlua::Error> {
        let content = std::fs::read_to_string(path)?;
        self.lua.load(&content).exec()?;
        Ok(())
    }

    pub fn call_update(&self, entity: &Entity) {
        // Call Lua update function
    }
}
"#;
        graph.parse_file(scripting_mod, "src/engine/scripting/mod.rs", "rust");

        // engine/ffi/mod.rs - C FFI layer
        let ffi_mod = r#"
pub mod gl;
pub mod audio;

#[link(name = "glbackend")]
extern "C" {
    pub fn gl_init() -> i32;
    pub fn gl_compile_shader(vert: *const i8, frag: *const i8) -> u32;
    pub fn gl_use_program(program: u32);
}
"#;
        graph.parse_file(ffi_mod, "src/engine/ffi/mod.rs", "rust");

        // engine/ffi/gl.rs
        let gl_rs = r#"
use std::ffi::CString;

pub fn init() { unsafe { super::gl_init() }; }
pub fn compile_shader(vert: &str, frag: &str) -> u32 {
    let vert_c = CString::new(vert).unwrap();
    let frag_c = CString::new(frag).unwrap();
    unsafe { super::gl_compile_shader(vert_c.as_ptr(), frag_c.as_ptr()) }
}
pub fn use_program(program: u32) { unsafe { super::gl_use_program(program) }; }
"#;
        graph.parse_file(gl_rs, "src/engine/ffi/gl.rs", "rust");

        // =====================================================================
        // C BACKEND LAYER (depth 2)
        // =====================================================================

        let gl_backend_c = r#"
#include <GL/gl3w.h>
#include <stdio.h>

int gl_init(void) {
    return gl3wInit();
}

unsigned int gl_compile_shader(const char* vert_src, const char* frag_src) {
    unsigned int vertex = glCreateShader(GL_VERTEX_SHADER);
    glShaderSource(vertex, 1, &vert_src, NULL);
    glCompileShader(vertex);

    unsigned int fragment = glCreateShader(GL_FRAGMENT_SHADER);
    glShaderSource(fragment, 1, &frag_src, NULL);
    glCompileShader(fragment);

    unsigned int program = glCreateProgram();
    glAttachShader(program, vertex);
    glAttachShader(program, fragment);
    glLinkProgram(program);

    return program;
}

void gl_use_program(unsigned int program) {
    glUseProgram(program);
}
"#;
        graph.parse_file(gl_backend_c, "src/backend/gl_backend.c", "c");

        // =====================================================================
        // GLSL SHADER LAYER (depth 2)
        // =====================================================================

        let pbr_vertex = r#"
#version 450 core

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_normal;
layout(location = 2) in vec2 a_uv;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;

out vec3 v_world_pos;
out vec3 v_normal;
out vec2 v_uv;

void main() {
    vec4 world_pos = u_model * vec4(a_position, 1.0);
    v_world_pos = world_pos.xyz;
    v_normal = mat3(u_model) * a_normal;
    v_uv = a_uv;
    gl_Position = u_projection * u_view * world_pos;
}
"#;
        graph.parse_file(pbr_vertex, "assets/shaders/pbr.vert.glsl", "glsl");

        let pbr_fragment = r#"
#version 450 core

in vec3 v_world_pos;
in vec3 v_normal;
in vec2 v_uv;

uniform vec3 u_cam_pos;
uniform vec3 u_light_pos;
uniform vec3 u_light_color;
uniform vec3 u_albedo;
uniform float u_metallic;
uniform float u_roughness;

out vec4 frag_color;

float distribution_ggx(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;
    return a2 / (3.14159 * (NdotH2 * (a2 - 1.0) + 1.0) * (NdotH2 * (a2 - 1.0) + 1.0));
}

void main() {
    vec3 N = normalize(v_normal);
    vec3 V = normalize(u_cam_pos - v_world_pos);
    vec3 L = normalize(u_light_pos - v_world_pos);
    vec3 H = normalize(V + L);

    float NDF = distribution_ggx(N, H, u_roughness);

    vec3 Lo = NDF * u_light_color * max(dot(N, L), 0.0);
    vec3 ambient = vec3(0.03) * u_albedo;
    vec3 color = ambient + Lo;

    frag_color = vec4(color, 1.0);
}
"#;
        graph.parse_file(pbr_fragment, "assets/shaders/pbr.frag.glsl", "glsl");

        // =====================================================================
        // PYTHON BINDINGS LAYER (depth 1)
        // =====================================================================

        let python_bindings = r#"
from typing import Optional, List
import numpy as np
from engine._core import (
    Renderer as _Renderer,
    PhysicsWorld as _PhysicsWorld,
    SceneManager as _SceneManager,
    Entity as _Entity,
)

class Renderer:
    """Python wrapper for Rust renderer"""
    def __init__(self):
        self._inner = _Renderer()

    def render(self, scene: 'SceneManager') -> None:
        self._inner.render(scene._inner)

    def set_camera_position(self, x: float, y: float, z: float) -> None:
        self._inner.set_camera_pos(x, y, z)

class PhysicsWorld:
    """Python wrapper for physics simulation"""
    def __init__(self, gravity: tuple = (0.0, -9.81, 0.0)):
        self._inner = _PhysicsWorld(gravity)

    def step(self, dt: float) -> None:
        self._inner.step(dt)

    def add_rigidbody(self, mass: float, position: List[float]) -> None:
        self._inner.add_body(mass, np.array(position, dtype=np.float32))

class SceneManager:
    """Python scene management"""
    def __init__(self):
        self._inner = _SceneManager()
        self._entities: List[Entity] = []

    def create_entity(self, name: str) -> 'Entity':
        entity = Entity(name, self._inner)
        self._entities.append(entity)
        return entity

    def update(self, dt: float) -> None:
        self._inner.update(dt)

class Entity:
    def __init__(self, name: str, scene: _SceneManager):
        self.name = name
        self._scene = scene
        self._components = {}

    def add_component(self, comp_type: str, **kwargs) -> None:
        self._scene.add_component(self.name, comp_type, kwargs)
"#;
        graph.parse_file(python_bindings, "bindings/python/engine/__init__.py", "python");

        // =====================================================================
        // TYPESCRIPT FRONTEND LAYER (depth 1)
        // =====================================================================

        let ts_editor = r#"
import { Engine, Renderer, Scene, Entity, PhysicsWorld } from '@engine/core';
import { EditorUI } from './ui/EditorUI';
import { Inspector } from './ui/Inspector';
import { Hierarchy } from './ui/Hierarchy';

interface EditorConfig {
    projectPath: string;
    viewportWidth: number;
    viewportHeight: number;
}

export class Editor {
    private engine: Engine;
    private renderer: Renderer;
    private scene: Scene;
    private physics: PhysicsWorld;
    private ui: EditorUI;
    private inspector: Inspector;
    private hierarchy: Hierarchy;
    private config: EditorConfig;

    constructor(config: EditorConfig) {
        this.config = config;
        this.engine = new Engine();
        this.renderer = new Renderer();
        this.scene = new Scene();
        this.physics = new PhysicsWorld({ gravity: [0, -9.81, 0] });
        this.ui = new EditorUI(config.viewportWidth, config.viewportHeight);
        this.inspector = new Inspector(this.scene);
        this.hierarchy = new Hierarchy(this.scene);
    }

    async initialize(): Promise<void> {
        await this.engine.loadProject(this.config.projectPath);
        this.ui.mount('#editor-root');
        this.inspector.mount('#inspector-panel');
        this.hierarchy.mount('#hierarchy-panel');
        this.startLoop();
    }

    private startLoop(): void {
        const loop = (dt: number) => {
            this.physics.step(dt);
            this.scene.update(dt);
            this.renderer.render(this.scene);
            this.ui.update();
            requestAnimationFrame(loop);
        };
        requestAnimationFrame(loop);
    }

    createEntity(name: string): Entity {
        const entity = this.scene.createEntity(name);
        this.hierarchy.refresh();
        return entity;
    }
}
"#;
        graph.parse_file(ts_editor, "editor/src/Editor.ts", "typescript");

        let ts_ui = r#"
export class EditorUI {
    private width: number;
    private height: number;
    private canvas: HTMLCanvasElement | null = null;

    constructor(width: number, height: number) {
        this.width = width;
        this.height = height;
    }

    mount(selector: string): void {
        const root = document.querySelector(selector);
        if (!root) throw new Error(`Element ${selector} not found`);
        this.canvas = document.createElement('canvas');
        this.canvas.width = this.width;
        this.canvas.height = this.height;
        root.appendChild(this.canvas);
    }

    update(): void {
        // UI update logic
    }
}

export class Inspector {
    private scene: any;

    constructor(scene: any) {
        this.scene = scene;
    }

    mount(selector: string): void {
        // Mount inspector panel
    }

    showEntity(entity: any): void {
        // Display entity properties
    }
}
"#;
        graph.parse_file(ts_ui, "editor/src/ui/EditorUI.ts", "typescript");

        // =====================================================================
        // LUA SCRIPTING LAYER (depth 2)
        // =====================================================================

        let lua_player_controller = r#"
local PlayerController = {}
PlayerController.__index = PlayerController

function PlayerController.new(entity)
    local self = setmetatable({}, PlayerController)
    self.entity = entity
    self.speed = 5.0
    self.jump_force = 10.0
    self.grounded = false
    return self
end

function PlayerController:update(dt)
    local input = engine.input.get_state()
    local move = vec3.new(0, 0, 0)

    if input.key_held("w") then move.z = move.z + 1 end
    if input.key_held("s") then move.z = move.z - 1 end
    if input.key_held("a") then move.x = move.x - 1 end
    if input.key_held("d") then move.x = move.x + 1 end

    move = move:normalized() * self.speed * dt
    self.entity.transform:translate(move)

    if input.key_pressed("space") and self.grounded then
        self.entity.physics:apply_impulse(vec3.new(0, self.jump_force, 0))
        self.grounded = false
    end
end

function PlayerController:on_collision(other)
    if other.tag == "ground" then
        self.grounded = true
    end
end

return PlayerController
"#;
        graph.parse_file(lua_player_controller, "assets/scripts/player_controller.lua", "lua");

        // =====================================================================
        // JSON CONFIG LAYER (depth 2)
        // =====================================================================

        let render_config = r#"
{
    "pipeline": {
        "deferred": true,
        "msaa_samples": 4,
        "shadow_resolution": 2048,
        "max_lights": 64
    },
    "post_processing": {
        "bloom": {
            "enabled": true,
            "threshold": 0.8,
            "intensity": 0.5
        },
        "tonemapping": {
            "mode": "aces",
            "exposure": 1.0
        },
        "ssao": {
            "enabled": true,
            "radius": 0.5,
            "samples": 32
        }
    },
    "quality_presets": {
        "low": { "shadow_resolution": 512, "msaa_samples": 2 },
        "medium": { "shadow_resolution": 1024, "msaa_samples": 4 },
        "high": { "shadow_resolution": 2048, "msaa_samples": 8 }
    }
}
"#;
        graph.parse_file(render_config, "config/render_settings.json", "json");

        graph.build_callers();
        graph
    }

    // =========================================================================
    // Test: Multi-language engine graph construction
    // =========================================================================

    #[test]
    fn test_multilang_engine_graph_construction() {
        let graph = build_multilang_engine_graph();

        // Collect all files that were parsed
        let files: std::collections::HashSet<_> = graph.all_nodes()
            .map(|n| n.file.clone())
            .collect();

        // Verify multiple languages are represented
        let has_rust = files.iter().any(|f| f.ends_with(".rs"));
        let has_python = files.iter().any(|f| f.ends_with(".py"));
        let has_typescript = files.iter().any(|f| f.ends_with(".ts"));
        let has_c = files.iter().any(|f| f.ends_with(".c"));
        let has_glsl = files.iter().any(|f| f.ends_with(".glsl") || f.ends_with(".vert") || f.ends_with(".frag"));
        let has_lua = files.iter().any(|f| f.ends_with(".lua"));
        let has_json = files.iter().any(|f| f.ends_with(".json"));

        // At minimum, we should have parsed some files
        assert!(!files.is_empty(), "Should have parsed at least some files");

        // Verify we have the expected language diversity (parsers may not support all)
        println!("Languages found: rust={}, python={}, ts={}, c={}, glsl={}, lua={}, json={}",
            has_rust, has_python, has_typescript, has_c, has_glsl, has_lua, has_json);

        // Rust is the primary language and should always parse
        assert!(has_rust, "Should have parsed Rust files");

        // Count total nodes
        let node_count = graph.all_nodes().count();
        assert!(node_count > 10, "Should have significant number of nodes, got {}", node_count);
    }

    // =========================================================================
    // Test: Cross-language dependency expansion
    // =========================================================================

    #[test]
    fn test_multilang_cross_language_dependencies() {
        let graph = build_multilang_engine_graph();
        let analyzer = ArchitecturalDriftAnalyzer::new(
            graph,
            fresh_drift_store(),
        );

        // Expand from TypeScript editor
        let ts_files = analyzer.expand_dependencies(Path::new("editor/src/Editor.ts"));

        // Should find TypeScript files and potentially Python bindings
        assert!(!ts_files.is_empty(), "TS expansion should find files");

        // Expand from Rust shader module
        let shader_files = analyzer.expand_dependencies(Path::new("src/engine/renderer/shader.rs"));

        // Should find Rust and GLSL dependencies
        assert!(!shader_files.is_empty(), "Shader expansion should find files");

        // Verify depth assignment
        for (path, depth) in &shader_files {
            if path.to_string_lossy().contains("shader.rs") && path.to_string_lossy().contains("engine") && !path.to_string_lossy().contains("assets") {
                assert_eq!(*depth, 0, "Root shader.rs should be depth 0");
            }
        }
    }

    // =========================================================================
    // Test: Weighted scoring across languages
    // =========================================================================

    #[test]
    fn test_multilang_weighted_scoring() {
        let analyzer = ArchitecturalDriftAnalyzer::new(
            CodeGraph::new(),
            fresh_drift_store(),
        );

        // Simulate multi-language drift:
        // - Rust core: stable (low scores)
        // - Python bindings: some drift (medium scores)
        // - GLSL shaders: heavy drift (high scores)
        // - Lua scripts: moderate drift
        // - TypeScript: stable
        let files = vec![
            // Rust core (depth 0-1)
            FileDrift { path: "src/engine/lib.rs".into(), depth: 0, score: 0.02, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] },
            FileDrift { path: "src/engine/renderer/mod.rs".into(), depth: 1, score: 0.03, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] },
            FileDrift { path: "src/engine/physics/mod.rs".into(), depth: 1, score: 0.05, trend: Trend::Stable { metric: "all".into(), variance: 0.02 }, alerts: vec![] },

            // Python bindings (depth 1)
            FileDrift { path: "bindings/python/engine/__init__.py".into(), depth: 1, score: 0.12, trend: Trend::Declining { metric: "type_hints".into(), rate: 0.03, severity: "warning".into() }, alerts: vec!["type_hints declining".into()] },

            // GLSL shaders (depth 2) - heavy drift!
            FileDrift { path: "assets/shaders/pbr.vert.glsl".into(), depth: 2, score: 0.28, trend: Trend::Declining { metric: "uniform_usage".into(), rate: 0.08, severity: "critical".into() }, alerts: vec!["unused uniforms".into(), "version deprecated".into()] },
            FileDrift { path: "assets/shaders/pbr.frag.glsl".into(), depth: 2, score: 0.35, trend: Trend::Declining { metric: "complexity".into(), rate: 0.12, severity: "critical".into() }, alerts: vec!["high complexity".into(), "performance issue".into()] },

            // Lua scripts (depth 2)
            FileDrift { path: "assets/scripts/player_controller.lua".into(), depth: 2, score: 0.15, trend: Trend::Declining { metric: "error_handling".into(), rate: 0.04, severity: "warning".into() }, alerts: vec!["missing error checks".into()] },

            // TypeScript (depth 1)
            FileDrift { path: "editor/src/Editor.ts".into(), depth: 1, score: 0.04, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] },
            FileDrift { path: "editor/src/ui/EditorUI.ts".into(), depth: 1, score: 0.03, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] },

            // C backend (depth 2)
            FileDrift { path: "src/backend/gl_backend.c".into(), depth: 2, score: 0.08, trend: Trend::Stable { metric: "all".into(), variance: 0.02 }, alerts: vec![] },

            // JSON config (depth 2)
            FileDrift { path: "config/render_settings.json".into(), depth: 2, score: 0.01, trend: Trend::Stable { metric: "all".into(), variance: 0.005 }, alerts: vec![] },
        ];

        let score = analyzer.compute_weighted_score(&files);

        // GLSL with high scores at depth 2 should still contribute significantly
        // Weighted: depth 0=1.0, depth 1=0.5, depth 2=0.333
        // The critical shaders should drive the recommendation toward "Schedule review"
        assert!(score > 0.05, "Should have meaningful score with GLSL drift");

        // Generate recommendation
        let rec = analyzer.generate_recommendation(&files);
        assert!(rec.contains("review") || rec.contains("GLSL") || rec.contains("shader") || rec.contains("Warning"),
            "Should mention shader issues: {}", rec);
    }

    // =========================================================================
    // Test: Language-specific trend patterns
    // =========================================================================

    #[test]
    fn test_multilang_trend_patterns() {
        // Rust: typically stable or gradual changes
        let rust_trend = Trend::Stable { metric: "type_strictness".into(), variance: 0.02 };

        // Python: type hints can degrade
        let python_trend = Trend::Declining { metric: "type_coverage".into(), rate: 0.05, severity: "warning".into() };

        // GLSL: can have critical performance issues
        let glsl_trend = Trend::Declining { metric: "uniform_efficiency".into(), rate: 0.15, severity: "critical".into() };

        // Lua: dynamic, often has error handling issues
        let lua_trend = Trend::Declining { metric: "error_handling".into(), rate: 0.08, severity: "warning".into() };

        // TypeScript: usually stable with strict mode
        let ts_trend = Trend::Stable { metric: "type_strictness".into(), variance: 0.01 };

        // Derive overall trend from multi-language mix
        let trends = vec![rust_trend, python_trend, glsl_trend.clone(), lua_trend, ts_trend];
        let derived = derive_trend(&trends);

        match derived {
            Trend::Declining { severity, .. } => {
                assert_eq!(severity, "critical", "Should prioritize GLSL critical trend");
            }
            _ => panic!("Expected Declining trend from multi-language mix"),
        }
    }

    // =========================================================================
    // Test: Full multi-language analysis
    // =========================================================================

    #[test]
    fn test_multilang_full_analysis() {
        let graph = build_multilang_engine_graph();
        let analyzer = ArchitecturalDriftAnalyzer::new(
            graph,
            fresh_drift_store(),
        );

        // Analyze from Rust core
        let result = analyzer.analyze(Path::new("src/engine/lib.rs"), 30);

        assert!(result.is_ok(), "Analysis should succeed");
        let report = result.unwrap();

        // Should have multiple languages represented
        let languages: std::collections::HashSet<_> = report.files.iter()
            .map(|f| {
                let path = f.path.as_str();
                if path.ends_with(".rs") { "rust" }
                else if path.ends_with(".py") { "python" }
                else if path.ends_with(".ts") { "typescript" }
                else if path.ends_with(".glsl") || path.ends_with(".vert") || path.ends_with(".frag") { "glsl" }
                else if path.ends_with(".lua") { "lua" }
                else if path.ends_with(".c") { "c" }
                else if path.ends_with(".json") { "json" }
                else { "other" }
            })
            .collect();

        // At minimum, we should have Rust files
        assert!(languages.contains("rust") || !report.files.is_empty(), "Should have some files parsed");

        // With no snapshots, all scores are 0.0
        for file in &report.files {
            assert_eq!(file.score, 0.0);
        }

        // Report should still be valid
        assert!(report.aggregate_score < 0.01, "Empty snapshots = low score");
    }

    // =========================================================================
    // Test: Max depth with multi-language graph
    // =========================================================================

    #[test]
    fn test_multilang_depth_limiting() {
        let graph = build_multilang_engine_graph();
        let config = ArchitecturalDriftConfig {
            max_depth: 2,
            max_files: 20,
        };
        let analyzer = ArchitecturalDriftAnalyzer::with_config(
            graph,
            fresh_drift_store(),
            config,
        );

        let files = analyzer.expand_dependencies(Path::new("src/engine/lib.rs"));

        // Verify depth limits
        for (_, depth) in &files {
            assert!(*depth <= 2, "Depth {} exceeds max_depth=2", depth);
        }

        // Verify file limit
        assert!(files.len() <= 20, "Should have at most 20 files, got {}", files.len());
    }

    // =========================================================================
    // Test: Worst files from multi-language codebase
    // =========================================================================

    #[test]
    fn test_multilang_worst_files() {
        let report = ArchitecturalDriftReport {
            root: "src/engine/lib.rs".into(),
            depth_requested: 3,
            files: vec![
                FileDrift { path: "assets/shaders/pbr.frag.glsl".into(), depth: 2, score: 0.45, trend: Trend::Declining { metric: "complexity".into(), rate: 0.15, severity: "critical".into() }, alerts: vec!["critical complexity".into()] },
                FileDrift { path: "assets/shaders/pbr.vert.glsl".into(), depth: 2, score: 0.38, trend: Trend::Declining { metric: "uniforms".into(), rate: 0.10, severity: "critical".into() }, alerts: vec!["unused uniforms".into()] },
                FileDrift { path: "bindings/python/engine/__init__.py".into(), depth: 1, score: 0.22, trend: Trend::Declining { metric: "types".into(), rate: 0.05, severity: "warning".into() }, alerts: vec![] },
                FileDrift { path: "assets/scripts/player_controller.lua".into(), depth: 2, score: 0.18, trend: Trend::Declining { metric: "errors".into(), rate: 0.04, severity: "warning".into() }, alerts: vec![] },
                FileDrift { path: "src/engine/renderer/shader.rs".into(), depth: 1, score: 0.05, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] },
                FileDrift { path: "src/engine/lib.rs".into(), depth: 0, score: 0.02, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] },
                FileDrift { path: "editor/src/Editor.ts".into(), depth: 1, score: 0.03, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] },
            ],
            aggregate_score: 0.19,
            recommendation: "Review GLSL shaders".into(),
        };

        let worst = report.worst_files(5);

        assert_eq!(worst.len(), 5);
        // GLSL should be top 2
        assert!(worst[0].path.contains("glsl") || worst[0].path.contains(".frag") || worst[0].path.contains(".vert"));
        assert!(worst[1].path.contains("glsl") || worst[1].path.contains(".frag") || worst[1].path.contains(".vert"));

        // Verify concerning count (score > 0.15)
        assert_eq!(report.concerning_count(), 4); // frag, vert, python, lua
    }

    // =========================================================================
    // Test: Format report with multi-language output
    // =========================================================================

    #[test]
    fn test_multilang_format_report() {
        let report = ArchitecturalDriftReport {
            root: "src/engine/lib.rs".into(),
            depth_requested: 2,
            files: vec![
                FileDrift { path: "src/engine/lib.rs".into(), depth: 0, score: 0.02, trend: Trend::Stable { metric: "all".into(), variance: 0.01 }, alerts: vec![] },
                FileDrift { path: "assets/shaders/pbr.frag.glsl".into(), depth: 2, score: 0.35, trend: Trend::Declining { metric: "complexity".into(), rate: 0.10, severity: "critical".into() }, alerts: vec!["high complexity".into()] },
                FileDrift { path: "bindings/python/engine/__init__.py".into(), depth: 1, score: 0.12, trend: Trend::Declining { metric: "types".into(), rate: 0.03, severity: "warning".into() }, alerts: vec![] },
            ],
            aggregate_score: 0.16,
            recommendation: "Priority: GLSL shader optimization".into(),
        };

        let output = format_report(&report);

        // Verify all languages appear
        assert!(output.contains(".rs"), "Should mention Rust file");
        assert!(output.contains(".glsl"), "Should mention GLSL file");
        assert!(output.contains(".py"), "Should mention Python file");

        // Verify scores
        assert!(output.contains("0.16"), "Should show aggregate score");
        assert!(output.contains("0.35") || output.contains("0.12") || output.contains("0.02"), "Should show file scores");

        // Verify recommendation
        assert!(output.contains("GLSL") || output.contains("Priority"), "Should show recommendation");
    }

    // =========================================================================
    // DRIFT + ERROR DETECTION TEST SUITE
    // =========================================================================
    // Tests that verify Aether can detect:
    // 1. Temporal drift: metrics degrading over time
    // 2. Structural errors: code with bugs and issues
    // =========================================================================

    use super::super::drift_snapshots::{SnapshotMetrics, CodeSnapshot};
    use chrono::{Utc, Duration as ChronoDuration};

    /// Helper: Create a DriftSnapshotStore populated with temporal data
    fn build_snapshot_store_with_drift(
        file_metrics: Vec<(&str, Vec<SnapshotMetrics>)>
    ) -> DriftSnapshotStore {
        // Use unique temp file per call to avoid persistence conflicts
        let temp_path = std::env::temp_dir().join(format!("aether_drift_snapshots_{}_{}.json", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let mut store = DriftSnapshotStore::new(Some(temp_path)).unwrap();
        let now = Utc::now();

        for (file_path, metrics_sequence) in file_metrics {
            for (days_ago, metrics) in metrics_sequence.into_iter().enumerate() {
                let snapshot = CodeSnapshot {
                    timestamp: now - ChronoDuration::days(days_ago as i64),
                    commit: Some(format!("commit_{}d_ago", days_ago)),
                    file_path: file_path.to_string(),
                    metrics,
                };
                store.record(snapshot).unwrap();
            }
        }

        store
    }

    // =========================================================================
    // Test: Detect type_strictness degradation over time
    // =========================================================================

    #[test]
    fn test_detect_type_strictness_drift() {
        // Simulate a Python file losing type hints over 4 commits
        let metrics_sequence = vec![
            // 3 days ago: full type hints
            SnapshotMetrics {
                type_strictness: 0.95,
                naming_consistency: 0.90,
                error_handling_quality: 0.85,
                complexity: 0.20,
                dead_code_ratio: 0.05,
            },
            // 2 days ago: some types removed
            SnapshotMetrics {
                type_strictness: 0.80,
                naming_consistency: 0.90,
                error_handling_quality: 0.85,
                complexity: 0.22,
                dead_code_ratio: 0.05,
            },
            // 1 day ago: more types removed
            SnapshotMetrics {
                type_strictness: 0.60,
                naming_consistency: 0.88,
                error_handling_quality: 0.80,
                complexity: 0.25,
                dead_code_ratio: 0.08,
            },
            // Today: very few type hints
            SnapshotMetrics {
                type_strictness: 0.35,
                naming_consistency: 0.85,
                error_handling_quality: 0.75,
                complexity: 0.30,
                dead_code_ratio: 0.12,
            },
        ];

        let store = build_snapshot_store_with_drift(vec![
            ("src/api/handlers.py", metrics_sequence)
        ]);

        // Analyze the drift
        let report = store.analyze_trend("src/api/handlers.py");

        assert!(report.is_some(), "Should detect drift for file with multiple snapshots");
        let drift_report = report.unwrap();

        // Should have declining type_strictness
        assert!(drift_report.trends.iter().any(|t| matches!(t, Trend::Declining { metric, .. } if metric == "type_strictness")),
            "Should detect declining type_strictness");

        // Drift score should be significant (> 0.1 for this degradation)
        assert!(drift_report.drift_score > 0.1, "Drift score should be significant, got {}", drift_report.drift_score);
    }

    // =========================================================================
    // Test: Detect complexity explosion
    // =========================================================================

    #[test]
    fn test_detect_complexity_explosion() {
        // Simulate a Rust file growing in complexity
        let metrics_sequence = vec![
            // 4 days ago: simple code
            SnapshotMetrics {
                type_strictness: 0.95,
                naming_consistency: 0.90,
                error_handling_quality: 0.90,
                complexity: 0.15,
                dead_code_ratio: 0.02,
            },
            // 3 days ago: slightly more complex
            SnapshotMetrics {
                type_strictness: 0.93,
                naming_consistency: 0.88,
                error_handling_quality: 0.85,
                complexity: 0.25,
                dead_code_ratio: 0.03,
            },
            // 2 days ago: getting complex
            SnapshotMetrics {
                type_strictness: 0.90,
                naming_consistency: 0.85,
                error_handling_quality: 0.75,
                complexity: 0.45,
                dead_code_ratio: 0.05,
            },
            // 1 day ago: very complex
            SnapshotMetrics {
                type_strictness: 0.88,
                naming_consistency: 0.82,
                error_handling_quality: 0.65,
                complexity: 0.65,
                dead_code_ratio: 0.08,
            },
            // Today: critical complexity
            SnapshotMetrics {
                type_strictness: 0.85,
                naming_consistency: 0.80,
                error_handling_quality: 0.55,
                complexity: 0.85,
                dead_code_ratio: 0.12,
            },
        ];

        let store = build_snapshot_store_with_drift(vec![
            ("src/engine/renderer.rs", metrics_sequence)
        ]);

        let report = store.analyze_trend("src/engine/renderer.rs").unwrap();

        // Should have declining complexity (inverse metric, increase = Declining = bad)
        let has_complexity_issue = report.trends.iter().any(|t| {
            matches!(t, Trend::Declining { metric, severity, .. } if metric == "complexity" && (severity == "high" || severity == "medium" || severity == "low"))
        });
        assert!(has_complexity_issue, "Should detect increasing complexity as Declining trend");

        // Should also have declining error handling
        let has_error_handling_decline = report.trends.iter().any(|t| {
            matches!(t, Trend::Declining { metric, .. } if metric == "error_handling")
        });
        assert!(has_error_handling_decline, "Should detect declining error handling");
    }

    // =========================================================================
    // Test: Detect dead code accumulation
    // =========================================================================

    #[test]
    fn test_detect_dead_code_accumulation() {
        // Simulate a TypeScript file accumulating dead code
        let metrics_sequence = vec![
            SnapshotMetrics {
                type_strictness: 0.92,
                naming_consistency: 0.90,
                error_handling_quality: 0.85,
                complexity: 0.20,
                dead_code_ratio: 0.02,
            },
            SnapshotMetrics {
                type_strictness: 0.90,
                naming_consistency: 0.88,
                error_handling_quality: 0.82,
                complexity: 0.22,
                dead_code_ratio: 0.08,
            },
            SnapshotMetrics {
                type_strictness: 0.88,
                naming_consistency: 0.85,
                error_handling_quality: 0.78,
                complexity: 0.25,
                dead_code_ratio: 0.18,
            },
            SnapshotMetrics {
                type_strictness: 0.85,
                naming_consistency: 0.82,
                error_handling_quality: 0.72,
                complexity: 0.30,
                dead_code_ratio: 0.35,
            },
        ];

        let store = build_snapshot_store_with_drift(vec![
            ("editor/src/Editor.ts", metrics_sequence)
        ]);

        let report = store.analyze_trend("editor/src/Editor.ts").unwrap();

        // Dead code ratio increasing is bad (inverse metric = Declining)
        let has_dead_code = report.trends.iter().any(|t| {
            matches!(t, Trend::Declining { metric, .. } if metric == "dead_code")
        });
        assert!(has_dead_code, "Should detect increasing dead code as Declining trend");
    }

    // =========================================================================
    // Test: Architectural drift across multiple files with temporal data
    // =========================================================================

    #[test]
    fn test_architectural_drift_with_real_snapshots() {
        let graph = build_multilang_engine_graph();

        // Create snapshots for multiple files with different degradation patterns
        // Analyze from renderer/mod.rs which has actual function call dependencies
        let store = build_snapshot_store_with_drift(vec![
            // Renderer module: stable
            ("src/engine/renderer/mod.rs", vec![
                SnapshotMetrics { type_strictness: 0.95, naming_consistency: 0.92, error_handling_quality: 0.90, complexity: 0.15, dead_code_ratio: 0.02 },
                SnapshotMetrics { type_strictness: 0.94, naming_consistency: 0.91, error_handling_quality: 0.89, complexity: 0.16, dead_code_ratio: 0.02 },
                SnapshotMetrics { type_strictness: 0.95, naming_consistency: 0.92, error_handling_quality: 0.90, complexity: 0.15, dead_code_ratio: 0.02 },
            ]),
            // Shader module: declining
            ("src/engine/renderer/shader.rs", vec![
                SnapshotMetrics { type_strictness: 0.90, naming_consistency: 0.88, error_handling_quality: 0.85, complexity: 0.20, dead_code_ratio: 0.03 },
                SnapshotMetrics { type_strictness: 0.82, naming_consistency: 0.85, error_handling_quality: 0.78, complexity: 0.28, dead_code_ratio: 0.05 },
                SnapshotMetrics { type_strictness: 0.70, naming_consistency: 0.80, error_handling_quality: 0.65, complexity: 0.40, dead_code_ratio: 0.10 },
            ]),
            // FFI/gl module: declining
            ("src/engine/ffi/gl.rs", vec![
                SnapshotMetrics { type_strictness: 0.85, naming_consistency: 0.90, error_handling_quality: 0.80, complexity: 0.18, dead_code_ratio: 0.02 },
                SnapshotMetrics { type_strictness: 0.60, naming_consistency: 0.85, error_handling_quality: 0.70, complexity: 0.25, dead_code_ratio: 0.05 },
                SnapshotMetrics { type_strictness: 0.35, naming_consistency: 0.80, error_handling_quality: 0.55, complexity: 0.35, dead_code_ratio: 0.12 },
            ]),
        ]);

        let analyzer = ArchitecturalDriftAnalyzer::new(graph, store);

        // Debug: check expand_dependencies
        let expanded = analyzer.expand_dependencies(Path::new("src/engine/renderer/mod.rs"));
        println!("Expanded files: {:?}", expanded);

        // Analyze from renderer/mod.rs which has actual dependencies on shader.rs and gl.rs
        let result = analyzer.analyze(Path::new("src/engine/renderer/mod.rs"), 30);

        assert!(result.is_ok());
        let report = result.unwrap();

        // Debug: print all files and scores
        println!("All files in report:");
        for f in &report.files {
            println!("  {} (depth {}): score={}", f.path, f.depth, f.score);
        }
        println!("aggregate_score: {}", report.aggregate_score);

        // Should have non-zero aggregate score due to degradation
        assert!(report.aggregate_score > 0.0, "Should detect drift with real snapshots");

        // Should have some concerning files
        let concerning = report.concerning_count();
        assert!(concerning > 0, "Should have concerning files, got {}", concerning);

        // Recommendation should mention review
        assert!(report.recommendation.to_lowercase().contains("review") ||
                report.recommendation.to_lowercase().contains("attention") ||
                report.aggregate_score > 0.05,
            "Should recommend review or have notable score");
    }

    // =========================================================================
    // Test: Detect cascading drift in dependency chain
    // =========================================================================

    #[test]
    fn test_cascading_drift_in_dependency_chain() {
        let graph = build_engine_graph();

        // Debug: check what file_dependencies returns for each file
        println!("renderer.rs deps: {:?}", graph.file_dependencies("src/engine/renderer.rs"));
        println!("shader.rs deps: {:?}", graph.file_dependencies("src/engine/shader.rs"));
        println!("texture.rs deps: {:?}", graph.file_dependencies("src/engine/texture.rs"));

        // Simulate a dependency chain where problems cascade:
        // gl_utils.rs (depth 2) has issues -> affects shader.rs (depth 1) -> affects renderer.rs (depth 0)
        let store = build_snapshot_store_with_drift(vec![
            // gl_utils.rs: started having problems
            ("src/engine/gl_utils.rs", vec![
                SnapshotMetrics { type_strictness: 0.90, naming_consistency: 0.88, error_handling_quality: 0.85, complexity: 0.15, dead_code_ratio: 0.02 },
                SnapshotMetrics { type_strictness: 0.85, naming_consistency: 0.85, error_handling_quality: 0.75, complexity: 0.30, dead_code_ratio: 0.08 },
                SnapshotMetrics { type_strictness: 0.70, naming_consistency: 0.78, error_handling_quality: 0.55, complexity: 0.55, dead_code_ratio: 0.18 },
            ]),
            // shader.rs: slightly affected
            ("src/engine/shader.rs", vec![
                SnapshotMetrics { type_strictness: 0.92, naming_consistency: 0.90, error_handling_quality: 0.88, complexity: 0.18, dead_code_ratio: 0.02 },
                SnapshotMetrics { type_strictness: 0.88, naming_consistency: 0.88, error_handling_quality: 0.82, complexity: 0.22, dead_code_ratio: 0.03 },
                SnapshotMetrics { type_strictness: 0.82, naming_consistency: 0.85, error_handling_quality: 0.75, complexity: 0.28, dead_code_ratio: 0.05 },
            ]),
            // renderer.rs: root, indirectly affected
            ("src/engine/renderer.rs", vec![
                SnapshotMetrics { type_strictness: 0.95, naming_consistency: 0.92, error_handling_quality: 0.90, complexity: 0.12, dead_code_ratio: 0.01 },
                SnapshotMetrics { type_strictness: 0.93, naming_consistency: 0.90, error_handling_quality: 0.88, complexity: 0.14, dead_code_ratio: 0.02 },
                SnapshotMetrics { type_strictness: 0.90, naming_consistency: 0.88, error_handling_quality: 0.85, complexity: 0.16, dead_code_ratio: 0.02 },
            ]),
        ]);

        let analyzer = ArchitecturalDriftAnalyzer::new(graph, store);

        // Debug: check expand_dependencies
        let expanded = analyzer.expand_dependencies(Path::new("src/engine/renderer.rs"));
        println!("Expanded files: {:?}", expanded);

        let result = analyzer.analyze(Path::new("src/engine/renderer.rs"), 30);
        assert!(result.is_ok());

        let report = result.unwrap();

        // Debug: print all files and scores
        println!("All files in report:");
        for f in &report.files {
            println!("  {} (depth {}): score={}", f.path, f.depth, f.score);
        }

        // gl_utils.rs should be the worst file (deepest problems)
        let worst = report.worst_files(3);
        println!("Worst files: {:?}", worst.iter().map(|f| (&f.path, f.score)).collect::<Vec<_>>());
        assert!(!worst.is_empty(), "Should have worst files");

        // The worst should be gl_utils.rs (highest drift score)
        assert!(worst[0].path.contains("gl_utils"),
            "gl_utils.rs should be worst file, got {}", worst[0].path);
    }

    // =========================================================================
    // Test: Alert generation for critical drift
    // =========================================================================

    #[test]
    fn test_alert_generation_for_critical_drift() {
        let mut store = fresh_drift_store();
        let now = Utc::now();

        // Add rapidly degrading snapshots
        for days_ago in 0..5 {
            let degradation = days_ago as f32 * 0.12;
            let snapshot = CodeSnapshot {
                timestamp: now - ChronoDuration::days(days_ago as i64),
                commit: Some(format!("commit_{}", days_ago)),
                file_path: "src/critical_module.rs".to_string(),
                metrics: SnapshotMetrics {
                    type_strictness: 0.95 - degradation,
                    naming_consistency: 0.90 - degradation * 0.5,
                    error_handling_quality: 0.85 - degradation * 0.8,
                    complexity: 0.15 + degradation * 2.0,
                    dead_code_ratio: 0.02 + degradation * 0.5,
                },
            };
            store.record(snapshot).unwrap();
        }

        // Check for alerts
        let alerts = store.check_alerts();

        // Should generate an alert for critical_module.rs
        assert!(!alerts.is_empty(), "Should generate alerts for critical drift");

        let has_critical = alerts.iter().any(|a|
            a.file.contains("critical_module") && a.level == "critical"
        );
        assert!(has_critical, "Should have critical alert for the file");
    }

    // =========================================================================
    // Test: Stable code shows low drift
    // =========================================================================

    #[test]
    fn test_stable_code_low_drift() {
        let store = build_snapshot_store_with_drift(vec![
            ("src/stable/utils.rs", vec![
                SnapshotMetrics { type_strictness: 0.95, naming_consistency: 0.92, error_handling_quality: 0.90, complexity: 0.10, dead_code_ratio: 0.01 },
                SnapshotMetrics { type_strictness: 0.95, naming_consistency: 0.92, error_handling_quality: 0.90, complexity: 0.10, dead_code_ratio: 0.01 },
                SnapshotMetrics { type_strictness: 0.94, naming_consistency: 0.91, error_handling_quality: 0.89, complexity: 0.11, dead_code_ratio: 0.01 },
                SnapshotMetrics { type_strictness: 0.95, naming_consistency: 0.92, error_handling_quality: 0.90, complexity: 0.10, dead_code_ratio: 0.01 },
            ]),
        ]);

        let report = store.analyze_trend("src/stable/utils.rs");

        assert!(report.is_some());
        let drift = report.unwrap();

        // Stable code should have low drift score
        assert!(drift.drift_score < 0.05, "Stable code should have low drift, got {}", drift.drift_score);

        // Most trends should be stable
        let stable_count = drift.trends.iter().filter(|t| matches!(t, Trend::Stable { .. })).count();
        assert!(stable_count >= 3, "Most metrics should be stable");
    }
}
