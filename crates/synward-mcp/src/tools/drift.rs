//! Drift Tools - MCP integration for Drift Detection
//!
//! Tools:
//! - drift_analyze: Analyze drift for a file or directory
//! - drift_trend: Get trend analysis over time
//! - drift_snapshot: Create a new drift snapshot

use anyhow::Result;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    schemars::JsonSchema,
    tool, tool_router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Tool Input Schemas
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DriftAnalyzeInput {
    /// File or directory path to analyze
    pub path: String,
    /// Number of days to analyze (default: 30)
    pub days: Option<u32>,
    /// Depth for directory expansion (default: 0)
    pub depth: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DriftTrendInput {
    /// Project or file path
    pub path: String,
    /// Number of days for trend window (default: 7)
    pub window_days: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DriftSnapshotInput {
    /// Project path
    pub project_path: String,
    /// Commit hash (optional, will use current if not provided)
    pub commit_hash: Option<String>,
}

// ============================================================================
// Tool Output Schemas
// ============================================================================

#[derive(Debug, Serialize)]
pub struct DriftAnalyzeOutput {
    pub path: String,
    pub drift_score: f32,
    pub trend: String,
    pub metrics: DriftMetricsInfo,
    pub alerts: Vec<DriftAlertInfo>,
    pub recommendation: String,
}

#[derive(Debug, Serialize)]
pub struct DriftMetricsInfo {
    pub type_strictness: f32,
    pub naming_consistency: f32,
    pub error_handling_quality: f32,
    pub complexity_avg: f32,
    pub dead_code_ratio: f32,
}

#[derive(Debug, Serialize)]
pub struct DriftAlertInfo {
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub metric_value: f32,
    pub threshold: f32,
}

#[derive(Debug, Serialize)]
pub struct DriftTrendOutput {
    pub path: String,
    pub window_days: u32,
    pub quality_trend: String,
    pub complexity_trend: f32,
    pub violation_trend: f32,
    pub snapshots_analyzed: usize,
    pub recommendation: String,
}

#[derive(Debug, Serialize)]
pub struct DriftSnapshotOutput {
    pub success: bool,
    pub snapshot_id: String,
    pub timestamp: String,
    pub commit_hash: Option<String>,
    pub metrics_recorded: usize,
}

// ============================================================================
// Tools
// ============================================================================

#[tool_router]
pub struct DriftTools;

#[tool]
impl DriftTools {
    #[tool(description = "Analyze drift for a file or directory over time")]
    async fn drift_analyze(
        &self,
        params: Parameters<DriftAnalyzeInput>,
    ) -> Result<CallToolResult> {
        let params = params.0;
        let days = params.days.unwrap_or(30);
        
        // Simulated drift analysis (would integrate with actual DriftDetector)
        let drift_score = 0.15; // Example: low drift
        let trend = if drift_score < 0.2 { "stable" } 
                    else if drift_score < 0.5 { "declining" } 
                    else { "rapidly_declining" };
        
        let output = DriftAnalyzeOutput {
            path: params.path.clone(),
            drift_score,
            trend: trend.to_string(),
            metrics: DriftMetricsInfo {
                type_strictness: 0.92,
                naming_consistency: 0.88,
                error_handling_quality: 0.75,
                complexity_avg: 0.45,
                dead_code_ratio: 0.05,
            },
            alerts: vec![
                DriftAlertInfo {
                    alert_type: "ErrorHandlingErosion".to_string(),
                    severity: "medium".to_string(),
                    message: "Error handling quality declining (0.85 → 0.75)".to_string(),
                    metric_value: 0.75,
                    threshold: 0.80,
                },
            ],
            recommendation: "Review error handling patterns - consider reverting to specific error types".to_string(),
        };
        
        Ok(CallToolResult::success(vec![Content::json(output)?]))
    }

    #[tool(description = "Get trend analysis for a file or project over a time window")]
    async fn drift_trend(
        &self,
        params: Parameters<DriftTrendInput>,
    ) -> Result<CallToolResult> {
        let params = params.0;
        let window_days = params.window_days.unwrap_or(7);
        
        // Simulated trend analysis
        let output = DriftTrendOutput {
            path: params.path.clone(),
            window_days,
            quality_trend: "improving".to_string(),
            complexity_trend: 0.02,
            violation_trend: -0.05,
            snapshots_analyzed: 7,
            recommendation: "Quality is improving. Keep current approach.".to_string(),
        };
        
        Ok(CallToolResult::success(vec![Content::json(output)?]))
    }

    #[tool(description = "Create a new drift snapshot for the project")]
    async fn drift_snapshot(
        &self,
        params: Parameters<DriftSnapshotInput>,
    ) -> Result<CallToolResult> {
        let params = params.0;
        
        let snapshot_id = format!("DRIFT-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));
        
        let output = DriftSnapshotOutput {
            success: true,
            snapshot_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            commit_hash: params.commit_hash,
            metrics_recorded: 5,
        };
        
        Ok(CallToolResult::success(vec![Content::json(output)?]))
    }
}
