// tools/fs.rs — ReadTool, WriteTool, EditTool, GlobTool, GrepTool

use super::{Tool, ToolResult};
use async_trait::async_trait;
use regex::Regex;
use std::path::Path;

// ─── ReadTool ─────────────────────────────────────────────────────────────────

pub struct ReadTool;

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str { "read" }
    fn description(&self) -> &str {
        "Read a file. Args: {\"path\": \"src/main.rs\", \"offset\": 1, \"limit\": 100}"
    }

    async fn run(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let path = match args["path"].as_str() {
            Some(p) => p.to_string(),
            None => return Ok(ToolResult::fail("read: missing 'path' arg")),
        };

        let content = match tokio::fs::read_to_string(&path).await {
            Ok(c) => c,
            Err(e) => return Ok(ToolResult::fail(format!("read: {e}"))),
        };

        let lines: Vec<&str> = content.lines().collect();
        let offset = args["offset"].as_u64().unwrap_or(1).saturating_sub(1) as usize;
        let limit = args["limit"].as_u64().unwrap_or(200) as usize;

        let slice = lines
            .iter()
            .skip(offset)
            .take(limit)
            .enumerate()
            .map(|(i, line)| format!("{}: {}", offset + i + 1, line))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ToolResult::ok(if slice.is_empty() { "(empty file)".into() } else { slice }))
    }
}

// ─── WriteTool ────────────────────────────────────────────────────────────────

pub struct WriteTool;

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str { "write" }
    fn description(&self) -> &str {
        "Write/overwrite a file. Args: {\"path\": \"file.txt\", \"content\": \"...\"}"
    }

    async fn run(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let path = match args["path"].as_str() {
            Some(p) => p.to_string(),
            None => return Ok(ToolResult::fail("write: missing 'path' arg")),
        };
        let content = args["content"].as_str().unwrap_or("");

        // Create parent directories if needed
        if let Some(parent) = Path::new(&path).parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent).await.ok();
            }
        }

        match tokio::fs::write(&path, content).await {
            Ok(_) => Ok(ToolResult::ok(format!("Written: {path}"))),
            Err(e) => Ok(ToolResult::fail(format!("write: {e}"))),
        }
    }
}

// ─── EditTool ─────────────────────────────────────────────────────────────────

pub struct EditTool;

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str { "edit" }
    fn description(&self) -> &str {
        "Find-and-replace in a file. Args: {\"path\": \"...\", \"old\": \"...\", \"new\": \"...\"}"
    }

    async fn run(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let path = match args["path"].as_str() {
            Some(p) => p.to_string(),
            None => return Ok(ToolResult::fail("edit: missing 'path' arg")),
        };
        let old = match args["old"].as_str() {
            Some(o) => o.to_string(),
            None => return Ok(ToolResult::fail("edit: missing 'old' arg")),
        };
        let new = args["new"].as_str().unwrap_or("");

        let content = match tokio::fs::read_to_string(&path).await {
            Ok(c) => c,
            Err(e) => return Ok(ToolResult::fail(format!("edit: read failed: {e}"))),
        };

        if !content.contains(&old) {
            return Ok(ToolResult::fail(format!("edit: oldString not found in {path}")));
        }

        let count = content.matches(&old).count();
        if count > 1 {
            return Ok(ToolResult::fail(format!(
                "edit: found {count} matches for oldString in {path} — provide more context to make it unique"
            )));
        }

        let updated = content.replacen(&old, new, 1);
        match tokio::fs::write(&path, updated).await {
            Ok(_) => Ok(ToolResult::ok(format!("Edited: {path}"))),
            Err(e) => Ok(ToolResult::fail(format!("edit: write failed: {e}"))),
        }
    }
}

// ─── GlobTool ─────────────────────────────────────────────────────────────────

pub struct GlobTool;

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str { "glob" }
    fn description(&self) -> &str {
        "Find files matching a glob pattern. Args: {\"pattern\": \"**/*.rs\"}"
    }

    async fn run(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let pattern = match args["pattern"].as_str() {
            Some(p) => p.to_string(),
            None => return Ok(ToolResult::fail("glob: missing 'pattern' arg")),
        };

        let matches = tokio::task::spawn_blocking(move || {
            glob::glob(&pattern)
                .map(|paths| {
                    paths
                        .filter_map(|p| p.ok())
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .await?;

        if matches.is_empty() {
            Ok(ToolResult::ok("(no matches)"))
        } else {
            Ok(ToolResult::ok(matches.join("\n")))
        }
    }
}

// ─── GrepTool ─────────────────────────────────────────────────────────────────

pub struct GrepTool;

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str { "grep" }
    fn description(&self) -> &str {
        "Search file contents by regex. Args: {\"pattern\": \"fn main\", \"path\": \"src/\"}"
    }

    async fn run(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let pattern = match args["pattern"].as_str() {
            Some(p) => p.to_string(),
            None => return Ok(ToolResult::fail("grep: missing 'pattern' arg")),
        };
        let search_path = args["path"].as_str().unwrap_or(".").to_string();

        let regex = match Regex::new(&pattern) {
            Ok(r) => r,
            Err(e) => return Ok(ToolResult::fail(format!("grep: invalid regex: {e}"))),
        };

        let results = tokio::task::spawn_blocking(move || {
            let mut out = Vec::new();
            for entry in walkdir::WalkDir::new(&search_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let path = entry.path().to_string_lossy().to_string();
                // Skip binary-likely files and build artifacts
                if path.contains("/target/") || path.ends_with(".lock") {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for (i, line) in content.lines().enumerate() {
                        if regex.is_match(line) {
                            out.push(format!("{}:{}: {}", path, i + 1, line.trim()));
                        }
                    }
                }
                if out.len() > 200 {
                    out.push("... (truncated at 200 matches)".into());
                    break;
                }
            }
            out
        })
        .await?;

        if results.is_empty() {
            Ok(ToolResult::ok("(no matches)"))
        } else {
            Ok(ToolResult::ok(results.join("\n")))
        }
    }
}
