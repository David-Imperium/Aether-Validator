//! Graph Tools - MCP dependency graph tools
//!
//! Tools:
//! - build_graph: Build dependency graph
//! - who_calls: Find callers of a function
//! - impact_analysis: Analyze impact of changes
//! - file_dependencies: Get file dependencies
//! - file_dependents: Get files that depend on this
//! - find_call_chain: Find call chain between functions

use anyhow::Result;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    schemars::JsonSchema,
    tool, tool_router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BuildGraphInput {
    pub directory: String,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WhoCallsInput {
    pub function_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImpactAnalysisInput {
    pub node_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileDepsInput {
    pub file_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CallChainInput {
    pub from_function: String,
    pub to_function: String,
}

#[tool_router]
pub struct GraphTools;

#[tool]
impl GraphTools {
    #[tool(description = "Build dependency graph from directory")]
    async fn build_graph(
        &self,
        _params: Parameters<BuildGraphInput>,
    ) -> Result<CallToolResult> {
        Ok(CallToolResult::success(vec![Content::text(
            "Build graph tool - see main.rs for implementation"
        )]))
    }

    #[tool(description = "Find all callers of a function")]
    async fn who_calls(
        &self,
        _params: Parameters<WhoCallsInput>,
    ) -> Result<CallToolResult> {
        Ok(CallToolResult::success(vec![Content::text(
            "Who calls tool - see main.rs for implementation"
        )]))
    }

    #[tool(description = "Analyze impact of modifications")]
    async fn impact_analysis(
        &self,
        _params: Parameters<ImpactAnalysisInput>,
    ) -> Result<CallToolResult> {
        Ok(CallToolResult::success(vec![Content::text(
            "Impact analysis tool - see main.rs for implementation"
        )]))
    }

    #[tool(description = "Get dependencies of a file")]
    async fn file_dependencies(
        &self,
        _params: Parameters<FileDepsInput>,
    ) -> Result<CallToolResult> {
        Ok(CallToolResult::success(vec![Content::text(
            "File dependencies tool - see main.rs for implementation"
        )]))
    }

    #[tool(description = "Get files that depend on this file")]
    async fn file_dependents(
        &self,
        _params: Parameters<FileDepsInput>,
    ) -> Result<CallToolResult> {
        Ok(CallToolResult::success(vec![Content::text(
            "File dependents tool - see main.rs for implementation"
        )]))
    }

    #[tool(description = "Find call chain between two functions")]
    async fn find_call_chain(
        &self,
        _params: Parameters<CallChainInput>,
    ) -> Result<CallToolResult> {
        Ok(CallToolResult::success(vec![Content::text(
            "Call chain tool - see main.rs for implementation"
        )]))
    }
}
