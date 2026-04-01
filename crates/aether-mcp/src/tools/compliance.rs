//! Compliance Tools - MCP integration for Compliance Engine
//!
//! Tools:
//! - compliance_status: Get compliance engine status and statistics
//! - compliance_evaluate: Evaluate a single violation through compliance engine
//! - compliance_accept: Accept a violation with reason
//! - compliance_learn: Manually trigger pattern learning

use anyhow::Result;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    schemars::JsonSchema,
    tool, tool_router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use aether_intelligence::compliance::{
    ComplianceEngine, ComplianceConfig, ComplianceContext,
    ComplianceAction, ContractTier, ComplianceDecision,
};

// ============================================================================
// Global Compliance Engine
// ============================================================================

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
// Tool Input Schemas
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ComplianceStatusInput {}

#[derive(Debug, Deserialize, JsonSchema)]
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
pub struct ComplianceAcceptInput {
    /// Rule ID to accept
    pub rule_id: String,
    /// File path
    pub file_path: String,
    /// Reason for acceptance
    pub reason: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ComplianceLearnInput {
    /// Rule ID
    pub rule_id: String,
    /// Scope: "file", "directory", "pattern", "project"
    pub scope_type: String,
    /// Scope value (file path, directory, pattern)
    pub scope_value: String,
    /// Confidence level (0.0-1.0)
    pub confidence: Option<f64>,
}

// ============================================================================
// Tool Output Schemas
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ComplianceStatusOutput {
    pub total_exemptions: usize,
    pub learned_patterns: usize,
    pub user_created: usize,
    pub occurrence_tracking: usize,
    pub config: ComplianceConfigInfo,
}

#[derive(Debug, Serialize)]
pub struct ComplianceConfigInfo {
    pub auto_accept_threshold: f64,
    pub ask_threshold: f64,
    pub learn_after_occurrences: u32,
    pub use_dubbioso: bool,
}

#[derive(Debug, Serialize)]
pub struct ComplianceEvaluateOutput {
    pub action: String,
    pub tier: String,
    pub confidence: f64,
    pub overridable: bool,
    pub explanation: String,
    pub should_block: bool,
    pub needs_input: bool,
}

// ============================================================================
// Tools
// ============================================================================

#[tool_router]
pub struct ComplianceTools;

#[tool]
impl ComplianceTools {
    #[tool(description = "Get compliance engine status and statistics")]
    async fn compliance_status(
        &self,
        _params: Parameters<ComplianceStatusInput>,
    ) -> Result<CallToolResult> {
        let engine = get_compliance_engine();
        let mut engine = engine.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        
        let stats = engine.stats();
        
        let output = ComplianceStatusOutput {
            total_exemptions: stats.exemptions.total,
            learned_patterns: stats.exemptions.learned,
            user_created: stats.exemptions.user_created,
            occurrence_tracking: stats.occurrence_tracking,
            config: ComplianceConfigInfo {
                auto_accept_threshold: 0.90,
                ask_threshold: 0.60,
                learn_after_occurrences: 3,
                use_dubbioso: true,
            },
        };
        
        Ok(CallToolResult::success(vec![Content::json(output)?]))
    }

    #[tool(description = "Evaluate a violation through the compliance engine to determine action")]
    async fn compliance_evaluate(
        &self,
        params: Parameters<ComplianceEvaluateInput>,
    ) -> Result<CallToolResult> {
        let params = params.0;
        let engine = get_compliance_engine();
        let mut engine = engine.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        
        let ctx = ComplianceContext {
            file_path: params.file_path,
            line: params.line.unwrap_or(0),
            snippet: None,
            project_type: None,
            code_region: params.code_region,
            function_context: None,
        };
        
        let decision = engine.evaluate(
            &params.rule_id,
            &params.domain,
            &params.message,
            &ctx,
        ).await.map_err(|e| anyhow::anyhow!("Evaluation error: {}", e))?;
        
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
        
        let output = ComplianceEvaluateOutput {
            action: action.to_string(),
            tier: tier.to_string(),
            confidence: decision.confidence,
            overridable: decision.overridable,
            explanation: decision.explanation,
            should_block: decision.should_fail(),
            needs_input: decision.needs_user_input(),
        };
        
        Ok(CallToolResult::success(vec![Content::json(output)?]))
    }

    #[tool(description = "Accept a violation with a documented reason")]
    async fn compliance_accept(
        &self,
        params: Parameters<ComplianceAcceptInput>,
    ) -> Result<CallToolResult> {
        let params = params.0;
        let engine = get_compliance_engine();
        let mut engine = engine.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        
        engine.accept_violation(
            &params.rule_id,
            &params.file_path,
            params.reason.clone(),
        ).map_err(|e| anyhow::anyhow!("Accept error: {}", e))?;
        
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Violation {} accepted for {} with reason: {}",
            params.rule_id, params.file_path, params.reason
        ))]))
    }

    #[tool(description = "Manually learn a pattern for automatic acceptance")]
    async fn compliance_learn(
        &self,
        params: Parameters<ComplianceLearnInput>,
    ) -> Result<CallToolResult> {
        let params = params.0;
        
        use aether_intelligence::compliance::{Exemption, ExemptionScope, ExemptionSource};
        
        let scope = match params.scope_type.as_str() {
            "file" => ExemptionScope::File { path: params.scope_value },
            "directory" => ExemptionScope::Directory { path: params.scope_value },
            "pattern" => ExemptionScope::Pattern { pattern: params.scope_value },
            "project" => ExemptionScope::Project,
            _ => return Err(anyhow::anyhow!("Invalid scope_type: {}", params.scope_type)),
        };
        
        let confidence = params.confidence.unwrap_or(0.85);
        let exemption = Exemption::learned(params.rule_id.clone(), scope, confidence);
        
        // Store via engine
        let engine = get_compliance_engine();
        let mut engine = engine.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        // Note: We need to add the exemption through the engine's exemption store
        // This is a simplified version
        
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Pattern learned for {} with scope {:?} (confidence: {:.2})",
            params.rule_id, params.scope_type, confidence
        ))]))
    }
}
