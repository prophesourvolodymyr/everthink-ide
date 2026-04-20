// tui/mod.rs — App struct, async event loop, key handling, layout

pub mod chat;
pub mod input;
pub mod status;

use chat::{ChatMessage, MessageRole};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};
use std::fmt;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::commands::review::{run_review, DiffTarget};
use crate::core::audit::AuditSession;
use crate::core::autonomous::{AutonomousBuild, AutonomousConfig};
use crate::core::spec_writer::SpecWriter;
use crate::providers::{LLMProvider, ProviderMessage, ProviderRegistry, StreamEvent};
use crate::skills::SkillsManager;
use crate::storage::{MemPalace, Session, SessionStore};
use crate::tools::verify::run_verify;
use crate::tools::fs::GrepTool;
use crate::tools::Tool;
use serde_json::json;

// ─── Compression mode (Caveman concept) ──────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CompressionMode {
    Off,
    Lite,
    Full,
    Ultra,
    Wenyan,
}

impl CompressionMode {
    pub fn label(&self) -> &str {
        match self {
            CompressionMode::Off    => "off",
            CompressionMode::Lite   => "🪶 lite",
            CompressionMode::Full   => "🪨 full",
            CompressionMode::Ultra  => "🔥 ultra",
            CompressionMode::Wenyan => "📜 wenyan",
        }
    }

    pub fn badge(&self) -> Option<&str> {
        match self {
            CompressionMode::Off    => None,
            CompressionMode::Lite   => Some("COMPRESS:LITE"),
            CompressionMode::Full   => Some("COMPRESS:FULL"),
            CompressionMode::Ultra  => Some("COMPRESS:ULTRA"),
            CompressionMode::Wenyan => Some("COMPRESS:文言文"),
        }
    }

    /// System prompt suffix injected when this mode is active.
    pub fn system_prompt(&self) -> Option<&str> {
        match self {
            CompressionMode::Off => None,
            CompressionMode::Lite => Some(
                "\n\nRESPONSE STYLE — LITE COMPRESSION: \
                Drop filler words and pleasantries. Keep full grammar. \
                No 'I would be happy to', no 'certainly', no 'just', no 'basically'. \
                Professional and direct. Same technical accuracy, no fluff."
            ),
            CompressionMode::Full => Some(
                "\n\nRESPONSE STYLE — FULL COMPRESSION (caveman): \
                Talk like caveman. Drop articles, filler, pleasantries. \
                Use fragments. Short synonyms. Pattern: [thing] [action] [reason]. [next step]. \
                Example: 'New object ref each render. Inline prop = new ref = re-render. Wrap in useMemo.' \
                Same fix. 75% less word. Brain still big. Code blocks unchanged."
            ),
            CompressionMode::Ultra => Some(
                "\n\nRESPONSE STYLE — ULTRA COMPRESSION: \
                Maximum compression. Telegraphic. Abbreviate everything safe to abbreviate. \
                Symbols over words (→, =, +). No sentences, only signal. \
                Example: 'Inline obj prop → new ref → re-render. useMemo.' \
                Code unchanged. Every word must earn its place."
            ),
            CompressionMode::Wenyan => Some(
                "\n\nRESPONSE STYLE — 文言文 (WENYAN) COMPRESSION: \
                Respond in Classical Chinese literary style. \
                Maximum token efficiency. Ancient scholar on a budget. \
                Technical terms, code, file paths, commands stay in English/original. \
                Example: '物出新參照，致重繪。useMemo Wrap之。' \
                Same accuracy. Minimum character."
            ),
        }
    }

    pub fn from_str(s: &str) -> Option<CompressionMode> {
        match s.to_lowercase().as_str() {
            "off" | "none" | "normal" => Some(CompressionMode::Off),
            "lite" | "light"          => Some(CompressionMode::Lite),
            "full"                    => Some(CompressionMode::Full),
            "ultra" | "max"           => Some(CompressionMode::Ultra),
            "wenyan" | "文言文"        => Some(CompressionMode::Wenyan),
            _                         => None,
        }
    }
}

// ─── Agent ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Agent {
    Build,
    Plan,
    General,
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Agent::Build => write!(f, "Build"),
            Agent::Plan => write!(f, "Plan"),
            Agent::General => write!(f, "General"),
        }
    }
}

// ─── Slash commands ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SlashCommand {
    pub trigger: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct SlashMode {
    pub active: bool,
    pub query: String,
    pub commands: Vec<SlashCommand>,
    pub selected: usize,
}

impl SlashMode {
    fn new() -> Self {
        let commands = vec![
            SlashCommand { trigger: "/compression".into(), description: "Set response compression: lite / full / ultra / wenyan / off".into() },
            SlashCommand { trigger: "/skills".into(),      description: "Browse / search / install skills".into() },
            SlashCommand { trigger: "/review".into(),      description: "Code review + blast radius [staged|head|<branch>]".into() },
            SlashCommand { trigger: "/build".into(),       description: "Build current feature".into() },
            SlashCommand { trigger: "/add".into(),      description: "Add a new feature (AUDIT phase)".into() },
            SlashCommand { trigger: "/cancel".into(),   description: "Cancel current AUDIT session".into() },
            SlashCommand { trigger: "/verify".into(),   description: "Run cargo clippy + build + test".into() },
            SlashCommand { trigger: "/clear".into(),    description: "Clear chat history".into() },
            SlashCommand { trigger: "/commit".into(),   description: "AI-generated git commit [optional message override]".into() },
            SlashCommand { trigger: "/search".into(),   description: "Search codebase with Greppy".into() },
            SlashCommand { trigger: "/remember".into(), description: "Load topic from memory".into() },
            SlashCommand { trigger: "/agent".into(),    description: "Switch active agent".into() },
            SlashCommand { trigger: "/model".into(),    description: "List or switch LLM provider: /model [provider-id]".into() },
            SlashCommand { trigger: "/context".into(),  description: "Show context stats (messages, token estimate)".into() },
            SlashCommand { trigger: "/session".into(),  description: "Manage sessions: list / save / load <id>".into() },
            SlashCommand { trigger: "/help".into(),     description: "Show all commands".into() },
        ];
        SlashMode { active: false, query: String::new(), commands, selected: 0 }
    }

    pub fn filtered(&self) -> Vec<&SlashCommand> {
        if self.query.is_empty() || self.query == "/" {
            return self.commands.iter().collect();
        }
        self.commands
            .iter()
            .filter(|c| c.trigger.contains(self.query.as_str()))
            .collect()
    }
}

// ─── App ─────────────────────────────────────────────────────────────────────

pub struct App {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub input_cursor: usize,
    pub scroll_offset: usize,
    pub agent: Agent,
    pub slash_mode: SlashMode,
    pub quit: bool,
    pub model: String,
    pub token_count: u32,

    // Phase 3: provider + streaming
    pub provider: Arc<dyn LLMProvider>,
    pub stream_rx: Option<mpsc::UnboundedReceiver<StreamEvent>>,
    pub is_streaming: bool,
    pub streaming_msg_idx: Option<usize>,
    pub system_prompt: String,

    // Phase 4: AUDIT session
    pub audit_session: Option<AuditSession>,

    // Phase 6: session persistence + memory
    pub session: Session,
    pub session_store: SessionStore,
    pub mem_palace: MemPalace,

    // Compression mode (Caveman concept)
    pub compression_mode: CompressionMode,

    // Phase 8: Skills system
    pub skills: SkillsManager,

    // Phase 10: provider registry for live model switching
    pub registry: ProviderRegistry,
}

impl App {
    fn new(provider: Arc<dyn LLMProvider>, registry: ProviderRegistry, config_diag: String) -> Self {
        let model = format!("{}/{}", provider.id(), provider.model());
        let session = Session::new(&model, "General");
        App {
            messages: vec![ChatMessage {
                role: MessageRole::System,
                content: format!(
                    "Welcome to Everthink IDE. Type a message or / for commands.\n[debug] {config_diag}"
                ),
            }],
            input: String::new(),
            input_cursor: 0,
            scroll_offset: 0,
            agent: Agent::General,
            slash_mode: SlashMode::new(),
            quit: false,
            model,
            token_count: 0,
            provider,
            stream_rx: None,
            is_streaming: false,
            streaming_msg_idx: None,
            system_prompt: "You are Everthink IDE, an expert AI coding assistant. \
                You help users design, build, and debug software efficiently. \
                Be concise and precise. Use code blocks for code examples."
                .into(),
            audit_session: None,
            session,
            session_store: SessionStore::from_cwd(),
            mem_palace: MemPalace::from_cwd(),
            compression_mode: CompressionMode::Off,
            skills: SkillsManager::load(
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
            ),
            registry,
        }
    }

    // ── Message helpers ───────────────────────────────────────────────────────

    fn push_system(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: MessageRole::System,
            content: content.into(),
        });
    }

    fn push_user(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: content.into(),
        });
    }

    // ── Provider helpers ──────────────────────────────────────────────────────

    /// Build the message list to send to the LLM.
    /// Starts with the system prompt, then includes only User/Assistant messages.
    fn build_provider_messages(&self) -> Vec<ProviderMessage> {
        // Combine base system prompt with active compression style
        let system = match self.compression_mode.system_prompt() {
            Some(suffix) => format!("{}{}", self.system_prompt, suffix),
            None => self.system_prompt.clone(),
        };

        let mut msgs = vec![ProviderMessage {
            role: "system".into(),
            content: system,
        }];
        msgs.extend(
            self.messages
                .iter()
                .filter(|m| !matches!(m.role, MessageRole::System))
                .filter(|m| !m.content.is_empty()) // skip empty streaming placeholder
                .map(|m| ProviderMessage {
                    role: match m.role {
                        MessageRole::User => "user".into(),
                        MessageRole::Assistant => "assistant".into(),
                        MessageRole::System => "system".into(),
                    },
                    content: m.content.clone(),
                }),
        );
        msgs
    }

    // ── Session save ──────────────────────────────────────────────────────────

    fn save_session(&mut self) {
        // Snapshot non-system messages into the persistent session
        self.session.messages = self
            .messages
            .iter()
            .filter(|m| !matches!(m.role, MessageRole::System))
            .map(|m| crate::storage::SavedMessage {
                role: match m.role {
                    MessageRole::User => "user".into(),
                    MessageRole::Assistant => "assistant".into(),
                    MessageRole::System => "system".into(),
                },
                content: m.content.clone(),
            })
            .collect();
        self.session.agent = self.agent.to_string();
        if let Err(e) = self.session_store.save(&mut self.session) {
            // Non-fatal — can't push a system message at quit time
            eprintln!("[everthink] session save failed: {e}");
        }
    }

    // ── Compression ───────────────────────────────────────────────────────────

    fn set_compression(&mut self, mode_str: String) {
        if mode_str.is_empty() {
            // Show current mode + options
            self.push_system(format!(
                "Compression: {} (current)\n\
                 \n\
                 Modes:\n\
                 /compression lite    — 🪶 drop filler, keep grammar\n\
                 /compression full    — 🪨 caveman fragments, no articles\n\
                 /compression ultra   — 🔥 telegraphic, symbols over words\n\
                 /compression wenyan  — 📜 classical Chinese (文言文)\n\
                 /compression off     — normal mode",
                self.compression_mode.label()
            ));
            return;
        }

        match CompressionMode::from_str(&mode_str) {
            Some(mode) => {
                let label = mode.label().to_string();
                self.compression_mode = mode;
                if self.compression_mode == CompressionMode::Off {
                    self.push_system("Compression off. Normal mode restored.");
                } else {
                    self.push_system(format!(
                        "Compression set to {}. All responses will now be compressed.\n\
                         Same answer. {} less word. Brain still big.",
                        label,
                        match &self.compression_mode {
                            CompressionMode::Lite   => "~30%",
                            CompressionMode::Full   => "~75%",
                            CompressionMode::Ultra  => "~85%",
                            CompressionMode::Wenyan => "~70%",
                            CompressionMode::Off    => "0%",
                        }
                    ));
                }
            }
            None => {
                self.push_system(format!(
                    "Unknown mode '{}'. Use: lite / full / ultra / wenyan / off",
                    mode_str
                ));
            }
        }
    }

    // ── Search ────────────────────────────────────────────────────────────────

    fn run_search(&mut self, query: String) {
        if self.is_streaming {
            self.push_system("[search] Busy — wait for current operation to finish.");
            return;
        }
        self.push_user(format!("/search {}", query));

        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: String::new(),
        });
        self.streaming_msg_idx = Some(self.messages.len() - 1);
        let (tx, rx) = mpsc::unbounded_channel::<StreamEvent>();
        self.stream_rx = Some(rx);
        self.is_streaming = true;

        tokio::spawn(async move {
            let _ = tx.send(StreamEvent::Token(format!(
                "Searching codebase for: {}\n\n",
                query
            )));
            let tool = GrepTool;
            match tool.run(json!({"pattern": query, "path": "."})).await {
                Ok(result) => {
                    let _ = tx.send(StreamEvent::Token(result.output));
                }
                Err(e) => {
                    let _ = tx.send(StreamEvent::Token(format!("Error: {e}")));
                }
            }
            let _ = tx.send(StreamEvent::Done);
        });
    }

    fn run_remember(&mut self, topic: String) {
        if self.is_streaming {
            self.push_system("[remember] Busy — wait for current operation to finish.");
            return;
        }
        self.push_user(format!("/remember {}", topic));

        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: String::new(),
        });
        self.streaming_msg_idx = Some(self.messages.len() - 1);
        let (tx, rx) = mpsc::unbounded_channel::<StreamEvent>();
        self.stream_rx = Some(rx);
        self.is_streaming = true;

        let mem = MemPalace::from_cwd();
        tokio::spawn(async move {
            let _ = tx.send(StreamEvent::Token(format!(
                "Searching memory for: {}\n\n",
                topic
            )));
            match mem.search(&topic) {
                Ok(results) if results.is_empty() => {
                    let _ = tx.send(StreamEvent::Token(
                        "(no memory entries found for that topic)".into(),
                    ));
                }
                Ok(results) => {
                    let text = results
                        .iter()
                        .map(|r| format!("[{}/{}] {}", r.wing, r.room, r.line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let _ = tx.send(StreamEvent::Token(text));
                }
                Err(e) => {
                    let _ = tx.send(StreamEvent::Token(format!("Error: {e}")));
                }
            }
            let _ = tx.send(StreamEvent::Done);
        });
    }

    // ── Send message ──────────────────────────────────────────────────────────

    fn send_message(&mut self) {
        if self.is_streaming {
            return;
        }
        let content = self.input.trim().to_string();
        if content.is_empty() {
            return;
        }

        // ── Slash command typed directly (e.g. /search fn main) ──────────────
        if content.starts_with('/') && !content.contains('\n') {
            self.input.clear();
            self.input_cursor = 0;
            if content.starts_with("/search ") {
                let query = content["/search ".len()..].trim().to_string();
                if query.is_empty() {
                    self.push_system("Usage: /search <pattern>");
                } else {
                    self.run_search(query);
                }
            } else if content.starts_with("/remember ") {
                let topic = content["/remember ".len()..].trim().to_string();
                if topic.is_empty() {
                    self.push_system("Usage: /remember <topic>");
                } else {
                    self.run_remember(topic);
                }
            } else if content.starts_with("/compression") {
                let mode_str = content["/compression".len()..].trim().to_string();
                self.set_compression(mode_str);
            } else if content.starts_with("/skills") {
                let args = content["/skills".len()..].trim().to_string();
                self.run_skills(args);
            } else if content.starts_with("/review") {
                let args = content["/review".len()..].trim().to_string();
                self.run_review(args);
            } else if content.starts_with("/commit") {
                let msg = content["/commit".len()..].trim().to_string();
                self.run_commit(msg);
            } else if content.starts_with("/model") {
                let args = content["/model".len()..].trim().to_string();
                self.run_model(args);
            } else if content.starts_with("/context") {
                let text = self.run_context();
                self.push_system(text);
            } else if content.starts_with("/session") {
                let args = content["/session".len()..].trim().to_string();
                self.run_session(args);
            } else {
                let trigger = content
                    .split_whitespace()
                    .next()
                    .unwrap_or(&content)
                    .to_string();
                self.execute_slash(&trigger);
            }
            return;
        }

        // ── AUDIT mode: route input as an answer ──────────────────────────────
        if self.audit_session.is_some() {
            // Allow /cancel to abort
            if content == "/cancel" {
                if let Some(ref mut s) = self.audit_session {
                    s.cancel();
                }
                self.audit_session = None;
                self.input.clear();
                self.input_cursor = 0;
                self.push_system("[AUDIT] Cancelled.");
                return;
            }

            self.push_user(content.clone());
            self.input.clear();
            self.input_cursor = 0;

            let next_msg = if let Some(ref mut session) = self.audit_session {
                match session.submit_answer(&content) {
                    Some((progress, text, hint)) => {
                        Some(Some(format!(
                            "[AUDIT] Question {}: {}\nHint: {}",
                            progress, text, hint,
                        )))
                    }
                    None => Some(None), // complete
                }
            } else {
                None
            };

            match next_msg {
                Some(Some(msg)) => {
                    self.push_system(msg);
                }
                Some(None) => {
                    // Session complete — write files
                    let session = self.audit_session.take().unwrap();
                    let dir = SpecWriter::default_project_dir();
                    match SpecWriter::write_all(&session, &dir) {
                        Ok(paths) => {
                            self.push_system(AuditSession::completion_message(&paths));
                        }
                        Err(e) => {
                            self.push_system(format!("[AUDIT] Error writing files: {e}"));
                        }
                    }
                }
                None => {}
            }
            return;
        }

        // ── Normal LLM mode ───────────────────────────────────────────────────
        self.push_user(content);
        self.input.clear();
        self.input_cursor = 0;
        self.scroll_offset = 0;

        let provider_msgs = self.build_provider_messages();

        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: String::new(),
        });
        self.streaming_msg_idx = Some(self.messages.len() - 1);

        let (tx, rx) = mpsc::unbounded_channel::<StreamEvent>();
        self.stream_rx = Some(rx);
        self.is_streaming = true;

        let provider = self.provider.clone();
        tokio::spawn(async move {
            if let Err(e) = provider.complete(&provider_msgs, tx.clone()).await {
                let _ = tx.send(StreamEvent::Error(e.to_string()));
            }
        });
    }

    // ── Agent cycling ─────────────────────────────────────────────────────────

    fn cycle_agent(&mut self) {
        self.agent = match self.agent {
            Agent::Build => Agent::Plan,
            Agent::Plan => Agent::General,
            Agent::General => Agent::Build,
        };
        self.push_system(format!("Agent switched to: {}", self.agent));
    }

    // ── Skills command ────────────────────────────────────────────────────────

    /// Handle `/skills [subcommand] [args]`
    ///
    /// Subcommands:
    ///   (none / list)         — list favorites, then installed
    ///   search <query>        — search all three tiers
    ///   install <name>        — install a library skill (creates stub SKILL.md)
    ///   fav <name>            — toggle favorite on an installed skill
    ///   unfav <name>          — remove from favorites
    ///   status                — show counts summary
    fn run_skills(&mut self, args: String) {
        let mut parts = args.splitn(2, char::is_whitespace);
        let sub = parts.next().unwrap_or("").trim();
        let rest = parts.next().unwrap_or("").trim().to_string();

        match sub {
            "" | "list" => {
                let msg = self.skills.display_list("");
                self.push_system(msg);
            }
            "search" => {
                if rest.is_empty() {
                    self.push_system("Usage: /skills search <query>");
                    return;
                }
                let msg = self.skills.display_list(&rest);
                self.push_system(msg);
            }
            "install" => {
                if rest.is_empty() {
                    self.push_system("Usage: /skills install <name>");
                    return;
                }
                // Find in library
                let entry = self.skills.library.iter().find(|e| e.name == rest).cloned();
                let content = match entry {
                    Some(ref e) => format!(
                        "# Skill: {}\n\nSource: {}\nPurpose: {}\n\n## Instructions\n\n\
                         [Adapt this template for your project.]\n",
                        e.name, e.source, e.purpose
                    ),
                    None => format!(
                        "# Skill: {rest}\n\nSource: local\n\n## Instructions\n\n\
                         [Describe what this skill does.]\n"
                    ),
                };
                match self.skills.install(&rest, &content) {
                    Ok(()) => self.push_system(format!(
                        "Skill '{}' installed to .skills/installed/{}/SKILL.md\n\
                         Edit that file to customise it, then use `/skills fav {}` to favourite it.",
                        rest, rest, rest
                    )),
                    Err(e) => self.push_system(format!("[skills] install error: {e}")),
                }
            }
            "fav" | "favorite" => {
                if rest.is_empty() {
                    self.push_system("Usage: /skills fav <name>");
                    return;
                }
                match self.skills.favorite(&rest) {
                    Ok(()) => self.push_system(format!("⭐ '{}' added to favorites.", rest)),
                    Err(e) => self.push_system(format!("[skills] {e}")),
                }
            }
            "unfav" | "unfavorite" => {
                if rest.is_empty() {
                    self.push_system("Usage: /skills unfav <name>");
                    return;
                }
                match self.skills.unfavorite(&rest) {
                    Ok(()) => self.push_system(format!("'{}' removed from favorites.", rest)),
                    Err(e) => self.push_system(format!("[skills] {e}")),
                }
            }
            "status" => {
                self.push_system(format!("[skills] {}", self.skills.status_summary()));
            }
            other => {
                self.push_system(format!(
                    "Unknown skills subcommand: '{}'\n\
                     Usage: /skills [list | search <q> | install <name> | fav <name> | unfav <name> | status]",
                    other
                ));
            }
        }
    }

    // ── Review command ────────────────────────────────────────────────────────

    /// Handle `/review [staged|head|<branch>]`
    ///
    /// 1. Runs `git diff` synchronously to get changed files + raw diff.
    /// 2. Computes blast radius (which files import the changed modules).
    /// 3. Displays the summary immediately in chat.
    /// 4. Streams an AI code review to the chat pane using the existing provider.
    fn run_review(&mut self, args: String) {
        if self.is_streaming {
            self.push_system("[review] Busy — wait for the current operation to finish.");
            return;
        }

        let target = DiffTarget::from_str(args.trim());
        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        // Step 1: run review pipeline synchronously (all git + grep, no LLM yet)
        let result = match run_review(&cwd, target) {
            Ok(r) => r,
            Err(e) => {
                self.push_system(format!("[review] Error: {e}\n\nMake sure you are inside a git repository."));
                return;
            }
        };

        // Step 2: show blast-radius summary immediately
        self.push_system(result.format_summary());

        // If nothing changed, stop here — no point calling the LLM
        if result.changed.is_empty() {
            return;
        }

        // Step 3: stream AI review
        let prompt = result.build_review_prompt();
        let system = self.build_provider_messages(); // use compression-aware system
        let provider = self.provider.clone();

        self.messages.push(crate::tui::chat::ChatMessage {
            role: crate::tui::chat::MessageRole::Assistant,
            content: String::new(),
        });
        self.streaming_msg_idx = Some(self.messages.len() - 1);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<crate::providers::StreamEvent>();
        self.stream_rx = Some(rx);
        self.is_streaming = true;

        tokio::spawn(async move {
            // Build a minimal message list: system + the review prompt as "user"
            let mut msgs = system;
            // Remove any prior user/assistant turns — we only want the review prompt
            msgs.retain(|m| m.role == "system");
            msgs.push(crate::providers::ProviderMessage {
                role: "user".into(),
                content: prompt,
            });

            if let Err(e) = provider.complete(&msgs, tx.clone()).await {
                let _ = tx.send(crate::providers::StreamEvent::Token(
                    format!("\n[review] LLM error: {e}")
                ));
                let _ = tx.send(crate::providers::StreamEvent::Done);
            }
        });
    }

    // ── Slash command dispatch ────────────────────────────────────────────────

    /// `/commit [message]`
    ///
    /// 1. Checks for staged changes; if none, runs `git add -A` first.
    /// 2. Grabs `git diff --cached` as context.
    /// 3. Asks the LLM to write a concise conventional commit message.
    /// 4. Runs `git commit -m <message>` and shows the result.
    fn run_commit(&mut self, user_msg: String) {
        if self.is_streaming {
            self.push_system("[commit] Busy — wait for the current operation to finish.");
            return;
        }

        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        // Quick git check to see if we're in a repo
        use std::process::Command;
        let in_repo = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(&cwd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !in_repo {
            self.push_system("[commit] Not inside a git repository.");
            return;
        }

        // If user provided a message, skip LLM and commit directly
        if !user_msg.is_empty() {
            let staged = Command::new("git")
                .args(["diff", "--cached", "--name-only"])
                .current_dir(&cwd)
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();

            if staged.is_empty() {
                // Auto-stage everything
                let _ = Command::new("git").args(["add", "-A"]).current_dir(&cwd).output();
            }

            match Command::new("git")
                .args(["commit", "-m", &user_msg])
                .current_dir(&cwd)
                .output()
            {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let combined = format!("{}{}", stdout, stderr).trim().to_string();
                    self.push_system(format!("[commit] {}", combined));
                }
                Err(e) => self.push_system(format!("[commit] Error: {e}")),
            }
            return;
        }

        // No user message — ask LLM to generate one
        // Check/auto-stage
        let staged_check = Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(&cwd)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        if staged_check.is_empty() {
            let _ = Command::new("git").args(["add", "-A"]).current_dir(&cwd).output();
            self.push_system("[commit] Nothing staged — ran `git add -A`.");
        }

        // Get the staged diff for LLM context (truncated)
        let diff = Command::new("git")
            .args(["diff", "--cached"])
            .current_dir(&cwd)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        const MAX_DIFF: usize = 8_000;
        let diff_snippet = if diff.len() > MAX_DIFF {
            format!("{}\n\n[... truncated ...]", &diff[..MAX_DIFF])
        } else {
            diff.clone()
        };

        if diff_snippet.trim().is_empty() {
            self.push_system("[commit] Nothing to commit — working tree clean.");
            return;
        }

        let prompt = format!(
            "Write a concise conventional commit message for these staged changes.\n\
             Rules:\n\
             - One line only (max 72 chars)\n\
             - Format: <type>(<scope>): <description>   e.g. feat(tui): add /commit command\n\
             - Types: feat / fix / refactor / chore / docs / test / style\n\
             - No body, no footer\n\
             - Output ONLY the commit message, nothing else\n\n\
             DIFF:\n```diff\n{diff_snippet}\n```"
        );

        let provider = self.provider.clone();
        let cwd_clone = cwd.clone();

        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: String::new(),
        });
        self.streaming_msg_idx = Some(self.messages.len() - 1);
        let (tx, rx) = mpsc::unbounded_channel::<StreamEvent>();
        self.stream_rx = Some(rx);
        self.is_streaming = true;

        tokio::spawn(async move {
            let msgs = vec![
                ProviderMessage {
                    role: "system".into(),
                    content: "You are a git commit message writer. Output only the commit message, nothing else.".into(),
                },
                ProviderMessage { role: "user".into(), content: prompt },
            ];

            // Spawn the provider completion into its own task so we can drive it
            let (buf_tx, mut buf_rx) = mpsc::unbounded_channel::<StreamEvent>();
            let prov = provider.clone();
            let msgs_clone = msgs.clone();
            tokio::spawn(async move {
                let _ = prov.complete(&msgs_clone, buf_tx).await;
            });

            // Drain streaming tokens — show them in TUI AND collect for git commit
            let mut commit_msg = String::new();
            loop {
                match buf_rx.recv().await {
                    Some(StreamEvent::Token(t)) => {
                        commit_msg.push_str(&t);
                        let _ = tx.send(StreamEvent::Token(t));
                    }
                    Some(StreamEvent::Done) | None => break,
                    Some(StreamEvent::Error(e)) => {
                        let _ = tx.send(StreamEvent::Token(format!("\n\n[commit] LLM error: {e}")));
                        let _ = tx.send(StreamEvent::Done);
                        return;
                    }
                }
            }

            let commit_msg = commit_msg.trim().trim_matches('"').trim_matches('`').trim().to_string();

            if commit_msg.is_empty() {
                let _ = tx.send(StreamEvent::Token("\n\n[commit] LLM returned empty message — commit aborted.".into()));
                let _ = tx.send(StreamEvent::Done);
                return;
            }

            // Run git commit
            let result = std::process::Command::new("git")
                .args(["commit", "-m", &commit_msg])
                .current_dir(&cwd_clone)
                .output();

            let status_msg = match result {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    format!("\n\n[commit] {}", format!("{}{}", stdout, stderr).trim())
                }
                Err(e) => format!("\n\n[commit] Error running git: {e}"),
            };

            let _ = tx.send(StreamEvent::Token(status_msg));
            let _ = tx.send(StreamEvent::Done);
        });
    }

    /// `/model [provider]`  — show available providers or switch live.
    ///
    /// - `/model`          → list available providers + current active
    /// - `/model anthropic` → switch to anthropic provider
    /// - `/model openai`    → switch to openai
    /// - `/model openrouter`→ switch to openrouter
    fn run_model(&mut self, args: String) {
        let arg = args.trim().to_string();

        if arg.is_empty() {
            // List all providers
            let names: Vec<String> = self.registry.list()
                .iter()
                .map(|p| {
                    let marker = if p.id() == self.provider.id() { " ← active" } else { "" };
                    format!("  {}/{}{}", p.id(), p.model(), marker)
                })
                .collect();
            let msg = format!(
                "/model — available providers:\n{}\n\nUsage: /model <provider-id>",
                names.join("\n")
            );
            self.push_system(msg);
            return;
        }

        // Try switching
        match self.registry.get(&arg) {
            Some(p) => {
                let label = format!("{}/{}", p.id(), p.model());
                self.provider = p;
                self.model = label.clone();
                self.push_system(format!("[model] Switched to {label}"));
            }
            None => {
                let ids: Vec<String> = self.registry.list().iter().map(|p| p.id().to_string()).collect();
                self.push_system(format!(
                    "[model] Unknown provider '{arg}'. Available: {}",
                    ids.join(", ")
                ));
            }
        }
    }

    /// `/context` — show current context stats.
    fn run_context(&self) -> String {
        let user_msgs = self.messages.iter().filter(|m| m.role == MessageRole::User).count();
        let asst_msgs = self.messages.iter().filter(|m| m.role == MessageRole::Assistant).count();
        let total_chars: usize = self.messages.iter().map(|m| m.content.len()).sum();
        // Rough token estimate: 1 token ≈ 4 chars
        let est_tokens = total_chars / 4;

        format!(
            "── Context ──\n\
             Messages : {total}  (user: {user}, assistant: {asst})\n\
             Est. tokens : ~{est}\n\
             Compression : {comp}\n\
             Provider    : {model}\n\
             Agent       : {agent}",
            total = self.messages.len(),
            user  = user_msgs,
            asst  = asst_msgs,
            est   = est_tokens,
            comp  = self.compression_mode.label(),
            model = self.model,
            agent = self.agent,
        )
    }

    /// `/session [list|save|load <id>]`
    fn run_session(&mut self, args: String) {
        let arg = args.trim().to_string();

        match arg.as_str() {
            "" | "list" => {
                let store = SessionStore::from_cwd();
                match store.list() {
                    Ok(ids) if ids.is_empty() => {
                        self.push_system("[session] No saved sessions found.\nUse /session save to save current session.");
                    }
                    Ok(ids) => {
                        let lines = ids.iter().enumerate()
                            .map(|(i, id)| format!("  [{}] {}", i + 1, id))
                            .collect::<Vec<_>>()
                            .join("\n");
                        self.push_system(format!("[session] Saved sessions:\n{lines}\n\nUse: /session load <id>"));
                    }
                    Err(e) => self.push_system(format!("[session] Error listing sessions: {e}")),
                }
            }
            "save" => {
                let mut session = self.session.clone();
                for msg in &self.messages {
                    session.messages.push(crate::storage::SavedMessage {
                        role: match msg.role {
                            MessageRole::User => "user".into(),
                            MessageRole::Assistant => "assistant".into(),
                            MessageRole::System => "system".into(),
                        },
                        content: msg.content.clone(),
                    });
                }
                let store = SessionStore::from_cwd();
                match store.save(&mut session) {
                    Ok(()) => self.push_system(format!("[session] Saved session '{}'.", session.id)),
                    Err(e) => self.push_system(format!("[session] Error saving: {e}")),
                }
            }
            other if other.starts_with("load ") => {
                let id = other[5..].trim();
                let store = SessionStore::from_cwd();
                match store.load(id) {
                    Ok(saved) => {
                        self.messages.clear();
                        for msg in &saved.messages {
                            let role = match msg.role.as_str() {
                                "user" => MessageRole::User,
                                "assistant" => MessageRole::Assistant,
                                _ => MessageRole::System,
                            };
                            self.messages.push(ChatMessage { role, content: msg.content.clone() });
                        }
                        self.scroll_offset = 0;
                        self.push_system(format!("[session] Loaded '{}' — {} messages.", id, self.messages.len()));
                    }
                    Err(e) => self.push_system(format!("[session] Error loading '{id}': {e}")),
                }
            }
            _ => {
                self.push_system("[session] Usage:\n  /session list\n  /session save\n  /session load <id>");
            }
        }
    }

    fn execute_slash(&mut self, trigger: &str) {
        match trigger {
            "/compression" => {
                self.set_compression(String::new());
            }
            "/skills" => {
                self.run_skills(String::new());
            }
            "/review" => {
                self.run_review(String::new());
            }
            "/clear" => {
                self.messages.clear();
                self.scroll_offset = 0;
            }
            "/help" => {
                self.push_system(
                    "Commands: /build /add /verify /review /skills /clear /commit /search /remember /agent /model /context /session /compression /help",
                );
            }
            "/build" => {
                if self.is_streaming {
                    self.push_system("[build] Busy — wait for current operation to finish.");
                    return;
                }

                self.push_user("/build");
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: String::new(),
                });
                self.streaming_msg_idx = Some(self.messages.len() - 1);
                let (stream_tx, rx) = mpsc::unbounded_channel::<StreamEvent>();
                self.stream_rx = Some(rx);
                self.is_streaming = true;

                let provider = self.provider.clone();
                tokio::spawn(async move {
                    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();
                    let build = AutonomousBuild::new(AutonomousConfig::default(), provider);

                    tokio::spawn(async move {
                        if let Err(e) = build.run_all(out_tx).await {
                            eprintln!("[build] error: {e}");
                        }
                    });

                    // Forward String output → StreamEvent::Token for TUI
                    while let Some(line) = out_rx.recv().await {
                        let _ = stream_tx.send(StreamEvent::Token(line));
                    }
                    let _ = stream_tx.send(StreamEvent::Done);
                });
            }
            "/add" => {
                if self.audit_session.is_some() {
                    self.push_system("[AUDIT] An AUDIT session is already active. Type /cancel to abort it first.");
                } else {
                    let session = AuditSession::new("new feature");
                    let msg = session.opening_message();
                    self.audit_session = Some(session);
                    self.push_system(msg);
                }
            }
            "/verify" => {
                if self.is_streaming {
                    self.push_system("[verify] Busy — wait for current operation to finish.");
                    return;
                }
                let cwd = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .to_string_lossy()
                    .to_string();

                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: String::new(),
                });
                self.streaming_msg_idx = Some(self.messages.len() - 1);
                let (tx, rx) = mpsc::unbounded_channel::<StreamEvent>();
                self.stream_rx = Some(rx);
                self.is_streaming = true;

                tokio::spawn(async move {
                    let _ = tx.send(StreamEvent::Token(
                        "Running cargo clippy + build + test...\n\n".into(),
                    ));
                    match run_verify(&cwd).await {
                        Ok(report) => {
                            let _ = tx.send(StreamEvent::Token(report.output));
                        }
                        Err(e) => {
                            let _ = tx.send(StreamEvent::Token(format!("Error: {e}")));
                        }
                    }
                    let _ = tx.send(StreamEvent::Done);
                });
            }
            "/commit" => {
                self.run_commit(String::new());
            }
            "/search" => {
                self.push_system("Usage: type  /search <pattern>  then press Enter to search the codebase.");
            }
            "/remember" => {
                self.push_system("Usage: type  /remember <topic>  then press Enter to recall from memory.");
            }
            "/agent" => {
                self.cycle_agent();
            }
            "/cancel" => {
                if let Some(ref mut s) = self.audit_session {
                    s.cancel();
                    self.audit_session = None;
                    self.push_system("[AUDIT] Cancelled.");
                } else {
                    self.push_system("No active AUDIT session to cancel.");
                }
            }
            "/model" => {
                self.run_model(String::new());
            }
            "/context" => {
                let text = self.run_context();
                self.push_system(text);
            }
            "/session" => {
                self.run_session(String::new());
            }
            other => {
                self.push_system(format!("Unknown command: {}", other));
            }
        }
    }
}

// ─── Stream token handler ─────────────────────────────────────────────────────

fn handle_stream_event(app: &mut App, event: StreamEvent) {
    match event {
        StreamEvent::Token(s) => {
            if let Some(idx) = app.streaming_msg_idx {
                if let Some(msg) = app.messages.get_mut(idx) {
                    msg.content.push_str(&s);
                    app.token_count += 1;
                }
            }
        }
        StreamEvent::Done => {
            app.stream_rx = None;
            app.is_streaming = false;
            app.streaming_msg_idx = None;
        }
        StreamEvent::Error(e) => {
            app.stream_rx = None;
            app.is_streaming = false;
            app.streaming_msg_idx = None;
            // Remove empty placeholder if it exists
            if let Some(idx) = app.streaming_msg_idx {
                if app.messages.get(idx).map(|m| m.content.is_empty()).unwrap_or(false) {
                    app.messages.remove(idx);
                }
            }
            app.push_system(format!("Error: {}", e));
        }
    }
}

// ─── Async receiver helper ─────────────────────────────────────────────────────

/// Awaits the next stream token or stays pending forever when no stream is active.
async fn recv_stream_token(
    rx: &mut Option<mpsc::UnboundedReceiver<StreamEvent>>,
) -> Option<StreamEvent> {
    match rx {
        Some(rx) => rx.recv().await,
        None => std::future::pending().await,
    }
}

// ─── Entry point (async) ──────────────────────────────────────────────────────

pub async fn run() -> anyhow::Result<()> {
    // Load config + build provider
    let config = crate::config::Config::load().unwrap_or_default();
    let _ = crate::config::Config::write_sample_if_missing();
    let diag = config.diagnostic();
    let registry = crate::providers::ProviderRegistry::new(&config);
    let provider = registry.default_provider();

    let mut terminal = ratatui::init();
    let mut app = App::new(provider, registry, diag);
    let mut event_stream = EventStream::new();

    loop {
        terminal.draw(|frame| ui(frame, &app))?;

        tokio::select! {
            // Keyboard / mouse events
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => handle_key(&mut app, key),
                    Some(Err(e)) => {
                        ratatui::restore();
                        return Err(e.into());
                    }
                    _ => {}
                }
            }
            // Streaming tokens from provider
            maybe_token = recv_stream_token(&mut app.stream_rx) => {
                if let Some(event) = maybe_token {
                    handle_stream_event(&mut app, event);
                }
            }
            // Periodic tick (cursor blink, status refresh)
            _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {}
        }

        if app.quit {
            break;
        }
    }

    // Auto-save session before exit
    app.save_session();

    ratatui::restore();
    Ok(())
}

/// Launch TUI pre-populated with messages from a saved session.
pub async fn run_with_session(session: crate::storage::Session) -> anyhow::Result<()> {
    let config = crate::config::Config::load().unwrap_or_default();
    let _ = crate::config::Config::write_sample_if_missing();
    let diag = config.diagnostic();
    let registry = crate::providers::ProviderRegistry::new(&config);
    let provider = registry.default_provider();

    let mut terminal = ratatui::init();
    let mut app = App::new(provider, registry, diag);

    // Restore saved messages
    for msg in &session.messages {
        match msg.role.as_str() {
            "user" => app.messages.push(ChatMessage {
                role: MessageRole::User,
                content: msg.content.clone(),
            }),
            "assistant" => app.messages.push(ChatMessage {
                role: MessageRole::Assistant,
                content: msg.content.clone(),
            }),
            _ => {}
        }
    }
    app.push_system(format!(
        "[session restored: {} — {} messages]",
        session.id,
        session.messages.len()
    ));

    let mut event_stream = EventStream::new();
    loop {
        terminal.draw(|frame| ui(frame, &app))?;

        tokio::select! {
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => handle_key(&mut app, key),
                    Some(Err(e)) => {
                        ratatui::restore();
                        return Err(e.into());
                    }
                    _ => {}
                }
            }
            maybe_token = recv_stream_token(&mut app.stream_rx) => {
                if let Some(event) = maybe_token {
                    handle_stream_event(&mut app, event);
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {}
        }

        if app.quit {
            break;
        }
    }

    app.save_session();
    ratatui::restore();
    Ok(())
}

// ─── Layout ──────────────────────────────────────────────────────────────────

fn ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // status bar
        Constraint::Min(0),    // chat pane
        Constraint::Length(3), // input bar
    ])
    .split(frame.area());

    status::render(frame, chunks[0], app);
    chat::render(frame, chunks[1], app);
    input::render(frame, chunks[2], app);

    // Slash popup overlay — rendered on top
    if app.slash_mode.active {
        let filtered = app.slash_mode.filtered();
        if !filtered.is_empty() {
            let popup_height = (filtered.len() as u16 + 2).min(12);
            let popup_y = chunks[2].y.saturating_sub(popup_height);
            let popup_rect = Rect {
                x: chunks[2].x,
                y: popup_y,
                width: chunks[2].width.min(64),
                height: popup_height,
            };
            render_slash_popup(frame, popup_rect, app, &filtered);
        }
    }
}

fn render_slash_popup<'a>(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    filtered: &[&'a SlashCommand],
) {
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let is_selected = i == app.slash_mode.selected;
            let trigger_style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::Cyan)
            };
            let desc_style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<18}", cmd.trigger), trigger_style),
                Span::styled(format!("  {}", cmd.description), desc_style),
            ]))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Commands (↑↓ navigate, Enter select, Esc close) ");

    frame.render_widget(List::new(items).block(block), area);
}

// ─── Key handling ─────────────────────────────────────────────────────────────

fn handle_key(app: &mut App, key: KeyEvent) {
    match (key.modifiers, key.code) {
        // ── Global ────────────────────────────────────────────────────────
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            app.quit = true;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
            app.messages.clear();
            app.scroll_offset = 0;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
            app.execute_slash("/commit");
        }

        // ── Escape ────────────────────────────────────────────────────────
        (_, KeyCode::Esc) => {
            if app.slash_mode.active {
                app.slash_mode.active = false;
                app.slash_mode.query.clear();
                app.slash_mode.selected = 0;
                app.input.clear();
                app.input_cursor = 0;
            }
        }

        // ── Shift+Enter — newline in input ────────────────────────────────
        (KeyModifiers::SHIFT, KeyCode::Enter) => {
            app.input.push('\n');
            app.input_cursor += 1;
        }

        // ── Enter ─────────────────────────────────────────────────────────
        (_, KeyCode::Enter) => {
            if app.slash_mode.active {
                let filtered = app.slash_mode.filtered();
                if let Some(cmd) = filtered.get(app.slash_mode.selected) {
                    let trigger = cmd.trigger.clone();
                    app.slash_mode.active = false;
                    app.slash_mode.query.clear();
                    app.slash_mode.selected = 0;
                    app.input.clear();
                    app.input_cursor = 0;
                    app.execute_slash(&trigger);
                } else if !app.input.is_empty() {
                    // No popup match — fall through as direct input (e.g. /search <query>)
                    app.slash_mode.active = false;
                    app.slash_mode.query.clear();
                    app.slash_mode.selected = 0;
                    app.send_message();
                }
            } else {
                app.send_message();
            }
        }

        // ── Tab ───────────────────────────────────────────────────────────
        (_, KeyCode::Tab) => {
            if app.slash_mode.active {
                let count = app.slash_mode.filtered().len();
                if count > 0 {
                    app.slash_mode.selected = (app.slash_mode.selected + 1) % count;
                }
            } else {
                app.cycle_agent();
            }
        }

        // ── Up ────────────────────────────────────────────────────────────
        (_, KeyCode::Up) => {
            if app.slash_mode.active {
                if app.slash_mode.selected > 0 {
                    app.slash_mode.selected -= 1;
                }
            } else {
                app.scroll_offset += 1;
            }
        }

        // ── Down ──────────────────────────────────────────────────────────
        (_, KeyCode::Down) => {
            if app.slash_mode.active {
                let count = app.slash_mode.filtered().len();
                if count > 0 && app.slash_mode.selected < count - 1 {
                    app.slash_mode.selected += 1;
                }
            } else {
                app.scroll_offset = app.scroll_offset.saturating_sub(1);
            }
        }

        // ── PageUp / PageDown ─────────────────────────────────────────────
        (_, KeyCode::PageUp) => {
            app.scroll_offset += 10;
        }
        (_, KeyCode::PageDown) => {
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
        }

        // ── Backspace ─────────────────────────────────────────────────────
        (_, KeyCode::Backspace) => {
            if !app.input.is_empty() {
                app.input.pop();
                app.input_cursor = app.input_cursor.saturating_sub(1);
                if app.slash_mode.active {
                    if app.input.is_empty() {
                        app.slash_mode.active = false;
                        app.slash_mode.query.clear();
                        app.slash_mode.selected = 0;
                    } else {
                        app.slash_mode.query = app.input.clone();
                        app.slash_mode.selected = 0;
                    }
                }
            }
        }

        // ── Character input ───────────────────────────────────────────────
        (_, KeyCode::Char(c)) => {
            app.input.push(c);
            app.input_cursor += 1;
            if app.input == "/" {
                app.slash_mode.active = true;
                app.slash_mode.query = String::new();
                app.slash_mode.selected = 0;
            } else if app.slash_mode.active {
                app.slash_mode.query = app.input.clone();
                app.slash_mode.selected = 0;
            }
        }

        _ => {}
    }
}
