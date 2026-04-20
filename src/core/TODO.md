# src/core — TODO (Phase 4: AUDIT System)

## Context

The AUDIT system is the Q&A phase that runs BEFORE any feature is built.
When the user types `/add` in the TUI (or `everthink add` from CLI), a sequential
Q&A session begins. The user answers questions about the feature. At the end,
three files are written to the current project directory:

- `AUDIT.md`  — raw Q&A log
- `INTENT.md` — goals and constraints summary
- `SPEC.md`   — full technical spec ready for the build phase

The AUDIT session runs INSIDE the TUI chat pane:
- Each question appears as a [AUDIT] system message
- User types answers normally (Enter to submit)
- Answers are captured instead of sent to LLM
- Progress shown: "Question 3/8"
- Completion triggers file write + approval prompt

## Todos

- [x] Create this file
- [ ] `src/core/audit.rs` — AuditSession, AuditQuestion, AuditAnswer, AuditState
- [ ] `src/core/spec_writer.rs` — writes AUDIT.md, INTENT.md, builds SPEC prompt
- [ ] `src/core/mod.rs` — replace stub with pub mod audit; pub mod spec_writer;
- [ ] `src/tui/mod.rs` — add `audit_session: Option<audit::AuditSession>` to App
- [ ] `src/tui/mod.rs` — route Enter key to audit when session is active
- [ ] `src/tui/mod.rs` — execute_slash("/add") starts AuditSession
- [ ] `cargo build` — zero errors

## Dependencies

- Phase 2 (TUI) — complete ✅
- Phase 3 (LLM) — complete ✅ (SPEC generation uses LLM in Phase 7, plain text for now)

## Blockers

None.

## Definition of Done

`/add` in TUI launches a Q&A loop with 8 questions. After all answers are given,
AUDIT.md + INTENT.md + SPEC.md are written to the current working directory.
User sees "AUDIT complete. Files written." message in chat.
