//! Watch Tools - MCP file watching tools
//!
//! Tools:
//! - watch_start: Start watching a directory
//! - watch_check: Check for file changes
//! - watch_stop: Stop watching

use anyhow::Result;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    schemars::JsonSchema,
    tool, tool_router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WatchStartInput {
    pub directory: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WatchCheckInput {
    pub watch_id: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WatchStopInput {
    pub watch_id: u32,
}

#[tool_router]
pub struct WatchTools;

#[tool]
impl WatchTools {
    #[tool(description = "Start watching a directory for file changes")]
    async fn watch_start(
        &self,
        _params: Parameters<WatchStartInput>,
    ) -> Result<CallToolResult> {
        Ok(CallToolResult::success(vec![Content::text(
            "Watch start tool - see main.rs for implementation"
        )]))
    }

    #[tool(description = "Check for file changes since last check")]
    async fn watch_check(
        &self,
        _params: Parameters<WatchCheckInput>,
    ) -> Result<CallToolResult> {
        Ok(CallToolResult::success(vec![Content::text(
            "Watch check tool - see main.rs for implementation"
        )]))
    }

    #[tool(description = "Stop watching a directory")]
    async fn watch_stop(
        &self,
        _params: Parameters<WatchStopInput>,
    ) -> Result<CallToolResult> {
        Ok(CallToolResult::success(vec![Content::text(
            "Watch stop tool - see main.rs for implementation"
        )]))
    }
}
