# TUI — TODO (Phase 2)

## Context

Build the full Ratatui terminal UI. No LLM yet — this is the shell the chat will live in.

Layout (from SPEC.md §4.2):
```
┌─────────────────────────────────────────────────────┐
│ everthink  model: claude-sonnet-4-5  tokens: 12,450 │  ← status bar (1 row)
├─────────────────────────────────────────────────────┤
│  [Build]                                             │
│  > you: hello                                        │
│  ◆ everthink: [stub] Processing...                   │
│  (scrollable chat pane)                              │
├─────────────────────────────────────────────────────┤
│ > _                                                  │  ← input bar (3 rows)
└─────────────────────────────────────────────────────┘
```

Files: `src/tui/mod.rs`, `src/tui/chat.rs`, `src/tui/input.rs`, `src/tui/status.rs`

**Key reference:** SPEC.md §4.2, OpenCode session/prompt patterns

---

## Todos

### src/tui/TODO.md
- [x] Created

### src/tui/chat.rs
- [ ] `ChatMessage` struct with `MessageRole` enum (User/Assistant/System)
- [ ] `render(frame, area, app)` — scrollable List of messages
- [ ] Color: User=Cyan prefix, Assistant=Green prefix, System=DarkGray
- [ ] Auto-scroll to bottom (scroll_offset=0 = at bottom)

### src/tui/status.rs
- [ ] `render(frame, area, app)` — single-row status bar
- [ ] Show: app name | model name | token count | key hints

### src/tui/input.rs
- [ ] `render(frame, area, app)` — bordered input bar
- [ ] Show placeholder when empty
- [ ] Position cursor with `frame.set_cursor_position()`

### src/tui/mod.rs
- [ ] `App` struct: messages, input, input_cursor, scroll_offset, agent, slash_mode, slash_selected, quit, model, token_count
- [ ] `Agent` enum: Build / Plan / General
- [ ] `SlashMode` struct: query, commands list
- [ ] `SlashCommand` struct: trigger, description
- [ ] `App::new()` with welcome system message
- [ ] `App::update_slash_mode()` — filter slash commands by query
- [ ] `App::handle_enter()` — send message or execute slash command
- [ ] `App::execute_slash()` — handle /clear /help /commit /review /context /agent /model /session
- [ ] `App::cycle_agent()` — Build → Plan → General → Build
- [ ] `run()` — init terminal, event loop, restore
- [ ] `handle_key()` — all key bindings
- [ ] `ui()` — layout: status(1) + chat(flex) + input(3)
- [ ] `render_slash_popup()` — overlay above input bar

### Key bindings
- [ ] Ctrl+C — quit
- [ ] Ctrl+L — /clear
- [ ] Ctrl+S — /commit
- [ ] Tab (no slash) — cycle agent
- [ ] Tab (slash open) — move selection down
- [ ] Up/Down (slash open) — navigate slash list
- [ ] Up/Down (no slash) — scroll chat
- [ ] PageUp/PageDown — scroll chat by 10
- [ ] Enter (slash open) — execute selected command
- [ ] Enter (no slash) — send message
- [ ] Shift+Enter — newline in input
- [ ] Backspace — delete char
- [ ] Esc — close slash popup

### src/cli/mod.rs update
- [ ] Make `command: Option<Commands>`
- [ ] None arm → call `crate::tui::run()`
- [ ] Some arm → existing stub handlers

---

## Dependencies

- Phase 1 (complete)

---

## Blockers

- None

---

## Definition of Done

- `everthink` (no args) launches TUI without panic
- Status bar shows model + token count
- Chat pane shows messages, scrolls with Up/Down/PageUp/PageDown
- Input bar accepts text, shows placeholder when empty, cursor visible
- `/` triggers slash command dropdown
- Enter selects slash command or sends message stub
- Tab cycles agent (Build/Plan/General) shown in chat pane title
- Ctrl+C exits cleanly, terminal restored
- `cargo build` zero errors
