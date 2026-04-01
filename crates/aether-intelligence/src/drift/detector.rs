//! Drift Detector - Main entry point for drift detection

use crate::drift::{CodeSnapshot, DriftReport, TrendAnalyzer, GitIntegration, SnapshotMetrics};
use crate::error::{Error, Result};
use crate::patterns::FeatureExtractor;
use std::path::PathBuf;
use std::collections::HashMap;

/// Detect code quality drift over time
pub struct DriftDetector {
    /// Git repository path
    repo_path: Option<PathBuf>,

    /// Git integration
    git: Option<GitIntegration>,

    /// Historical snapshots
    snapshots: HashMap<String, Vec<CodeSnapshot>>,

    /// Trend analyzer
    analyzer: TrendAnalyzer,
}

impl DriftDetector {
    /// Create a new detector
    pub fn new(repo_path: Option<PathBuf>) -> Result<Self> {
        let git = repo_path.as_ref()
            .and_then(|p| GitIntegration::new(p.clone()).ok());

        Ok(Self {
            repo_path,
            git,
            snapshots: HashMap::new(),
            analyzer: TrendAnalyzer::new(),
        })
    }

    /// Check if git is available
    pub fn has_git(&self) -> bool {
        self.git.as_ref().map(|g| g.is_valid()).unwrap_or(false)
    }

    /// Analyze drift for a file
    pub async fn analyze_file(&self, file_path: &str) -> Result<DriftReport> {
        let snapshots = self.snapshots.get(file_path).cloned().unwrap_or_default();

        if snapshots.is_empty() {
            return Ok(DriftReport {
                file_path: file_path.to_string(),
                drift_score: 0.0,
                trends: vec![],
                snapshots_analyzed: 0,
            });
        }

        let trends = self.analyzer.analyze(&snapshots);
        let drift_score = self.calculate_drift_score(&trends);

        Ok(DriftReport {
            file_path: file_path.to_string(),
            drift_score,
            trends,
            snapshots_analyzed: snapshots.len(),
        })
    }

    /// Analyze drift for entire project
    pub async fn analyze_project(&self) -> Result<Vec<DriftReport>> {
        let mut reports = Vec::new();

        for file_path in self.snapshots.keys() {
            let report = self.analyze_file(file_path).await?;
            reports.push(report);
        }

        // Sort by drift score (highest first)
        reports.sort_by(|a, b| b.drift_score.partial_cmp(&a.drift_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(reports)
    }

    /// Add a snapshot
    pub fn add_snapshot(&mut self, snapshot: CodeSnapshot) {
        self.snapshots
            .entry(snapshot.file_path.clone())
            .or_default()
            .push(snapshot);
    }

    /// Load snapshots from git history
    pub async fn load_from_git(&mut self, last_n_commits: usize) -> Result<()> {
        let git = self.git.as_ref().ok_or_else(|| {
            Error::Git("Git integration not available".to_string())
        })?;

        if !git.is_valid() {
            return Err(Error::Git("Not a valid git repository".to_string()));
        }

        let commits = git.get_commits(last_n_commits)?;
        tracing::info!("Loaded {} commits from git history", commits.len());

        // Collect all files that have changed
        let mut files_to_track: std::collections::HashSet<String> = std::collections::HashSet::new();
        for commit in &commits {
            for file in &commit.files_changed {
                // Only track source files
                if is_source_file(file) {
                    files_to_track.insert(file.clone());
                }
            }
        }

        // For each tracked file, get historical versions
        for file_path in files_to_track {
            let mut file_snapshots = Vec::new();

            for commit in &commits {
                if commit.files_changed.contains(&file_path) {
                    if let Ok(content) = git.get_file_at_commit(&commit.hash, &file_path) {
                        // Extract features
                        let extractor = FeatureExtractor::new();
                        let lang = detect_language(&file_path);
                        let features = extractor.extract(&content, &lang);
                        let metrics = SnapshotMetrics::from_features(&features);

                        let snapshot = CodeSnapshot {
                            timestamp: commit.timestamp,
                            commit: Some(commit.hash.clone()),
                            file_path: file_path.clone(),
                            metrics,
                        };

                        file_snapshots.push(snapshot);
                    }
                }
            }

            if !file_snapshots.is_empty() {
                self.snapshots.insert(file_path, file_snapshots);
            }
        }

        Ok(())
    }

    /// Get tracked files
    pub fn tracked_files(&self) -> Vec<&str> {
        self.snapshots.keys().map(|s| s.as_str()).collect()
    }

    fn calculate_drift_score(&self, trends: &[crate::drift::Trend]) -> f32 {
        if trends.is_empty() {
            return 0.0;
        }

        let total: f32 = trends
            .iter()
            .map(|t| match t {
                crate::drift::Trend::Declining { severity, .. } => {
                    match severity.as_str() {
                        "high" => 0.3,
                        "medium" => 0.15,
                        _ => 0.05,
                    }
                }
                crate::drift::Trend::Increasing { metric, severity, .. } => {
                    if metric == "complexity" {
                        match severity.as_str() {
                            "high" => 0.25,
                            "medium" => 0.1,
                            _ => 0.03,
                        }
                    } else {
                        0.0
                    }
                }
                crate::drift::Trend::Stable { .. } => 0.0,
            })
            .sum();

        total.min(1.0)
    }
}

/// Check if file is a source file
fn is_source_file(path: &str) -> bool {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    matches!(ext, "rs" | "py" | "js" | "ts" | "go" | "java" | "cpp" | "c" | "lua")
}

/// Detect language from file extension
fn detect_language(path: &str) -> &'static str {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    match ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "go" => "go",
        "java" => "java",
        "cpp" | "cc" | "cxx" => "cpp",
        "c" => "c",
        "lua" => "lua",
        _ => "generic",
    }
}
