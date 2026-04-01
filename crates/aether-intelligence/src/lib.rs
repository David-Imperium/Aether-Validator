//! Aether Intelligence - Lightweight validation enhancement
//!
//! This crate provides intelligence layers for Aether validation:
//!
//! - **Layer 2: Memory** - Hybrid memory system (4 sub-layers)
//!   - 2A: Code Graph (AST-based)
//!   - 2B: Decision Log (Knowledge Graph)
//!   - 2C: Validation State (File-based)
//!   - 2D: Drift Snapshots (Time-series)
//! - **Layer 3: Patterns** - Rule-based pattern discovery
//! - **Layer 4: Intent** - External LLM API (optional)
//! - **Layer 5: Drift** - Git-based drift detection
//! - **Phase 4: Semantic Search** - TF-IDF + candle-transformers
//!
//! # Features
//!
//! - `memory` (default) - Semantic memory layer
//! - `patterns` - Rule-based pattern discovery
//! - `intent-api` - External LLM integration
//! - `drift` - Git-based drift detection
//! - `semantic-search` - Vector-backed semantic search (Pro tier)
//!
//! # Example
//!
//! ```ignore
//! use aether_intelligence::{AetherIntelligence, Config, MemoryQuery};
//!
//! let ai = AetherIntelligence::new(Config::default())?;
//!
//! // Unified memory API
//! let result = ai.recall(MemoryQuery::WhyExists {
//!     file: "src/main.rs".into(),
//!     line: 42,
//! })?;
//! ```

pub mod error;
pub mod memory;
pub mod knowledge;
pub mod semantic;
pub mod dubbioso;
pub mod mcp_questions;
pub mod dubbioso_patterns;
pub mod dubbioso_validator;
pub mod learner;

#[cfg(feature = "tree-sitter")]
pub mod tree_sitter_parser;

// Phase 4: Semantic Search
pub mod semantic_search;

#[cfg(feature = "patterns")]
pub mod patterns;

#[cfg(feature = "intent-api")]
pub mod intent;

#[cfg(feature = "drift")]
pub mod drift;

pub use error::{Error, Result};

// Semantic Analysis (for Dubbioso Mode)
pub use semantic::{
    SemanticAnalyzer, SemanticContext, FunctionSemanticContext, ErrorHandlingStyle,
};

// Dubbioso Mode (Confidence-based validation)
pub use dubbioso::{
    DubbiosoAnalyzer, DubbiosoConfig, ConfidenceResult, ConfidenceLevel, DubbiosoPreset,
};

// MCP Question Protocol (for Dubbioso Mode)
pub use mcp_questions::{
    McpQuestion, McpResponse, McpQuestionManager, QuestionType,
    QuestionOption, QuestionContext, MemoryImpact, ResponseResult, MemoryUpdate,
};

// Dubbioso Pattern Persistence
pub use dubbioso_patterns::{
    DubbiosoPattern, DubbiosoPatternStore, PatternUpdate,
};

// Phase 4: Semantic Search
pub use semantic_search::{
    SearchEngine, SearchResult, TfidfSearch, TfidfConfig, ModelInfo,
};

#[cfg(feature = "semantic-search")]
pub use semantic_search::{
    VectorSearch, VectorConfig, HybridSearch, HybridConfig,
    EmbeddingModel, EmbeddingConfig,
};

// Dubbioso Validator Integration
pub use dubbioso_validator::{
    DubbiosoValidator, DubbiosoValidationResult, ViolationInput,
};

// Compliance Engine - Intelligent Contract Enforcement
pub mod compliance;
pub use compliance::{
    ComplianceEngine, ComplianceConfig, ComplianceContext, ComplianceStats,
    ContractClassifier, ContractTier, TierMetadata,
    ComplianceDecision, ComplianceAction, DecisionReason,
    ExemptionStore, Exemption, ExemptionScope, ExemptionSource,
};

#[cfg(feature = "memory")]
pub use memory::{
    MemoryEntry, MemoryStore, MemoryRetriever, ProjectContext, MemoryType,
    // Layer 2A: Code Graph
    CodeGraph, CodeNode, CodeNodeType, ImpactResult, FunctionContext,
    // Code Property Graph (CPG) — for neural feature extraction
    CPGNode, CPGEdge, CPGEdgeType, CPGNodeType, CodePropertyGraph, EdgeIndex, CPGBuilder,
    // Layer 2B: Decision Log
    DecisionLog, DecisionNode, DecisionType, DecisionAuthor, DecisionStatus,
    // Layer 2C: Validation State
    ProjectState, FileState, ViolationRecord, AcceptedViolation, Severity, ValidationState,
    // Layer 2D: Drift Snapshots
    DriftSnapshotStore, DriftAlert, AlertThresholds,
    // Architectural Drift Analysis
    ArchitecturalDriftAnalyzer, ArchitecturalDriftReport, format_report as format_drift_report,
    // Unified API
    MemoryQuery,
    // Learned Configuration (Memory-Driven Core)
    LearnedConfig, ConfigId, CustomRule, WhitelistedPattern,
    StyleConventions, NamingConventions, FormattingConventions,
    ImportConventions, ConfigStats,
    // Project Configuration (.aether.toml)
    ProjectConfig, WhitelistSection, WhitelistEntry,
    StyleSection, NamingSection, FormattingSection,
    RulesSection, CustomRuleEntry, ProjectMetadataSection,
    DubbiosoSection,
    // Bundled Presets
    Preset, PresetManager, PresetRules, PresetRule,
    PresetStyle, PresetNaming, PresetFormatting, PresetMeta,
    export_as_preset, import_preset,
};

#[cfg(feature = "patterns")]
pub use patterns::{CodeFeatures, FeatureExtractor, AnomalyDetector, RuleGenerator};

#[cfg(feature = "intent-api")]
pub use intent::{Intent, IntentInferrer};

#[cfg(feature = "drift")]
pub use drift::{DriftMetrics, DriftDetector, DriftReport, Trend};

pub use knowledge::{TypeStubLoader, ApiSignature, LlmApiResolver};
pub use learner::{PatternLearner, LearnedPatterns, NamingPatterns, DerivePatterns, DocPatterns};

// Contract Generator
pub mod contract_generator;
pub use contract_generator::{
    ContractGenerator, NamingRule, NamingRuleType, DeriveRule, DocRule,
    DocRuleType, ContractMetadata, AetherContract,
};

#[cfg(feature = "tree-sitter")]
pub use tree_sitter_parser::{
    TreeSitterParser, Language, AstNode, ParsedFile, DeriveInfo,
};

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration for Aether Intelligence
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Config {
    /// Memory store path
    pub memory_path: Option<PathBuf>,

    /// Decision log path
    pub decision_log_path: Option<PathBuf>,

    /// Validation state path
    pub validation_state_path: Option<PathBuf>,

    /// Code graph path (for persistence)
    pub code_graph_path: Option<PathBuf>,

    /// External LLM API endpoint
    #[cfg(feature = "intent-api")]
    pub llm_api_endpoint: Option<String>,

    /// Git repository path for drift analysis
    #[cfg(feature = "drift")]
    pub git_repo_path: Option<PathBuf>,
}


/// Main entry point for Aether Intelligence
pub struct AetherIntelligence {
    config: Config,

    #[cfg(feature = "memory")]
    memory: MemoryStore,

    #[cfg(feature = "memory")]
    decision_log: DecisionLog,

    #[cfg(feature = "memory")]
    validation_state: ValidationState,

    #[cfg(feature = "memory")]
    code_graph: CodeGraph,

    #[cfg(feature = "memory")]
    drift_snapshots: DriftSnapshotStore,

    #[cfg(feature = "patterns")]
    patterns: FeatureExtractor,

    #[cfg(feature = "intent-api")]
    intent: IntentInferrer,

    #[cfg(feature = "drift")]
    drift: DriftDetector,

    knowledge: TypeStubLoader,
}

impl AetherIntelligence {
    /// Create new Aether Intelligence instance
    pub fn new(config: Config) -> Result<Self> {
        #[cfg(feature = "memory")]
        let memory = MemoryStore::new(config.memory_path.clone())?;

        #[cfg(feature = "memory")]
        let decision_log = DecisionLog::new(config.decision_log_path.clone())?;

        #[cfg(feature = "memory")]
        let validation_state = ValidationState::new(config.validation_state_path.clone())?;

        #[cfg(feature = "memory")]
        let code_graph = if let Some(ref path) = config.code_graph_path {
            if path.exists() {
                CodeGraph::load(path).unwrap_or_else(|_| CodeGraph::new())
            } else {
                CodeGraph::new()
            }
        } else {
            CodeGraph::new()
        };

        #[cfg(feature = "memory")]
        let drift_snapshots = DriftSnapshotStore::new(None)?;

        #[cfg(feature = "patterns")]
        let patterns = FeatureExtractor::new();

        #[cfg(feature = "intent-api")]
        let intent = {
            // Auto-detect Ollama if no endpoint configured
            let endpoint = config.llm_api_endpoint.clone().or_else(|| {
                Self::detect_ollama()
            });
            IntentInferrer::new(endpoint)
        };

        #[cfg(feature = "drift")]
        let drift = DriftDetector::new(config.git_repo_path.clone())?;

        let knowledge = TypeStubLoader::new();

        Ok(Self {
            config,
            #[cfg(feature = "memory")]
            memory,
            #[cfg(feature = "memory")]
            decision_log,
            #[cfg(feature = "memory")]
            validation_state,
            #[cfg(feature = "memory")]
            code_graph,
            #[cfg(feature = "memory")]
            drift_snapshots,
            #[cfg(feature = "patterns")]
            patterns,
            #[cfg(feature = "intent-api")]
            intent,
            #[cfg(feature = "drift")]
            drift,
            knowledge,
        })
    }

    /// Unified memory API: Query the memory system
    ///
    /// This is the main entry point for the `aether recall` CLI command.
    #[cfg(feature = "memory")]
    pub fn recall(&self, query: MemoryQuery) -> Result<MemoryResult> {
        let mut result = MemoryResult::default();

        match query {
            MemoryQuery::WhyExists { file, line } => {
                let decisions = self.decision_log.why_exists(&file, line);
                result.decisions = Some(decisions.into_iter().cloned().collect());
            }

            MemoryQuery::IsAccepted { violation_id, file } => {
                let global = self.decision_log.is_accepted(&violation_id);
                let project_accepted = self.validation_state.projects.values().next()
                    .map(|project: &ProjectState| project.is_accepted(&violation_id, &file))
                    .unwrap_or(false);
                result.is_accepted = global.is_some() || project_accepted;
                if let Some(node) = global {
                    result.reason = Some(node.content.clone());
                }
            }

            MemoryQuery::SemanticRecall { query, limit } => {
                let entries = self.memory.recall(&query, limit)?;
                result.memory_entries = Some(entries);
            }

            MemoryQuery::DriftTrend { file, days } => {
                // Use DriftSnapshotStore (Layer 2D) for trend analysis
                if let Some(file_path) = file {
                    if let Some(report) = self.drift_snapshots.analyze_trend_days(&file_path, days) {
                        result.drift_score = Some(report.drift_score);
                        result.trends = Some(report.trends.iter().map(|t| format!("{:?}", t)).collect());
                        tracing::info!("DriftTrend: {} has drift score {:.2}", file_path, report.drift_score);
                    } else {
                        result.drift_score = Some(0.0);
                        result.trends = Some(vec!["Insufficient snapshots for analysis".to_string()]);
                    }
                } else {
                    // Project-wide: check alerts
                    let alerts = self.drift_snapshots.check_alerts();
                    if !alerts.is_empty() {
                        let avg_score = alerts.iter().map(|a| a.score).sum::<f32>() / alerts.len() as f32;
                        result.drift_score = Some(avg_score);
                        result.trends = Some(alerts.iter()
                            .flat_map(|a| a.trends.clone())
                            .take(10)
                            .map(|t| format!("{:?}", t))
                            .collect());
                        tracing::warn!("Drift alerts: {} files with issues", alerts.len());
                    } else {
                        result.drift_score = Some(0.0);
                        result.trends = Some(vec!["No drift detected".to_string()]);
                    }
                }
            }

            MemoryQuery::ImpactAnalysis { file, function } => {
                // Use CodeGraph (Layer 2A) to analyze impact
                let impact = self.code_graph.impact_analysis(&function, &file);
                result.affected_files = Some(impact.affected_files);
                tracing::info!("ImpactAnalysis query: file={}, function={}", file, function);
            }

            MemoryQuery::WhoCalls { function, file } => {
                // Use CodeGraph (Layer 2A) to find callers
                let callers = self.code_graph.who_calls(&function, &file);
                result.callers = Some(callers.iter().map(|n| format!("{}:{}:{}", n.file, n.line, n.name)).collect());
                tracing::info!("WhoCalls query: function={}, file={}", function, file);
            }
        }

        Ok(result)
    }

    /// Analyze architectural drift for a file and its dependencies
    #[cfg(feature = "memory")]
    pub fn architectural_drift(
        &self,
        root: &std::path::Path,
        days: usize,
        depth: usize,
    ) -> Result<ArchitecturalDriftReport> {
        use memory::{ArchitecturalDriftAnalyzer, ArchitecturalDriftConfig};

        let config = ArchitecturalDriftConfig {
            max_depth: depth,
            max_files: 50,
        };

        let analyzer = ArchitecturalDriftAnalyzer::with_config(
            self.code_graph.clone(),
            self.drift_snapshots.clone(),
            config,
        );

        analyzer.analyze(root, days as u32)
    }

    /// Enhance validation with intelligence layers
    #[cfg(feature = "memory")]
    pub async fn enhance_validation(&self, code: &str) -> Result<IntelligenceResult> {
        let mut result = IntelligenceResult::default();

        // Layer 2: Recall similar code from memory
        let similar = self.memory.recall(code, 5)?;
        result.memory_context = Some(similar);

        // Layer 3: Extract features and detect anomalies
        #[cfg(feature = "patterns")]
        {
            let features = self.patterns.extract(code, "auto");
            let detector = AnomalyDetector::new();
            result.discovered_patterns = Some(detector.detect(&features));
        }

        // Layer 4: Infer intent via external API
        #[cfg(feature = "intent-api")]
        if let Some(ref _endpoint) = self.config.llm_api_endpoint {
            let intent = self.intent.infer(code).await?;
            result.intent = Some(intent);
        }

        Ok(result)
    }

    /// Remember validation result for future reference
    #[cfg(feature = "memory")]
    pub fn remember(&mut self, entry: MemoryEntry) -> Result<()> {
        self.memory.save(entry)
    }

    /// Record a decision in the knowledge graph
    #[cfg(feature = "memory")]
    pub fn record_decision(&mut self, node: DecisionNode) -> Result<memory::MemoryId> {
        self.decision_log.record(node)
    }

    /// Get validation state for a project
    #[cfg(feature = "memory")]
    pub fn get_validation_state(&mut self, project_root: &Path) -> &mut ProjectState {
        self.validation_state.get_project(project_root)
    }

    /// Check API signature against knowledge base
    pub fn check_api(&self, module: &str, function: &str, args: &[crate::knowledge::ArgInfo]) -> Option<crate::knowledge::ApiCheckResult> {
        self.knowledge.check_api_call(module, function, args).ok()
    }

    /// Index a project's source files into the code graph
    #[cfg(feature = "memory")]
    pub fn index_project(&mut self, project_root: &Path) -> Result<()> {
        use std::fs;

        fn should_index(path: &std::path::Path) -> bool {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            matches!(ext, "rs" | "py" | "js" | "ts")
        }

        fn get_language(path: &std::path::Path) -> &'static str {
            match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
                "rs" => "rust",
                "py" => "python",
                "js" => "javascript",
                "ts" => "typescript",
                _ => "unknown",
            }
        }

        fn visit_dir(dir: &std::path::Path, graph: &mut CodeGraph, project_root: &std::path::Path) -> std::io::Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Skip hidden dirs and common exclusions
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !name.starts_with('.') && !matches!(name, "target" | "node_modules" | "venv" | "__pycache__") {
                        visit_dir(&path, graph, project_root)?;
                    }
                } else if should_index(&path) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let relative = path.strip_prefix(project_root).unwrap_or(&path);
                        // Normalize path separators to forward slashes for consistency
                        let file_str = relative.to_string_lossy().replace('\\', "/");
                        graph.parse_file(&content, &file_str, get_language(&path));
                    }
                }
            }
            Ok(())
        }

        visit_dir(project_root, &mut self.code_graph, project_root)?;
        self.code_graph.build_callers();

        // Persist code graph if path configured
        if let Some(ref path) = self.config.code_graph_path {
            if let Err(e) = self.code_graph.save(path) {
                tracing::warn!("Failed to save code graph: {}", e);
            }
        }

        tracing::info!("Indexed project: {} nodes in code graph", self.code_graph.all_nodes().count());
        Ok(())
    }

    /// Get the code graph (read-only)
    #[cfg(feature = "memory")]
    pub fn code_graph(&self) -> &CodeGraph {
        &self.code_graph
    }

    // ========================================================================
    // Memory-Driven Core Methods (Phase 16)
    // ========================================================================

    /// Load learned configuration for a project.
    ///
    /// This enables Memory-Driven validation where past decisions
    /// influence current validation behavior.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = ai.load_config(&project_root)?;
    /// let pipeline = ValidationPipeline::new()
    ///     .with_config(config.to_json()?);
    /// ```
    #[cfg(feature = "memory")]
    pub fn load_config(&self, project_root: &Path) -> Result<LearnedConfig> {
        self.memory.load_config(project_root)
    }

    /// Save learned configuration for a project.
    #[cfg(feature = "memory")]
    pub fn save_config(&self, config: &LearnedConfig) -> Result<()> {
        self.memory.save_config(config)
    }

    /// Record feedback from validation results.
    ///
    /// This is the feedback loop that enables learning:
    /// 1. User accepts/rejects violations
    /// 2. Config is updated based on acceptance patterns
    /// 3. Future validations use the evolved config
    ///
    /// # Arguments
    ///
    /// * `config` - Mutable reference to LearnedConfig to update
    /// * `violations` - All violations from validation
    /// * `accepted_ids` - IDs of violations the user accepted
    #[cfg(feature = "memory")]
    pub fn record_feedback(
        &self,
        config: &mut LearnedConfig,
        violations: &[ViolationRecord],
        accepted_ids: &[String],
    ) -> Result<()> {
        self.memory.update_config_from_feedback(config, violations, accepted_ids)
    }

    /// Validate and learn from accepted violations.
    ///
    /// This is the unified feedback loop entry point that:
    /// 1. Loads or creates LearnedConfig for the project
    /// 2. Records acceptance decisions in DecisionLog
    /// 3. Updates LearnedConfig based on feedback
    /// 4. Persists all changes atomically
    ///
    /// # Arguments
    ///
    /// * `project_root` - Project root directory
    /// * `violations` - All violations found during validation
    /// * `accepted_ids` - IDs of violations the user accepted
    /// * `reason` - Reason for accepting the violations
    ///
    /// # Returns
    ///
    /// The updated LearnedConfig, or error.
    #[cfg(feature = "memory")]
    pub fn validate_and_learn(
        &mut self,
        project_root: &Path,
        violations: &[ViolationRecord],
        accepted_ids: &[String],
        reason: &str,
    ) -> Result<LearnedConfig> {

        // Ensure .aether directory exists
        let aether_dir = project_root.join(".aether");
        std::fs::create_dir_all(&aether_dir).ok();

        // 1. Load or create LearnedConfig
        let mut config = self.load_config(project_root)?;

        // 2. Record decisions in DecisionLog for each accepted violation
        for violation_id in accepted_ids {
            // Find the violation to get file/line info
            let violation = violations.iter().find(|v| &v.id == violation_id);

            let mut decision = DecisionNode::new(DecisionType::AcceptViolation, reason.to_string())
                .by(DecisionAuthor::User)
                .with_status(DecisionStatus::Accepted);

            if let Some(v) = violation {
                decision = decision.at(v.file.clone(), v.line);
            }

            decision = decision.with_tags(&[violation_id]);
            self.decision_log.record(decision)?;
        }

        // DecisionLog::record() already persists internally via persist()

        // 3. Apply feedback to LearnedConfig
        self.record_feedback(&mut config, violations, accepted_ids)?;

        // 4. Save LearnedConfig
        self.save_config(&config)?;

        tracing::info!(
            "validate_and_learn: {} violations accepted, config updated",
            accepted_ids.len()
        );

        Ok(config)
    }

    /// Auto-detect Ollama endpoint (localhost:11434)
    ///
    /// Returns Some(endpoint) if Ollama is running, None otherwise.
    /// This enables Layer 4 (Intent Inference) automatically when available.
    #[cfg(feature = "intent-api")]
    fn detect_ollama() -> Option<String> {
        // Try common Ollama ports
        let ports = [11434, 11435];

        for port in &ports {
            let addr: std::net::SocketAddr = format!("127.0.0.1:{}", port).parse().ok()?;
            if let Ok(stream) = std::net::TcpStream::connect_timeout(
                &addr,
                std::time::Duration::from_millis(100),
            ) {
                drop(stream);
                let endpoint = format!("http://localhost:{}/api/generate", port);
                tracing::info!("Ollama detected at {}", endpoint);
                return Some(endpoint);
            }
        }

        tracing::debug!("Ollama not detected, Layer 4 (Intent Inference) disabled");
        None
    }
}

/// Result of a memory query
#[derive(Debug, Default, Serialize)]
pub struct MemoryResult {
    /// Decision nodes (for WhyExists queries)
    #[cfg(feature = "memory")]
    pub decisions: Option<Vec<DecisionNode>>,

    /// Memory entries (for SemanticRecall queries)
    #[cfg(feature = "memory")]
    pub memory_entries: Option<Vec<MemoryEntry>>,

    /// Is the violation accepted?
    pub is_accepted: bool,

    /// Reason for acceptance (if any)
    pub reason: Option<String>,

    /// Drift score (for DriftTrend queries)
    #[cfg(feature = "memory")]
    pub drift_score: Option<f32>,

    /// Drift trends (for DriftTrend queries)
    #[cfg(feature = "memory")]
    pub trends: Option<Vec<String>>,

    /// Affected files (for ImpactAnalysis queries)
    pub affected_files: Option<Vec<String>>,

    /// Callers (for WhoCalls queries)
    pub callers: Option<Vec<String>>,
}

/// Result of intelligence analysis
#[derive(Debug, Default, Serialize)]
pub struct IntelligenceResult {
    /// Similar code from memory (Layer 2)
    #[cfg(feature = "memory")]
    pub memory_context: Option<Vec<MemoryEntry>>,

    /// Discovered patterns/anomalies (Layer 3)
    #[cfg(feature = "patterns")]
    pub discovered_patterns: Option<Vec<crate::patterns::Anomaly>>,

    /// Inferred intent (Layer 4)
    #[cfg(feature = "intent-api")]
    pub intent: Option<Intent>,

    /// Drift analysis (Layer 5)
    #[cfg(feature = "drift")]
    pub drift: Option<DriftReport>,
}
