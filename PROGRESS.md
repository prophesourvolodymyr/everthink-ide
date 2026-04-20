# Everthink IDE — PROGRESS.md

**Version:** 6.0
**Updated:** 2026-04-20
**Status:** Phase 10 complete — Slash Commands (/commit AI, /model live switch, /context, /session list/save/load)

---

## How This Works

PROGRESS.md is the **overview only**. Each phase has a dedicated `TODO.md` inside its module directory with full context, specific tasks, dependencies, and blockers.

When starting a phase: open its TODO.md, work through it, update status here when done.

---

## Build Order Overview

| Phase | Name | Status | Todo File |
|-------|------|--------|-----------|
| 1 | Rust Scaffold | ✅ Complete | `src/cli/TODO.md` |
| 2 | TUI (Ratatui) | ✅ Complete | `src/tui/TODO.md` |
| 3 | LLM Integration | ✅ Complete | `src/providers/TODO.md` |
| 4 | AUDIT System | ✅ Complete | `src/core/TODO.md` |
| 5 | Tools System | ✅ Complete | `src/tools/TODO.md` |
| 6 | Memory + Sessions | ✅ Complete | `src/storage/TODO.md` |
| 7 | Autonomous Build | ✅ Complete | `src/core/TODO.md` |
| 8 | Skills System | ✅ Complete | `src/skills/TODO.md` |
| 9 | Code Review + Blast Radius | ✅ Complete | `src/commands/TODO.md` |
| 10 | Slash Commands | ✅ Complete | `src/commands/TODO.md` |
| 11 | Config + First Launch | ⬜ Pending | `src/config/TODO.md` |
| 12 | Error Handling + Security | ⬜ Pending | `src/core/TODO.md` |
| 13 | Testing + Benchmarks | ⬜ Pending | `tests/TODO.md` |

---

## Phase 1: Rust Scaffold → See `src/cli/TODO.md`

**Status:** ✅ Complete
**Scope:** Cargo project init, Cargo.toml dependencies, module folder structure, CLI arg parsing, main entry point.
**Output:** A compilable `everthink` binary that routes commands.

---

## Phase 2: TUI (Ratatui) → See `src/tui/TODO.md`

**Status:** ✅ Complete
**Scope:** Full Ratatui app — event loop, chat pane, input bar, status bar, slash command autocomplete, keyboard shortcuts.
**Output:** Working TUI shell. `everthink` (no args) launches a full interactive chat UI. Stub LLM responses. Ready for Phase 3 provider wiring.

---

## Phase 3: LLM Integration → See `src/providers/TODO.md`

**Status:** ✅ Complete
**Scope:** Provider trait, Anthropic + OpenAI streaming HTTP, StubProvider fallback, Config loading, async TUI event loop with `tokio::select!`.
**Output:** `everthink` sends messages to real LLMs and streams tokens back into the chat pane. Streaming cursor `▊` visible during generation. No config → stub shows setup instructions.

---

## Phase 4: AUDIT System → See `src/core/TODO.md`

**Status:** ✅ Complete
**Scope:** `/add` slash command starts an 8-question sequential Q&A session inside the TUI chat pane. Answers are captured (not sent to LLM). On completion, writes `AUDIT.md`, `INTENT.md`, `SPEC.md` to current working directory. `/cancel` aborts the session. Status bar shows `[AUDIT n/8]` badge while active.
**Output:** `/add` in TUI launches a full AUDIT Q&A loop. After 8 answers, three files are written to the project directory.

---

## Phase 5: Tools System → See `src/tools/TODO.md`

**Status:** ✅ Complete
**Scope:** Tool trait + ToolResult, ToolRegistry, BashTool, ReadTool/WriteTool/EditTool/GlobTool/GrepTool (fs.rs), WebFetchTool, VerifyTool + `run_verify()` pipeline (clippy→build→test), PythonEngine subprocess wrapper, McpStub. TUI wired: `/verify` streams the full pipeline output into the chat pane; `/search <pattern>` runs GrepTool and streams results. Slash command Enter handler updated to fall through when no popup match (enables `/search <pattern>` typed directly).
**Output:** `/verify` runs real `cargo clippy + build + test` and streams results. `/search fn main` searches the codebase. All tools compile cleanly. `cargo build` clean (26 dead-code warnings only — expected for stubs).

---

## Phase 6: Memory + Sessions → See `src/storage/TODO.md`

**Status:** ✅ Complete
**Scope:** `Session` (id/timestamps/model/agent/messages), `SessionStore` (save to `sessions/{ulid}.json`, `latest` pointer, load by ID or latest), `MemPalace` (Wing/Room/Drawer — `memory/{wing}/{room}.md` files, `search()`, `append_drawer()`), `DecisionsLog` (append-only `DECISIONS.md`). TUI wired: auto-saves session on `Ctrl+C` quit. `/remember <topic>` streams MemPalace search results. `everthink continue` loads last session and pre-populates chat via `run_with_session()`.
**Output:** Sessions persist across launches. `everthink continue` restores last session. `/remember <topic>` searches memory. Wing/Room/Drawer model fully implemented. `cargo build` clean.

---

## Phase 7: Autonomous Build → See `src/core/TODO.md`

**Status:** ✅ Complete
**Scope:** `ProgressManager` (parse/update PROGRESS.md table), `LoopDetector` (PentAGI — same call ≥3 times in a row), `ContextCompressor` (Caveman — summarise older messages when count > threshold), `AutonomousBuild::run_all()` (GSD-2 fresh context per task, LLM call, verify, mark done). `everthink build --all [--yolo]` runs the full autonomous pipeline from CLI. `/build` in TUI streams the same engine output into the chat pane.
**Output:** `everthink build --all` reads pending phases from PROGRESS.md, calls LLM with fresh context, runs `cargo clippy + build + test`, marks phases complete. `/build` in TUI shows the same output streamed into chat. `cargo build` clean.

---

## Phase 8: Skills System → See `src/skills/TODO.md`

**Status:** ✅ Complete
**Scope:** `SkillsManager` with 3-tier architecture (Favorites → Installed → Library). Built-in library of 15 skills (skills.sh + awesome-claude-skills). Jaro-Winkler fuzzy search across all tiers. Disk persistence: `.skills/installed/<name>/SKILL.md`, `.skills/favorites.yaml`, `.skills/index.yaml`. TUI commands: `/skills` (list), `/skills search <q>`, `/skills install <name>`, `/skills fav/unfav <name>`, `/skills status`. `SkillsManager` field on `App`, loaded from cwd on startup. Use-count tracking for favorites.
**Output:** `/skills search pdf` returns ranked results across all tiers. `/skills install api-client` creates a stub SKILL.md. `/skills fav api-client` adds to favorites and persists. Status bar-ready `status_summary()`. `cargo build` clean.

---

## Phase 9: Code Review + Blast Radius → See `src/commands/TODO.md`

**Status:** ✅ Complete
**Scope:** `src/commands/review.rs` — `DiffTarget` (Staged/Head/Branch), `ChangedFile`, `AffectedFile`, `RiskLevel` (Low/Medium/High), `ReviewResult` with `format_summary()` + `build_review_prompt()`. `run_review()` public entry point. Blast radius via regex grep for `crate::<module>` (no Tree-sitter). TUI wired: `/review [staged|head|<branch>]` shows blast radius immediately then streams AI review.
**Output:** `/review` shows changed files, blast radius dependents, risk level, then streams AI code review. `cargo build` clean.

---

## Phase 10: Slash Commands → See `src/commands/TODO.md`

**Status:** ✅ Complete
**Scope:** `/commit` (AI-generated conventional commit message via LLM stream → `git commit -m`; auto-stages if nothing staged; or use `/commit <msg>` to skip LLM). `/model` (list available providers or switch live via `ProviderRegistry.get(id)`; `registry` field added to `App`). `/context` (message count, user/assistant breakdown, estimated tokens, compression mode, provider, agent). `/session` (list saved sessions, save current session, load session by ID; uses existing `SessionStore`). Added `/context` + `/session` to slash popup. Added `PartialEq` to `MessageRole`.
**Output:** All four commands work. `/commit` generates real git commits. `/model anthropic` switches provider live. `/context` shows stats. `/session save` persists to disk. `cargo build` clean.

---

## Phase 11: Config + First Launch → See `src/config/TODO.md`

**Status:** ⬜ Pending
**Scope:** Global config (`~/.config/everthink/config.toml`), project config (`everthink.toml`), first-launch wizard, `everthink config set`.
**Output:** First run shows wizard. Config persists. All providers load keys from config.

---

## Phase 12: Error Handling + Security → See `src/core/TODO.md`

**Status:** ⬜ Pending
**Scope:** `EverthinkError` enum, retry logic, dangerous command blocklist, sandbox, permission system.
**Output:** All errors are typed and handled. Dangerous commands are blocked. Retry works on LLM/build failures.

---

## Phase 13: Testing + Benchmarks → See `tests/TODO.md`

**Status:** ⬜ Pending
**Scope:** Unit tests (tool registry, permissions, skills search, context logic), integration tests (session cycle, verify pipeline), E2E (autonomous build), benchmarks.
**Output:** Full test suite passing. Benchmarks logged against targets.

---

## Decisions Log

| Date | Decision | Reason |
|------|----------|--------|
| 2026-04-19 | Full V6 system — no phased MVP | AUDIT answer: "everything" |
| 2026-04-19 | Ratatui for TUI | Best Rust TUI crate, actively maintained |
| 2026-04-19 | All 22+ providers from day one | AUDIT answer |
| 2026-04-19 | Pure file storage (no DB) | Human-readable, git-friendly |
| 2026-04-19 | Plaintext config at ~/.config/everthink/ | Simplicity over keychain |
| 2026-04-19 | Full MemPalace Wing/Room/Drawer | AUDIT answer |
| 2026-04-19 | Greppy as Python subprocess | AUDIT answer |
| 2026-04-19 | Blast radius on every /review | AUDIT answer |
| 2026-04-19 | Loop detection (3 calls) + 50 tool limit | AUDIT answer |
| 2026-04-19 | Study + build in parallel | AUDIT answer |
| 2026-04-19 | Full crate set upfront | AUDIT answer |
| 2026-04-19 | Hybrid todo system: PROGRESS.md overview + per-module TODO.md | Improves specificity and context |
