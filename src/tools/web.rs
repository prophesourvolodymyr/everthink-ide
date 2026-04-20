// tools/web.rs — WebFetchTool: HTTP GET a URL and return the response body

use super::{Tool, ToolResult};
use async_trait::async_trait;
use reqwest::Client;

pub struct WebFetchTool {
    client: Client,
}

impl WebFetchTool {
    pub fn new() -> Self {
        WebFetchTool { client: Client::new() }
    }
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str { "web_fetch" }

    fn description(&self) -> &str {
        "Fetch a URL via HTTP GET. Args: {\"url\": \"https://...\"}"
    }

    async fn run(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let url = match args["url"].as_str() {
            Some(u) => u.to_string(),
            None => return Ok(ToolResult::fail("web_fetch: missing 'url' arg")),
        };

        let response = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => return Ok(ToolResult::fail(format!("web_fetch: request failed: {e}"))),
        };

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if status.is_success() {
            // Truncate very large responses
            let truncated = if body.len() > 8192 {
                format!("{}\n\n[... truncated at 8KB]", &body[..8192])
            } else {
                body
            };
            Ok(ToolResult::ok(truncated))
        } else {
            Ok(ToolResult::fail(format!("HTTP {status}: {body}")))
        }
    }
}
