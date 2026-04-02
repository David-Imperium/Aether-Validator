//! Layer 2: Memory System (Hybrid Architecture)
//!
//! Provides persistent memory for validation context.
//!
//! ## Architecture (Context Rot Solution)
//!
//! - **Layer 2A: Code Graph** (AST-based) - `who_calls()`, `what_depends_on()`
//! - **Layer 2B: Decision Log** (Knowledge Graph) - `why_exists()`, `is_accepted()`
//! - **Layer 2C: Validation State** (File-based) - JSON persistence, delta detection
//! - **Layer 2D: Drift Snapshots** (Time-series) - `analyze_trend()`, alerting
//! - **Architectural Drift** - Multi-file drift analysis with dependency expansion
//! - **LearnedConfig** - Dynamic configuration learned from validation history
//! - **ProjectConfig** - `.synward.toml` file parsing and merge
//!
//! ## v3.0 Features
//!
//! - **Temporal Decision Log**: Track decision evolution with `superseded_by`/`supersedes`
//! - **Multi-Signal Scoring**: Composite score from relevance, recency, importance
//! - **Memory Hierarchy**: STM (5min) → MTM (1hr) → LTM (persistent)
//! - **Audit Trail**: Complete history for compliance

mod store;
mod retrieval;
mod project_context;
mod decision_log;
mod validation_state;
mod code_graph;
mod drift_snapshots;
mod architectural_drift;
pub mod learned_config;
mod project_config;
mod presets;

// v3.0 Memory Hierarchy
mod tier;
mod hierarchy;
mod tiers;

// Phase 5: Deduplication
mod dedup;

// v4.0: Hybrid Memory Architecture
mod scope;
mod git_store;

pub use store::{MemoryEntry, MemoryStore, MemoryType};
pub use retrieval::MemoryRetriever;
pub use project_context::ProjectContext;

// Layer 2A: Code Graph
pub use code_graph::{
    CodeGraph, CodeNode, CodeNodeType, ImpactResult, FunctionContext,
    // Code Property Graph (CPG) — for neural feature extraction
    CPGNode, CPGEdge, CPGEdgeType, CPGNodeType, CodePropertyGraph, EdgeIndex, CPGBuilder,
};

// Layer 2B: Decision Log (v3.0)
pub use decision_log::{
    DecisionLog, DecisionNode, DecisionType, DecisionAuthor,
    DecisionStatus, DecisionEdge, DecisionRelation, CodeLocation,
    // v3.0 additions
    MultiSignalScore,
};

// Layer 2C: Validation State
pub use validation_state::{
    ProjectState, FileState, ViolationRecord, AcceptedViolation,
    Severity, FileDelta, ProjectMetadata, ValidationState, ViolationDelta,
};

// Layer 2D: Drift Snapshots
pub use drift_snapshots::{
    DriftSnapshotStore, DriftAlert, AlertThresholds, CodeSnapshot, SnapshotMetrics, Trend, DriftReport,
};

// Architectural Drift Analysis
pub use architectural_drift::{
    ArchitecturalDriftAnalyzer, ArchitecturalDriftConfig, ArchitecturalDriftReport,
    FileDrift, format_report,
};

// Learned Configuration (Memory-Driven Core)
pub use learned_config::{
    LearnedConfig, ConfigId, CustomRule, WhitelistedPattern,
    StyleConventions, NamingConventions, FormattingConventions,
    ImportConventions, ConfigStats,
};

// Project Configuration (.synward.toml)
pub use project_config::{
    ProjectConfig, WhitelistSection, WhitelistEntry,
    StyleSection, NamingSection, FormattingSection,
    RulesSection, CustomRuleEntry, ProjectMetadataSection,
    DubbiosoSection,
};

// Bundled Presets
pub use presets::{
    Preset, PresetManager, PresetRules, PresetRule,
    PresetStyle, PresetNaming, PresetFormatting, PresetMeta,
    export_as_preset, import_preset,
};

// v3.0 Memory Hierarchy
pub use tier::{DecisionId, DecisionEntry, MemoryTier, TierError};
pub use hierarchy::{MemoryHierarchy, MaintenanceReport, HierarchyStats};
pub use tiers::{STM, MTM, LTM};

// Phase 5: Deduplication
pub use dedup::{DedupEngine, DedupConfig, DedupReport, DuplicatePair, DuplicateType};

// v4.0: Hybrid Memory Architecture
pub use scope::{MemoryScope, MemoryPath};
pub use git_store::{GitMemoryStore, SnapshotInfo, CommitHash};

use serde::{Deserialize, Serialize};

/// Unique identifier for a memory entry
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(pub String);

impl Default for MemoryId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

/// Similarity score between 0.0 and 1.0
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SimilarityScore(pub f32);

impl SimilarityScore {
    pub fn new(score: f32) -> Self {
        Self(score.clamp(0.0, 1.0))
    }

    pub fn is_similar(&self, threshold: f32) -> bool {
        self.0 >= threshold
    }
}

/// Query types for unified memory API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryQuery {
    /// Who calls this function?
    WhoCalls { function: String, file: String },

    /// Why does this code exist?
    WhyExists { file: String, line: usize },

    /// Is this violation accepted?
    IsAccepted { violation_id: String, file: String },

    /// Semantic recall (search)
    SemanticRecall { query: String, limit: usize },

    /// Drift trend analysis
    DriftTrend { file: Option<String>, days: usize },

    /// Impact analysis for changes
    ImpactAnalysis { file: String, function: String },
}
