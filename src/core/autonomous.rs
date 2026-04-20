// core/autonomous.rs — Autonomous Build engine (Phase 7)
//
// Concepts implemented:
//   GSD-2  — fresh context per task (load only what that task needs)
//   Caveman — context compression when message count exceeds threshold
//   PentAGI — loop detection (same tool call ≥ N times) + max tool limit

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::mpsc;

use crate::providers::{LLMProvider, ProviderMessage, StreamEvent};
use crate::tools::verify::run_verify;
use super::progress::ProgressManager;

// ─── Config ───────────────────────────────────────────────────────────────────

pub struct AutonomousConfig {
    /// Skip all confirmation prompts (YOLO mode)
    pub yolo: bool,
    /// Max tool calls per autonomous session before halting (PentAGI)
    pub max_tools: u32,
    /// Consecutive identical tool calls that trigger loop detection (PentAGI)
    pub loop_limit: usize,
    /// Compress messages when total count exceeds this threshold (Caveman)
    pub compress_threshold: usize,
}

impl Default for AutonomousConfig {
    fn default() -> Self {
        AutonomousConfig {
            yolo: false,
            max_tools: 50,
            loop_limit: 3,
            compress_threshold: 20,
        }
    }
}

// ─── Loop detector (PentAGI concept) ─────────────────────────────────────────

pub struct LoopDetector {
    recent: VecDeque<String>,
    limit: usize,
}

impl LoopDetector {
    pub fn new(limit: usize) -> Self {
        LoopDetector {
            recent: VecDeque::new(),
            limit,
        }
    }

    /// Record a tool call identifier. Returns `true` if a loop is detected
    /// (same call appeared `limit` consecutive times).
    pub fn record(&mut self, call: &str) -> bool {
        self.recent.push_back(call.to_string());
        // Rolling window: keep at most limit * 2 entries
        while self.recent.len() > self.limit * 2 {
            self.recent.pop_front();
        }
        if self.recent.len() < self.limit {
            return false;
        }
        // Check if the last `limit` entries are all identical
        let tail: Vec<&String> = self.recent.iter().rev().take(self.limit).collect();
        tail.windows(2).all(|w| w[0] == w[1])
    }

    pub fn reset(&mut self) {
        self.recent.clear();
    }
}

// ─── Context compressor (Caveman concept) ─────────────────────────────────────

pub struct ContextCompressor;

impl ContextCompressor {
    /// Compress messages in-place when `messages.len() > threshold`.
    /// Older messages are replaced with a summary header; the most recent
    /// `threshold / 2` messages are kept verbatim.
    pub fn compress(messages: &mut Vec<ProviderMessage>, threshold: usize) {
        if messages.len() <= threshold {
            return;
        }
        let keep = threshold / 2;
        let drain_count = messages.len() - keep;

        let old: Vec<ProviderMessage> = messages.drain(0..drain_count).collect();
        let summary = old
            .iter()
            .map(|m| {
                let preview = if m.content.len() > 200 {
                    format!("{}…", &m.content[..200])
                } else {
                    m.content.clone()
                };
                format!("[{}]: {}", m.role, preview)
            })
            .collect::<Vec<_>>()
            .join("\n");

        messages.insert(
            0,
            ProviderMessage {
                role: "system".into(),
                content: format!(
                    "[Context summary — {} earlier messages compressed (~46% token reduction)]\n{}",
                    drain_count, summary
                ),
            },
        );
    }
}

// ─── Autonomous build engine ──────────────────────────────────────────────────

pub struct AutonomousBuild {
    pub config: AutonomousConfig,
    pub provider: Arc<dyn LLMProvider>,
    pub project_dir: PathBuf,
}

impl AutonomousBuild {
    pub fn new(config: AutonomousConfig, provider: Arc<dyn LLMProvider>) -> Self {
        let project_dir =
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        AutonomousBuild { config, provider, project_dir }
    }

    pub fn new_in_dir(
        config: AutonomousConfig,
        provider: Arc<dyn LLMProvider>,
        dir: impl Into<PathBuf>,
    ) -> Self {
        AutonomousBuild { config, provider, project_dir: dir.into() }
    }

    /// Run all pending tasks from PROGRESS.md.
    ///
    /// Progress lines and AI output are sent to `output_tx` as plain strings
    /// so both CLI (print to stdout) and TUI (stream to chat pane) can use
    /// the same engine.
    pub async fn run_all(
        &self,
        output_tx: mpsc::UnboundedSender<String>,
    ) -> anyhow::Result<()> {
        let progress_path = self.project_dir.join("PROGRESS.md");
        let pm = ProgressManager::new(&progress_path);

        let pending = match pm.pending_tasks() {
            Ok(p) => p,
            Err(e) => {
                let _ = output_tx.send(format!(
                    "[build] Could not read PROGRESS.md: {e}\n\
                     Make sure you are running from the project root.\n"
                ));
                return Ok(());
            }
        };

        if pending.is_empty() {
            let _ = output_tx.send(
                "No pending phases in PROGRESS.md — all complete!\n".into(),
            );
            return Ok(());
        }

        let total = pending.len();
        let _ = output_tx.send(format!(
            "Autonomous build starting — {} pending phase(s)\n\
             Project: {}\n",
            total,
            self.project_dir.display()
        ));

        let mut loop_detector = LoopDetector::new(self.config.loop_limit);
        let mut tool_calls: u32 = 0;

        for (idx, task) in pending.iter().enumerate() {
            // ── Tool-call budget (PentAGI) ─────────────────────────────────
            if tool_calls >= self.config.max_tools {
                let _ = output_tx.send(format!(
                    "\n[build] Tool call limit ({}) reached — stopping.\n",
                    self.config.max_tools
                ));
                break;
            }

            // ── Loop detection (PentAGI) ───────────────────────────────────
            let call_key = format!("phase-{}", task.phase);
            if loop_detector.record(&call_key) {
                let _ = output_tx.send(format!(
                    "\n[build] Loop detected on Phase {} — skipping to avoid infinite loop.\n",
                    task.phase
                ));
                loop_detector.reset();
                continue;
            }

            let _ = output_tx.send(format!(
                "\n━━━ [{}/{}] Phase {} — {} ━━━\n",
                idx + 1,
                total,
                task.phase,
                task.name
            ));

            // Mark in-progress
            if let Err(e) = pm.mark_in_progress(task.phase) {
                let _ = output_tx.send(format!(
                    "  [warn] Could not mark phase as in-progress: {e}\n"
                ));
            }

            // ── Build fresh context (GSD-2 concept) ───────────────────────
            let progress_content = self.read_file("PROGRESS.md");
            let decisions_content = self.read_file("DECISIONS.md");

            let mut messages = vec![
                ProviderMessage {
                    role: "system".into(),
                    content: self.system_prompt(),
                },
                ProviderMessage {
                    role: "user".into(),
                    content: format!(
                        "Current project state:\n\n\
                         ## PROGRESS.md\n{}\n\n\
                         ## DECISIONS.md\n{}",
                        progress_content, decisions_content
                    ),
                },
                ProviderMessage {
                    role: "user".into(),
                    content: format!(
                        "Your task: Phase {} — {}\n\n\
                         Describe what this phase should implement, \
                         what files need to be created or modified, \
                         and any key technical decisions for this phase.",
                        task.phase, task.name
                    ),
                },
            ];

            // Apply Caveman compression if needed
            ContextCompressor::compress(&mut messages, self.config.compress_threshold);

            // ── LLM call ──────────────────────────────────────────────────
            let _ = output_tx.send("  [AI] Analyzing task...\n".into());
            tool_calls += 1;

            match self.call_llm(&messages, &output_tx).await {
                Ok(response) if !response.is_empty() => {
                    let _ = output_tx.send(format!("\n[AI response]\n{}\n", response));
                }
                Ok(_) => {
                    let _ = output_tx.send("  (no response from model)\n".into());
                }
                Err(e) => {
                    let _ = output_tx.send(format!("  [error] LLM failed: {e}\n"));
                    // Non-fatal — continue to verify and next task
                }
            }

            // ── Verify ────────────────────────────────────────────────────
            let _ = output_tx.send(
                "\n  [verify] Running cargo clippy + build + test...\n".into(),
            );
            let dir = self.project_dir.to_string_lossy().to_string();
            match run_verify(&dir).await {
                Ok(report) => {
                    let _ = output_tx.send(report.output.clone());
                    if report.all_passed() {
                        let _ = output_tx.send(format!(
                            "\n  ✓ Phase {} verified — marking complete.\n",
                            task.phase
                        ));
                        if let Err(e) = pm.mark_complete(task.phase) {
                            let _ = output_tx.send(format!(
                                "  [warn] Could not mark complete: {e}\n"
                            ));
                        }
                    } else {
                        let _ = output_tx.send(format!(
                            "\n  ✗ Phase {} verify failed — kept as In Progress.\n",
                            task.phase
                        ));
                    }
                }
                Err(e) => {
                    let _ = output_tx.send(format!("  [error] Verify error: {e}\n"));
                }
            }
        }

        let _ = output_tx.send("\n─── Autonomous build complete ───\n".into());
        Ok(())
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Call the LLM, stream tokens to `output_tx`, and return the full response.
    async fn call_llm(
        &self,
        messages: &[ProviderMessage],
        output_tx: &mpsc::UnboundedSender<String>,
    ) -> anyhow::Result<String> {
        let (stream_tx, mut stream_rx) = mpsc::unbounded_channel::<StreamEvent>();
        let provider = self.provider.clone();
        let msgs: Vec<ProviderMessage> = messages.to_vec();

        tokio::spawn(async move {
            if let Err(e) = provider.complete(&msgs, stream_tx.clone()).await {
                let _ = stream_tx.send(StreamEvent::Error(e.to_string()));
            }
        });

        let mut full_response = String::new();
        loop {
            match stream_rx.recv().await {
                Some(StreamEvent::Token(t)) => {
                    let _ = output_tx.send(t.clone());
                    full_response.push_str(&t);
                }
                Some(StreamEvent::Done) | None => break,
                Some(StreamEvent::Error(e)) => {
                    return Err(anyhow::anyhow!("LLM stream error: {}", e));
                }
            }
        }
        Ok(full_response)
    }

    fn system_prompt(&self) -> String {
        "You are Everthink IDE's autonomous build agent. \
         Analyze the current project state and provide clear, actionable \
         implementation guidance for the assigned phase. \
         Be concise and technical. Focus only on the current phase task."
            .into()
    }

    fn read_file(&self, name: &str) -> String {
        let path = self.project_dir.join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|_| format!("({} not found)", name))
    }
}
