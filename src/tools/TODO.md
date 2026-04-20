# src/tools — TODO (Phase 5: Tools System)

## Context

The tools system gives the AI (and slash commands) the ability to act on the
filesystem, run shell commands, search code, and fetch URLs.

Tools are registered in `ToolRegistry`. Each tool implements the `Tool` trait
(async `run(args) -> ToolResult`). The TUI uses the same `StreamEvent` channel
to stream tool output into the chat pane — so `/verify` shows cargo output
line-by-line as it runs.

## Todos

- [x] Create this file
- [ ] `Cargo.toml` — add `glob`, `regex` crates
- [ ] `src/tools/mod.rs` — Tool trait, ToolResult, re-exports
- [ ] `src/tools/registry.rs` — ToolRegistry (HashMap + register/get/list)
- [ ] `src/tools/bash.rs` — BashTool: spawn command, stream stdout+stderr
- [ ] `src/tools/fs.rs` — ReadTool, WriteTool, EditTool, GlobTool, GrepTool
- [ ] `src/tools/web.rs` — WebFetchTool: HTTP GET via reqwest
- [ ] `src/tools/verify.rs` — VerifyTool: cargo clippy + cargo build + cargo test
- [ ] `src/tools/python_engine.rs` — PythonEngine: subprocess to python3 script
- [ ] `src/tools/mcp.rs` — McpStub (logs call, returns placeholder)
- [ ] `src/tui/mod.rs` — wire /verify → VerifyTool, /search → GrepTool

## Dependencies

- Phase 3 (LLM streaming) ✅ — reuses StreamEvent channel for tool output

## Blockers

None.

## Definition of Done

- `/verify` runs real `cargo clippy && cargo build && cargo test` and streams output
- `/search <query>` runs GrepTool and shows matching lines in chat
- All tool structs compile with zero errors
