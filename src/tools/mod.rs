// tools/mod.rs — Tool trait, ToolResult, registry, all tool modules

pub mod bash;
pub mod fs;
pub mod mcp;
pub mod python_engine;
pub mod registry;
pub mod verify;
pub mod web;

use async_trait::async_trait;

// ─── Result type ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub output: String,
    pub success: bool,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        ToolResult { output: output.into(), success: true, error: None }
    }

    pub fn fail(error: impl Into<String>) -> Self {
        let msg = error.into();
        ToolResult { output: msg.clone(), success: false, error: Some(msg) }
    }
}

// ─── Tool trait ───────────────────────────────────────────────────────────────

#[async_trait]
pub trait Tool: Send + Sync {
    /// Short identifier used in tool calls, e.g. "bash", "read", "glob"
    fn name(&self) -> &str;
    /// One-line description shown in /help
    fn description(&self) -> &str;
    /// Execute the tool with JSON args. Returns a ToolResult.
    async fn run(&self, args: serde_json::Value) -> anyhow::Result<ToolResult>;
}
