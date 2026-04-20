// storage/mod.rs — Session management, MemPalace (Wing/Room/Drawer), Decisions log
// Phase 6

use anyhow::Context;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ulid::Ulid;

// ─── Saved message ────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedMessage {
    pub role: String,
    pub content: String,
}

// ─── Session ──────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub model: String,
    pub agent: String,
    pub messages: Vec<SavedMessage>,
}

impl Session {
    pub fn new(model: &str, agent: &str) -> Self {
        Session {
            id: Ulid::new().to_string(),
            started_at: Utc::now().to_rfc3339(),
            ended_at: None,
            model: model.to_string(),
            agent: agent.to_string(),
            messages: Vec::new(),
        }
    }

    pub fn close(&mut self) {
        self.ended_at = Some(Utc::now().to_rfc3339());
    }

    /// Non-system messages only (user + assistant)
    pub fn chat_messages(&self) -> Vec<&SavedMessage> {
        self.messages
            .iter()
            .filter(|m| m.role == "user" || m.role == "assistant")
            .collect()
    }
}

// ─── SessionStore ─────────────────────────────────────────────────────────────

pub struct SessionStore {
    dir: PathBuf,
}

impl SessionStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        SessionStore { dir: dir.into() }
    }

    /// Create from `sessions/` relative to cwd.
    pub fn from_cwd() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        SessionStore::new(cwd.join("sessions"))
    }

    fn ensure_dir(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.dir)
            .with_context(|| format!("creating sessions dir {:?}", self.dir))
    }

    fn latest_pointer(&self) -> PathBuf {
        self.dir.join("latest")
    }

    /// Save a session to `sessions/{id}.json` and update `latest` pointer.
    pub fn save(&self, session: &mut Session) -> anyhow::Result<()> {
        self.ensure_dir()?;
        session.close();
        let path = self.dir.join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(session)?;
        std::fs::write(&path, &json)
            .with_context(|| format!("writing session {:?}", path))?;
        std::fs::write(self.latest_pointer(), &session.id)?;
        Ok(())
    }

    /// Load the session recorded in the `latest` pointer file.
    pub fn load_latest(&self) -> anyhow::Result<Option<Session>> {
        let ptr = self.latest_pointer();
        if !ptr.exists() {
            return Ok(None);
        }
        let id = std::fs::read_to_string(&ptr)
            .context("reading latest pointer")?
            .trim()
            .to_string();
        if id.is_empty() {
            return Ok(None);
        }
        self.load(&id).map(Some)
    }

    /// Load a session by ID.
    pub fn load(&self, id: &str) -> anyhow::Result<Session> {
        let path = self.dir.join(format!("{}.json", id));
        let json = std::fs::read_to_string(&path)
            .with_context(|| format!("reading session {:?}", path))?;
        serde_json::from_str(&json).context("parsing session JSON")
    }

    /// List all sessions (newest first by ULID which is time-ordered).
    pub fn list(&self) -> anyhow::Result<Vec<String>> {
        if !self.dir.exists() {
            return Ok(vec![]);
        }
        let mut ids: Vec<String> = std::fs::read_dir(&self.dir)?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.strip_suffix(".json").map(|s| s.to_string())
            })
            .collect();
        ids.sort_by(|a, b| b.cmp(a)); // newest first
        Ok(ids)
    }
}

// ─── MemPalace — Wing / Room / Drawer ────────────────────────────────────────
//
//  memory/
//    {wing}/
//      {room}.md    ← each room is a markdown file
//
//  A "Drawer" is a bullet-point fact inside a room.

pub struct MemPalace {
    dir: PathBuf,
}

pub struct MemResult {
    pub wing: String,
    pub room: String,
    pub line: String,
}

impl MemPalace {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        MemPalace { dir: dir.into() }
    }

    pub fn from_cwd() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        MemPalace::new(cwd.join("memory"))
    }

    fn ensure_wing(&self, wing: &str) -> anyhow::Result<PathBuf> {
        let path = self.dir.join(wing);
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    /// Write / overwrite a room (markdown file) inside a wing.
    pub fn save_room(&self, wing: &str, room: &str, content: &str) -> anyhow::Result<()> {
        let wing_dir = self.ensure_wing(wing)?;
        let path = wing_dir.join(format!("{}.md", room));
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Append a drawer (bullet fact) to a room.
    pub fn append_drawer(&self, wing: &str, room: &str, fact: &str) -> anyhow::Result<()> {
        let wing_dir = self.ensure_wing(wing)?;
        let path = wing_dir.join(format!("{}.md", room));
        let mut existing = if path.exists() {
            std::fs::read_to_string(&path)?
        } else {
            format!("# {}\n\n", room)
        };
        existing.push_str(&format!("- {}\n", fact));
        std::fs::write(path, existing)?;
        Ok(())
    }

    /// Case-insensitive search across all rooms. Returns matched lines with context.
    pub fn search(&self, query: &str) -> anyhow::Result<Vec<MemResult>> {
        if !self.dir.exists() {
            return Ok(vec![]);
        }
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for wing_entry in std::fs::read_dir(&self.dir)?.filter_map(|e| e.ok()) {
            if !wing_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let wing = wing_entry.file_name().to_string_lossy().to_string();
            for room_entry in std::fs::read_dir(wing_entry.path())?.filter_map(|e| e.ok()) {
                let name = room_entry.file_name().to_string_lossy().to_string();
                if !name.ends_with(".md") {
                    continue;
                }
                let room = name.trim_end_matches(".md").to_string();
                if let Ok(content) = std::fs::read_to_string(room_entry.path()) {
                    for line in content.lines() {
                        if line.to_lowercase().contains(&query_lower) {
                            results.push(MemResult {
                                wing: wing.clone(),
                                room: room.clone(),
                                line: line.to_string(),
                            });
                        }
                    }
                }
            }
        }
        Ok(results)
    }

    /// Read a whole room as markdown.
    pub fn read_room(&self, wing: &str, room: &str) -> anyhow::Result<String> {
        let path = self.dir.join(wing).join(format!("{}.md", room));
        std::fs::read_to_string(&path)
            .with_context(|| format!("reading room {wing}/{room}"))
    }

    /// List all wings.
    pub fn wings(&self) -> Vec<String> {
        if !self.dir.exists() { return vec![]; }
        std::fs::read_dir(&self.dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// List rooms in a wing.
    pub fn rooms(&self, wing: &str) -> Vec<String> {
        let wing_dir = self.dir.join(wing);
        if !wing_dir.exists() { return vec![]; }
        std::fs::read_dir(&wing_dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        name.strip_suffix(".md").map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ─── Decisions log ────────────────────────────────────────────────────────────

pub struct DecisionsLog {
    path: PathBuf,
}

impl DecisionsLog {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        DecisionsLog { path: path.into() }
    }

    pub fn from_cwd() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        DecisionsLog::new(cwd.join("DECISIONS.md"))
    }

    /// Append a decision entry.
    pub fn append(&self, decision: &str, reason: &str) -> anyhow::Result<()> {
        let date = Utc::now().format("%Y-%m-%d %H:%M").to_string();
        let entry = format!("| {} | {} | {} |\n", date, decision, reason);

        let mut content = if self.path.exists() {
            std::fs::read_to_string(&self.path)?
        } else {
            "# Decisions Log\n\n| Date | Decision | Reason |\n|------|----------|--------|\n".to_string()
        };

        content.push_str(&entry);
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    pub fn read(&self) -> anyhow::Result<String> {
        if !self.path.exists() {
            return Ok("(no decisions recorded yet)".to_string());
        }
        std::fs::read_to_string(&self.path).context("reading DECISIONS.md")
    }
}

// (types are used via crate::storage::{Session, SessionStore, MemPalace, ...})
