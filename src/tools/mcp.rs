// tools/mcp.rs — McpStub: placeholder for MCP (Model Context Protocol) tools
// Full implementation in Phase 10+

use super::{Tool, ToolResult};
use async_trait::async_trait;

pub struct McpStub;

#[async_trait]
impl Tool for McpStub {
    fn name(&self) -> &str { "mcp" }

    fn description(&self) -> &str {
        "MCP tool bridge (stub — Phase 10). Args: {\"tool\": \"name\", \"args\": {}}"
    }

    async fn run(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let tool_name = args["tool"].as_str().unwrap_or("unknown");
        Ok(ToolResult::fail(format!(
            "MCP tool '{}' not yet implemented (Phase 10). Args: {}",
            tool_name, args
        )))
    }
}
