// commands/review.rs — /review command: git diff + blast radius + AI review
//
// Blast radius algorithm (no Tree-sitter, pure regex):
//   For each changed file → derive its crate module path → grep all other
//   .rs files for `use crate::<module>` → those files are in the blast radius.
//
// Inspired by code-review-graph (github.com/tirth8205/code-review-graph)
// but implemented in pure Rust using what we already have.

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

// ──────────────────────────────────────────────────────────────────────────────
// Public types
// ──────────────────────────────────────────────────────────────────────────────

/// A file that changed in the diff.
#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    /// +added / -removed line counts from the diff stat
    pub added: usize,
    pub removed: usize,
}

/// A file that was NOT changed but imports something that did change.
#[derive(Debug, Clone)]
pub struct AffectedFile {
    pub path: String,
    /// Which changed module(s) it imports.
    pub depends_on: Vec<String>,
}

/// Risk level of a change based on blast radius size.
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,    // 0-2 affected files
    Medium, // 3-7 affected files
    High,   // 8+ affected files
}

impl RiskLevel {
    fn from_count(n: usize) -> Self {
        match n {
            0..=2 => Self::Low,
            3..=7 => Self::Medium,
            _ => Self::High,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Low    => "🟢 LOW",
            Self::Medium => "🟡 MEDIUM",
            Self::High   => "🔴 HIGH",
        }
    }
}

/// Full result of running a review.
#[derive(Debug)]
pub struct ReviewResult {
    /// Human-readable target description ("staged changes", "HEAD", etc.)
    pub target: String,
    pub changed: Vec<ChangedFile>,
    pub blast_radius: Vec<AffectedFile>,
    pub risk: RiskLevel,
    /// Raw unified diff text (truncated to MAX_DIFF_BYTES if needed).
    pub diff: String,
    /// Total added lines across all changed files.
    pub total_added: usize,
    /// Total removed lines across all changed files.
    pub total_removed: usize,
}

impl ReviewResult {
    /// Format the blast-radius summary for display in TUI (no diff, no AI).
    pub fn format_summary(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!(
            "── /review: {} ──\n\n",
            self.target
        ));

        // Changed files
        if self.changed.is_empty() {
            out.push_str("No changes detected.\n");
            return out;
        }

        out.push_str(&format!(
            "CHANGED ({} file{})  +{}/-{} lines\n",
            self.changed.len(),
            if self.changed.len() == 1 { "" } else { "s" },
            self.total_added,
            self.total_removed,
        ));
        for f in &self.changed {
            out.push_str(&format!(
                "  {} (+{} -{})\n",
                f.path, f.added, f.removed
            ));
        }

        // Blast radius
        out.push('\n');
        out.push_str(&format!(
            "BLAST RADIUS ({} file{})  Risk: {}\n",
            self.blast_radius.len(),
            if self.blast_radius.len() == 1 { "" } else { "s" },
            self.risk.label(),
        ));
        if self.blast_radius.is_empty() {
            out.push_str("  No dependents found.\n");
        } else {
            for f in &self.blast_radius {
                out.push_str(&format!(
                    "  {}  ← uses [{}]\n",
                    f.path,
                    f.depends_on.join(", ")
                ));
            }
        }

        out
    }

    /// Build the prompt to send to the LLM for AI review.
    pub fn build_review_prompt(&self) -> String {
        const MAX_DIFF: usize = 12_000; // chars — keep prompt manageable

        let diff_snippet = if self.diff.len() > MAX_DIFF {
            format!(
                "{}\n\n[... diff truncated — {} chars total ...]",
                &self.diff[..MAX_DIFF],
                self.diff.len()
            )
        } else {
            self.diff.clone()
        };

        let blast_lines = if self.blast_radius.is_empty() {
            "  None detected.".to_string()
        } else {
            self.blast_radius
                .iter()
                .map(|f| format!("  {} (uses: {})", f.path, f.depends_on.join(", ")))
                .collect::<Vec<_>>()
                .join("\n")
        };

        format!(
            "You are an expert code reviewer. Review the following changes concisely.\n\
             Focus on: bugs, logic errors, missing error handling, breaking API changes, \
             security issues, and test gaps.\n\
             Be specific — reference file names and line numbers where relevant.\n\
             Do NOT praise style. Do NOT restate what the diff does unless there is a problem.\n\n\
             TARGET: {target}\n\
             RISK: {risk}  ({changed} changed, {affected} in blast radius)\n\n\
             BLAST RADIUS (files that depend on changed code):\n\
             {blast}\n\n\
             DIFF:\n\
             ```diff\n{diff}\n```\n\n\
             Provide your review now:",
            target   = self.target,
            risk     = self.risk.label(),
            changed  = self.changed.len(),
            affected = self.blast_radius.len(),
            blast    = blast_lines,
            diff     = diff_snippet,
        )
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Git helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Run a git command in `repo_dir` and return stdout as a String.
fn git(repo_dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo_dir)
        .output()
        .with_context(|| format!("git {:?}", args))?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Detect the git root from a starting directory.
fn git_root(start: &Path) -> Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(start)
        .output()
        .context("git rev-parse --show-toplevel")?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        anyhow::bail!("Not inside a git repository");
    }
    Ok(PathBuf::from(s))
}

// ──────────────────────────────────────────────────────────────────────────────
// Diff parsing
// ──────────────────────────────────────────────────────────────────────────────

/// Parse `git diff --stat` output into per-file add/remove counts.
/// Example stat line: " src/foo.rs | 12 ++++---"
fn parse_stat(stat_output: &str) -> HashMap<String, (usize, usize)> {
    let mut map = HashMap::new();
    for line in stat_output.lines() {
        // Skip the summary line ("3 files changed, …")
        if line.trim_start().starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, '|').collect();
        if parts.len() != 2 { continue; }
        let path = parts[0].trim().to_string();
        let counts = parts[1];
        let added   = counts.chars().filter(|&c| c == '+').count();
        let removed = counts.chars().filter(|&c| c == '-').count();
        map.insert(path, (added, removed));
    }
    map
}

/// Get changed files for a given diff target.
fn changed_files(repo: &Path, target: &DiffTarget) -> Result<Vec<ChangedFile>> {
    let (diff_args, stat_args): (Vec<&str>, Vec<&str>) = match target {
        DiffTarget::Staged => (
            vec!["diff", "--cached", "--name-only"],
            vec!["diff", "--cached", "--stat"],
        ),
        DiffTarget::Head => (
            vec!["diff", "HEAD", "--name-only"],
            vec!["diff", "HEAD", "--stat"],
        ),
        DiffTarget::Branch(base) => (
            vec!["diff", base.as_str(), "--name-only"],
            vec!["diff", base.as_str(), "--stat"],
        ),
    };

    let names  = git(repo, &diff_args)?;
    let stat   = git(repo, &stat_args)?;
    let counts = parse_stat(&stat);

    let files = names
        .lines()
        .filter(|l| !l.is_empty())
        .map(|path| {
            let (added, removed) = counts.get(path).copied().unwrap_or((0, 0));
            ChangedFile { path: path.to_string(), added, removed }
        })
        .collect();

    Ok(files)
}

/// Get the raw unified diff.
fn raw_diff(repo: &Path, target: &DiffTarget) -> Result<String> {
    let args: Vec<&str> = match target {
        DiffTarget::Staged      => vec!["diff", "--cached"],
        DiffTarget::Head        => vec!["diff", "HEAD"],
        DiffTarget::Branch(b)  => vec!["diff", b.as_str()],
    };
    git(repo, &args)
}

// ──────────────────────────────────────────────────────────────────────────────
// Blast radius
// ──────────────────────────────────────────────────────────────────────────────

/// Convert a file path (relative to repo root) into a Rust crate module string.
/// e.g. "src/providers/anthropic.rs" → "providers::anthropic"
///      "src/tui/mod.rs"             → "tui"
fn path_to_module(path: &str) -> Option<String> {
    let p = Path::new(path);
    // Only handle Rust source files
    if p.extension().and_then(|e| e.to_str()) != Some("rs") {
        return None;
    }

    // Strip leading "src/" if present
    let stripped = path
        .strip_prefix("src/")
        .unwrap_or(path);

    // Remove the .rs extension
    let without_ext = stripped.strip_suffix(".rs").unwrap_or(stripped);

    // "mod" file → use parent only
    let module = if without_ext.ends_with("/mod") || without_ext == "mod" {
        without_ext
            .strip_suffix("/mod")
            .or_else(|| without_ext.strip_suffix("mod"))
            .unwrap_or(without_ext)
    } else {
        without_ext
    };

    // "main" or "lib" → skip (no meaningful import path)
    if module == "main" || module == "lib" {
        return None;
    }

    Some(module.replace('/', "::"))
}

/// Find all Rust source files under `root`, excluding the given set of paths.
fn all_rust_files(root: &Path, exclude: &HashSet<String>) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().and_then(|x| x.to_str()) == Some("rs")
        })
        .filter_map(|e| {
            let rel = e.path().strip_prefix(root).ok()?.to_string_lossy().to_string();
            if exclude.contains(&rel) { None } else { Some(e.into_path()) }
        })
        .collect()
}

/// For each candidate file, check if it contains a `use crate::<module>` pattern.
/// Returns a map of file_path → list of matched module names.
fn find_dependents(
    root: &Path,
    candidate_files: &[PathBuf],
    module_names: &[String],
) -> HashMap<String, Vec<String>> {
    // Build one regex per module: `use crate::providers` etc.
    let patterns: Vec<(String, Regex)> = module_names
        .iter()
        .filter_map(|m| {
            // Match `use crate::m` or `crate::m::` in any position
            let pat = format!(r"crate::{}", regex::escape(m));
            Regex::new(&pat).ok().map(|r| (m.clone(), r))
        })
        .collect();

    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    for file in candidate_files {
        let Ok(content) = std::fs::read_to_string(file) else { continue };
        let rel = file
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file.to_string_lossy().to_string());

        for (mod_name, regex) in &patterns {
            if regex.is_match(&content) {
                result.entry(rel.clone()).or_default().push(mod_name.clone());
            }
        }
    }

    result
}

/// Compute the blast radius: files that depend on any of the changed files.
fn compute_blast_radius(
    repo: &Path,
    changed: &[ChangedFile],
) -> Vec<AffectedFile> {
    // Derive module paths from changed file paths
    let modules: Vec<String> = changed
        .iter()
        .filter_map(|f| path_to_module(&f.path))
        .collect();

    if modules.is_empty() {
        return Vec::new();
    }

    // Build exclusion set (the changed files themselves)
    let exclude: HashSet<String> = changed.iter().map(|f| f.path.clone()).collect();

    let candidates = all_rust_files(repo, &exclude);
    let dependents = find_dependents(repo, &candidates, &modules);

    let mut affected: Vec<AffectedFile> = dependents
        .into_iter()
        .map(|(path, depends_on)| AffectedFile { path, depends_on })
        .collect();

    // Sort by path for stable output
    affected.sort_by(|a, b| a.path.cmp(&b.path));
    affected
}

// ──────────────────────────────────────────────────────────────────────────────
// Public entry point
// ──────────────────────────────────────────────────────────────────────────────

/// What to diff against.
#[derive(Debug, Clone)]
pub enum DiffTarget {
    /// `git diff --cached`  (staged changes only)
    Staged,
    /// `git diff HEAD`      (all uncommitted changes)
    Head,
    /// `git diff <branch>`  (changes vs another branch)
    Branch(String),
}

impl DiffTarget {
    /// Parse from a user-provided string (e.g. "staged", "head", "main").
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "staged" | "cached" | "index" => Self::Staged,
            "" | "head" | "working"       => Self::Head,
            branch                         => Self::Branch(branch.to_string()),
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Staged     => "staged changes".into(),
            Self::Head       => "uncommitted changes (HEAD)".into(),
            Self::Branch(b)  => format!("changes vs `{b}`"),
        }
    }
}

/// Run the full review pipeline synchronously.
///
/// Returns a `ReviewResult` with everything needed to display the summary and
/// build the LLM prompt. The LLM call itself is done in the TUI async task.
pub fn run_review(cwd: &str, target: DiffTarget) -> Result<ReviewResult> {
    let start = PathBuf::from(cwd);
    let repo  = git_root(&start).context("Finding git root")?;

    let changed = changed_files(&repo, &target)?;
    let diff    = raw_diff(&repo, &target)?;

    let blast_radius = compute_blast_radius(&repo, &changed);
    let risk = RiskLevel::from_count(blast_radius.len());

    let total_added   = changed.iter().map(|f| f.added).sum();
    let total_removed = changed.iter().map(|f| f.removed).sum();

    Ok(ReviewResult {
        target: target.label(),
        changed,
        blast_radius,
        risk,
        diff,
        total_added,
        total_removed,
    })
}
