# Everthink IDE — SPEC.md

**Version:** 6.0
**Date:** 2026-04-19
**Status:** Ready for implementation
**Based on:** AUDIT.md (2026-04-19)

---

## 1. Overview

Everthink IDE is a native Rust CLI/TUI AI coding assistant. It is:

- A **chat-focused TUI** (like OpenCode, written in TypeScript — we rewrite in Rust)
- Backed by the **V6 system**: AUDIT + Skills + Memory + Autonomous build
- A **native binary** with no Node.js/npm runtime dependency

**Not** a VS Code clone. **Not** a file tree IDE. A terminal chat interface that builds apps autonomously.

---

## 2. Architecture

```
everthink/
├── src/
│   ├── main.rs              ← Entry point, command routing
│   ├── cli/                 ← CLI argument parsing (clap)
│   │   └── mod.rs
│   ├── tui/                 ← Ratatui TUI layer
│   │   ├── mod.rs           ← App state + event loop
│   │   ├── chat.rs          ← Chat message pane
│   │   ├── input.rs         ← Input bar
│   │   └── status.rs        ← Status bar (model/tokens)
│   ├── core/                ← Core logic
│   │   ├── mod.rs
│   │   ├── audit.rs         ← AUDIT phase (Q&A flow)
│   │   ├── llm.rs           ← LLM client + provider trait
│   │   ├── context.rs       ← Context management (GSD-2)
│   │   ├── autonomous.rs    ← Autonomous build loop
│   │   ├── permissions.rs   ← Permission system (ask/yolo/allow)
│   │   └── agents.rs        ← Build/Plan/General agents
│   ├── providers/           ← 22+ LLM providers
│   │   ├── mod.rs           ← Provider registry + trait
│   │   ├── anthropic.rs
│   │   ├── openai.rs
│   │   ├── google.rs
│   │   ├── groq.rs
│   │   ├── mistral.rs
│   │   ├── openrouter.rs
│   │   └── ... (one file per provider)
│   ├── tools/               ← Tool system
│   │   ├── mod.rs           ← Tool registry + trait
│   │   ├── bash.rs          ← Bash execution
│   │   ├── file.rs          ← Read/Write/Edit/Glob/Grep
│   │   ├── web.rs           ← WebFetch + WebSearch
│   │   ├── skill.rs         ← Skill tool
│   │   ├── python_engine.rs ← Python subprocess bridge (Greppy)
│   │   └── mcp.rs           ← MCP tool stub
│   ├── skills/              ← Skills system (3-tier)
│   │   ├── mod.rs
│   │   ├── library.rs       ← Fetch from skills.sh / awesome-claude-skills
│   │   ├── installed.rs     ← Per-project installed skills
│   │   └── favorites.rs     ← Favorites (.skills/favorites.yaml)
│   ├── storage/             ← File-based storage
│   │   ├── mod.rs
│   │   ├── session.rs       ← Session save/load/resume
│   │   ├── progress.rs      ← PROGRESS.md read/write
│   │   ├── decisions.rs     ← DECISIONS.md append
│   │   └── memory.rs        ← Wing/Room/Drawer (MemPalace)
│   ├── commands/            ← Slash command handlers
│   │   ├── mod.rs
│   │   ├── commit.rs        ← /commit
│   │   ├── review.rs        ← /review + blast radius
│   │   ├── model.rs         ← /model
│   │   ├── session.rs       ← /session
│   │   ├── agent.rs         ← /agent
│   │   ├── context.rs       ← /context show
│   │   ├── clear.rs         ← /clear
│   │   └── help.rs          ← /help
│   └── config/              ← Config loading
│       ├── mod.rs
│       ├── global.rs        ← ~/.config/everthink/config.toml
│       └── project.rs       ← everthink.toml per project
├── engines/                 ← Python engine wrappers
│   ├── greppy.py            ← Greppy semantic search wrapper
│   └── caveman.py           ← Caveman token compression wrapper
├── .skills/                 ← Skills for IDE development
├── .config/                 ← Dev config
├── tests/                   ← Integration + E2E tests
│   ├── unit/
│   ├── integration/
│   └── e2e/
├── Cargo.toml
├── everthink.toml           ← Project config (example)
└── AGENTS.md
```

---

## 3. Cargo.toml Dependencies

```toml
[package]
name = "everthink"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "everthink"
path = "src/main.rs"

[dependencies]
# TUI
ratatui = "0.28"
crossterm = "0.28"

# Async runtime
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"

# HTTP client (LLM API calls + streaming)
reqwest = { version = "0.12", features = ["json", "stream"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
serde_yaml = "0.9"

# CLI argument parsing
clap = { version = "4", features = ["derive"] }

# Error handling
anyhow = "1"
thiserror = "1"

# Filesystem paths
directories = "5"

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Unique IDs
ulid = "1"

# Text search (for skills BM25 similarity)
tantivy = "0.22"

# Clipboard
arboard = "3"

# Async utilities
futures = "0.3"

# Regex
regex = "1"

# String similarity (for skills matching)
strsim = "0.11"

# Markdown rendering in TUI
pulldown-cmark = "0.11"

# Syntax highlighting (code blocks in chat)
syntect = "5"

# Process execution
tokio-process = "0.2"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
```

---

## 4. Module Specifications

### 4.1 CLI Entry (`src/main.rs` + `src/cli/`)

Commands parsed with `clap`:

```
everthink                        # Launch TUI
everthink init [idea]            # New project
everthink add [feature]          # AUDIT + add feature
everthink build                  # Build current feature
everthink build all [--yolo]     # Autonomous build all
everthink continue               # Resume last session
everthink verify                 # Run verification
everthink search [query]         # Greppy code search
everthink remember [topic]       # Load from memory
```

### 4.2 TUI (`src/tui/`)

Built with **Ratatui**. Layout:

```
┌─────────────────────────────────────────────────────┐
│ everthink  model: claude-sonnet-4-5  tokens: 12,450 │  ← status bar
├─────────────────────────────────────────────────────┤
│                                                      │
│  [Build]  ← agent indicator (Tab to switch)         │
│                                                      │
│  > you: init "habit tracker"                         │
│                                                      │
│  ◆ everthink: Running AUDIT for habit tracker...     │
│    Q1: What problem does this solve?                 │
│                                                      │
│  (scrollable chat pane)                              │
│                                                      │
├─────────────────────────────────────────────────────┤
│ > _                                                  │  ← input bar
└─────────────────────────────────────────────────────┘
```

**Keyboard shortcuts:**
- `Tab` — toggle Build ↔ Plan agent
- `Ctrl+Tab` — cycle subagents
- `Ctrl+L` — clear session
- `Ctrl+S` — /commit
- `Ctrl+C` — exit
- Mouse select — copy to clipboard

**Slash command autocomplete:** Show dropdown of available commands when `/` is typed.

### 4.3 LLM Providers (`src/providers/`)

Provider trait:

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;
    fn models(&self) -> Vec<String>;
    async fn complete(&self, req: LLMRequest) -> Result<LLMResponse>;
    async fn stream(&self, req: LLMRequest) -> Result<Pin<Box<dyn Stream<Item=Result<String>> + Send>>>;
}
```

All 22 providers implement this trait. API keys loaded from `~/.config/everthink/config.toml`.

Provider list:
1. anthropic (claude-opus-4-5, claude-sonnet-4-5, claude-haiku-3-5)
2. openai (gpt-4o, gpt-4o-mini, gpt-4-turbo)
3. google (gemini-2-pro, gemini-2-flash)
4. google-vertex
5. amazon-bedrock
6. azure
7. mistral (mistral-large, mistral-small)
8. groq (llama3-70b, mixtral-8x7b)
9. deepinfra
10. cerebras
11. cohere (command-r-plus)
12. togetherai
13. perplexity
14. vercel
15. xai (grok-2-beta)
16. openrouter (gateway to all)
17. openai-compatible (custom endpoints)
18. gateway
19. gitlab
20. github-copilot
21. vertex-anthropic
22. opencode (minimax)

### 4.4 AUDIT System (`src/core/audit.rs`)

Flow:
1. User runs `everthink add [feature]`
2. AUDIT asks 3-5 sequential questions specific to feature type
3. AI adds 2-3 ambition suggestions
4. Saves `features/[name]/AUDIT.md`
5. Asks: "Ready for SPEC? (yes/no)"
6. Creates `features/[name]/INTENT.md` + `features/[name]/SPEC.md`
7. Creates `features/[name]/PROGRESS.md` with task breakdown

### 4.5 Memory System (`src/storage/memory.rs`)

**MemPalace Wing/Room/Drawer model:**

| Level | Scope | File | Purpose |
|-------|-------|------|---------|
| Wing | Project | `PROJECT.md`, `DECISIONS.md` | Project-wide context |
| Room | Feature | `features/[name]/` | Feature-specific context |
| Drawer | Session | `sessions/[date]-[name].md` | Conversation history |

Auto-save: Every message appended to current Drawer (session file).

`everthink continue` loads:
1. Latest Drawer (recent session)
2. Current Room PROGRESS.md
3. Wing DECISIONS.md
4. Relevant installed skills

### 4.6 Context Manager (`src/core/context.rs`)

**GSD-2 fresh context per task:**

```
Token thresholds:
  - 0–150k: Normal operation
  - 150k+: Auto-compress via Caveman (target: 46% reduction → ~108k)
  - 200k:   Force fresh context for next task
```

Fresh context loads only what the current task needs (task files + relevant sessions + decisions).

### 4.7 Autonomous Build (`src/core/autonomous.rs`)

```
everthink build all [--yolo]

1. Load PROGRESS.md → get all pending tasks
2. For each task:
   a. Fresh context (clear, load task-specific files)
   b. AI builds the code
   c. Run verify (cargo clippy + build + test)
   d. If fail → self-correct (up to 3 retries) → retry verify
   e. If pass → update PROGRESS.md, save to session
   f. Check token count → compress if >150k
   g. Check loop detection → halt if 3 identical tool calls
   h. Check tool limit → halt if >50 calls on one task
3. Print summary
```

YOLO mode: Skip all permission prompts. All tools allowed.

### 4.8 Skills System (`src/skills/`)

Three-tier priority:

```
1. Check FAVORITES (.skills/favorites.yaml) → use immediately
2. Check INSTALLED (.skills/installed/)     → use immediately
3. Search LIBRARY  (live fetch)             → install → personalize → use
```

Live fetch sources:
- `https://github.com/ComposioHQ/awesome-claude-skills` (1000+)
- `https://api.skills.sh/v1` (200+)

Skills index cached in `.skills/index.yaml` (TTL: 24h).

### 4.9 Greppy Integration (`src/tools/python_engine.rs` + `engines/greppy.py`)

Greppy runs as a Python subprocess:

```rust
PythonEngine::new("engines/greppy.py")
    .execute("search", json!({ "query": "login auth", "path": "src/" }))
```

Returns: ranked results with file paths, line numbers, relevance scores.

### 4.10 Code Review + Blast Radius (`src/commands/review.rs`)

Runs on every `/review`:
1. Get git diff (changed files)
2. Run Code Review Graph blast radius on changed files
3. Display: changed files + affected downstream files
4. Generate AI code review with blast context

### 4.11 Permission System (`src/core/permissions.rs`)

Default mode: **Ask**

| Mode | Behavior |
|------|----------|
| `ask` | Prompt before dangerous operations (rm, sudo, curl\|bash) |
| `yolo` | No prompts — all tools allowed |
| `allow` | AI decides what's safe |

Per-rule config in `everthink.toml`. Hardcoded blocklist (rm -rf /, shutdown, mkfs, etc.) always active regardless of mode.

### 4.12 Session Management (`src/storage/session.rs`)

- Auto-save every message to `sessions/YYYY-MM-DD-[feature]-[ULID].md`
- `/session` — list all sessions with timestamps
- `/session load [id]` — restore session
- `/session export [id]` — export to JSON
- `everthink continue` — auto-loads most recent session

### 4.13 Configuration

**Global config** `~/.config/everthink/config.toml`:

```toml
[providers]
anthropic.key = "sk-ant-..."
openai.key = "sk-..."
google.key = "AIza..."

[defaults]
provider = "anthropic"
model = "claude-sonnet-4-5"

[ui]
theme = "dark"

[keys]
commit = "Ctrl+s"
clear = "Ctrl+l"
```

**Project config** `everthink.toml` (per project):

```toml
[project]
name = "my-project"

[llm]
provider = "anthropic"
model = "claude-sonnet-4-5"

[autonomous]
fresh_context = true
auto_compact = true
yolo_mode = false

[skills]
auto_discover = true
favorites = []

[permissions]
mode = "ask"
rules = [
    { tool = "Bash", pattern = "git *", action = "allow" },
    { tool = "Bash", pattern = "rm *", action = "ask" },
]
```

### 4.14 First Launch Wizard

When `~/.config/everthink/config.toml` doesn't exist:

```
Welcome to Everthink IDE!
──────────────────────────────────
First time setup:

1. Choose provider:
   > anthropic

2. Enter API key:
   > [hidden input]

3. Choose default model:
   > claude-sonnet-4-5

Done! Type: everthink init "your project idea"
──────────────────────────────────
```

Also available via commands: `everthink config set provider.anthropic.key sk-...`

---

## 5. Source Repos to Study

Study each repo in parallel with building its corresponding feature:

| Phase | Study Before | Repo |
|-------|-------------|------|
| Phase 2 (TUI) | OpenCode TUI patterns | `/Users/volodymurvasualkiw/Desktop/opencode/packages/app/src` |
| Phase 3 (LLM) | OpenCode provider system | `/Users/volodymurvasualkiw/Desktop/opencode/packages/opencode/src/provider` |
| Phase 5 (Tools) | OpenCode tool system | `/Users/volodymurvasualkiw/Desktop/opencode/packages/opencode/src/tool` |
| Phase 6 (Memory) | MemPalace | Clone + study Wing/Room/Drawer |
| Phase 7 (Autonomous) | GSD-2, Caveman, PentAGI | Clone + study each |
| Phase 8 (Skills) | OpenCode skill system | `/Users/volodymurvasualkiw/Desktop/opencode/packages/opencode/src/skill` |
| Phase 9 (Review) | Code Review Graph, Greppy | Clone + study each |

---

## 6. Performance Targets

| Operation | Target |
|-----------|--------|
| Cold start | < 1s |
| TUI first render | < 100ms |
| Tool execution | < 100ms |
| Context load | < 500ms |
| Verification (cargo) | < 30s |
| Token compression | < 1s |
| Greppy search | < 1ms |

---

## 7. Error Handling

All errors typed via `thiserror`. Retry-able errors (LLM, Build, Verify) auto-retry up to 3 times. Hard errors surface to user with clear message + suggested fix.

---

## 8. Security

- Hardcoded dangerous command blocklist always active
- Sandbox restricts tool execution to project directory
- No API keys in environment or git — plaintext config only in `~/.config/`
- PentAGI loop detection: halt after 3 identical tool calls
- PentAGI tool limit: max 50 tool calls per autonomous task
