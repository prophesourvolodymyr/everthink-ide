// core/progress.rs — PROGRESS.md parser and phase status updater
// Phase 7

use std::path::PathBuf;
use anyhow::Context;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
}

impl TaskStatus {
    pub fn emoji(&self) -> &str {
        match self {
            TaskStatus::Pending => "⬜",
            TaskStatus::InProgress => "🔄",
            TaskStatus::Complete => "✅",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            TaskStatus::Pending => "Pending",
            TaskStatus::InProgress => "In Progress",
            TaskStatus::Complete => "Complete",
        }
    }

    fn from_col(col: &str) -> TaskStatus {
        if col.contains('✅') {
            TaskStatus::Complete
        } else if col.contains('🔄') {
            TaskStatus::InProgress
        } else {
            TaskStatus::Pending
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgressTask {
    pub phase: u8,
    pub name: String,
    pub status: TaskStatus,
}

// ─── Manager ──────────────────────────────────────────────────────────────────

pub struct ProgressManager {
    path: PathBuf,
}

impl ProgressManager {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        ProgressManager { path: path.into() }
    }

    pub fn from_cwd() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        ProgressManager::new(cwd.join("PROGRESS.md"))
    }

    /// Parse all tasks from PROGRESS.md.
    pub fn tasks(&self) -> anyhow::Result<Vec<ProgressTask>> {
        let content = std::fs::read_to_string(&self.path)
            .with_context(|| format!("reading {:?}", self.path))?;
        Ok(parse_tasks(&content))
    }

    /// Return only Pending tasks.
    pub fn pending_tasks(&self) -> anyhow::Result<Vec<ProgressTask>> {
        Ok(self
            .tasks()?
            .into_iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .collect())
    }

    /// Mark a phase as InProgress in PROGRESS.md.
    pub fn mark_in_progress(&self, phase: u8) -> anyhow::Result<()> {
        self.update_status(phase, TaskStatus::InProgress)
    }

    /// Mark a phase as Complete in PROGRESS.md.
    pub fn mark_complete(&self, phase: u8) -> anyhow::Result<()> {
        self.update_status(phase, TaskStatus::Complete)
    }

    fn update_status(&self, phase: u8, new_status: TaskStatus) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(&self.path)
            .with_context(|| format!("reading {:?}", self.path))?;

        let updated: Vec<String> = content
            .lines()
            .map(|line| {
                if is_phase_row(line, phase) {
                    rewrite_row(line, &new_status)
                } else {
                    line.to_string()
                }
            })
            .collect();

        let mut final_content = updated.join("\n");
        if content.ends_with('\n') {
            final_content.push('\n');
        }

        std::fs::write(&self.path, final_content)
            .with_context(|| format!("writing {:?}", self.path))?;
        Ok(())
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Parse table rows from PROGRESS.md content.
/// Expects rows like: | 7 | Autonomous Build | ⬜ Pending | `src/core/TODO.md` |
fn parse_tasks(content: &str) -> Vec<ProgressTask> {
    let mut tasks = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cols: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
        // cols[0]="", cols[1]=phase, cols[2]=name, cols[3]=status, cols[4]=todo, cols[5]=""
        if cols.len() < 4 {
            continue;
        }
        let phase_col = cols[1];
        let name_col = cols[2];
        let status_col = cols[3];

        // Skip header / separator rows
        if name_col == "Name" || name_col.is_empty() || phase_col == "Phase" {
            continue;
        }
        if let Ok(phase) = phase_col.parse::<u8>() {
            tasks.push(ProgressTask {
                phase,
                name: name_col.to_string(),
                status: TaskStatus::from_col(status_col),
            });
        }
    }
    tasks
}

/// Returns true if this line is the table row for the given phase number.
fn is_phase_row(line: &str, phase: u8) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return false;
    }
    let cols: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
    if cols.len() < 3 {
        return false;
    }
    cols[1].parse::<u8>().ok() == Some(phase)
}

/// Rewrite the status column (index 3) of a pipe-delimited table row.
fn rewrite_row(line: &str, status: &TaskStatus) -> String {
    let mut parts: Vec<String> = line.split('|').map(|s| s.to_string()).collect();
    if parts.len() >= 4 {
        parts[3] = format!(" {} {} ", status.emoji(), status.label());
    }
    parts.join("|")
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
| Phase | Name | Status | Todo File |
|-------|------|--------|-----------|
| 1 | Rust Scaffold | ✅ Complete | `src/cli/TODO.md` |
| 7 | Autonomous Build | ⬜ Pending | `src/core/TODO.md` |
| 8 | Skills System | ⬜ Pending | `src/skills/TODO.md` |
";

    #[test]
    fn parse_finds_pending() {
        let tasks = parse_tasks(SAMPLE);
        assert_eq!(tasks.len(), 3);
        let pending: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::Pending).collect();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].phase, 7);
        assert_eq!(pending[0].name, "Autonomous Build");
    }

    #[test]
    fn rewrite_row_changes_status() {
        let line = "| 7 | Autonomous Build | ⬜ Pending | `src/core/TODO.md` |";
        let result = rewrite_row(line, &TaskStatus::InProgress);
        assert!(result.contains("🔄"));
        assert!(result.contains("In Progress"));
    }
}
