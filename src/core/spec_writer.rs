// core/spec_writer.rs — writes AUDIT.md, INTENT.md, SPEC.md from a completed AuditSession

use super::audit::AuditSession;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct SpecWriter;

impl SpecWriter {
    /// Write all three output files. Returns the list of absolute paths written.
    pub fn write_all(session: &AuditSession, project_dir: &Path) -> Result<Vec<String>> {
        let mut written = Vec::new();

        let audit_path = project_dir.join("AUDIT.md");
        std::fs::write(&audit_path, Self::render_audit_md(session))?;
        written.push(audit_path.display().to_string());

        let intent_path = project_dir.join("INTENT.md");
        std::fs::write(&intent_path, Self::render_intent_md(session))?;
        written.push(intent_path.display().to_string());

        let spec_path = project_dir.join("SPEC.md");
        std::fs::write(&spec_path, Self::render_spec_md(session))?;
        written.push(spec_path.display().to_string());

        Ok(written)
    }

    /// Returns the default project directory (current working directory).
    pub fn default_project_dir() -> PathBuf {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    // ── Renderers ─────────────────────────────────────────────────────────────

    /// AUDIT.md — raw Q&A log, timestamped
    fn render_audit_md(session: &AuditSession) -> String {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        let mut out = format!(
            "# AUDIT — {}\n\n**Feature:** {}\n**Date:** {}\n**Questions:** {}\n\n---\n\n",
            session.feature_name,
            session.feature_name,
            now,
            session.answers.len(),
        );

        for (i, answer) in session.answers.iter().enumerate() {
            out.push_str(&format!(
                "## Q{}: {}\n\n{}\n\n---\n\n",
                i + 1,
                answer.question_text,
                answer.answer.trim(),
            ));
        }

        out
    }

    /// INTENT.md — structured summary of goals and constraints
    fn render_intent_md(session: &AuditSession) -> String {
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();

        // Pull answers by question ID
        let get = |id: &str| -> &str {
            session
                .answers
                .iter()
                .find(|a| a.question_id == id)
                .map(|a| a.answer.as_str())
                .unwrap_or("(not answered)")
        };

        format!(
            "# INTENT — {feature}\n\n\
             **Date:** {date}\n\n\
             ---\n\n\
             ## Problem\n\n{problem}\n\n\
             ## Users\n\n{users}\n\n\
             ## Requirements\n\n{requirements}\n\n\
             ## Inputs\n\n{inputs}\n\n\
             ## Outputs\n\n{outputs}\n\n\
             ## Integration\n\n{integration}\n\n\
             ## Risks\n\n{risks}\n\n\
             ## Definition of Done\n\n{done}\n",
            feature = session.feature_name,
            date = now,
            problem = get("problem"),
            users = get("users"),
            requirements = get("requirements"),
            inputs = get("inputs"),
            outputs = get("outputs"),
            integration = get("integration"),
            risks = get("risks"),
            done = get("done"),
        )
    }

    /// SPEC.md — technical spec ready for the build phase
    fn render_spec_md(session: &AuditSession) -> String {
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();
        let get = |id: &str| -> &str {
            session
                .answers
                .iter()
                .find(|a| a.question_id == id)
                .map(|a| a.answer.as_str())
                .unwrap_or("(not answered)")
        };

        format!(
            "# SPEC — {feature}\n\n\
             **Status:** Draft\n\
             **Date:** {date}\n\
             **Phase:** Ready to build\n\n\
             ---\n\n\
             ## Overview\n\n\
             This spec was generated from the AUDIT phase for the `{feature}` feature.\n\n\
             **Problem being solved:**\n{problem}\n\n\
             ---\n\n\
             ## Functional Requirements\n\n\
             {requirements}\n\n\
             ---\n\n\
             ## Technical Design\n\n\
             ### Inputs / Triggers\n\n{inputs}\n\n\
             ### Outputs / Side Effects\n\n{outputs}\n\n\
             ### Module Integration\n\n{integration}\n\n\
             ---\n\n\
             ## Risk Register\n\n{risks}\n\n\
             ---\n\n\
             ## Definition of Done\n\n{done}\n\n\
             ---\n\n\
             ## Build Instructions\n\n\
             1. Review this spec\n\
             2. Type `/build` in Everthink IDE to start the autonomous build loop\n\
             3. The build loop will implement each requirement, run `cargo build`, and self-correct\n",
            feature = session.feature_name,
            date = now,
            problem = get("problem"),
            requirements = get("requirements"),
            inputs = get("inputs"),
            outputs = get("outputs"),
            integration = get("integration"),
            risks = get("risks"),
            done = get("done"),
        )
    }
}
