// tools/verify.rs — VerifyTool: cargo clippy + cargo build + cargo test

use super::{Tool, ToolResult};
use async_trait::async_trait;
use tokio::process::Command;

pub struct VerifyTool;

/// Result of a full verify run.
pub struct VerifyReport {
    pub clippy_ok: bool,
    pub build_ok: bool,
    pub tests_ok: bool,
    pub output: String,
}

impl VerifyReport {
    pub fn all_passed(&self) -> bool {
        self.clippy_ok && self.build_ok && self.tests_ok
    }

    pub fn summary(&self) -> &str {
        if self.all_passed() { "PASS" } else { "FAIL" }
    }
}

/// Run the full verification pipeline: clippy → build → test.
/// Returns a VerifyReport with combined output.
pub async fn run_verify(project_dir: &str) -> anyhow::Result<VerifyReport> {
    let mut full_output = String::new();
    let mut clippy_ok = false;
    let mut build_ok = false;
    let mut tests_ok = false;

    // ── cargo clippy ──────────────────────────────────────────────────────────
    full_output.push_str("=== cargo clippy ===\n");
    let clippy = Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings"])
        .current_dir(project_dir)
        .output()
        .await?;

    let clippy_out = combine_output(&clippy.stdout, &clippy.stderr);
    clippy_ok = clippy.status.success();
    full_output.push_str(&clippy_out);
    full_output.push_str(if clippy_ok { "\n✓ clippy passed\n\n" } else { "\n✗ clippy failed\n\n" });

    // ── cargo build ───────────────────────────────────────────────────────────
    full_output.push_str("=== cargo build ===\n");
    let build = Command::new("cargo")
        .args(["build"])
        .current_dir(project_dir)
        .output()
        .await?;

    let build_out = combine_output(&build.stdout, &build.stderr);
    build_ok = build.status.success();
    full_output.push_str(&build_out);
    full_output.push_str(if build_ok { "\n✓ build passed\n\n" } else { "\n✗ build failed\n\n" });

    // ── cargo test (only if build passed) ─────────────────────────────────────
    full_output.push_str("=== cargo test ===\n");
    if build_ok {
        let test = Command::new("cargo")
            .args(["test"])
            .current_dir(project_dir)
            .output()
            .await?;

        let test_out = combine_output(&test.stdout, &test.stderr);
        tests_ok = test.status.success();
        full_output.push_str(&test_out);
        full_output.push_str(if tests_ok { "\n✓ tests passed\n" } else { "\n✗ tests failed\n" });
    } else {
        full_output.push_str("(skipped — build failed)\n");
    }

    // ── Summary ───────────────────────────────────────────────────────────────
    full_output.push_str(&format!(
        "\n─── VERIFY {} ─── clippy:{} build:{} tests:{}\n",
        if clippy_ok && build_ok && tests_ok { "PASS ✓" } else { "FAIL ✗" },
        if clippy_ok { "✓" } else { "✗" },
        if build_ok { "✓" } else { "✗" },
        if tests_ok { "✓" } else { "✗" },
    ));

    Ok(VerifyReport { clippy_ok, build_ok, tests_ok, output: full_output })
}

fn combine_output(stdout: &[u8], stderr: &[u8]) -> String {
    let out = String::from_utf8_lossy(stdout);
    let err = String::from_utf8_lossy(stderr);
    match (out.trim().is_empty(), err.trim().is_empty()) {
        (true, true) => "(no output)\n".into(),
        (true, false) => format!("{}\n", err.trim()),
        (false, true) => format!("{}\n", out.trim()),
        (false, false) => format!("{}\n{}\n", out.trim(), err.trim()),
    }
}

// ─── Tool impl ────────────────────────────────────────────────────────────────

#[async_trait]
impl Tool for VerifyTool {
    fn name(&self) -> &str { "verify" }
    fn description(&self) -> &str {
        "Run cargo clippy + build + test. Args: {\"dir\": \".\"} (defaults to cwd)"
    }

    async fn run(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let dir = args["dir"].as_str().unwrap_or(".");
        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        let project_dir = if dir == "." {
            cwd.to_string_lossy().to_string()
        } else {
            dir.to_string()
        };

        let report = run_verify(&project_dir).await?;
        if report.all_passed() {
            Ok(ToolResult::ok(report.output))
        } else {
            Ok(ToolResult { output: report.output, success: false, error: Some("Verify failed".into()) })
        }
    }
}
