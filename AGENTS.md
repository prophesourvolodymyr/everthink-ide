# Everthink IDE — Builder Agent

**IMPORTANT:** This is the EVERTHING IDE project. You are building the IDE itself.

**DO NOT** work on other projects. **DO NOT** create features for other apps. This repo is for building the Everthink IDE tool.

---

## What Are You Building?

You are building **Everthink IDE** — an AI coding assistant written in Rust, like OpenCode but with our custom features.

This is the **BUILDER** of the IDE, not a user project.

---

## The System (V6)

Everthink IDE uses the V6 system which includes:

- **AUDIT** — Question + idea phase for every feature
- **Skills** — Three-tier (Library / Installed / Favorites)
- **22+ LLM Providers** — Via OpenCode provider system
- **Autonomous Build** — Fresh context, auto-compact, YOLO mode
- **Memory** — sessions/ + PROGRESS.md + DECISIONS.md

---

## File Structure

```
Everthink IDE/
├── AGENTS.md              ← YOU ARE HERE
├── README.md              ← Project overview
├── SPECIFICATION.md       ← Full V6 spec (imported)
├── src/                  ← Rust source code
├── engines/              ← Python engine wrappers
├── .skills/              ← Skills for building the IDE
├── .config/              ← Configuration
└── tests/                ← Test files
```

---

## Key Rules

### 1. You Are The Builder

You are building the TOOL. Not using it. Not working on another project.

**WRONG:** "Let me create a feature for a habit tracker app"
**RIGHT:** "Let me implement the AUDIT phase in the CLI"

### 2. Work In This Folder

ALL your code goes in this folder: `/Users/volodymurvasualkiw/Desktop/Opensource/Everthink IDE/`

**DO NOT** create files in other folders like `USER PROJECTS/`, `DATABASE/`, or anywhere else.

### 3. Follow V6 System

When building features for the IDE, follow the V6 specification:
- Use AUDIT for features
- Track progress in PROGRESS.md
- Store sessions in sessions/
- Use the same command structure

---

## Commands You Can Use

For building this IDE:

| Command | Purpose |
|---------|---------|
| `init [name]` | Initialize new component/module |
| `add [feature]` | Add new feature to the IDE |
| `build` | Build the IDE |
| `test` | Run tests |
| `verify` | Run verification |

---

## Starting The Project

When you start, you MUST run AUDIT first. This is how it works:

### Step 1: Initial Prompt

Send this to the AI:

```
I'm building Everthink IDE — an AI coding assistant in Rust.

Work in this folder: /Users/volodymurvasualkiw/Desktop/Opensource/Everthink IDE/

Read AGENTS.md and SPECIFICATION.md first to understand the system.

Let's start with the AUDIT phase. Ask me questions about what features the IDE should have.
```

### Step 2: AI Runs AUDIT

The AI will ask you questions about:
- Core features
- LLM providers
- Skills system
- TUI interface
- Commands
- Memory system
- Autonomous build

### Step 3: You Answer

Answer the questions. The AI will document everything in AUDIT.md.

### Step 4: AI Creates SPEC

After AUDIT, AI creates SPEC.md for the IDE itself.

### Step 5: Build Begins

Only then does building start.

---

## Your First Task

When you start working with the AI, remind it:

1. Read AGENTS.md
2. Read SPECIFICATION.md  
3. Run AUDIT (ask questions)
4. Create SPEC.md
5. Then start building

Don't let the AI skip AUDIT!

---

## Questions?

If unsure about what to build, read SPECIFICATION.md first.

**Remember:** You are building the TOOL. Not using it. Work in THIS folder only.