# Rust Scaffold — TODO

## Context

This is **Phase 1** — the foundation everything else builds on. We are:

1. Initializing the Cargo project that produces the `everthink` binary
2. Locking in all crate dependencies upfront (decided in AUDIT)
3. Creating the full module folder structure that matches `SPEC.md §2`
4. Wiring up CLI argument parsing so every command has a registered entry point
5. Writing `main.rs` to route each command to its handler

Nothing else can start until this phase is complete. The binary must compile (even if commands are stubs) before Phase 2 begins.

**Key reference:** `SPEC.md §2` (architecture), `SPEC.md §3` (Cargo.toml), `SPEC.md §4.1` (CLI commands)
**Study first:** OpenCode's `packages/opencode/src/` for module organization patterns

---

## Todos

### Cargo Project Init
- [ ] Run `cargo new everthink --bin` in this directory (or convert existing to Cargo workspace)
- [ ] Verify binary name is `everthink` in `[[bin]]` section

### Cargo.toml — Lock In All Dependencies
- [ ] Add TUI crates: `ratatui = "0.28"`, `crossterm = "0.28"`
- [ ] Add async runtime: `tokio = { version = "1", features = ["full"] }`, `tokio-stream = "0.1"`, `futures = "0.3"`
- [ ] Add HTTP client: `reqwest = { version = "0.12", features = ["json", "stream"] }`
- [ ] Add serialization: `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `toml = "0.8"`, `serde_yaml = "0.9"`
- [ ] Add CLI parsing: `clap = { version = "4", features = ["derive"] }`
- [ ] Add error handling: `anyhow = "1"`, `thiserror = "1"`
- [ ] Add filesystem/paths: `directories = "5"`
- [ ] Add time/IDs: `chrono = { version = "0.4", features = ["serde"] }`, `ulid = "1"`
- [ ] Add search: `tantivy = "0.22"`
- [ ] Add clipboard: `arboard = "3"`
- [ ] Add string similarity: `strsim = "0.11"`
- [ ] Add markdown + highlighting: `pulldown-cmark = "0.11"`, `syntect = "5"`
- [ ] Add async utils: `async-trait = "0.1"`
- [ ] Add dev dependencies: `tokio-test = "0.4"`, `tempfile = "3"`
- [ ] Run `cargo build` — confirm all crates resolve with no conflicts

### Module Structure
- [ ] Create `src/main.rs` — entry point, calls `cli::run()`
- [ ] Create `src/cli/mod.rs` — CLI struct with all subcommands defined via clap
- [ ] Create `src/tui/mod.rs` — empty stub, exports `App`
- [ ] Create `src/core/mod.rs` — empty stub, exports `audit`, `llm`, `context`, `autonomous`, `permissions`, `agents`
- [ ] Create `src/providers/mod.rs` — empty stub, exports `LLMProvider` trait + `ProviderRegistry`
- [ ] Create `src/tools/mod.rs` — empty stub, exports `Tool` trait + `ToolRegistry`
- [ ] Create `src/skills/mod.rs` — empty stub, exports `SkillsManager`
- [ ] Create `src/storage/mod.rs` — empty stub, exports `SessionManager`, `ProgressTracker`, `MemoryStore`
- [ ] Create `src/commands/mod.rs` — empty stub, exports all slash command handlers
- [ ] Create `src/config/mod.rs` — empty stub, exports `GlobalConfig`, `ProjectConfig`

### CLI Argument Parsing (`src/cli/mod.rs`)
- [ ] Define `Cli` struct with `clap::Parser`
- [ ] Define `Commands` enum: `Init`, `Add`, `Build`, `Continue`, `Verify`, `Search`, `Remember`
- [ ] `Init { idea: String }` — new project
- [ ] `Add { feature: String }` — AUDIT + feature scaffold
- [ ] `Build { all: bool, yolo: bool }` — build current feature or all
- [ ] `Continue` — resume last session
- [ ] `Verify` — run cargo clippy + build + test
- [ ] `Search { query: String }` — Greppy code search
- [ ] `Remember { topic: String }` — load from memory

### Main Entry Point (`src/main.rs`)
- [ ] Parse `Cli` args with `Cli::parse()`
- [ ] Match on `Commands` enum — call stub handler per command (return `todo!()` for now)
- [ ] Run `cargo build` — binary compiles clean with no warnings
- [ ] Run `everthink --help` — all commands listed correctly
- [ ] Run `everthink init "test"` — hits stub, no panic

---

## Dependencies

- **None** — Phase 1 has no upstream dependencies. This is the start.

---

## Blockers

- None yet. First thing to build.

---

## Definition of Done

- `cargo build` succeeds with zero errors
- `everthink --help` shows all 7 commands
- `everthink [any command]` hits a stub without panicking
- All module `mod.rs` files exist and are imported from `main.rs`
- All crates in `Cargo.toml` resolve (no version conflicts)
