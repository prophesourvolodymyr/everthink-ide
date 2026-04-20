// tools/python_engine.rs — PythonEngine: subprocess bridge to python3 scripts
// Used by Greppy and Caveman engines

use anyhow::Result;

pub struct PythonEngine {
    script_path: String,
}

impl PythonEngine {
    pub fn new(script: impl Into<String>) -> Self {
        PythonEngine { script_path: script.into() }
    }

    /// Execute a python3 script with --tool and --args flags.
    /// Returns stdout on success, stderr on failure.
    pub async fn execute(&self, tool: &str, args: &serde_json::Value) -> Result<String> {
        let output = tokio::process::Command::new("python3")
            .arg(&self.script_path)
            .arg("--tool")
            .arg(tool)
            .arg("--args")
            .arg(args.to_string())
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(anyhow::anyhow!("Python error: {err}"))
        }
    }

    /// Check if python3 is available on this system.
    pub async fn available() -> bool {
        tokio::process::Command::new("python3")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
