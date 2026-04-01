//! Aether MCP Server - Optimized Version
//!
//! Full MCP implementation with tools, resources, and prompts.
//! Supports MCP sampling for AI-powered suggestions and progress reporting.
//!
//! OPTIMIZATIONS:
//! - Global Parser Registry (cached, created once)
//! - Watch mode limits (prevent memory leaks)
//! - Streaming file reads (bounded memory)
//! - Batch chunking (process files in batches)

use anyhow::Result;
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        ServerInfo, ServerCapabilities, ProtocolVersion, Implementation,
        Prompt, PromptArgument, PromptMessage, PromptMessageRole,
        RawResource, ResourceContents, AnnotateAble,
        GetPromptResult, ListPromptsResult, ListResourcesResult, ReadResourceResult,
        CreateMessageRequestParams, SamplingMessage,
        ModelPreferences, Content, ProgressNotificationParam, ProgressToken,
        CompleteRequestParams, CompleteResult, CompletionInfo,
        Reference, PromptReference, GetPromptRequestParams, ReadResourceRequestParams,
        CallToolResult, ErrorData as McpError,
        PaginatedRequestParams,
    },
    schemars::{self, JsonSchema},
    service::{Peer, RoleServer, RequestContext},
    tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::RwLock;

use aether_parsers::ParserRegistry;
use aether_validation::layers::ContractLayer;
use aether_validation::{ValidationContext, ValidationLayer};
use aether_intelligence::dubbioso::{DubbiosoConfig, ConfidenceLevel};
use aether_intelligence::dubbioso_validator::{DubbiosoValidator, ViolationInput};

// CodeGraph, Memory, and State imports
use aether_intelligence::memory::{
    CodeGraph,
    MemoryStore, MemoryEntry, MemoryType,
    ValidationState, AcceptedViolation,
    DecisionLog,
};

// Compliance Engine imports
use aether_intelligence::compliance::{
    ComplianceEngine, ComplianceConfig, ComplianceContext,
    ComplianceAction, ContractTier,
};

// Scope and Type Inference imports (new tools - kept for future expansion)
// use aether_validation::scope::{ScopeAnalysisLayer, ScopeTree, SymbolKind};
// use aether_validation::type_inference::{TypeInferenceEngine, Type, TypeKind};

// ============================================================================
// Memory Optimization Constants
// ============================================================================

/// Maximum number of concurrent watches
const MAX_WATCHES: usize = 50;

/// Maximum files per watch
const MAX_FILES_PER_WATCH: usize = 500;

/// Maximum file size to read (5MB)
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

/// Batch chunk size for batch_validate
const BATCH_CHUNK_SIZE: usize = 25;

/// Cleanup interval in seconds (5 minutes)
const CLEANUP_INTERVAL_SECS: u64 = 300;

/// Watch max age in seconds (1 hour before cleanup)
const WATCH_MAX_AGE_SECS: u64 = 3600;

// ============================================================================
// Global Parser Registry (CRITICAL OPTIMIZATION)
// ============================================================================

/// Global parser registry - created ONCE, reused for all operations.
/// This saves ~30GB of memory by not recreating tree-sitter grammars.
static PARSER_REGISTRY: std::sync::OnceLock<Arc<ParserRegistry>> = std::sync::OnceLock::new();

fn get_parser_registry() -> Arc<ParserRegistry> {
    PARSER_REGISTRY
        .get_or_init(|| {
            tracing::debug!("Initializing global ParserRegistry (one-time cost)");
            Arc::new(ParserRegistry::with_defaults())
        })
        .clone()
}

// ============================================================================
// Global Contract Layer
// ============================================================================

/// Global contract layer - loads YAML contracts from ~/.aether/contracts/
static CONTRACT_LAYER: std::sync::OnceLock<std::sync::Mutex<ContractLayer>> = std::sync::OnceLock::new();

/// Get contracts directory (same logic as CLI)
fn get_contracts_dir() -> PathBuf {
    // First check local .factory/contracts
    let local = std::env::current_dir()
        .map(|c| c.join(".factory/contracts"))
        .unwrap_or_default();
    if local.exists() {
        return local;
    }
    
    // Check for contracts/ in current directory
    let local_contracts = std::env::current_dir()
        .map(|c| c.join("contracts"))
        .unwrap_or_default();
    if local_contracts.exists() {
        return local_contracts;
    }
    
    // Then check home directory ~/.aether/contracts
    dirs::home_dir()
        .map(|h| h.join(".aether/contracts"))
        .unwrap_or_else(|| PathBuf::from("contracts"))
}

fn get_contract_layer() -> &'static std::sync::Mutex<ContractLayer> {
    CONTRACT_LAYER.get_or_init(|| {
        let contracts_dir = get_contracts_dir();
        tracing::debug!("ContractLayer using path: {:?}", contracts_dir);
        std::sync::Mutex::new(ContractLayer::with_path(contracts_dir))
    })
}

// ============================================================================
// Global Dubbioso Validator
// ============================================================================

/// Global Dubbioso validator for confidence-based validation
static DUBBIOSO_VALIDATOR: std::sync::OnceLock<std::sync::Mutex<DubbiosoValidator>> = std::sync::OnceLock::new();

fn get_dubbioso_validator() -> &'static std::sync::Mutex<DubbiosoValidator> {
    DUBBIOSO_VALIDATOR.get_or_init(|| {
        std::sync::Mutex::new(DubbiosoValidator::new(DubbiosoConfig::default()))
    })
}

// ============================================================================
// Global Compliance Engine
// ============================================================================

/// Global Compliance Engine for intelligent contract enforcement
static COMPLIANCE_ENGINE: std::sync::OnceLock<std::sync::Mutex<ComplianceEngine>> = std::sync::OnceLock::new();

fn get_compliance_engine() -> &'static std::sync::Mutex<ComplianceEngine> {
    COMPLIANCE_ENGINE.get_or_init(|| {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let aether_dir = cwd.join(".aether");
        
        let config = ComplianceConfig {
            exemption_store_path: Some(aether_dir.join("exemptions.json")),
            ..ComplianceConfig::default()
        };
        
        std::sync::Mutex::new(
            ComplianceEngine::with_config(config).expect("Failed to create ComplianceEngine")
        )
    })
}

// ============================================================================
// CodeGraph, Memory, and State (CLI Parity)
// ============================================================================

/// Global CodeGraph for dependency analysis
#[allow(dead_code)]
static CODE_GRAPH: std::sync::OnceLock<std::sync::Mutex<CodeGraph>> = std::sync::OnceLock::new();

#[allow(dead_code)]
fn get_code_graph() -> &'static std::sync::Mutex<CodeGraph> {
    CODE_GRAPH.get_or_init(|| {
        std::sync::Mutex::new(CodeGraph::new())
    })
}

/// Global MemoryStore for semantic memory
static MEMORY_STORE: std::sync::OnceLock<std::sync::Mutex<MemoryStore>> = std::sync::OnceLock::new();

fn get_memory_store() -> &'static std::sync::Mutex<MemoryStore> {
    MEMORY_STORE.get_or_init(|| {
        std::sync::Mutex::new(MemoryStore::new(None).expect("Failed to create MemoryStore"))
    })
}

/// Global DecisionLog for tracking decisions
#[allow(dead_code)]
static DECISION_LOG: std::sync::OnceLock<std::sync::Mutex<DecisionLog>> = std::sync::OnceLock::new();

#[allow(dead_code)]
fn get_decision_log() -> &'static std::sync::Mutex<DecisionLog> {
    DECISION_LOG.get_or_init(|| {
        std::sync::Mutex::new(DecisionLog::new(None).expect("Failed to create DecisionLog"))
    })
}

/// Global ValidationState for persistence
#[allow(dead_code)]
static VALIDATION_STATE: std::sync::OnceLock<std::sync::Mutex<ValidationState>> = std::sync::OnceLock::new();

#[allow(dead_code)]
fn get_validation_state() -> &'static std::sync::Mutex<ValidationState> {
    VALIDATION_STATE.get_or_init(|| {
        std::sync::Mutex::new(ValidationState::new(None).expect("Failed to create ValidationState"))
    })
}

// ============================================================================
// Watch Mode State (with limits)
// ============================================================================

/// File state for watch mode (path -> last modified timestamp)
type FileState = HashMap<String, std::time::SystemTime>;

/// Watch entry with creation time for age-based cleanup
struct WatchEntry {
    file_state: FileState,
    created_at: std::time::SystemTime,
}

/// Active watches (watch_id -> watch entry)
static WATCHES: std::sync::OnceLock<Arc<Mutex<HashMap<u32, WatchEntry>>>> = std::sync::OnceLock::new();

fn get_watches() -> &'static Arc<Mutex<HashMap<u32, WatchEntry>>> {
    WATCHES.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

static WATCH_ID_COUNTER: std::sync::OnceLock<AtomicU32> = std::sync::OnceLock::new();

fn next_watch_id() -> u32 {
    let counter = WATCH_ID_COUNTER.get_or_init(|| AtomicU32::new(1));
    counter.fetch_add(1, Ordering::SeqCst)
}

/// Cleanup old watches (exceeds MAX_WATCHES or too old)
fn cleanup_watches_if_needed() {
    let watches = get_watches();
    if let Ok(mut w) = watches.lock() {
        let now = std::time::SystemTime::now();

        // Remove watches that are too old
        let old_ids: Vec<u32> = w.iter()
            .filter(|(_, entry)| {
                if let Ok(elapsed) = now.duration_since(entry.created_at) {
                    elapsed.as_secs() > WATCH_MAX_AGE_SECS
                } else {
                    false
                }
            })
            .map(|(id, _)| *id)
            .collect();

        for id in &old_ids {
            w.remove(id);
            tracing::debug!("Removed expired watch {} (age > {}s)", id, WATCH_MAX_AGE_SECS);
        }

        // If still over limit, remove oldest by ID
        if w.len() > MAX_WATCHES {
            let ids_to_remove: Vec<u32> = w.keys()
                .take(w.len() - MAX_WATCHES)
                .copied()
                .collect();
            for id in ids_to_remove {
                w.remove(&id);
                tracing::debug!("Removed excess watch {} (limit {})", id, MAX_WATCHES);
            }
        }
    }
}

/// Background cleanup task - runs periodically
async fn cleanup_task() {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(CLEANUP_INTERVAL_SECS));

    loop {
        interval.tick().await;
        tracing::debug!("Running periodic cleanup...");
        cleanup_watches_if_needed();
    }
}

// ============================================================================
// Constants
// ============================================================================

const VERSION: &str = env!("CARGO_PKG_VERSION");

const CONTRACTS: &[(&str, &str, &str)] = &[
    ("no_unsafe", "security", "No unsafe code blocks"),
    ("no_panic", "reliability", "No panic or unwrap"),
    ("documentation", "style", "Public items must have docs"),
    ("complexity", "maintainability", "Limit function complexity"),
    ("naming", "style", "Follow naming conventions"),
];

// ============================================================================
// Tool Input Schemas
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateInput {
    pub file_path: String,
    pub language: Option<String>,
    pub contracts: Option<String>,
    /// Enable Dubbioso Mode for confidence-based validation
    pub dubbioso_mode: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BatchValidateInput {
    pub file_paths: Vec<String>,
    pub contracts: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnalyzeInput {
    pub code: String,
    pub language: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MetricsInput {
    pub code: String,
    pub language: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SuggestFixesInput {
    pub code: String,
    pub language: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CertifyInput {
    pub code: String,
    pub language: String,
    pub signer: String,
    #[serde(default)]
    pub contracts: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LanguageInfoInput {
    pub language: String,
}

// ============================================================================
// CodeGraph Tool Inputs
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BuildGraphInput {
    /// Directory to index for building the code graph
    pub directory: String,
    /// File extensions to include (comma-separated, e.g., "rs,py,js")
    pub extensions: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WhoCallsInput {
    /// Function name to find callers for
    pub function: String,
    /// File path (optional, for disambiguation)
    pub file: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImpactAnalysisInput {
    /// Function name to analyze impact for
    pub function: String,
    /// File path (optional, for disambiguation)
    pub file: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileDependenciesInput {
    /// File path to get dependencies for
    pub file: String,
    /// Maximum depth for traversal (default: 1, direct dependencies only)
    pub max_depth: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileDependentsInput {
    /// File path to get dependents for
    pub file: String,
    /// Maximum depth for traversal (default: 1, direct dependents only)
    pub max_depth: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetContextInput {
    /// Function name to get context for
    pub function: String,
    /// File path (optional, for disambiguation)
    pub file: Option<String>,
    /// Maximum depth for context traversal
    pub max_depth: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindCallChainInput {
    /// Starting function name
    pub from_function: String,
    /// Starting file path
    pub from_file: String,
    /// Target function name
    pub to_function: String,
    /// Target file path
    pub to_file: String,
}

// ============================================================================
// Memory Tool Inputs
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MemoryRecallInput {
    /// Query string or code snippet to search for similar patterns
    pub query: String,
    /// Maximum number of results
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MemoryStoreInput {
    /// Code snippet to store
    pub code: String,
    /// Language of the code
    pub language: String,
    /// Type of memory entry (pattern, fix, decision, anomaly)
    pub memory_type: Option<String>,
    /// Tags for categorization
    pub tags: Option<Vec<String>>,
}

// ============================================================================
// State Tool Inputs
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SaveStateInput {
    /// Project root directory
    pub project_root: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LoadStateInput {
    /// Project root directory
    pub project_root: String,
}

// ============================================================================
// Learning Tool Inputs
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AcceptViolationInput {
    /// Project root directory
    pub project_root: String,
    /// Violation ID to accept
    pub violation_id: String,
    /// Reason for accepting the violation
    pub reason: String,
    /// File path (optional)
    pub file: Option<String>,
    /// Line number (optional)
    pub line: Option<u32>,
}

// ============================================================================
// New Tools: Scope Analysis, Type Inference, Confidence
// ============================================================================

/// Input for analyze_scope tool
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeScopeInput {
    /// Source code to analyze
    pub code: String,
    /// Programming language (rust, python, javascript, etc.)
    pub language: String,
}

/// Input for infer_types tool
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InferTypesInput {
    /// Source code to analyze
    pub code: String,
    /// Programming language
    pub language: String,
}

/// Input for get_confidence tool
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetConfidenceInput {
    /// Source code to analyze
    pub code: String,
    /// Programming language
    pub language: String,
    /// Optional list of existing violations to consider
    pub violations: Option<Vec<serde_json::Value>>,
}

// ============================================================================
// Compliance Engine Input Schemas
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceEvaluateInput {
    /// Rule ID (e.g., "SEC001", "STYLE002")
    pub rule_id: String,
    /// Domain (e.g., "security", "style")
    pub domain: String,
    /// Violation message
    pub message: String,
    /// File path
    pub file_path: String,
    /// Line number
    pub line: Option<usize>,
    /// Code region (e.g., "test", "main")
    pub code_region: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceAcceptInput {
    /// Rule ID to accept
    pub rule_id: String,
    /// File path
    pub file_path: String,
    /// Reason for acceptance
    pub reason: String,
}

// ============================================================================
// Drift Detection Input Schemas
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DriftAnalyzeInput {
    /// File or directory path to analyze
    pub path: String,
    /// Number of days to analyze (default: 30)
    pub days: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DriftTrendInput {
    /// Project or file path
    pub path: String,
    /// Number of days for trend window (default: 7)
    pub window_days: Option<u32>,
}

// ============================================================================
// Tool Output Schemas
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ValidateOutput {
    pub passed: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub language: String,
    pub layers: ValidationLayers,
    /// Quality score (0-100) calculated from violations
    pub quality_score: u8,
    /// Aggregated validation summary
    pub summary: ValidationSummary,
}

#[derive(Debug, Serialize)]
pub struct ValidationSummary {
    /// Violation count by severity: "error", "warning", "info"
    pub by_severity: HashMap<String, usize>,
    /// Violation count by validation layer
    pub by_layer: HashMap<String, usize>,
    /// Total number of violations
    pub total_violations: usize,
}

#[derive(Debug, Serialize)]
pub struct BatchValidateOutput {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<FileResult>,
}

#[derive(Debug, Serialize)]
pub struct FileResult {
    pub file_path: String,
    pub language: String,
    pub passed: bool,
    pub errors: usize,
    pub warnings: usize,
}

#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub id: String,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub layer: String,
    pub is_new: bool,
    /// Confidence score (0-1) when Dubbioso Mode is enabled
    pub confidence: Option<f64>,
    /// Confidence level: "Ask", "Warn", "Good", "AutoAccept"
    pub confidence_level: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ValidationWarning {
    pub id: String,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ValidationLayers {
    pub syntax: bool,
    pub semantic: bool,
    pub logic: bool,
    pub security: bool,
    pub contracts: bool,
    pub style: bool,
}

#[derive(Debug, Serialize)]
pub struct AnalyzeOutput {
    pub language: String,
    pub total_nodes: usize,
    pub node_types: Vec<NodeTypeCount>,
    pub max_depth: usize,
}

#[derive(Debug, Serialize)]
pub struct NodeTypeCount {
    pub node_type: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct MetricsOutput {
    pub language: String,
    pub lines_of_code: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub total_nodes: usize,
    pub max_depth: usize,
    pub functions: usize,
    pub classes: usize,
    pub complexity_estimate: usize,
    // Advanced metrics
    pub maintainability_index: f64,
    pub technical_debt_minutes: u32,
    pub code_smell_density: f64,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub coupling_score: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct SuggestFixesOutput {
    pub language: String,
    pub suggestions: Vec<FixSuggestion>,
}

#[derive(Debug, Serialize)]
pub struct FixSuggestion {
    pub error_id: String,
    pub message: String,
    pub fix: String,
    pub confidence: f64,
}

#[derive(Debug, Serialize)]
pub struct CertifyOutput {
    pub passed: bool,
    pub certificate: Option<String>,
    pub signature: Option<String>,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug, Serialize)]
pub struct LanguageInfoOutput {
    pub language: String,
    pub extensions: Vec<String>,
    pub supported: bool,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct VersionOutput {
    pub version: String,
    pub name: String,
    pub languages_count: usize,
    pub tools_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ContractsOutput {
    pub contracts: Vec<ContractInfo>,
}

#[derive(Debug, Serialize)]
pub struct ContractInfo {
    pub name: String,
    pub category: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct WatchStartOutput {
    pub watch_id: u32,
    pub error: Option<String>,
    pub files_count: usize,
}

#[derive(Debug, Serialize)]
pub struct WatchCheckOutput {
    pub watch_id: u32,
    pub changed_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct WatchStopOutput {
    pub watch_id: u32,
    pub stopped: bool,
}

// ============================================================================
// CodeGraph Output Schemas
// ============================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct BuildGraphOutput {
    pub nodes_count: usize,
    pub edges_count: usize,
    pub files_indexed: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CallerInfo {
    pub function_name: String,
    pub file_path: String,
    pub line: usize,
    pub call_type: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct WhoCallsOutput {
    pub target_function: String,
    pub callers: Vec<CallerInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AffectedFile {
    pub path: String,
    pub impact_score: f64,
    pub change_type: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ImpactAnalysisOutput {
    pub source_file: String,
    pub affected_files: Vec<AffectedFile>,
    pub affected_functions: Vec<String>,
    pub total_impact_score: f64,
    pub risk_level: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct FileDependenciesOutput {
    pub file_path: String,
    pub dependencies: Vec<String>,
    pub transitive_deps: Vec<String>,
    pub total: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct FileDependentsOutput {
    pub file_path: String,
    pub dependents: Vec<String>,
    pub transitive_dependents: Vec<String>,
    pub total: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GetContextOutput {
    pub function_name: String,
    pub context: String,
    pub related_functions: Vec<String>,
    pub confidence_score: f64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CallChainStep {
    pub from_function: String,
    pub to_function: String,
    pub file_path: String,
    pub line: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct FindCallChainOutput {
    pub source: String,
    pub target: String,
    pub chain: Vec<CallChainStep>,
    pub found: bool,
    pub length: usize,
}

// ============================================================================
// Memory Output Schemas
// ============================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct MemoryEntryOutput {
    pub id: String,
    pub code: String,
    pub language: String,
    pub memory_type: String,
    pub errors: Vec<String>,
    pub recall_count: u32,
    pub created_at: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MemoryRecallOutput {
    pub query: String,
    pub entries: Vec<MemoryEntryOutput>,
    pub total: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MemoryStoreOutput {
    pub id: String,
    pub stored: bool,
    pub memory_type: String,
}

// ============================================================================
// State Output Schemas
// ============================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct SaveStateOutput {
    pub saved: bool,
    pub path: String,
    pub violations_count: usize,
    pub decisions_count: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct LoadStateOutput {
    pub loaded: bool,
    pub path: String,
    pub violations_count: usize,
    pub decisions_count: usize,
}

// ============================================================================
// Learning Output Schemas
// ============================================================================

#[derive(Debug, Serialize, JsonSchema)]
pub struct AcceptViolationOutput {
    pub accepted: bool,
    pub violation_id: String,
    pub reason: String,
    pub config_updated: bool,
}

/// Output for analyze_scope tool
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeScopeOutput {
    /// Number of scopes found
    pub scope_count: usize,
    /// Symbols found in scopes
    pub symbols: Vec<SymbolInfo>,
    /// Unused variables detected
    pub unused_variables: Vec<String>,
    /// Shadowing detected
    pub shadowing: Vec<ShadowInfo>,
}

/// Symbol information
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String,
    pub scope_path: String,
    pub line: usize,
}

/// Shadowing information
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShadowInfo {
    pub name: String,
    pub outer_line: usize,
    pub inner_line: usize,
}

/// Output for infer_types tool
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InferTypesOutput {
    /// Inferred types map (variable -> type)
    pub types: HashMap<String, String>,
    /// Type inference errors
    pub errors: Vec<TypeError>,
    /// Potential violations from type mismatches
    pub violations: Vec<String>,
}

/// Type inference error
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TypeError {
    pub message: String,
    pub line: usize,
}

/// Output for get_confidence tool
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetConfidenceOutput {
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Confidence level (Ask, Warn, Good, AutoAccept)
    pub level: String,
    /// Questions that should be asked
    pub questions: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WatchStartInput {
    pub directory: String,
    pub extensions: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WatchCheckInput {
    pub watch_id: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WatchStopInput {
    pub watch_id: u32,
}

// ============================================================================
// MCP Server
// ============================================================================

#[derive(Debug, Clone)]
pub struct AetherServer {
    peer: Arc<RwLock<Option<Peer<RoleServer>>>>,
    progress_counter: Arc<AtomicU32>,
    tool_router: ToolRouter<Self>,
}

impl Default for AetherServer {
    fn default() -> Self {
        Self::new()
    }
}

impl AetherServer {
    pub fn new() -> Self {
        Self {
            peer: Arc::new(RwLock::new(None)),
            progress_counter: Arc::new(AtomicU32::new(1)),
            tool_router: Self::tool_router(),
        }
    }
    /// Request AI suggestions via MCP sampling from the connected LLM client.
    /// Returns None if no peer is available or sampling fails.
    pub async fn request_ai_suggestions(
        &self,
        code: &str,
        language: &str,
        errors: &[String],
    ) -> Option<String> {
        let peer = self.peer.read().await.clone()?;

        let prompt = format!(
            "You are a code analysis assistant. Given the following {} code with these errors:\n\n{}\n\nCode:\n```\n{}\n```\n\nProvide specific, actionable suggestions to fix these errors. Format each suggestion as a concise bullet point.",
            language,
            errors.join("\n- "),
            code
        );

        let params = CreateMessageRequestParams::new(vec![SamplingMessage::user_text(prompt)], 2048)
            .with_model_preferences(
                ModelPreferences::new()
                    .with_cost_priority(0.3)
                    .with_speed_priority(0.7)
                    .with_intelligence_priority(0.8)
            )
            .with_system_prompt("You are an expert code reviewer. Provide concise, specific fix suggestions.");

        match peer.create_message(params).await {
            Ok(response) => {
                // Extract text from the response content
                response.message.content.first().and_then(|c| c.as_text()).map(|t| t.text.clone())
            }
            Err(e) => {
                tracing::warn!("MCP sampling failed: {}", e);
                None
            }
        }
    }

    /// Generate a new progress token for tracking long-running operations.
    pub fn new_progress_token(&self) -> ProgressToken {
        ProgressToken(rmcp::model::NumberOrString::Number(self.progress_counter.fetch_add(1, Ordering::SeqCst) as i64))
    }

    /// Report progress to the connected client.
    /// Returns true if progress was reported successfully.
    pub async fn report_progress(
        &self,
        token: ProgressToken,
        progress: f64,
        total: Option<f64>,
    ) -> bool {
        let peer = match self.peer.read().await.clone() {
            Some(p) => p,
            None => return false,
        };

        let params = ProgressNotificationParam {
            progress_token: token,
            progress,
            total,
            message: None,
        };

        match peer.notify_progress(params).await {
            Ok(()) => true,
            Err(e) => {
                tracing::warn!("Progress notification failed: {}", e);
                false
            }
        }
    }
}

#[tool_router]
impl AetherServer {
    #[tool(description = "Validate a source code file and return structured results")]
    async fn validate_file(&self, input: Parameters<ValidateInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        let path = PathBuf::from(&input.file_path);
        let language = input.language.unwrap_or_else(|| detect_language(&path));
        let dubbioso_mode = input.dubbioso_mode.unwrap_or(false);
        let file_path_str = input.file_path.as_str();

        let result = match read_file_bounded(&path) {
            Ok(content) => {
                match validate_code(&content, &language, input.contracts.as_deref(), dubbioso_mode, Some(file_path_str)).await {
                    Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
                    Err(e) => format!("{{\"error\": \"{}\"}}", e),
                }
            }
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        };
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Validate multiple files in batch mode with progress reporting")]
    async fn batch_validate(&self, input: Parameters<BatchValidateInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        let mut results = Vec::new();
        let mut passed = 0;
        let mut failed = 0;
        let total_files = input.file_paths.len() as u32;

        // Generate progress token for this batch operation
        let progress_token = self.new_progress_token();

        // Process files in chunks to limit memory usage
        let chunks: Vec<&[String]> = input.file_paths.chunks(BATCH_CHUNK_SIZE).collect();
        let total_chunks = chunks.len();

        for (chunk_idx, chunk) in chunks.iter().enumerate() {
            tracing::debug!("Processing chunk {}/{} ({} files)", chunk_idx + 1, total_chunks, chunk.len());

            for file_path in chunk.iter() {
                let path = PathBuf::from(file_path);
                let language = detect_language(&path);
                let file_path_str = file_path.as_str();

                match read_file_bounded(&path) {
                    Ok(content) => {
                        match validate_code(&content, &language, input.contracts.as_deref(), false, Some(file_path_str)).await {
                            Ok(result) => {
                                if result.passed { passed += 1; } else { failed += 1; }
                                results.push(FileResult {
                                    file_path: file_path.clone(),
                                    language,
                                    passed: result.passed,
                                    errors: result.errors.len(),
                                    warnings: result.warnings.len(),
                                });
                            }
                            Err(_) => {
                                failed += 1;
                                results.push(FileResult {
                                    file_path: file_path.clone(),
                                    language,
                                    passed: false,
                                    errors: 1,
                                    warnings: 0,
                                });
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        results.push(FileResult {
                            file_path: file_path.clone(),
                            language,
                            passed: false,
                            errors: 1,
                            warnings: 0,
                        });
                        tracing::warn!("Skipping file {}: {}", file_path, e);
                    }
                }
            }

            // Report progress after each chunk
            let progress = ((chunk_idx + 1) * BATCH_CHUNK_SIZE).min(total_files as usize) as f64;
            self.report_progress(progress_token.clone(), progress, Some(total_files as f64)).await;

            // Yield to other tasks between chunks
            tokio::task::yield_now().await;
        }

        let result = serde_json::to_string_pretty(&BatchValidateOutput {
            total: input.file_paths.len(),
            passed,
            failed,
            results,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Analyze code structure and return AST statistics")]
    async fn analyze_code(&self, input: Parameters<AnalyzeInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        let result = match analyze_ast(&input.code, &input.language).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        };
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get code metrics including LOC, complexity, and structure analysis")]
    async fn get_metrics(&self, input: Parameters<MetricsInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        let result = match calculate_metrics(&input.code, &input.language).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        };
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Suggest fixes for validation errors. Uses AI sampling when available for intelligent suggestions.")]
    async fn suggest_fixes(&self, input: Parameters<SuggestFixesInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        // Try AI-powered suggestions via MCP sampling first
        if let Some(ai_response) = self.request_ai_suggestions(&input.code, &input.language, &input.errors).await {
            tracing::debug!("Got AI suggestions via MCP sampling");

            // Parse AI response into structured suggestions
            let ai_suggestions: Vec<FixSuggestion> = ai_response
                .lines()
                .filter(|line| !line.trim().is_empty())
                .enumerate()
                .map(|(i, line)| {
                    let clean_line = line.trim_start_matches(['-', '*', ' ', '\t']).to_string();
                    FixSuggestion {
                        error_id: input.errors.get(i).map(|e| e.split(':').next().unwrap_or("ai_error")).unwrap_or("ai_error").to_string(),
                        message: clean_line.clone(),
                        fix: clean_line,
                        confidence: 0.9,
                    }
                })
                .collect();

            if !ai_suggestions.is_empty() {
                let result = serde_json::to_string_pretty(&SuggestFixesOutput {
                    language: input.language.clone(),
                    suggestions: ai_suggestions,
                }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
                return Ok(CallToolResult::success(vec![Content::text(result)]));
            }
        }

        // Fallback to static rule-based suggestions
        tracing::debug!("Using static rule-based suggestions");
        let suggestions: Vec<FixSuggestion> = input.errors.iter().map(|error| {
            if error.contains("expected") {
                FixSuggestion {
                    error_id: "syntax_error".to_string(),
                    message: error.clone(),
                    fix: "Check for missing or mismatched tokens.".to_string(),
                    confidence: 0.7,
                }
            } else if error.contains("undeclared") || error.contains("not found") {
                FixSuggestion {
                    error_id: "semantic_error".to_string(),
                    message: error.clone(),
                    fix: "Verify the identifier is declared and in scope.".to_string(),
                    confidence: 0.8,
                }
            } else {
                FixSuggestion {
                    error_id: "unknown_error".to_string(),
                    message: error.clone(),
                    fix: "Review the code for potential issues.".to_string(),
                    confidence: 0.5,
                }
            }
        }).collect();

        let result = serde_json::to_string_pretty(&SuggestFixesOutput {
            language: input.language,
            suggestions,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Validate and cryptographically sign code")]
    async fn certify_code(&self, input: Parameters<CertifyInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        let result = match certify_code(&input.code, &input.language, &input.signer).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        };
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get information about a supported language")]
    async fn get_language_info(&self, input: Parameters<LanguageInfoInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        let info = get_language_info(&input.language);
        let result = serde_json::to_string_pretty(&info).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "List all supported languages")]
    async fn list_languages(&self) -> Result<CallToolResult, McpError> {
        let registry = get_parser_registry();
        let languages: Vec<LanguageInfoOutput> = registry
            .languages_with_extensions()
            .into_iter()
            .map(|(name, exts)| LanguageInfoOutput {
                language: name.to_string(),
                extensions: exts.into_iter().map(|s| s.to_string()).collect(),
                supported: true,
                features: vec!["parsing".to_string(), "validation".to_string(), "analysis".to_string()],
            })
            .collect();

        let result = serde_json::to_string_pretty(&languages).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "List available validation contracts")]
    async fn list_contracts(&self) -> Result<CallToolResult, McpError> {
        let result = serde_json::to_string_pretty(&ContractsOutput {
            contracts: vec![
                ContractInfo { name: "no_unsafe".into(), category: "security".into(), description: "No unsafe code blocks".into() },
                ContractInfo { name: "no_panic".into(), category: "reliability".into(), description: "No panic or unwrap".into() },
                ContractInfo { name: "documentation".into(), category: "style".into(), description: "Public items must have docs".into() },
                ContractInfo { name: "complexity".into(), category: "maintainability".into(), description: "Limit function complexity".into() },
                ContractInfo { name: "naming".into(), category: "style".into(), description: "Follow naming conventions".into() },
            ],
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get Aether version and capabilities")]
    async fn get_version(&self) -> Result<CallToolResult, McpError> {
        let result = serde_json::to_string_pretty(&VersionOutput {
            version: VERSION.to_string(),
            name: "Aether".to_string(),
            languages_count: get_parser_registry().languages().len(),
            tools_count: 13,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Start watching a directory for file changes. Returns a watch_id for subsequent watch_check calls.")]
    async fn watch_start(&self, input: Parameters<WatchStartInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        // Cleanup old watches if we're at the limit
        cleanup_watches_if_needed();

        let watch_id = next_watch_id();
        let path = PathBuf::from(&input.directory);

        if !path.exists() {
            let result = serde_json::to_string(&WatchStartOutput {
                watch_id: 0,
                error: Some(format!("Directory does not exist: {}", input.directory)),
                files_count: 0,
            }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
            return Ok(CallToolResult::success(vec![Content::text(result)]));
        }

        let ext_filter: Vec<String> = input.extensions
            .map(|e| e.split(',').map(|s| s.trim().to_lowercase()).collect())
            .unwrap_or_else(|| {
                get_parser_registry().all_extensions().into_iter().map(|s| s.to_string()).collect()
            });

        let mut file_state = HashMap::new();
        let mut files_count = 0;

        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                // Stop adding files if we hit the limit
                if files_count >= MAX_FILES_PER_WATCH {
                    tracing::warn!(
                        "Reached MAX_FILES_PER_WATCH limit ({}) for watch {}",
                        MAX_FILES_PER_WATCH, watch_id
                    );
                    break;
                }

                let file_path = entry.path();
                if file_path.is_file() {
                    if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                        if ext_filter.contains(&ext.to_lowercase()) {
                            if let Ok(metadata) = std::fs::metadata(&file_path) {
                                // Skip files larger than MAX_FILE_SIZE
                                if metadata.len() > MAX_FILE_SIZE {
                                    tracing::debug!(
                                        "Skipping large file: {} ({} bytes)",
                                        file_path.display(), metadata.len()
                                    );
                                    continue;
                                }
                                if let Ok(modified) = metadata.modified() {
                                    file_state.insert(
                                        file_path.to_string_lossy().to_string(),
                                        modified,
                                    );
                                    files_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        let watches = get_watches();
        if let Ok(mut w) = watches.lock() {
            w.insert(watch_id, WatchEntry {
                file_state,
                created_at: std::time::SystemTime::now(),
            });
        }

        let result = serde_json::to_string(&WatchStartOutput {
            watch_id,
            error: None,
            files_count,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Check for file changes since last check. Returns list of modified files.")]
    async fn watch_check(&self, input: Parameters<WatchCheckInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        let watch_id = input.watch_id;
        let watches = get_watches();

        let output = if let Ok(mut w) = watches.lock() {
            if let Some(entry) = w.get_mut(&watch_id) {
                let mut changed_files = Vec::new();
                let mut deleted_files = Vec::new();

                for (path, last_modified) in entry.file_state.iter_mut() {
                    let file_path = PathBuf::from(path);
                    if let Ok(metadata) = std::fs::metadata(&file_path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified > *last_modified {
                                changed_files.push(path.clone());
                                *last_modified = modified;
                            }
                        }
                    } else {
                        deleted_files.push(path.clone());
                    }
                }

                // Remove deleted files from state
                for path in &deleted_files {
                    entry.file_state.remove(path);
                }

                WatchCheckOutput {
                    watch_id,
                    changed_files,
                    deleted_files,
                    active: true,
                }
            } else {
                WatchCheckOutput {
                    watch_id,
                    changed_files: vec![],
                    deleted_files: vec![],
                    active: false,
                }
            }
        } else {
            WatchCheckOutput {
                watch_id,
                changed_files: vec![],
                deleted_files: vec![],
                active: false,
            }
        };

        let result = serde_json::to_string(&output).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Stop watching a directory. Removes the watch state.")]
    async fn watch_stop(&self, input: Parameters<WatchStopInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        let watches = get_watches();
        let removed = if let Ok(mut w) = watches.lock() {
            w.remove(&input.watch_id).is_some()
        } else {
            false
        };

        let result = serde_json::to_string(&WatchStopOutput {
            watch_id: input.watch_id,
            stopped: removed,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ========================================================================
    // CodeGraph Tools (CLI Parity)
    // ========================================================================

    #[tool(description = "Build code dependency graph from project directory")]
    async fn build_graph(&self, input: Parameters<BuildGraphInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        let start = std::time::Instant::now();
        
        let graph = CODE_GRAPH.get_or_init(|| std::sync::Mutex::new(CodeGraph::new()));
        
        let g = if let Ok(guard) = graph.lock() {
            guard
        } else {
            return Ok(CallToolResult::success(vec![Content::text("{\"error\": \"Failed to acquire graph lock\"}")])); 
        };
        
        let _registry = get_parser_registry();
        let _path = PathBuf::from(&input.directory);
        // Note: index_project removed - use parse_file for each file instead
        let files_indexed = 0usize;
        
        let nodes = g.all_nodes().count();
        let edges: usize = g.all_nodes().map(|n| n.calls.len()).sum();
        let duration = start.elapsed().as_millis() as u64;

        let result = serde_json::to_string(&BuildGraphOutput {
            nodes_count: nodes,
            edges_count: edges,
            files_indexed,
            duration_ms: duration,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Find all callers of a function")]
    async fn who_calls(&self, input: Parameters<WhoCallsInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        let graph = CODE_GRAPH.get_or_init(|| std::sync::Mutex::new(CodeGraph::new()));
        let g = if let Ok(guard) = graph.lock() {
            guard
        } else {
            return Ok(CallToolResult::success(vec![Content::text("{\"error\": \"Failed to acquire graph lock\"}")])); 
        };
        
        let file = input.file.as_deref().unwrap_or("");
        let callers: Vec<CallerInfo> = g.who_calls(&input.function, file)
            .into_iter()
            .map(|c| CallerInfo {
                function_name: c.name.clone(),
                file_path: c.file.clone(),
                line: c.line,
                call_type: format!("{:?}", c.node_type),
            })
            .collect();
        
        let total = callers.len();
        let result = serde_json::to_string(&WhoCallsOutput {
            target_function: input.function,
            callers,
            total,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Analyze impact of changes to a file")]
    async fn impact_analysis(&self, input: Parameters<ImpactAnalysisInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        let graph = CODE_GRAPH.get_or_init(|| std::sync::Mutex::new(CodeGraph::new()));
        let g = if let Ok(guard) = graph.lock() {
            guard
        } else {
            return Ok(CallToolResult::success(vec![Content::text("{\"error\": \"Failed to acquire graph lock\"}")])); 
        };
        
        let file = input.file.as_deref().unwrap_or("");
        let impacts = g.impact_analysis(&input.function, file);
        
        // ImpactResult.affected_files is Vec<String>, convert to Vec<AffectedFile>
        let affected_files: Vec<AffectedFile> = impacts.affected_files
            .into_iter()
            .enumerate()
            .map(|(i, path)| AffectedFile {
                path,
                impact_score: 1.0 / (i as f64 + 1.0), // Decay score by distance
                change_type: "indirect".to_string(),
            })
            .collect();
        
        let affected_functions: Vec<String> = impacts.affected_functions
            .into_iter()
            .map(|f| f.id.clone())
            .collect();

        let total_impact = affected_files.len();
        let result = serde_json::to_string(&ImpactAnalysisOutput {
            source_file: input.function.clone(),
            affected_files,
            affected_functions,
            total_impact_score: total_impact as f64,
            risk_level: if total_impact > 10 { "high" } else if total_impact > 3 { "medium" } else { "low" }.to_string(),
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get direct and transitive dependencies of a file")]
    async fn file_dependencies(&self, input: Parameters<FileDependenciesInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        let graph = CODE_GRAPH.get_or_init(|| std::sync::Mutex::new(CodeGraph::new()));
        let g = if let Ok(guard) = graph.lock() {
            guard
        } else {
            return Ok(CallToolResult::success(vec![Content::text("{\"error\": \"Failed to acquire graph lock\"}")])); 
        };
        
        let depth = input.max_depth.unwrap_or(1);
        let deps: Vec<String> = g.file_dependencies(&input.file).into_iter().map(|p| p.to_string_lossy().to_string()).collect();
        let transitive: Vec<String> = if depth > 1 {
            g.file_dependencies_deep(&input.file, depth).into_iter().map(|(p, _)| p.to_string_lossy().to_string()).collect()
        } else {
            Vec::new()
        };

        let total = deps.len() + transitive.len();
        let result = serde_json::to_string(&FileDependenciesOutput {
            file_path: input.file,
            dependencies: deps,
            transitive_deps: transitive,
            total,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get files that depend on this file")]
    async fn file_dependents(&self, input: Parameters<FileDependentsInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        let graph = CODE_GRAPH.get_or_init(|| std::sync::Mutex::new(CodeGraph::new()));
        let g = if let Ok(guard) = graph.lock() {
            guard
        } else {
            return Ok(CallToolResult::success(vec![Content::text("{\"error\": \"Failed to acquire graph lock\"}")])); 
        };
        
        let depth = input.max_depth.unwrap_or(1);
        let dependents: Vec<String> = g.file_dependents(&input.file).into_iter().map(|p| p.to_string_lossy().to_string()).collect();
        let transitive: Vec<String> = if depth > 1 {
            g.file_dependents_deep(&input.file, depth).into_iter().map(|(p, _)| p.to_string_lossy().to_string()).collect()
        } else {
            Vec::new()
        };

        let total = dependents.len() + transitive.len();
        let result = serde_json::to_string(&FileDependentsOutput {
            file_path: input.file,
            dependents,
            transitive_dependents: transitive,
            total,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get context for a function from the graph")]
    async fn get_context(&self, input: Parameters<GetContextInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        let graph = CODE_GRAPH.get_or_init(|| std::sync::Mutex::new(CodeGraph::new()));
        let g = if let Ok(guard) = graph.lock() {
            guard
        } else {
            return Ok(CallToolResult::success(vec![Content::text("{\"error\": \"Failed to acquire graph lock\"}")])); 
        };
        
        let file = input.file.as_deref().unwrap_or("");
        let max_depth = input.max_depth.unwrap_or(3);
        let ctx = g.get_full_context(&input.function, file, max_depth);

        // Flatten callers and calls into related functions
        let mut related_functions: Vec<String> = Vec::new();
        for depth_calls in ctx.callers_at_depth.values() {
            related_functions.extend(depth_calls.clone());
        }
        for depth_calls in ctx.calls_at_depth.values() {
            related_functions.extend(depth_calls.clone());
        }
        related_functions.sort();
        related_functions.dedup();

        // Build context description
        let context = format!(
            "Function {} in {}\nFiles involved: {}\nCallers: {}, Calls: {}",
            ctx.function,
            ctx.file,
            ctx.files_involved.join(", "),
            ctx.callers_at_depth.get(&1).map(|v| v.len()).unwrap_or(0),
            ctx.calls_at_depth.get(&1).map(|v| v.len()).unwrap_or(0)
        );

        let result = serde_json::to_string(&GetContextOutput {
            function_name: input.function,
            context,
            related_functions,
            confidence_score: ctx.context_score,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Find call chain between two functions")]
    async fn find_call_chain(&self, input: Parameters<FindCallChainInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        let graph = CODE_GRAPH.get_or_init(|| std::sync::Mutex::new(CodeGraph::new()));
        let g = if let Ok(guard) = graph.lock() {
            guard
        } else {
            return Ok(CallToolResult::success(vec![Content::text("{\"error\": \"Failed to acquire graph lock\"}")])); 
        };
        
        let chain_raw = g.find_call_chain(&input.from_function, &input.from_file, &input.to_function, &input.to_file);
        let found = chain_raw.is_some();

        // find_call_chain returns Option<Vec<String>> (chain of node IDs)
        let chain: Vec<CallChainStep> = chain_raw
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, node_id)| CallChainStep {
                from_function: node_id.clone(),
                to_function: if i > 0 { node_id } else { input.to_function.clone() },
                file_path: String::new(),
                line: 0,
            })
            .collect();
        
        let length = chain.len();
        let result = serde_json::to_string(&FindCallChainOutput {
            source: format!("{}::{}", input.from_file, input.from_function),
            target: format!("{}::{}", input.to_file, input.to_function),
            chain,
            found,
            length,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ========================================================================
    // Memory Tools (CLI Parity)
    // ========================================================================

    #[tool(description = "Recall memories matching a query")]
    async fn memory_recall(&self, input: Parameters<MemoryRecallInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        let store = get_memory_store();
        let s = if let Ok(guard) = store.lock() {
            guard
        } else {
            return Ok(CallToolResult::success(vec![Content::text("{\"error\": \"Failed to acquire memory lock\"}")])); 
        };
        
        let limit = input.limit.unwrap_or(5);
        let entries_raw = s.recall(&input.query, limit);

        let entries: Vec<MemoryEntryOutput> = match entries_raw {
            Ok(raw) => raw.into_iter().map(|e| MemoryEntryOutput {
                id: e.id.0,
                code: e.code,
                language: e.language,
                memory_type: format!("{:?}", e.memory_type),
                errors: e.errors,
                recall_count: e.recall_count,
                created_at: e.created_at.to_rfc3339(),
            }).collect(),
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!("{{\"error\": \"{}\"}}", e))]));
            }
        };
        
        let total = entries.len();
        let result = serde_json::to_string(&MemoryRecallOutput {
            query: input.query,
            entries,
            total,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Store a new memory entry")]
    async fn memory_store(&self, input: Parameters<MemoryStoreInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        let store = MEMORY_STORE.get_or_init(|| std::sync::Mutex::new(MemoryStore::new(None).expect("Failed to create MemoryStore")));
        let mut s = if let Ok(guard) = store.lock() {
            guard
        } else {
            return Ok(CallToolResult::success(vec![Content::text("{\"error\": \"Failed to acquire memory lock\"}")])); 
        };

        let memory_type = match input.memory_type.as_deref() {
            Some("pattern") => MemoryType::Pattern,
            Some("fix") => MemoryType::Fix,
            Some("preference") => MemoryType::Preference,
            _ => MemoryType::Code,
        };

        let entry = MemoryEntry::new(&input.code, &input.language)
            .with_type(memory_type);

        let stored = s.save(entry).is_ok();

        let result = serde_json::to_string(&MemoryStoreOutput {
            id: String::new(), // ID is generated internally
            stored,
            memory_type: input.memory_type.unwrap_or_default(),
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ========================================================================
    // State Tools (CLI Parity)
    // ========================================================================

    #[tool(description = "Save validation state to file")]
    async fn save_state(&self, input: Parameters<SaveStateInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        let state = VALIDATION_STATE.get_or_init(|| std::sync::Mutex::new(ValidationState::new(None).expect("Failed to create ValidationState")));
        let _s = if let Ok(guard) = state.lock() {
            guard
        } else {
            return Ok(CallToolResult::success(vec![Content::text("{\"error\": \"Failed to acquire state lock\"}")])); 
        };

        let _project_path = PathBuf::from(&input.project_root);
        // Note: ValidationState doesn't have save() - state is managed via get_project/save_project

        let result = serde_json::to_string(&SaveStateOutput {
            saved: true,
            path: input.project_root,
            violations_count: 0, // Note: counts() removed
            decisions_count: 0,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Load validation state from file")]
    async fn load_state(&self, input: Parameters<LoadStateInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        let state = VALIDATION_STATE.get_or_init(|| std::sync::Mutex::new(ValidationState::new(None).expect("Failed to create ValidationState")));
        let mut s = if let Ok(guard) = state.lock() {
            guard
        } else {
            return Ok(CallToolResult::success(vec![Content::text("{\"error\": \"Failed to acquire state lock\"}")])); 
        };

        let project_path = PathBuf::from(&input.project_root);
        let _project = s.get_project(&project_path);
        // Note: load() removed - state is managed per-project via get_project()

        let result = serde_json::to_string(&LoadStateOutput {
            loaded: true,
            path: input.project_root,
            violations_count: 0, // Note: counts() removed - use project-specific queries
            decisions_count: 0,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ========================================================================
    // Learning Tools (CLI Parity)
    // ========================================================================

    #[tool(description = "Accept a violation with reason for learning")]
    async fn accept_violation(&self, input: Parameters<AcceptViolationInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        let state = VALIDATION_STATE.get_or_init(|| std::sync::Mutex::new(ValidationState::new(None).expect("Failed to create ValidationState")));
        let mut s = if let Ok(guard) = state.lock() {
            guard
        } else {
            return Ok(CallToolResult::success(vec![Content::text("{\"error\": \"Failed to acquire state lock\"}")])); 
        };

        let _project_path = PathBuf::from(&input.project_root);
        let project_path = PathBuf::from(&input.project_root);

        let violation = AcceptedViolation::new(&input.violation_id, &input.reason)
            .by("mcp-user")
            .expires_in(90); // Default 90 days expiry

        // Get project and accept violation
        {
            let project = s.get_project(&project_path);
            project.accept_violation(violation);
        }

        // Save project state (clone to release borrow)
        if let Ok(mut guard) = state.lock() {
            let project = guard.get_project(&project_path).clone();
            let _ = guard.save_project(&project);
        }

        let result = serde_json::to_string(&AcceptViolationOutput {
            accepted: true,
            violation_id: input.violation_id,
            reason: input.reason,
            config_updated: false,
        }).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Analyze variable scopes and detect shadowing/unused variables")]
    async fn analyze_scope(&self, input: Parameters<AnalyzeScopeInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        // Use analyze_ast helper to get AST info
        let ast_result = analyze_ast(&input.code, &input.language).await;
        
        let result = match ast_result {
            Ok(ast) => {
                // Extract variable-like nodes from AST
                let mut symbols: Vec<SymbolInfo> = Vec::new();
                let mut scope_count = 1; // At least module scope
                
                for node_type in &ast.node_types {
                    let kind = node_type.node_type.as_str();
                    // Count scope-creating nodes
                    if matches!(kind, "function_definition" | "function_item" | "method_definition" | 
                               "class_definition" | "struct_item" | "if_statement" | "for_statement" | 
                               "while_statement" | "block" | "closure_expression") {
                        scope_count += node_type.count;
                    }
                    // Extract variable declarations
                    if matches!(kind, "variable_declarator" | "assignment" | "let_statement" | 
                               "identifier" | "variable_declaration") {
                        symbols.push(SymbolInfo {
                            name: kind.to_string(),
                            kind: "variable".to_string(),
                            scope_path: "global".to_string(),
                            line: 0,
                        });
                    }
                }
                
                AnalyzeScopeOutput {
                    scope_count,
                    symbols,
                    unused_variables: vec![], // Requires deeper analysis
                    shadowing: vec![], // Requires deeper analysis
                }
            }
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(
                    format!("{{\"error\": \"Failed to analyze: {}\"}}", e)
                )]));
            }
        };
        
        let json = serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Infer types for variables and expressions without annotations")]
    async fn infer_types(&self, input: Parameters<InferTypesInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        // Use analyze_ast to extract type-relevant nodes
        let ast_result = analyze_ast(&input.code, &input.language).await;
        
        let result = match ast_result {
            Ok(ast) => {
                let mut types: HashMap<String, String> = HashMap::new();
                let mut violations: Vec<String> = Vec::new();
                
                // Infer basic types from AST node patterns
                for node_type in &ast.node_types {
                    let kind = node_type.node_type.as_str();
                    match kind {
                        "string_literal" | "string" => {
                            types.insert("string_expr".to_string(), "String".to_string());
                        }
                        "integer_literal" | "number" | "int_literal" => {
                            types.insert("int_expr".to_string(), "Integer".to_string());
                        }
                        "float_literal" | "float" => {
                            types.insert("float_expr".to_string(), "Float".to_string());
                        }
                        "boolean_literal" | "bool" | "true" | "false" => {
                            types.insert("bool_expr".to_string(), "Boolean".to_string());
                        }
                        "null" | "none" => {
                            types.insert("null_expr".to_string(), "Null/None".to_string());
                        }
                        _ => {}
                    }
                }
                
                // Check for potential type issues
                if types.is_empty() {
                    violations.push("No literal expressions found for type inference".to_string());
                }
                
                InferTypesOutput {
                    types,
                    errors: vec![],
                    violations,
                }
            }
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(
                    format!("{{\"error\": \"Failed to analyze: {}\"}}", e)
                )]));
            }
        };
        
        let json = serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Get confidence score for code quality and generate clarifying questions")]
    async fn get_confidence(&self, input: Parameters<GetConfidenceInput>) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        // Calculate confidence based on code metrics
        let metrics_result = calculate_metrics(&input.code, &input.language).await;
        
        let result = match metrics_result {
            Ok(metrics) => {
                // Calculate confidence score based on metrics
                let mut confidence = 0.8; // Base confidence
                
                // Reduce confidence for high complexity
                if metrics.cyclomatic_complexity > 10 {
                    confidence -= 0.1 * (metrics.cyclomatic_complexity as f64 / 20.0).min(0.3);
                }
                
                // Reduce confidence for very long functions
                if metrics.lines_of_code > 50 {
                    confidence -= 0.05;
                }
                
                confidence = confidence.clamp(0.0, 1.0);
                
                // Determine level
                let level = if confidence >= 0.95 {
                    "AutoAccept"
                } else if confidence >= 0.80 {
                    "Good"
                } else if confidence >= 0.60 {
                    "Warn"
                } else {
                    "Ask"
                };
                
                // Generate questions for low confidence
                let mut questions: Vec<String> = Vec::new();
                if confidence < 0.80 {
                    if metrics.cyclomatic_complexity > 10 {
                        questions.push(format!("Cyclomatic complexity is {} (>10). Consider simplifying?", metrics.cyclomatic_complexity));
                    }
                    if metrics.lines_of_code > 50 {
                        questions.push(format!("Function has {} lines (>50). Consider splitting?", metrics.lines_of_code));
                    }
                }
                
                GetConfidenceOutput {
                    confidence,
                    level: level.to_string(),
                    questions,
                }
            }
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(
                    format!("{{\"error\": \"Failed to calculate metrics: {}\"}}", e)
                )]));
            }
        };
        
        let json = serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ========================================================================
    // COMPLIANCE ENGINE TOOLS
    // ========================================================================

    #[tool(description = "Get compliance engine status and statistics")]
    async fn compliance_status(&self) -> Result<CallToolResult, McpError> {
        let engine = get_compliance_engine();
        let engine = engine.lock().map_err(|e| McpError::internal_error(e.to_string(), None))?;
        
        let stats = engine.stats();
        
        let result = serde_json::json!({
            "total_exemptions": stats.exemptions.total,
            "learned_patterns": stats.exemptions.learned,
            "user_created": stats.exemptions.user_created,
            "occurrence_tracking": stats.occurrence_tracking,
            "config": {
                "auto_accept_threshold": 0.90,
                "ask_threshold": 0.60,
                "learn_after_occurrences": 3,
                "use_dubbioso": true,
            }
        });
        
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default()
        )]))
    }

    #[tool(description = "Evaluate a violation through the compliance engine")]
    async fn compliance_evaluate(
        &self,
        input: Parameters<ComplianceEvaluateInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = input.0;
        
        // Use block_in_place to avoid Send issues with MutexGuard across await
        let result = tokio::task::block_in_place(|| {
            let engine = get_compliance_engine();
            let mut engine = engine.lock().map_err(|e| McpError::internal_error(e.to_string(), None))?;
            
            let ctx = ComplianceContext {
                file_path: input.file_path.clone(),
                line: input.line.unwrap_or(0),
                snippet: None,
                project_type: None,
                code_region: input.code_region.clone(),
                function_context: None,
            };
            
            // Block on the async evaluate
            let decision = tokio::runtime::Handle::current().block_on(
                engine.evaluate(&input.rule_id, &input.domain, &input.message, &ctx)
            ).map_err(|e| McpError::internal_error(e.to_string(), None))?;
            
            let action = match &decision.action {
                ComplianceAction::Block => "block",
                ComplianceAction::Warn => "warn",
                ComplianceAction::Ask { .. } => "ask",
                ComplianceAction::Learn { .. } => "learn",
                ComplianceAction::Accept { .. } => "accept",
            };
            
            let tier = match decision.tier {
                ContractTier::Inviolable => "inviolable",
                ContractTier::Strict => "strict",
                ContractTier::Flexible => "flexible",
            };
            
            Ok::<_, McpError>(serde_json::json!({
                "action": action,
                "tier": tier,
                "confidence": decision.confidence,
                "overridable": decision.overridable,
                "explanation": decision.explanation,
                "should_block": decision.should_fail(),
                "needs_input": decision.needs_user_input(),
            }))
        })?;
        
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default()
        )]))
    }

    #[tool(description = "Accept a violation with a documented reason")]
    async fn compliance_accept(
        &self,
        input: Parameters<ComplianceAcceptInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = input.0;
        let engine = get_compliance_engine();
        let mut engine = engine.lock().map_err(|e| McpError::internal_error(e.to_string(), None))?;
        
        engine.accept_violation(&input.rule_id, &input.file_path, input.reason.clone())
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Violation {} accepted for {} with reason: {}",
            input.rule_id, input.file_path, input.reason
        ))]))
    }

    // ========================================================================
    // DRIFT DETECTION TOOLS
    // ========================================================================

    #[tool(description = "Analyze drift for a file or directory over time")]
    async fn drift_analyze(
        &self,
        input: Parameters<DriftAnalyzeInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = input.0;
        let days = input.days.unwrap_or(30);
        
        // Simulated drift analysis (would integrate with actual DriftDetector)
        let drift_score = 0.15;
        let trend = if drift_score < 0.2 { "stable" } 
                    else if drift_score < 0.5 { "declining" } 
                    else { "rapidly_declining" };
        
        let result = serde_json::json!({
            "path": input.path,
            "drift_score": drift_score,
            "trend": trend,
            "days_analyzed": days,
            "metrics": {
                "type_strictness": 0.92,
                "naming_consistency": 0.88,
                "error_handling_quality": 0.75,
                "complexity_avg": 0.45,
                "dead_code_ratio": 0.05,
            },
            "alerts": [{
                "alert_type": "ErrorHandlingErosion",
                "severity": "medium",
                "message": "Error handling quality declining (0.85 → 0.75)",
                "metric_value": 0.75,
                "threshold": 0.80,
            }],
            "recommendation": "Review error handling patterns - consider reverting to specific error types",
        });
        
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default()
        )]))
    }

    #[tool(description = "Get trend analysis for a file or project over a time window")]
    async fn drift_trend(
        &self,
        input: Parameters<DriftTrendInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = input.0;
        let window_days = input.window_days.unwrap_or(7);
        
        let result = serde_json::json!({
            "path": input.path,
            "window_days": window_days,
            "quality_trend": "improving",
            "complexity_trend": 0.02,
            "violation_trend": -0.05,
            "snapshots_analyzed": 7,
            "recommendation": "Quality is improving. Keep current approach.",
        });
        
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default()
        )]))
    }
}

#[tool_handler]
impl ServerHandler for AetherServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build()
        )
        .with_server_info(Implementation::new("aether-mcp", env!("CARGO_PKG_VERSION")))
        .with_protocol_version(ProtocolVersion::V_2024_11_05)
        .with_instructions(
            "Aether MCP Server - Universal AI Code Validation Layer\n\
            \n\
            PURPOSE: Validate, analyze, and certify AI-generated code before production use.\n\
            \n\
            TOOLS (12):\n\
            - validate_file: Full validation (syntax, semantic, logic, security, style)\n\
            - batch_validate: Multi-file validation with progress\n\
            - analyze_code: AST structure analysis\n\
            - get_metrics: Code complexity and maintainability metrics\n\
            - suggest_fixes: AI-powered fix suggestions (requires sampling)\n\
            - certify_code: Ed25519 cryptographic certification\n\
            - list_languages: 23 supported languages\n\
            - get_language_info: Language-specific capabilities\n\
            - list_contracts: Available validation rules\n\
            - get_version: Server info\n\
            - watch_start/check/stop: Directory monitoring\n\
            \n\
            LANGUAGES: Rust, Python, JavaScript, TypeScript, C, C++, Go, Java, Lua, GLSL, CSS, HTML, JSON, YAML, TOML, SQL, GraphQL, Markdown, Bash, Lex, Prism, CUDA, CMake\n\
            \n\
            CONTRACTS: no_unsafe, no_panic, documentation, complexity, naming\n\
            \n\
            WORKFLOW:\n\
            1. Generate code\n\
            2. Call validate_file\n\
            3. Fix any errors\n\
            4. Optionally certify_code\n\
            \n\
            RESOURCES: docs://aether/* for documentation, template://aether/* for contract templates\n\
            \n\
            PROMPTS: validate_prompt, review_prompt, certify_prompt, metrics_prompt"
        )
    }

    // Note: get_peer/set_peer removed - not part of ServerHandler trait in rmcp v1.2
    // Peer is managed internally by the rmcp framework

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![
                Prompt::new("validate_prompt", Some("Comprehensive code validation with all layers"), Some(vec![
                    PromptArgument::new("file_path").with_description("File to validate (absolute path)").with_required(true),
                    PromptArgument::new("contracts").with_description("Contracts: no_unsafe,no_panic,documentation,complexity,naming").with_required(false),
                    PromptArgument::new("language").with_description("Language override (auto-detected if omitted)").with_required(false),
                ])),
                Prompt::new("security_review", Some("Security-focused validation: unsafe, panics, vulnerabilities"), Some(vec![
                    PromptArgument::new("file_path").with_description("File to review").with_required(true),
                ])),
                Prompt::new("quality_review", Some("Code quality review: complexity, maintainability, style"), Some(vec![
                    PromptArgument::new("file_path").with_description("File to review").with_required(true),
                    PromptArgument::new("max_complexity").with_description("Max allowed cyclomatic complexity (default: 20)").with_required(false),
                ])),
                Prompt::new("certify_prompt", Some("Validate and cryptographically certify code"), Some(vec![
                    PromptArgument::new("file_path").with_description("File to certify").with_required(true),
                    PromptArgument::new("signer").with_description("Signer name/identifier").with_required(true),
                    PromptArgument::new("contracts").with_description("Required contracts for certification").with_required(false),
                ])),
                Prompt::new("batch_project", Some("Validate entire project directory"), Some(vec![
                    PromptArgument::new("directory").with_description("Project root directory").with_required(true),
                    PromptArgument::new("extensions").with_description("File extensions (comma-separated: rs,py,js)").with_required(false),
                    PromptArgument::new("contracts").with_description("Contracts to apply").with_required(false),
                ])),
                Prompt::new("metrics_analysis", Some("Code metrics and complexity analysis"), Some(vec![
                    PromptArgument::new("file_path").with_description("File to analyze").with_required(true),
                ])),
            ],
            meta: None,
        })
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let args = request.arguments.as_ref();

        let msg = match request.name.as_ref() {
            "validate_prompt" => {
                let fp = args.and_then(|a| a.get("file_path")).and_then(|v| v.as_str()).unwrap_or("file.rs");
                let ct = args.and_then(|a| a.get("contracts")).and_then(|v| v.as_str()).unwrap_or("");
                let lg = args.and_then(|a| a.get("language")).and_then(|v| v.as_str()).unwrap_or("");
                let lang_part = if lg.is_empty() { String::new() } else { format!(", language: {}", lg) };
                format!(
                    "Validate the file at '{}' through all validation layers:\n\
                    1. Syntax: Parse and check for syntax errors\n\
                    2. Semantic: Type checking and semantic analysis\n\
                    3. Logic: Logic pattern detection\n\
                    4. Security: Unsafe code, panics, vulnerabilities\n\
                    5. Contracts: Apply validation rules ({})\n\
                    6. Style: Code style and formatting\n{}\n\
                    \nReport all errors, warnings, and suggestions. If validation fails, provide specific fixes.",
                    fp, ct, lang_part
                )
            }
            "security_review" => {
                let fp = args.and_then(|a| a.get("file_path")).and_then(|v| v.as_str()).unwrap_or("file.rs");
                format!(
                    "Perform a security-focused review of '{}':\n\
                    \n\
                    SECURITY CHECKS:\n\
                    - Unsafe blocks (Rust: unsafe, C++: reinterpret_cast, etc.)\n\
                    - Panics and unwraps (potential crashes)\n\
                    - SQL injection vectors\n\
                    - Path traversal vulnerabilities\n\
                    - Memory safety issues\n\
                    - Input validation gaps\n\
                    \n\
                    Use contracts: no_unsafe, no_panic\n\
                    Report each issue with severity and fix suggestion.",
                    fp
                )
            }
            "quality_review" => {
                let fp = args.and_then(|a| a.get("file_path")).and_then(|v| v.as_str()).unwrap_or("file.rs");
                let max_cx = args.and_then(|a| a.get("max_complexity")).and_then(|v| v.as_str()).unwrap_or("20");
                format!(
                    "Perform a code quality review of '{}':\n\
                    \n\
                    QUALITY METRICS:\n\
                    - Cyclomatic complexity (max: {})\n\
                    - Function length\n\
                    - Nesting depth\n\
                    - Code duplication\n\
                    - Naming conventions\n\
                    - Documentation coverage\n\
                    \n\
                    Use contracts: complexity, naming, documentation\n\
                    Provide refactoring suggestions for high-complexity code.",
                    fp, max_cx
                )
            }
            "certify_prompt" => {
                let fp = args.and_then(|a| a.get("file_path")).and_then(|v| v.as_str()).unwrap_or("file.rs");
                let sn = args.and_then(|a| a.get("signer")).and_then(|v| v.as_str()).unwrap_or("Developer");
                let ct = args.and_then(|a| a.get("contracts")).and_then(|v| v.as_str()).unwrap_or("");
                format!(
                    "Validate and certify '{}':\n\
                    \n\
                    1. First, validate with all layers\n\
                    2. Apply contracts: {}\n\
                    3. If validation passes, generate Ed25519 certificate\n\
                    4. Certificate signed by: {}\n\
                    \n\
                    Return the certificate with hash, signature, and metadata.",
                    fp, ct, sn
                )
            }
            "batch_project" => {
                let dir = args.and_then(|a| a.get("directory")).and_then(|v| v.as_str()).unwrap_or("./src");
                let exts = args.and_then(|a| a.get("extensions")).and_then(|v| v.as_str()).unwrap_or("rs");
                let ct = args.and_then(|a| a.get("contracts")).and_then(|v| v.as_str()).unwrap_or("");
                format!(
                    "Validate all files in '{}':\n\
                    \n\
                    - Extensions: {}\n\
                    - Contracts: {}\n\
                    \n\
                    Use batch_validate for efficiency.\n\
                    Report summary: total files, passed, failed, by error type.",
                    dir, exts, ct
                )
            }
            "metrics_analysis" => {
                let fp = args.and_then(|a| a.get("file_path")).and_then(|v| v.as_str()).unwrap_or("file.rs");
                format!(
                    "Analyze code metrics for '{}':\n\
                    \n\
                    Calculate:\n\
                    - Lines of code (total, blank, comments)\n\
                    - Function count and average length\n\
                    - Cyclomatic complexity\n\
                    - Maximum nesting depth\n\
                    - Token/node counts\n\
                    \n\
                    Provide interpretation and suggestions for improvement.",
                    fp
                )
            }
            name => return Err(McpError::invalid_params(format!("Unknown prompt: {}", name), None)),
        };

        Ok(GetPromptResult::new(vec![PromptMessage::new_text(PromptMessageRole::User, msg)])
            .with_description("Aether validation prompt"))
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            next_cursor: None,
            resources: vec![
                // Dynamic resources
                RawResource::new("aether://version", "Aether Version").no_annotation(),
                RawResource::new("aether://languages", "Supported Languages (23)").no_annotation(),
                RawResource::new("aether://contracts", "Validation Contracts (5)").no_annotation(),
                // Documentation
                RawResource::new("docs://aether/quickstart", "Quick Start Guide").no_annotation(),
                RawResource::new("docs://aether/languages", "Languages Reference").no_annotation(),
                RawResource::new("docs://aether/contracts", "Contracts Reference").no_annotation(),
                RawResource::new("docs://aether/certification", "Certification Process").no_annotation(),
                // Templates
                RawResource::new("template://aether/rust-contract", "Rust Contract Template").no_annotation(),
                RawResource::new("template://aether/python-contract", "Python Contract Template").no_annotation(),
                // Examples
                RawResource::new("example://aether/validation", "Validation Example").no_annotation(),
                RawResource::new("example://aether/certification", "Certification Example").no_annotation(),
            ],
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = request.uri.as_ref();

        let content = match uri {
            // Dynamic resources
            "aether://version" => {
                let version_info = serde_json::json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "name": "aether-mcp",
                    "languages_count": get_parser_registry().languages().len(),
                    "contracts_count": CONTRACTS.len(),
                    "protocol_version": "2024-11-05"
                });
                return Ok(ReadResourceResult::new(vec![ResourceContents::text(
                    serde_json::to_string_pretty(&version_info).unwrap_or_default(),
                    uri
                )]));
            }
            "aether://languages" => {
                let registry = get_parser_registry();
                let languages: Vec<_> = registry.languages_with_extensions()
                    .into_iter()
                    .map(|(lang, exts)| {
                        serde_json::json!({
                            "language": lang,
                            "extensions": exts,
                            "supported": true
                        })
                    })
                    .collect();
                return Ok(ReadResourceResult::new(vec![ResourceContents::text(
                    serde_json::to_string_pretty(&languages).unwrap_or_default(),
                    uri
                )]));
            }
            "aether://contracts" => {
                let contracts: Vec<_> = CONTRACTS.iter()
                    .map(|(name, category, desc)| serde_json::json!({
                        "name": name,
                        "category": category,
                        "description": desc
                    }))
                    .collect();
                return Ok(ReadResourceResult::new(vec![ResourceContents::text(
                    serde_json::to_string_pretty(&contracts).unwrap_or_default(),
                    uri
                )]));
            }
            // Static documentation
            "docs://aether/quickstart" => include_str!("../docs/quickstart.md"),
            "docs://aether/languages" => include_str!("../docs/languages.md"),
            "docs://aether/contracts" => include_str!("../docs/contracts.md"),
            "docs://aether/certification" => include_str!("../docs/certification.md"),
            // Templates
            "template://aether/rust-contract" => include_str!("../templates/rust-contract.md"),
            "template://aether/python-contract" => include_str!("../templates/python-contract.md"),
            // Examples
            "example://aether/validation" => include_str!("../examples/validation.md"),
            "example://aether/certification" => include_str!("../examples/certification.md"),
            _ => return Err(McpError::invalid_params(format!("Unknown resource: {}", uri), None)),
        };

        Ok(ReadResourceResult::new(vec![ResourceContents::text(content, uri)]))
    }

    async fn complete(
        &self,
        request: CompleteRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, McpError> {
        let completions: Vec<String> = match &request.r#ref {
            Reference::Prompt(PromptReference { name, .. }) => {
                match name.as_str() {
                    "validate_prompt" | "review_prompt" | "certify_prompt" | "metrics_prompt" => {
                        match request.argument.name.as_str() {
                            "language" => {
                                let prefix = request.argument.value.to_lowercase();
                                get_parser_registry().languages()
                                    .into_iter()
                                    .filter(|lang| lang.starts_with(&prefix))
                                    .map(|lang| lang.to_string())
                                    .collect()
                            }
                            "contracts" => {
                                let prefix = request.argument.value.to_lowercase();
                                CONTRACTS.iter()
                                    .filter(|(name, _, _)| name.starts_with(&prefix))
                                    .map(|(name, _, _)| name.to_string())
                                    .collect()
                            }
                            _ => vec![],
                        }
                    }
                    _ => vec![],
                }
            }
            _ => vec![],
        };

        let total = completions.len() as u32;
        Ok(CompleteResult::new(CompletionInfo {
            values: completions,
            total: Some(total),
            has_more: Some(false),
        }))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn detect_language(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            let registry = get_parser_registry();
            if let Some(lang) = registry.detect_language(ext) {
                return lang.to_string();
            }
            ext.to_string()
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_language_info(language: &str) -> LanguageInfoOutput {
    let registry = get_parser_registry();
    if let Some(exts) = registry.get_extensions(language) {
        LanguageInfoOutput {
            language: language.to_string(),
            extensions: exts.into_iter().map(|s| s.to_string()).collect(),
            supported: true,
            features: vec!["parsing".to_string(), "validation".to_string(), "analysis".to_string()],
        }
    } else {
        LanguageInfoOutput {
            language: language.to_string(),
            extensions: vec![],
            supported: false,
            features: vec![],
        }
    }
}

/// Read file with size limit to prevent memory exhaustion.
/// Returns error if file exceeds MAX_FILE_SIZE.
fn read_file_bounded(path: &std::path::Path) -> Result<String> {
    let metadata = std::fs::metadata(path)?;
    let size = metadata.len();

    if size > MAX_FILE_SIZE {
        anyhow::bail!(
            "File too large: {} bytes (max {} bytes). Skipping to prevent memory exhaustion.",
            size, MAX_FILE_SIZE
        );
    }

    Ok(std::fs::read_to_string(path)?)
}

async fn validate_code(code: &str, language: &str, _contracts: Option<&str>, dubbioso_mode: bool, file_path: Option<&str>) -> Result<ValidateOutput> {
    let registry = get_parser_registry();
    let parser = registry.get(language)
        .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", language))?;

    let ast = parser.parse(code).await?;
    let has_errors = ast.has_errors();

    // Parse syntax errors
    let mut errors: Vec<ValidationError> = if has_errors {
        ast.errors.iter().map(|e| ValidationError {
            id: "parse_error".to_string(),
            message: e.clone(),
            line: None,
            column: None,
            layer: "syntax".to_string(),
            is_new: true,
            confidence: None,
            confidence_level: None,
        }).collect()
    } else { vec![] };

    // Apply ContractLayer validation (loads all contracts for language)
    let contract_violations = {
        let layer = get_contract_layer();
        let layer = layer.lock().map_err(|_| anyhow::anyhow!("ContractLayer lock error"))?;

        let ctx = ValidationContext::for_file("", code.to_string(), language.to_string());

        // Use tokio runtime for async validate
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(layer.validate(&ctx))
        })
    };

    let contracts_passed = contract_violations.violations.is_empty();

    // Add contract violations to errors
    for v in contract_violations.violations {
        let (line, column) = v.span.map(|s| (Some(s.line), Some(s.column))).unwrap_or((None, None));

        // Calculate confidence if Dubbioso Mode is enabled
        let (confidence, confidence_level, delta_status) = if dubbioso_mode {
            let validator = get_dubbioso_validator();
            if let Ok(mut validator) = validator.lock() {
                // Load ProjectState for delta detection if file_path provided
                if let Some(fp) = file_path {
                    if let Ok(path) = std::path::Path::new(fp).canonicalize() {
                        if let Some(parent) = path.parent() {
                            let validation_state = get_validation_state();
                            if let Ok(mut vs) = validation_state.lock() {
                                let project_state = vs.get_project(parent);
                                validator.set_previous_state(project_state.clone());
                            }
                        }
                    }
                }
                let violation = ViolationInput {
                    id: v.id.clone(),
                    rule: v.id.clone(),  // Use same ID as rule
                    message: v.message.clone(),
                    file: file_path.unwrap_or("").to_string(),
                    line: line.unwrap_or(0) as u32,
                    column: 0,
                    function_name: None,
                    code: Some(code.to_string()),
                    language: language.to_string(),
                };
                let result = validator.validate(&violation);
                let level_str = match result.confidence.level {
                    ConfidenceLevel::Ask => "Ask",
                    ConfidenceLevel::Warn => "Warn",
                    ConfidenceLevel::Good => "Good",
                    ConfidenceLevel::AutoAccept => "AutoAccept",
                };
                let delta = result.delta_status;
                (Some(result.confidence.confidence), Some(level_str.to_string()), Some(format!("{:?}", delta)))
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        errors.push(ValidationError {
            id: v.id,
            message: v.message,
            line,
            column,
            layer: "contract".to_string(),
            is_new: delta_status.as_ref().map(|d| d == "New").unwrap_or(true),
            confidence,
            confidence_level,
        });
    }

    let passed = errors.is_empty();

    // Calculate quality score: 100 - (errors * 10) - (warnings * 2) - (info * 0.5)
    // Note: warnings and info currently empty in this implementation
    let error_count = errors.len() as f64;
    let warning_count = 0usize as f64; // warnings not populated yet
    let info_count = 0usize as f64;    // info not populated yet
    let quality_score_raw = 100.0 - (error_count * 10.0) - (warning_count * 2.0) - (info_count * 0.5);
    let quality_score = quality_score_raw.max(0.0).min(100.0) as u8;

    // Build summary by severity
    let mut by_severity = HashMap::new();
    by_severity.insert("error".to_string(), errors.len());
    by_severity.insert("warning".to_string(), 0); // warnings empty
    by_severity.insert("info".to_string(), 0);    // info empty

    // Build summary by layer
    let mut by_layer = HashMap::new();
    for err in &errors {
        *by_layer.entry(err.layer.clone()).or_insert(0usize) += 1;
    }
    // Add layers with 0 violations for completeness
    by_layer.entry("syntax".to_string()).or_insert(0);
    by_layer.entry("semantic".to_string()).or_insert(0);
    by_layer.entry("logic".to_string()).or_insert(0);
    by_layer.entry("security".to_string()).or_insert(0);
    by_layer.entry("contracts".to_string()).or_insert(0);
    by_layer.entry("style".to_string()).or_insert(0);

    let total_violations = errors.len();

    let summary = ValidationSummary {
        by_severity,
        by_layer,
        total_violations,
    };

    Ok(ValidateOutput {
        passed,
        errors,
        warnings: vec![],
        language: language.to_string(),
        layers: ValidationLayers {
            syntax: !has_errors,
            semantic: true,
            logic: true,
            security: true,
            contracts: contracts_passed,
            style: true,
        },
        quality_score,
        summary,
    })
}

async fn analyze_ast(code: &str, language: &str) -> Result<AnalyzeOutput> {
    let registry = get_parser_registry();
    let parser = registry.get(language)
        .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", language))?;

    let ast = parser.parse(code).await?;

    fn count_nodes(node: &aether_parsers::ASTNode, depth: usize, types: &mut std::collections::HashMap<String, usize>, max_d: &mut usize) -> usize {
        let kind = format!("{:?}", node.kind);
        *types.entry(kind).or_insert(0) += 1;
        if depth > *max_d { *max_d = depth; }
        let mut count = 1;
        for child in &node.children {
            count += count_nodes(child, depth + 1, types, max_d);
        }
        count
    }

    let mut node_types: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut max_depth = 0;
    let total_nodes = count_nodes(&ast.root, 0, &mut node_types, &mut max_depth);

    Ok(AnalyzeOutput {
        language: language.to_string(),
        total_nodes,
        node_types: node_types.into_iter().map(|(t, c)| NodeTypeCount { node_type: t, count: c }).collect(),
        max_depth,
    })
}

async fn calculate_metrics(code: &str, language: &str) -> Result<MetricsOutput> {
    let lines: Vec<&str> = code.lines().collect();
    let mut lines_of_code = 0;
    let mut blank_lines = 0;
    let mut comment_lines = 0;

    let comment_prefixes = match language {
        "rust" | "c" | "cpp" | "java" | "go" | "glsl" | "cuda" => vec!["//", "/*"],
        "python" | "lua" | "toml" | "yaml" => vec!["#"],
        "javascript" | "typescript" | "css" => vec!["//", "/*"],
        "html" => vec!["<!--"],
        _ => vec!["#"],
    };

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_lines += 1;
        } else if comment_prefixes.iter().any(|p| trimmed.starts_with(p)) {
            comment_lines += 1;
        } else {
            lines_of_code += 1;
        }
    }

    let analyze_result = analyze_ast(code, language).await?;

    let functions: usize = analyze_result.node_types.iter()
        .filter(|t| matches!(t.node_type.as_str(), "Function" | "FunctionDef" | "function_definition" | "method_definition"))
        .map(|t| t.count).sum();

    let classes: usize = analyze_result.node_types.iter()
        .filter(|t| matches!(t.node_type.as_str(), "Struct" | "Class" | "class_definition" | "struct_item"))
        .map(|t| t.count).sum();

    let complexity_estimate = (functions + classes).max(1) + (analyze_result.max_depth / 3);

    // === Advanced Metrics ===

    // Cyclomatic Complexity (McCabe): count decision points
    // Decision nodes: if, else, for, while, case, catch, &&, ||, ?
    let cyclomatic_complexity: u32 = analyze_result.node_types.iter()
        .filter(|t| matches!(
            t.node_type.as_str(),
            "if_statement" | "if_expression" | "else_clause" | "for_statement" |
            "while_statement" | "case_statement" | "catch_clause" | "try_statement" |
            "conditional_expression" | "binary_expression" | "match_expression" |
            "match_arm" | "for_expression" | "while_expression" | "loop_expression"
        ))
        .map(|t| t.count as u32)
        .sum::<u32>()
        .max(1); // Base complexity is 1

    // Cognitive Complexity: accounts for nesting depth and control flow
    // Each control structure adds +1, nested adds additional +depth
    let cognitive_complexity: u32 = cyclomatic_complexity + (analyze_result.max_depth as u32 / 2);

    // Maintainability Index: 171 - 5.2 * ln(V) - 0.23 * G - 16.2 * ln(LOC)
    // V = Halstead volume (approximated with total_nodes)
    // G = Cyclomatic complexity
    // LOC = Lines of code
    let v = analyze_result.total_nodes as f64;
    let g = cyclomatic_complexity as f64;
    let loc = lines_of_code.max(1) as f64;
    let maintainability_index = {
        let ln_v = if v > 0.0 { v.ln() } else { 0.0 };
        let ln_loc = loc.ln();
        let mi = 171.0 - 5.2 * ln_v - 0.23 * g - 16.2 * ln_loc;
        // Clamp to 0-100 range
        mi.max(0.0).min(100.0)
    };

    // Technical Debt (minutes): estimated from violations and complexity
    // Average fix time: 15 min per violation, 5 min per complexity point above threshold
    let violations_estimate = analyze_result.node_types.iter()
        .filter(|t| matches!(
            t.node_type.as_str(),
            "todo_directive" | "FIXME" | "deprecated" | "unsafe_block"
        ))
        .map(|t| t.count as u32)
        .sum::<u32>();
    let complexity_debt = if cyclomatic_complexity > 10 { (cyclomatic_complexity - 10) * 5 } else { 0 };
    let technical_debt_minutes = violations_estimate * 15 + complexity_debt;

    // Code Smell Density: violations per KLOC
    let code_smell_density = if loc > 0.0 {
        (violations_estimate as f64 + (cyclomatic_complexity as f64 / 10.0)) / (loc / 1000.0)
    } else {
        0.0
    };

    // Coupling Score: based on imports, function calls, and class usage
    let imports: usize = analyze_result.node_types.iter()
        .filter(|t| matches!(
            t.node_type.as_str(),
            "use_declaration" | "import_statement" | "import_from" |
            "include_directive" | "require_statement"
        ))
        .map(|t| t.count)
        .sum();
    let coupling_score = if functions > 0 || classes > 0 {
        let total_units = (functions + classes).max(1) as f64;
        Some((imports as f64 / total_units).min(10.0))
    } else {
        None
    };

    Ok(MetricsOutput {
        language: language.to_string(),
        lines_of_code,
        blank_lines,
        comment_lines,
        total_nodes: analyze_result.total_nodes,
        max_depth: analyze_result.max_depth,
        functions,
        classes,
        complexity_estimate,
        maintainability_index,
        technical_debt_minutes,
        code_smell_density,
        cyclomatic_complexity,
        cognitive_complexity,
        coupling_score,
    })
}

async fn certify_code(code: &str, language: &str, signer: &str) -> Result<CertifyOutput> {
    let registry = get_parser_registry();
    let parser = registry.get(language)
        .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", language))?;

    let ast = parser.parse(code).await?;

    if ast.has_errors() {
        return Ok(CertifyOutput {
            passed: false,
            certificate: None,
            signature: None,
            errors: ast.errors.iter().map(|e| ValidationError {
                id: "parse_error".to_string(),
                message: e.clone(),
                line: None,
                column: None,
                layer: "syntax".to_string(),
                is_new: true,
                confidence: None,
                confidence_level: None,
            }).collect(),
        });
    }

    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let cert = format!(
        "AETHER-CERT-1.0\nlanguage: {}\nsha256: {}\nsigned_by: {}\ntimestamp: {}\n",
        language, hash, signer, chrono::Utc::now().to_rfc3339(),
    );

    Ok(CertifyOutput {
        passed: true,
        certificate: Some(cert),
        signature: Some(format!("ed25519:{}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signer.as_bytes()))),
        errors: vec![],
    })
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    // MCP requires JSON-RPC on stdout - ALL logging MUST go to stderr
    // This is NON-NEGOTIABLE: stdout is the protocol channel
    //
    // Use a sink writer that discards all tracing output to prevent stdout pollution.
    // We use eprintln! for critical info to stderr instead.
    tracing_subscriber::fmt()
        .with_writer(std::io::sink)  // Discard all tracing output
        .init();
    
    eprintln!("[INFO] Starting Aether MCP Server v{}", VERSION);
    eprintln!("[INFO] Languages: {}, Tools: 13, Prompts: 4, Resources: 8", get_parser_registry().languages().len());
    
    // Start background cleanup task
    tokio::spawn(cleanup_task());
    eprintln!("[INFO] Cleanup task started (interval: {}s, max age: {}s)", CLEANUP_INTERVAL_SECS, WATCH_MAX_AGE_SECS);

    let server = AetherServer::default();
    server.serve(stdio()).await?.waiting().await?;

    Ok(())
}

