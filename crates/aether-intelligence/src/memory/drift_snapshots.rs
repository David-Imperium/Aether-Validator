//! Layer 2D: Drift Snapshots (Time-series Storage)
//!
//! Stores code metrics over time for trend analysis and drift detection.
//! Provides `analyze_trend()` functionality as part of the unified memory API.
//!
//! This module is self-contained and does not depend on the drift feature.

use crate::error::{Error, Result};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

use super::scope::MemoryPath;

// ============================================================================
// Types (self-contained, not importing from drift module)
// ============================================================================

/// Metrics at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetrics {
    /// Type strictness score (0-1)
    pub type_strictness: f32,

    /// Naming consistency score (0-1)
    pub naming_consistency: f32,

    /// Error handling quality (0-1)
    pub error_handling_quality: f32,

    /// Code complexity score (0-1, lower = better)
    pub complexity: f32,

    /// Dead code ratio (0-1)
    pub dead_code_ratio: f32,
}

impl Default for SnapshotMetrics {
    fn default() -> Self {
        Self {
            type_strictness: 1.0,
            naming_consistency: 1.0,
            error_handling_quality: 1.0,
            complexity: 0.0,
            dead_code_ratio: 0.0,
        }
    }
}

/// A snapshot of code at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnapshot {
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,

    /// Commit hash (if from git)
    pub commit: Option<String>,

    /// File path
    pub file_path: String,

    /// Metrics at this point
    pub metrics: SnapshotMetrics,
}

/// A detected trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trend {
    /// Metric is declining
    Declining {
        metric: String,
        rate: f32,
        severity: String,
    },

    /// Metric is increasing
    Increasing {
        metric: String,
        rate: f32,
        severity: String,
    },

    /// Metric is stable
    Stable {
        metric: String,
        variance: f32,
    },
}

/// Report on drift for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    /// File path
    pub file_path: String,

    /// Overall drift score (0-1, higher = more drift)
    pub drift_score: f32,

    /// Detected trends
    pub trends: Vec<Trend>,

    /// Number of snapshots analyzed
    pub snapshots_analyzed: usize,
}

/// Trend analyzer (simple implementation)
#[derive(Debug, Clone)]
pub struct TrendAnalyzer {
    /// Threshold for significant change
    change_threshold: f32,
}

impl TrendAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            change_threshold: 0.02,
        }
    }

    /// Analyze trends from snapshots
    pub fn analyze(&self, snapshots: &[CodeSnapshot]) -> Vec<Trend> {
        if snapshots.len() < 2 {
            return vec![];
        }

        let mut trends = Vec::new();

        // Get first and last metrics (safe after len check)
        let first = match snapshots.first() {
            Some(s) => &s.metrics,
            None => return vec![],
        };
        let last = match snapshots.last() {
            Some(s) => &s.metrics,
            None => return vec![],
        };

        // Analyze each metric
        trends.extend(self.analyze_metric("type_strictness", first.type_strictness, last.type_strictness, false));
        trends.extend(self.analyze_metric("naming_consistency", first.naming_consistency, last.naming_consistency, false));
        trends.extend(self.analyze_metric("error_handling", first.error_handling_quality, last.error_handling_quality, false));
        trends.extend(self.analyze_metric("complexity", first.complexity, last.complexity, true));
        trends.extend(self.analyze_metric("dead_code", first.dead_code_ratio, last.dead_code_ratio, true));

        trends
    }

    fn analyze_metric(&self, name: &str, old: f32, new: f32, inverse: bool) -> Vec<Trend> {
        let diff = new - old;
        let abs_diff = diff.abs();

        if abs_diff < self.change_threshold {
            return vec![Trend::Stable {
                metric: name.to_string(),
                variance: abs_diff,
            }];
        }

        // For inverse metrics (complexity, dead_code), increase is bad
        let is_decline = if inverse { diff > 0.0 } else { diff < 0.0 };

        let severity = if abs_diff > 0.1 {
            "high"
        } else if abs_diff > 0.05 {
            "medium"
        } else {
            "low"
        };

        if is_decline {
            vec![Trend::Declining {
                metric: name.to_string(),
                rate: abs_diff,
                severity: severity.to_string(),
            }]
        } else {
            vec![Trend::Increasing {
                metric: name.to_string(),
                rate: abs_diff,
                severity: severity.to_string(),
            }]
        }
    }
}

impl Default for TrendAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Drift Snapshot Store
// ============================================================================

/// Time-series storage for drift snapshots
#[derive(Debug, Clone)]
pub struct DriftSnapshotStore {
    /// Snapshots indexed by file path
    snapshots: HashMap<String, Vec<CodeSnapshot>>,

    /// Persistent storage path
    path: PathBuf,

    /// Trend analyzer
    analyzer: TrendAnalyzer,

    /// Maximum snapshots per file
    max_per_file: usize,

    /// Alert thresholds
    alert_thresholds: AlertThresholds,
}

/// Alert thresholds for drift detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Drift score threshold for warning (0-1)
    pub warning: f32,

    /// Drift score threshold for critical alert (0-1)
    pub critical: f32,

    /// Minimum snapshots before alerting
    pub min_snapshots: usize,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            warning: 0.15,
            critical: 0.30,
            min_snapshots: 3,
        }
    }
}

/// Alert type for drift
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftAlert {
    /// File path
    pub file: String,

    /// Alert level (warning/critical)
    pub level: String,

    /// Drift score
    pub score: f32,

    /// Detected trends
    pub trends: Vec<Trend>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Recommendation
    pub recommendation: String,
}

impl DriftSnapshotStore {
    /// Create a new drift snapshot store
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let path = path.unwrap_or_else(|| {
            MemoryPath::global_base()
                .join("drift_snapshots.json")
        });

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(Error::Io)?;
        }

        let mut store = Self {
            snapshots: HashMap::new(),
            path,
            analyzer: TrendAnalyzer::new(),
            max_per_file: 100,
            alert_thresholds: AlertThresholds::default(),
        };

        store.load()?;
        Ok(store)
    }

    /// Set alert thresholds
    pub fn with_thresholds(mut self, thresholds: AlertThresholds) -> Self {
        self.alert_thresholds = thresholds;
        self
    }

    /// Record a snapshot
    pub fn record(&mut self, snapshot: CodeSnapshot) -> Result<()> {
        let entry = self.snapshots.entry(snapshot.file_path.clone()).or_default();
        entry.push(snapshot);

        // Keep only recent snapshots
        if entry.len() > self.max_per_file {
            entry.remove(0);
        }

        self.persist()
    }

    /// Record from current metrics (creates timestamped snapshot)
    pub fn record_metrics(&mut self, file: String, metrics: SnapshotMetrics, commit: Option<String>) -> Result<()> {
        let snapshot = CodeSnapshot {
            timestamp: Utc::now(),
            commit,
            file_path: file,
            metrics,
        };
        self.record(snapshot)
    }

    /// Analyze trend for a file
    pub fn analyze_trend(&self, file: &str) -> Option<DriftReport> {
        let snapshots = self.snapshots.get(file)?;

        if snapshots.len() < 2 {
            return None;
        }

        let trends = self.analyzer.analyze(snapshots);

        // Calculate overall drift score
        let drift_score = self.calculate_drift_score(&trends);

        Some(DriftReport {
            file_path: file.to_string(),
            drift_score,
            trends,
            snapshots_analyzed: snapshots.len(),
        })
    }

    /// Analyze trend over last N days
    pub fn analyze_trend_days(&self, file: &str, days: usize) -> Option<DriftReport> {
        let snapshots = self.snapshots.get(file)?;
        let cutoff = Utc::now() - Duration::days(days as i64);

        let recent: Vec<_> = snapshots
            .iter()
            .filter(|s| s.timestamp > cutoff)
            .cloned()
            .collect();

        if recent.len() < 2 {
            return None;
        }

        let trends = self.analyzer.analyze(&recent);
        let drift_score = self.calculate_drift_score(&trends);

        Some(DriftReport {
            file_path: file.to_string(),
            drift_score,
            trends,
            snapshots_analyzed: recent.len(),
        })
    }

    /// Check for alerts across all files
    pub fn check_alerts(&self) -> Vec<DriftAlert> {
        let mut alerts = Vec::new();

        for (file, snapshots) in &self.snapshots {
            if snapshots.len() < self.alert_thresholds.min_snapshots {
                continue;
            }

            if let Some(report) = self.analyze_trend(file) {
                if report.drift_score >= self.alert_thresholds.critical {
                    let recommendation = self.generate_recommendation(&report, "critical");
                    alerts.push(DriftAlert {
                        file: file.to_string(),
                        level: "critical".to_string(),
                        score: report.drift_score,
                        trends: report.trends,
                        timestamp: Utc::now(),
                        recommendation,
                    });
                } else if report.drift_score >= self.alert_thresholds.warning {
                    let recommendation = self.generate_recommendation(&report, "warning");
                    alerts.push(DriftAlert {
                        file: file.to_string(),
                        level: "warning".to_string(),
                        score: report.drift_score,
                        trends: report.trends,
                        timestamp: Utc::now(),
                        recommendation,
                    });
                }
            }
        }

        alerts
    }

    /// Get snapshots for a file
    pub fn get_snapshots(&self, file: &str) -> Option<&[CodeSnapshot]> {
        self.snapshots.get(file).map(|v: &Vec<CodeSnapshot>| v.as_slice())
    }

    /// Get all tracked files
    pub fn tracked_files(&self) -> Vec<&str> {
        self.snapshots.keys().map(|s: &String| s.as_str()).collect()
    }

    /// Count total snapshots
    pub fn count(&self) -> usize {
        self.snapshots.values().map(|v: &Vec<CodeSnapshot>| v.len()).sum()
    }

    /// Clear snapshots for a file
    pub fn clear_file(&mut self, file: &str) -> Result<()> {
        self.snapshots.remove(file);
        self.persist()
    }

    /// Clear all snapshots
    pub fn clear(&mut self) -> Result<()> {
        self.snapshots.clear();
        self.persist()
    }

    /// Calculate drift score from trends
    fn calculate_drift_score(&self, trends: &[Trend]) -> f32 {
        if trends.is_empty() {
            return 0.0;
        }

        let mut score: f32 = 0.0;

        for trend in trends {
            match trend {
                Trend::Declining { metric, rate, severity } => {
                    // Declining quality metrics = drift
                    // Also handle complexity/dead_code (inverse metrics where Declining = increasing value = bad)
                    let weight: f32 = match severity.as_str() {
                        "high" => 0.3,
                        "medium" => 0.15,
                        _ => 0.05,
                    };
                    // Type strictness and error handling declines are worse
                    if metric == "type_strictness" || metric == "error_handling" {
                        score += rate * weight * 2.0;
                    } else if metric == "complexity" || metric == "dead_code" {
                        // Inverse metrics: Declining trend means value increased (bad)
                        score += rate * weight * 1.5;
                    } else {
                        score += rate * weight;
                    }
                }
                Trend::Increasing { metric, rate, severity } => {
                    // Increasing complexity or dead code = drift (only for non-inverse semantics)
                    // Note: With inverse metrics, these won't be Increasing, but handle for safety
                    if metric == "complexity" || metric == "dead_code" {
                        let weight: f32 = match severity.as_str() {
                            "high" => 0.25,
                            "medium" => 0.12,
                            _ => 0.04,
                        };
                        score += rate * weight;
                    }
                }
                Trend::Stable { .. } => {
                    // Stable is good, no contribution to drift
                }
            }
        }

        score.min(1.0)
    }

    /// Generate recommendation for drift
    fn generate_recommendation(&self, report: &DriftReport, level: &str) -> String {
        let mut rec = String::new();

        if level == "critical" {
            rec.push_str("CRITICAL: ");
        } else {
            rec.push_str("WARNING: ");
        }

        let declining: Vec<_> = report.trends.iter()
            .filter_map(|t| match t {
                Trend::Declining { metric, .. } => Some(metric.as_str()),
                _ => None,
            })
            .collect();

        if declining.contains(&"type_strictness") {
            rec.push_str("Type safety degrading. Consider adding type annotations. ");
        }
        if declining.contains(&"error_handling") {
            rec.push_str("Error handling declining. Review unwrap() and expect() usage. ");
        }
        if declining.contains(&"naming_consistency") {
            rec.push_str("Naming inconsistencies detected. Run style linter. ");
        }

        for trend in &report.trends {
            if let Trend::Increasing { metric, .. } = trend {
                if metric == "complexity" {
                    rec.push_str("Complexity increasing. Consider refactoring. ");
                }
                if metric == "dead_code" {
                    rec.push_str("Dead code accumulating. Run unused code cleanup. ");
                }
            }
        }

        if rec.is_empty() || rec.ends_with(": ") {
            rec.push_str("Review recent changes for quality issues.");
        }

        rec
    }

    /// Load from disk
    fn load(&mut self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }

        #[derive(Deserialize)]
        struct Serialized {
            snapshots: HashMap<String, Vec<CodeSnapshot>>,
            thresholds: Option<AlertThresholds>,
        }

        let content = fs::read_to_string(&self.path).map_err(Error::Io)?;
        let data: Serialized = serde_json::from_str(&content)?;

        self.snapshots = data.snapshots;
        if let Some(thresholds) = data.thresholds {
            self.alert_thresholds = thresholds;
        }

        tracing::info!("Loaded {} drift snapshot files from {:?}", self.snapshots.len(), self.path);
        Ok(())
    }

    /// Persist to disk
    fn persist(&self) -> Result<()> {
        #[derive(Serialize)]
        struct Serialized<'a> {
            snapshots: &'a HashMap<String, Vec<CodeSnapshot>>,
            thresholds: &'a AlertThresholds,
        }

        let data = Serialized {
            snapshots: &self.snapshots,
            thresholds: &self.alert_thresholds,
        };

        let content = serde_json::to_string_pretty(&data)?;
        fs::write(&self.path, content).map_err(Error::Io)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_metrics(type_strictness: f32, error_handling: f32) -> SnapshotMetrics {
        SnapshotMetrics {
            type_strictness,
            naming_consistency: 0.9,
            error_handling_quality: error_handling,
            complexity: 0.3,
            dead_code_ratio: 0.1,
        }
    }

    #[test]
    fn test_record_and_analyze() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("aether_test_drift.json");
        let mut store = DriftSnapshotStore::new(Some(temp_path.clone())).unwrap();

        // Record two snapshots with declining type strictness
        store.record_metrics(
            "src/main.rs".to_string(),
            make_metrics(0.9, 0.9),
            None,
        ).unwrap();

        store.record_metrics(
            "src/main.rs".to_string(),
            make_metrics(0.7, 0.85),
            None,
        ).unwrap();

        let report = store.analyze_trend("src/main.rs");
        assert!(report.is_some());

        let report = report.unwrap();
        assert_eq!(report.snapshots_analyzed, 2);
        assert!(report.drift_score > 0.0, "Should detect drift");

        // Cleanup
        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_analyze_days() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("aether_test_drift_days.json");
        let mut store = DriftSnapshotStore::new(Some(temp_path.clone())).unwrap();

        // Create old snapshot
        let old_snapshot = CodeSnapshot {
            timestamp: Utc::now() - Duration::days(10),
            commit: None,
            file_path: "src/old.rs".to_string(),
            metrics: make_metrics(0.9, 0.9),
        };
        store.record(old_snapshot).unwrap();

        // Create recent snapshot
        store.record_metrics(
            "src/old.rs".to_string(),
            make_metrics(0.8, 0.85),
            None,
        ).unwrap();

        // Analyze last 5 days - should find only recent snapshot
        let report = store.analyze_trend_days("src/old.rs", 5);
        assert!(report.is_none(), "Should not find enough recent snapshots");

        // Analyze last 15 days - should find both
        let report = store.analyze_trend_days("src/old.rs", 15);
        assert!(report.is_some());

        // Cleanup
        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_alerts() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("aether_test_drift_alerts.json");
        let mut store = DriftSnapshotStore::new(Some(temp_path.clone())).unwrap();

        // Set low thresholds for testing
        store.alert_thresholds = AlertThresholds {
            warning: 0.05,
            critical: 0.15,
            min_snapshots: 2,
        };

        // Create declining metrics
        for i in 0..3 {
            store.record_metrics(
                "src/alert.rs".to_string(),
                make_metrics(0.9 - (i as f32 * 0.15), 0.9 - (i as f32 * 0.1)),
                None,
            ).unwrap();
        }

        let alerts = store.check_alerts();
        assert!(!alerts.is_empty(), "Should generate alerts for declining code");

        // Cleanup
        std::fs::remove_file(&temp_path).ok();
    }
}
