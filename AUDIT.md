# AUDIT: Everthink IDE — V6 Full System

**Date:** 2026-04-19
**Version:** 6.0
**Status:** Complete

---

## Questions & Answers

| # | Question | Answer |
|---|----------|--------|
| 1 | MVP scope — what ships first? | Everything — full V6 system, no deferred features |
| 2 | TUI library? | Ratatui |
| 3 | V1 LLM providers? | All 22+ from day one |
| 4 | AUDIT interaction style? | Sequential Q&A (Q1 → answer → Q2 → ...) |
| 5 | Storage format for sessions/progress/decisions? | Pure files (Markdown + TOML + YAML) |
| 6 | API key storage? | Config file — `~/.config/everthink/config.toml` (plaintext) |
| 7 | Autonomous build context management? | Same process, manual context reset between tasks |
| 8 | Skills source in V1? | Live fetch from skills.sh + awesome-claude-skills |
| 9 | Verification pipeline scope? | Rust-only: `cargo clippy`, `cargo build`, `cargo test` |
| 10 | Default permission mode? | Ask mode (prompt before dangerous ops) |
| 11 | TUI layout? | OpenCode-style: single chat pane + input bar, no sidebars |
| 12 | Theme/colors? | Dark, terminal-native (respects terminal theme) |
| 13 | First launch experience? | Small wizard on first run + config commands for manual setup |
| 14 | MemPalace memory depth? | Full Wing/Room/Drawer — Wing=project, Room=feature, Drawer=session |
| 15 | Greppy integration? | Python subprocess via PythonEngine bridge |
| 16 | Code Review Graph blast radius trigger? | On every `/review` command |
| 17 | PentAGI safety limits? | Loop detection after 3 identical tool calls + 50 tool call max per task |
| 18 | Repo study strategy? | Study + build in parallel — study each source repo as its feature is built |
| 19 | Multi-agent switching? | Tab = toggle Build/Plan, `/agent` command, Ctrl+Tab = cycle subagents |
| 20 | Session auto-resume? | `everthink continue` auto-loads last session; `/session` lists all |
| 21 | Build component order? | Scaffold → TUI → LLM → AUDIT → Tools → Memory → Autonomous → Skills → Review → Config |
| 22 | Core Rust crates? | Full dependency set upfront (all crates locked in from start) |

---

## AI Suggestions

| Idea | Status |
|------|--------|
| Use OpenRouter as a fallback gateway for all 22+ providers | ⏸ Deferred |
| Add LSP (Language Server Protocol) for code intelligence in TUI | ⏸ Deferred (post-MVP) |
| Web dashboard companion (like OpenCode's SST web UI) | ⏸ Deferred (post-MVP) |
| Bundle a mock/demo LLM mode for offline development testing | ✅ Accepted |
| Built-in MCP server inside Everthink | ⏸ Deferred (post-MVP) |
| Image rendering via sixel/iTerm2 protocol | ⏸ Deferred (post-MVP) |

---

## Decisions Made

1. **Full V6 ships as one** — no phased MVP, all features built together
2. **Ratatui** for TUI — OpenCode-style single chat pane, input bar at bottom, status bar at top
3. **All 22+ providers** supported from day one via direct HTTP (reqwest), not an AI SDK
4. **Pure files** for all storage — human-readable, git-friendly, no database
5. **Plaintext config** for API keys at `~/.config/everthink/config.toml`
6. **MemPalace Wing/Room/Drawer** — full memory model, Wing=project, Room=feature, Drawer=session
7. **Greppy** runs as Python subprocess via PythonEngine bridge in `engines/`
8. **Code Review blast radius** runs automatically on every `/review` command
9. **Safety limits**: loop detection (3 identical calls triggers halt) + 50 tool calls max per task
10. **Study + build in parallel** — study each source repo when building its corresponding feature
11. **Build order**: Scaffold → TUI → LLM → AUDIT → Tools → Memory → Autonomous → Skills → Review → Config → Tests
12. **Full crate set upfront** — lock in all Cargo.toml dependencies before first line of feature code

---

## Source Repos (Study + Clone)

| Repo | Feature Being Cloned | Location |
|------|---------------------|----------|
| OpenCode | TUI, providers, tools, sessions, slash commands, MCP | `/Users/volodymurvasualkiw/Desktop/opencode` |
| MemPalace | Wing/Room/Drawer memory, semantic search, knowledge graph | Study before Memory phase |
| Caveman | 46% token compression, caveman output mode | Study before Autonomous phase |
| Greppy | BM25 + AI reranking code search, <1ms | Study before Tools phase |
| Code Review Graph | Blast radius, affected file tracing | Study before Review phase |
| GSD-2 | Fresh 200k context per task, context pre-loading | Study before Autonomous phase |
| PentAGI | Loop detection, tool call limits, execution monitoring | Study before Autonomous phase |
