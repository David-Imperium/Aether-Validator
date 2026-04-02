//! Drift analysis command
//!
//! Analyze architectural drift over time across a codebase.

use anyhow::Result;
use std::path::PathBuf;
use synward_intelligence::memory::{
    ArchitecturalDriftAnalyzer, ArchitecturalDriftConfig,
    CodeGraph, DriftSnapshotStore,
};

/// Run architectural drift analysis
pub fn analyze(root: &PathBuf, depth: usize, max_files: usize, days: u32) -> Result<()> {
    let config = ArchitecturalDriftConfig {
        max_depth: depth,
        max_files,
    };

    // Initialize dependencies
    let code_graph = CodeGraph::new();
    let drift_store = DriftSnapshotStore::new(None)?;

    let analyzer = ArchitecturalDriftAnalyzer::with_config(code_graph, drift_store, config);
    let report = analyzer.analyze(root, days)?;

    println!("Architectural Drift Analysis");
    println!("============================");
    println!("Root: {}", report.root);
    println!("Depth: {}", report.depth_requested);
    println!("Files analyzed: {}", report.files.len());
    println!();

    // Sort by score descending
    let mut files = report.files.clone();
    files.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // Show worst files
    let worst: Vec<_> = files.iter().take(10).collect();
    if !worst.is_empty() {
        println!("Top concerning files:");
        for f in worst {
            println!("  [{:.2}] {} (depth: {}, trend: {:?})",
                f.score, f.path, f.depth, f.trend);
            if !f.alerts.is_empty() {
                for alert in &f.alerts {
                    println!("    ! {}", alert);
                }
            }
        }
        println!();
    }

    // Aggregate score
    println!("Aggregate Score: {:.2}", report.aggregate_score);
    println!("Recommendation: {}", report.recommendation);

    // Exit with error if critical
    if report.aggregate_score > 0.7 {
        std::process::exit(1);
    }

    Ok(())
}

/// Show drift trend for a specific file
pub fn trend(file: &PathBuf, days: u32) -> Result<()> {
    // Initialize dependencies
    let code_graph = CodeGraph::new();
    let drift_store = DriftSnapshotStore::new(None)?;

    let analyzer = ArchitecturalDriftAnalyzer::new(code_graph, drift_store);

    // Analyze just this file with depth 0
    let report = analyzer.analyze(file, days)?;

    println!("Drift Trend for: {}", file.display());
    println!("================================");

    if let Some(f) = report.files.first() {
        println!("Score: {:.2}", f.score);
        println!("Trend: {:?}", f.trend);
        if !f.alerts.is_empty() {
            println!("\nAlerts:");
            for alert in &f.alerts {
                println!("  - {}", alert);
            }
        }
    } else {
        println!("No drift data available for this file.");
    }

    Ok(())
}
