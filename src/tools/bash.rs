// tools/bash.rs — BashTool: execute shell commands, capture stdout + stderr

use super::{Tool, ToolResult};
use async_trait::async_trait;

pub struct BashTool;

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str { "bash" }

    fn description(&self) -> &str {
        "Execute a shell command. Args: {\"command\": \"git status\"}"
    }

    async fn run(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let command = match args["command"].as_str() {
            Some(c) => c.to_string(),
            None => return Ok(ToolResult::fail("bash: missing 'command' arg")),
        };

        // Use sh -c so pipes, redirects, etc. all work
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let combined = if stderr.is_empty() {
            stdout.clone()
        } else if stdout.is_empty() {
            stderr.clone()
        } else {
            format!("{stdout}\n--- stderr ---\n{stderr}")
        };

        if output.status.success() {
            Ok(ToolResult::ok(if combined.is_empty() { "(no output)".into() } else { combined }))
        } else {
            let code = output.status.code().unwrap_or(-1);
            Ok(ToolResult::fail(format!("exit {code}\n{combined}")))
        }
    }
}
