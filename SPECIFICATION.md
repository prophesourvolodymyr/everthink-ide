# Everthink IDE — V6 IDEA

**Version:** 6.0 (Ultimate)
**Created:** April 2026
**Status:** Complete system specification with ALL features

---

## Executive Summary

V6 is the complete Everthink IDE specification integrating every feature discovered through our sessions:

- **CORE system prompt** identifying the IDE
- **AUDIT phase** for every feature with fresh questions + AI ambition
- **Skills system** (three tiers: Library / Installed / Favorites)
- **Provider system** (22+ LLM providers + Skills sources)
- **Memory system** (sessions + PROGRESS + DECISIONS combined)
- **Autonomous build** (fresh context, auto-compact, YOLO mode)
- **All TUI features** (/commit, /review, /model, /session, etc)
- **Complete tool system** + verification
- **Multi-agent system** (Build, Plan, subagents)
- **Permission system** (YOLO mode)

This is the ultimate AI coding IDE.

---

## Repositories Being Cloned & Studied

This IDE is built by studying and cloning features from these repos in your Desktop folder:

### 1. OpenCode
**Repo:** `https://github.com/anomalyco/opencode`
**Location:** `/Users/volodymurvasualkiw/Desktop/opencode`

**What's Being Cloned:**
- Chat interface design (TUI-style)
- Provider system (22+ LLM providers)
- Tool system (Bash, Glob, Grep, Read, Edit, Write, etc)
- Agent system (Build, Plan modes)
- Session management
- Slash commands (/commit, /review, /model, /session)
- MCP integration

### 2. MemPalace
**Repo:** `https://github.com/MemPalace/mempalace`

**What's Being Cloned:**
- Palace memory structure (Wing/Room/Drawer)
- Semantic search for sessions
- Knowledge graph with temporal windows
- Verbatim storage (not summarized)

### 3. Caveman
**Repo:** `https://github.com/JuliusBrussee/caveman`

**What's Being Cloned:**
- Token compression (46% reduction)
- Caveman output mode (75% fewer tokens)
- Context pre-loading

### 4. Greppy
**Repo:** `https://github.com/KBLCode/greppy`

**What's Being Cloned:**
- Semantic code search
- <1ms latency
- BM25 + AI reranking

### 5. Code Review Graph
**Repo:** `https://github.com/tirth8205/code-review-graph`

**What's Being Cloned:**
- Blast radius analysis
- Trace affected files on change

### 6. GSD-2
**Repo:** `https://github.com/gsd-build/gsd-2`

**What's Being Cloned:**
- Fresh 200k context per task
- Context pre-loading

### 7. PentAGI
**Repo:** `https://github.com/vxcontrol/pentagi`

**What's Being Cloned:**
- Execution monitoring
- Loop detection
- Tool call limits

---

## Clone These First

Before starting, clone and study:
1. **OpenCode** — Main reference for TUI chat interface
2. **MemPalace** — Memory system  
3. **Caveman** — Token compression
4. **Grepp** — Code search
5. **Code Review Graph** — Blast radius
6. **GSD-2** — Fresh context
7. **PentAGI** — Execution monitoring

Study each repo to understand how to implement its features in Rust.

---

## Part I: Identity (CORE System Prompt)

### 1.0 Interface Type

**This is a Chat-Focused TUI** — Like OpenCode, but written in Rust.

```
┌─────────────────────────────────────┐
│  EVERTHINK (Rust TUI Chat)         │
│                                     │
│  AI Chat Pane                      │
│  ─────────────────────            │
│  Type commands + chat               │
│                                     │
│  Input Area                        │
└─────────────────────────────────────┘
```

Not a full IDE. Just chat interface where:
- User types messages
- AI responds
- Commands via slash (like OpenCode)
- Same backend (V6 system)

Same as OpenCode, but:
- Written in Rust (not TypeScript)
- Our custom features (AUDIT, Skills, etc)

**This is NOT:**
- VS Code clone
- Full IDE with file tree
- Terminal file editor

**This IS:**
- AI chat interface
- Command line tool
- Build/progress shown in chat

### 1.1 Who The IDE Is

Embedded in the IDE code:

```rust
fn system_prompt() -> String {
    r#"You are Everthink IDE, an autonomous AI coding assistant.

## Your Identity
- You are Everthink, an AI coding assistant
- You build applications with full autonomy
- You remember everything across sessions
- You verify your code automatically

## Your Job
Help users build complete applications through human-AI collaboration.
The human owns WHAT (intent, approval), you own HOW (implementation).

## Your Workflow
1. User gives idea → Run AUDIT (questions + ideas)
2. User approves → Create SPEC.md
3. Build code → Verify (lint + build + test)
4. Update PROGRESS.md automatically
5. Remember everything in sessions/

## Autonomous Mode
- When user says "build all" or "continue":
  - Read PROGRESS.md for tasks
  - Work through tasks one by one
  - Fresh 200k context per task (GSD-2)
  - Auto-compact context when high (caveman)
  - Verify automatically
  - Update progress after each task
  - Ask only when blocked

## Key Rules
- AUDIT EVERY FEATURE before building
- Always verify before marking done
- Work autonomously in "build all" mode
- Use/skills when helpful
- Never forget: use PROGRESS.md track

## Commands
- `init [idea]` - Start new project
- `add [feature]` - Add feature (runs AUDIT)
- `build` - Build current feature
- `build all` - Build ALL pending tasks
- `verify` - Run verification
- `continue` - Continue from last task
- `search [query]` - Code search
- `remember [topic]` - Load history
- `/commit` - Smart git commit
- `/review` - Code review
- `/model [name]` - Switch model
- `/session` - Manage sessions
- `--yolo` - Skip all prompts

Just say what you want to build. I'll handle the rest autonomously.
"#.to_string()
}
```

### 1.2 AGENTS.md (Project Rules)

In `PROJECT/.opencode/AGENTS.md`:

```markdown
# Everthink Builder Agent

## AUDIT Phase (Mandatory for EVERY Feature)

For EVERY feature, you MUST run AUDIT:

1. Ask 3-5 questions SPECIFIC to the feature type
2. Add 2-3 AI suggestions (ambition)
3. Save to AUDIT.md
4. Then create SPEC.md

Never skip AUDIT. Every feature needs AUDIT first.

## Questions by Feature Type

### Authentication
- Email-only or username?
- Social login (Google/Apple)?
- Password reset flow?
- Session duration?
- 2FA?

### Database
- SQLite or PostgreSQL?
- Cloud sync?
- Need migrations?
- ORM or raw queries?

### Search
- Full-text search?
- Filters?
- Ranking algorithm?

### Payments
- Stripe or PayPal?
- Webhook handling?
- Refund flow?

### User Profile
- Photo upload?
- Bio field?
- Social links?

## AI Suggestions (Always Add)

- Add biometric login (Face ID/Touch ID)
- Consider passwordless magic links
- Add caching for performance
- Consider webhook for async

## Save Format
Save to features/[feature]/AUDIT.md

## Autonomous Build Mode

When user says "build all" or "continue":
1. Load PROGRESS.md
2. Take first pending task
3. Fresh 200k context (GSD-2)
4. Build + auto-verify
5. Update PROGRESS
6. If context high → compress (caveman)
7. Repeat until all done

Use --yolo flag to skip all permission prompts.

## Don't Proceed Without AUDIT
Never build without running AUDIT first.
```

---

## Part II: The AUDIT Phase

### 2.1 Fresh Per Feature

Each feature gets FRESH AUDIT with unique questions:

| Feature | AUDIT Questions |
|---------|-----------------|
| Login | Email-only? Social? Reset? Sessions? 2FA? |
| Database | SQLite? PostgreSQL? Sync? Migrations? ORM? |
| Biometric | Face ID? Touch ID? Fallback? |
| Search | Full-text? Filters? Ranking? |
| Payments | Stripe? PayPal? Webhooks? Refunds? |
| Profile | Photo? Bio? Links? |

### 2.2 AUDIT.md File

```markdown
# AUDIT: [Feature Name]
**Date:** 2025-04-19

## Questions & Answers

| # | Question | Answer |
|---|----------|--------|
| 1 | [Question from above adapted] | [User answer] |
| 2 | [Question from above adapted] | [User answer] |
| 3 | [Question from above adapted] | [User answer] |

---

## AI Suggestions

| Idea | Status |
|------|--------|
| [AI suggestion 1] | ✅ Accepted / ❌ Rejected / ⏸ Deferred |
| [AI suggestion 2] | ✅ Accepted / ❌ Rejected / ⏸ Deferred |
| [AI suggestion 3] | ✅ Accepted / ❌ Rejected / ⏸ Deferred |

---

## Decisions Made
- [Decision 1]
- [Decision 2]
```

### 2.3 User Experience

```
> everthink add login

Running AUDIT for login...
─────────────────────────────────────

Q1: Username or email-only?
> Email only

Q2: Add social login (Google/Apple)?
> No, V2 maybe

Q3: Password reset flow?
> Email link

Q4: Session duration?
> 30 days

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

AI Suggestions:
💡 Add biometric (Face ID)?
> Yes, add it

💡 Passwordless magic links?
> Cool but defer to V2

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

AUDIT complete. Ready for SPEC?
> yes
```

---

## Part III: Skills System (Three Tiers)

### 3.1 Three-Tier Architecture

```
┌─────────────────────────────────────────────┐
│        SKILLS LIBRARY (Searchable)          │
│                                             │
│   Sources:                                   │
│   • skills.sh (200+)                        │
│   • awesome-claude-skills (1000+)           │
│   • skillsmp.com (500+)                       │
│   • community                               │
└─────────────────────────────────────────────┘
                     ↓ AI searches when needed
                     
┌─────────────────────────────────────────────┐
│        INSTALLED SKILLS (Per Project)        │
│                                             │
│   .skills/installed/                        │
│   ├── pdf-generator/                        │
│   ├── email-sender/                         │
│   └── webhook-handler/                       │
└─────────────────────────────────────────────┘
                     ↓ User picks favorites
                     
┌─────────────────────────────────────────────┐
│        FAVORITE SKILLS (Quick Access)        │
│                                             │
│   .skills/favorites.yaml                    │
│   - pdf-generator ⭐                         │
│   - api-client ⭐                            │
│   - database-migration ⭐                    │
└─────────────────────────────────────────────┘
```

### 3.2 Skill Selection Flow

```
Task: "Add PDF invoice export"

Step 1: Check FAVORITES
→ pdf-generator available! → Use immediately

Step 2: Check INSTALLED
→ Not found → Search LIBRARY

Step 3: Search LIBRARY
→ "invoice pdf" → Found: "invoice-pdf" (92%)
→ Found: "pdf-generator" (87%)

Step 4: Install + Personalize
→ Download skill → Adapt for project → Save to INSTALLED
```

### 3.3 Skills Index

```yaml
# .skills/index.yaml
skills:
  - name: pdf-generator
    source: skills.sh
    purpose: Generate PDFs
    tags: [pdf, document, export]
    
  - name: email-sender
    source: awesome-skills  
    purpose: Send emails
    tags: [email, notification]
    
  - name: webhook-handler
    source: skills.sh
    purpose: Handle webhooks
    tags: [api, webhook]
```

### 3.4 Favorites.yaml

```yaml
# .skills/favorites.yaml
favorites:
  - name: pdf-generator
    added: 2025-04-10
    use_count: 45
    
  - name: database-migration
    added: 2025-04-12
    use_count: 23
    
  - name: api-client
    added: 2025-04-15
    use_count: 67
```

### 3.5 Personalized Skill

```markdown
# Personalized: Invoice PDF

Adapted from: awesome-claude-skills/invoice-pdf
For: [Project Name]

## Modifications Made

### Brand Integration
- Changed colors to brand: --ec-color-primary
- Added logo path

### Data Schema
- Adapted to OurInvoices table
- Added fields: habit_id, completed_at

### Configuration
skill:
  source: awesome-claude-skills/invoice-pdf
  personalized: true
  created: 2025-04-19
  project: my-project
```

---

## Part IV: Provider System

### 4.1 LLM Providers (22+)

```yaml
# .config/providers.yaml
llm_providers:
  - name: anthropic
    sdk: @ai-sdk/anthropic
    models: [claude-opus-4-5, claude-sonnet-4-5, claude-haiku-3-5]
    
  - name: openai
    sdk: @ai-sdk/openai
    models: [gpt-4o, gpt-4o-mini, gpt-4-turbo, gpt-5]
    
  - name: google
    sdk: @ai-sdk/google
    models: [gemini-2-pro, gemini-2-flash]
    
  - name: google-vertex
    sdk: @ai-sdk/google-vertex
    models: [gemini-vertex-pro, gemini-vertex-ai]
    
  - name: amazon-bedrock
    sdk: @ai-sdk/amazon-bedrock
    models: [claude3-bedrock, llama3-bedrock]
    
  - name: azure
    sdk: @ai-sdk/azure
    models: [gpt-4-azure, gpt-35-turbo]
    
  - name: mistral
    sdk: @ai-sdk/mistral
    models: [mistral-large, mistral-small]
    
  - name: groq
    sdk: @ai-sdk/groq
    models: [llama3-70b-groq, mixtral-8x7b]
    
  - name: deepinfra
    sdk: @ai-sdk/deepinfra
    models: [Llama-3-70B, Mistral-7B]
    
  - name: cerebras
    sdk: @ai-sdk/cerebras
    models: [cerebras-llama3]
    
  - name: cohere
    sdk: @ai-sdk/cohere
    models: [command-r-plus]
    
  - name: togetherai
    sdk: @ai-sdk/togetherai
    models: [Llama-3-70B, Mixtral-8x7B]
    
  - name: perplexity
    sdk: @ai-sdk/perplexity
    models: [llama-3-sonnet-perplexity]
    
  - name: vercel
    sdk: @ai-sdk/vercel
    models: [gpt-4o-vercel]
    
  - name: xai
    sdk: @ai-sdk/xai
    models: [grok-2-beta]
    
  - name: openrouter
    sdk: @openrouter/ai-sdk-provider
    models: [claude-sonnet, gpt-4o]
    
  - name: openai-compatible
    sdk: @ai-sdk/openai-compatible
    models: [custom]
    
  - name: gateway
    sdk: @ai-sdk/gateway
    
  - name: gitlab
    sdk: @gitlab/gitlab-ai-provider
    
  - name: github-copilot
    sdk: @ai-sdk/github-copilot
    
  - name: vertex-anthropic
    sdk: @ai-sdk/google-vertex/anthropic
    
  - name: opencode
    sdk: custom
    models: [opencode-minimax]
```

### 4.2 Skills Providers

```yaml
# .config/skills-providers.yaml
skills_providers:
  - name: skills.sh
    source: https://github.com/skills.sh/skill-directory
    count: 200+
    
  - name: awesome-claude-skills
    source: https://github.com/ComposioHQ/awesome-claude-skills
    count: 1000+
    
  - name: skillsmp
    source: https://skillsmp.com
    count: 500+
    
  - name: local
    source: .skills/
    count: project-specific
```

### 4.3 Usage

```bash
# Switch LLM provider
/model anthropic/claude-sonnet-4-5
/model openai/gpt-4o
/model google/gemini-2-pro

# Skills are automatic
> Add PDF export
→ AI searches skills → finds match → uses it
```

---

## Part V: File Structure

### 5.1 Complete Project Hierarchy

```
PROJECT/
├── GM.md                       ← Brand (if multi-app)
├── PROJECT.md                 ← Vision + tech stack
├── STATUS.md                  ← Current state (AI updates)
├── DECISIONS.md              ← Decision log with timestamps
├── sessions/                 ← ALL conversation history
│   ├── 2025-04-01-init.md
│   ├── 2025-04-02-audit-login.md
│   ├── 2025-04-03-spec-login.md
│   └── ...
│
├── features/                ← ROOMS
│   ├── login/
│   │   ├── AUDIT.md        ← Fresh questions + ideas
│   │   ├── INTENT.md       ← User's original intent
│   │   ├── SPEC.md         ← Technical spec
│   │   ├── PROGRESS.md     ← Task checkboxes
│   │   ├── BUILD/          ← Code files
│   │   │   ├── LoginView.ts
│   ��   │   ├── auth.ts
│   │   │   └── jwt.ts
│   │   └── VERIFY/          ← Test results
│   │       ├── lint.txt
│   │       └── test.txt
│   │
│   ├── database/
│   │   ├── AUDIT.md
│   │   ├── INTENT.md
│   │   ├── SPEC.md
│   │   ├── PROGRESS.md
│   │   └── BUILD/
│   │
│   └── [more features...]
│
├── .design/                  ← TOKENS
│   └── DESIGN.md            ← Colors, typography
│
├── .skills/                  ← SKILLS
│   ├── index.yaml           ← All available (index)
│   ├── installed/            ← Installed skills
│   │   ├── pdf-generator/
│   │   └── email-sender/
│   └── favorites.yaml       ← User favorites ⭐
│
└── .opencode/                ← CONFIG
    ├── AGENTS.md            ← Project rules
    └── config.yaml          ← Provider config
```

---

## Part VI: Memory System

### 6.1 Three Memory Types Combined

| Memory | What | Format | Purpose |
|--------|------|--------|---------|
| **sessions/** | Conversations | Markdown | Full history |
| **PROGRESS.md** | Tasks | Checkboxes | Current work |
| **DECISIONS.md** | Choices | Timestamped | Why choices |

### 6.2 Auto-Detection On Continue

```rust
fn load_autonomous_context(project_path) -> Context {
    // 1. Load main PROGRESS for current feature
    let main = read(project_path + "PROGRESS.md");
    let current = main.current_feature();
    
    // 2. Load feature-specific tasks
    let tasks = read(project_path + "features/" + current + "/PROGRESS.md");
    
    // 3. Load recent sessions
    let history = search_sessions(project_path, current, limit=3);
    
    // 4. Load decisions
    let decisions = read(project_path + "DECISIONS.md");
    
    // 5. Check skills (favorites + installed)
    let skills = load_skills(project_path);
    
    return Context {
        feature: current,
        tasks: tasks,
        history: history,
        decisions: decisions,
        skills: skills
    };
}
```

---

## Part VII: Autonomous Build Mode

### 7.1 Fresh Context Per Task (GSD-2)

```
AUTONOMOUS BUILD:
─────────────────────────────────────

Task 1: Build login email input
→ Fresh 200k context for this task only
→ Build → verify → done
→ Clear context

Task 2: Build JWT validation  
→ NEW fresh 200k context ← starts clean!
→ Build → verify → done
→ Clear context

Task 3: Database schema
→ NEW fresh 200k context ← starts clean!
→ Build → verify → done

Result: No context pollution. Each task focused.
```

### 7.2 Auto-Compaction (caveman)

```rust
fn maybe_compress(context_tokens: u32) {
    if context_tokens > 150_000 {
        let compressed = caveman_compress(context);
        info!("Compressed: {} → {} tokens", 
              context_tokens, compressed.tokens);
        // 46% reduction
    }
}
```

### 7.3 YOLO Mode (Skip Permissions)

```bash
# With YOLO - no prompts
> everthink build all --yolo

# Without YOLO - ask for sensitive ops
> everthink build all
```

### 7.4 Permissions Config

```yaml
# .config/permissions.yaml

# Mode: yolo (no prompts), ask (always ask), allow (let AI decide)
mode: ask

# Per-tool rules (only applies when mode: ask)
rules:
  Bash:
    allow: ["git *", "npm *", "cargo *"]
    ask: ["rm *", "sudo *", "curl | bash"]
    
  Edit:
    allow: ["src/*", "tests/*"]
    ask: ["*"]
    
  WebFetch:
    allow: ["https://api.*"]
    ask: ["*"]
```

### 7.5 Full Autonomous Flow

```rust
fn autonomous_build(project_path) {
    let tasks = load_pending_tasks(project_path);
    
    while let Some(task) = tasks.next() {
        print!("Building: {}...", task.name);
        
        // Fresh context for this task
        let ctx = fresh_context(task);
        
        // Build
        let result = ai_build(ctx, task);
        
        // Verify
        let verified = verify(result);
        
        if verified {
            // Update progress
            update_progress(task, done: true);
            update_sessions(task, result);
        } else {
            // Self-correct and retry
            let fixed = ai_fix(result.error);
            retry!(fixed, verify);
        }
        
        // Compress if needed
        maybe_compress(current_tokens);
    }
    
    print!("All done!");
}
```

---

## Part VIII: TUI Features

### 8.1 Slash Commands

| Command | Description |
|---------|-------------|
| `/commit` | Smart git commit |
| `/review` | Code review of changes |
| `/model [name]` | Switch LLM provider |
| `/clear` | Clear session |
| `/session` | List/switch/manage sessions |
| `/agent [name]` | Switch agent (Build/Plan) |
| `/help` | Show all commands |
| `/for [cmd]` | Run command for selected text |

### 8.2 Tab Mode Switching

| Key | Action |
|-----|--------|
| `Tab` | Switch between Build/Plan agents |
| `Ctrl+Tab` | Cycle through subagents |

### 8.3 Select = Copy

Mouse select automatically copies to clipboard.

### 8.4 Context Show

```
/context show

─────────────────────────────────────
Context: 45,000 tokens
Features: login, database
Sessions: 4 conversations
Last: Today 10:30am
─────────────────────────────────────
```

---

## Part IX: Tool System

### 9.1 Core Tools

| Tool | Purpose | Example |
|------|---------|---------|
| **Bash** | Execute commands | `git add .` |
| **Glob** | Find files | `**/*.ts` |
| **Grep** | Search content | `login auth` |
| **Read** | View files | `src/main.ts` |
| **Write** | Create files | `new file.txt` |
| **Edit** | Modify files | `change line 42` |
| **Patch** | Apply diffs | `git diff` |
| **WebFetch** | HTTP GET/POST | Get API docs |
| **WebSearch** | Search web | Find solutions |
| **CodeSearch** | API search | Context7 |
| **Tool** | Call subagent | `@general` |
| **Skill** | Load skill | `pdf-generator` |
| **MCP** | External tools | Custom |

### 9.2 Verification Suite

```rust
fn verify_build() -> Verification {
    let lint = run("cargo clippy --fix");
    let build = run("cargo build");
    let test = run("cargo test");
    
    return Verification {
        lint: lint.passed,
        build: build.passed,
        tests: test.passed,
        count: test.passed_count
    };
}
```

---

## Part X: Agent System

### 10.1 Built-in Agents

| Agent | Purpose | Permissions |
|-------|---------|-------------|
| **Build** | Full development | All tools |
| **Plan** | Read-only exploration | No edits, ask for bash |
| **General** | Complex multi-step | Subagent |

### 10.2 Switching

```bash
# Switch to Build agent
/agent build
# or press Tab

# Switch to Plan (read-only)
/agent plan

# Use subagent
@general "Refactor all auth code"
```

---

## Part XI: Session Management

### 11.1 Auto-Save

Every message saved to `sessions/` automatically.

### 11.2 Resume Mode

```
> /session
Sessions:
- Login feature (today 10:30am) [active]
- Database design (Apr 15)
- Auth audit (Apr 12)

> /resume login
Resumed: Login feature
Full context loaded
```

### 11.3 Export/Import

```
/session export login.json
/session import login.json
```

---

## Part XII: GM/G Simplified

### 12.1 Single Project

```
PROJECT/
├── PROJECT.md     ← Project vision
```

### 12.2 Multi-Project (Same Brand)

```
my-brand/                        ← Brand (GM.md shared)
├── GM.md                        ← Brand identity
│
├── app-ios/                     ← Repo 1
│   ├── GM.md (symlink)
│   └── PROJECT.md
│
├── app-android/                  ← Repo 2
│   ├── GM.md (symlink)
│   └── PROJECT.md
│
└── web/                         ← Repo 3
    ├── GM.md (symlink)
    └── PROJECT.md
```

---

## Part XIII: Ownership Model

### 13.1 Clear Separation

| Human Owns | AI Owns |
|------------|----------|
| WHAT (PROJECT.md) | ARCH (tech stack) |
| INTENT (feature intent) | SPEC (technical) |
| APPROVAL (checkpoints) | CODE (implementation) |
| DECISIONS (final choices) | MAINTAIN (PROGRESS.md) |

### 13.2 Three Stop Points

1. **PROJECT.md** — Before work begins
2. **SPEC.md** — Before code written
3. **Final output** — Before feature complete

---

## Part XIV: Complete Command Reference

### 14.1 Core Commands

| Command | Purpose |
|---------|---------|
| `init [idea]` | New project |
| `add [feature]` | Add feature (runs AUDIT) |
| `build` | Build current feature |
| `build all` | Autonomous: build all pending |
| `verify` | Run verification |
| `continue` | Continue from last |

### 14.2 Context Commands

| Command | Purpose |
|---------|---------|
| `search [query]` | Code search |
| `remember [topic]` | Load history |
| `why [decision]` | Show decision |

### 14.3 Slash Commands

| Command | Purpose |
|---------|---------|
| `/commit` | Smart commit |
| `/review` | Code review |
| `/model [name]` | Switch model |
| `/clear` | Clear session |
| `/session` | Manage sessions |
| `/agent [name]` | Switch agent |
| `/help` | Show help |

### 14.4 Flags

| Flag | Purpose |
|------|---------|
| `--yolo` | Skip all prompts |
| `--fresh` | Always fresh context |
| `--all` | Build all pending |

---

## Part XV: Comparison

### 15.1 vs Claude Code

| Feature | Claude Code | Everthink V6 |
|--------|-------------|--------------|
| Memory | Session | Persistent |
| AUDIT | None | Every feature |
| Skills | Manual | 3-tier auto |
| Providers | 1 | 22+ |
| Autonomous | No | Full |
| YOLO | No | Yes |

### 15.2 vs OpenCode

| Feature | OpenCode | Everthink V6 |
|--------|----------|--------------|
| Language | TypeScript | Rust |
| Binary | Node.js | Native |
| Skills | Basic | 3-tier |
| Providers | 22 | 22 + Skills |
| AUDIT | None | Full |
| Autonomous | No | Full |

---

## Part XVI: Glossary

| Term | Definition |
|------|-------------|
| **AUDIT** | AI questioning + ambition phase |
| **Wing** | Project-level memory |
| **Room** | Feature-level memory |
| **Drawer** | Session storage |
| **YOLO mode** | Skip all prompts |
| **GSD-2** | Fresh 200k context per task |
| **caveman** | 46% token compression |
| **Favorites** | Quick-access skills |

---

## Part XVII: Technical Implementation

### 17.1 Rust Core Architecture

```rust
// src/main.rs
mod cli;
mod core;
mod tools;
mod storage;

fn main() {
    let args = CLI::parse();
    
    match args.command {
        Command::Init(idea) => core::init(idea),
        Command::Add(feature) => core::audit(feature),
        Command::Build(all) => core::build(all),
        Command::Continue => core::continue_autonomous(),
        Command::Verify => core::verify(),
        Command::Search(query) => tools::greppy::search(query),
        Command::Remember(topic) => storage::memory::remember(topic),
        Command::Model(name) => core::switch_model(name),
        Command::Session(cmd) => storage::session::manage(cmd),
        Command::Agent(name) => core::switch_agent(name),
    }
}
```

### 17.2 Python Engine Integration

```rust
// src/tools/python_engine.rs
use std::process::Command;

pub struct PythonEngine {
    script_path: String,
}

impl PythonEngine {
    pub fn new(script: &str) -> Self {
        Self {
            script_path: script.to_string(),
        }
    }
    
    pub fn execute(&self, tool: &str, args: serde_json::Value) -> Result<String> {
        let output = Command::new("python3")
            .arg(&self.script_path)
            .arg("--tool")
            .arg(tool)
            .arg("--args")
            .arg(args.to_string())
            .output()?;
            
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(Error::from(format!("Python error: {}", String::from_utf8_lossy(&output.stderr))))
        }
    }
}
```

### 17.3 Tool Registry

```rust
// src/tools/registry.rs
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn register<T: Tool + 'static>(&mut self, name: &str, tool: T) {
        self.tools.insert(name.to_string(), Box::new(tool));
    }
    
    pub fn get(&self, name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.get(name)
    }
    
    pub fn all(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
}

pub trait Tool {
    fn name(&self) -> &str;
    fn execute(&self, args: Value) -> Result<Value>;
    fn description(&self) -> &str;
}
```

### 17.4 LLM Provider Integration

```rust
// src/core/llm.rs
pub struct LLMClient {
    provider: Provider,
    model: String,
    api_key: String,
}

impl LLMClient {
    pub fn new(provider: Provider, model: &str) -> Result<Self> {
        let api_key = Self::load_api_key(&provider)?;
        Ok(Self { provider, model: model.to_string(), api_key })
    }
    
    pub async fn complete(&self, messages: Vec<Message>) -> Result<Response> {
        let request = Request {
            model: &self.model,
            messages,
            temperature: 0.7,
            max_tokens: None,
        };
        
        let client = reqwest::Client::new();
        let response = client
            .post(&self.provider.endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;
            
        Ok(response.json()?)
    }
}
```

### 17.5 Skills Discovery

```rust
// src/tools/skills.rs
pub struct SkillsManager {
    library: SkillsLibrary,
    installed: Vec<InstalledSkill>,
    favorites: Vec<FavoriteSkill>,
}

impl SkillsManager {
    pub fn search(&self, query: &str) -> Vec<SkillMatch> {
        let mut results = Vec::new();
        
        // Search library
        for skill in &self.library.skills {
            let score = self.similarity(query, &skill.name);
            if score > 0.7 {
                results.push(SkillMatch { skill, score, source: "library" });
            }
        }
        
        // Search installed
        for skill in &self.installed {
            let score = self.similarity(query, &skill.name);
            if score > 0.7 {
                results.push(SkillMatch { skill, score, source: "installed" });
            }
        }
        
        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }
    
    pub fn get_favorites(&self) -> Vec<&FavoriteSkill> {
        self.favorites.iter().collect()
    }
}
```

### 17.6 Context Fresh (GSD-2 Integration)

```rust
// src/core/context.rs
pub struct ContextManager {
    max_tokens: u32,
    compression_threshold: f64,
    current_tokens: u32,
}

impl ContextManager {
    pub fn new_task_context(&mut self, task: &Task) -> Context {
        // Clear previous context
        self.clear();
        
        // Load task-specific context
        let mut ctx = Context::new();
        
        // Add project files
        ctx.add_files(self.load_project_files(task));
        
        // Add relevant sessions
        ctx.add_sessions(self.load_relevant_sessions(task));
        
        // Add decisions
        ctx.add_decisions(self.load_decisions(task));
        
        // Add skills
        ctx.add_skills(self.load_skills(task));
        
        self.current_tokens = ctx.token_count();
        ctx
    }
    
    pub fn maybe_compress(&mut self, ctx: &mut Context) {
        if self.current_tokens as f64 > self.max_tokens as f64 * self.compression_threshold {
            let compressed = caveman_compress(ctx);
            *ctx = compressed;
            self.current_tokens = ctx.token_count();
        }
    }
}
```

### 17.7 Session Management

```rust
// src/storage/session.rs
pub struct SessionManager {
    sessions_dir: PathBuf,
    current_session: Option<Session>,
}

impl SessionManager {
    pub fn new_session(&mut self, project: &Project) -> Session {
        let id = ULID::new();
        let session = Session {
            id,
            project: project.name.clone(),
            created_at: Utc::now(),
            messages: Vec::new(),
            context_tokens: 0,
        };
        
        self.current_session = Some(session.clone());
        self.save_session(&session);
        
        session
    }
    
    pub fn save_message(&mut self, message: &Message) {
        if let Some(ref mut session) = self.current_session {
            session.messages.push(message.clone());
            self.save_session(session);
        }
    }
    
    pub fn load_session(&self, id: &str) -> Option<Session> {
        let path = self.sessions_dir.join(format!("{}.json", id));
        if path.exists() {
            Some(serde_json::from_file(&path).ok())
        } else {
            None
        }
    }
    
    pub fn export_session(&self, id: &str) -> Result<String> {
        let session = self.load_session(id)
            .ok_or(Error::SessionNotFound)?;
        Ok(serde_json::to_string_pretty(&session)?)
    }
    
    pub fn import_session(&mut self, json: &str) -> Result<Session> {
        let session: Session = serde_json::from_str(json)?;
        self.save_session(&session);
        Ok(session)
    }
}
```

### 17.8 Verification Pipeline

```rust
// src/tools/verify.rs
pub struct VerificationPipeline {
    linter: Linter,
    builder: Builder,
    tester: Tester,
}

impl VerificationPipeline {
    pub fn run(&self, project: &Project) -> VerificationResult {
        let mut result = VerificationResult::new();
        
        // Step 1: Lint
        print!("Running lint...");
        match self.linter.run(project) {
            Ok(output) => result.lint = Some(output),
            Err(e) => result.errors.push(format!("Lint failed: {}", e)),
        }
        
        // Step 2: Build
        print!("Building...");
        match self.builder.run(project) {
            Ok(output) => result.build = Some(output),
            Err(e) => result.errors.push(format!("Build failed: {}", e)),
        }
        
        // Step 3: Test
        print!("Running tests...");
        match self.tester.run(project) {
            Ok(output) => result.tests = Some(output),
            Err(e) => result.errors.push(format!("Tests failed: {}", e)),
        }
        
        result.success = result.errors.is_empty();
        result
    }
    
    pub fn auto_fix(&self, result: &mut VerificationResult) {
        // Try to fix lint errors
        if let Some(ref mut lint) = result.lint {
            if let Ok(fixed) = self.linter.auto_fix(lint) {
                *lint = fixed;
            }
        }
        
        // Retry verification
        // ... implementation
    }
}
```

### 17.9 Permission System

```rust
// src/core/permissions.rs
pub struct PermissionManager {
    mode: PermissionMode,
    rules: Vec<PermissionRule>,
}

#[derive(Clone)]
pub enum PermissionMode {
    Yolo,        // No prompts
    Ask,         // Always ask
    Allow,       // Let AI decide
}

impl PermissionManager {
    pub fn check(&self, tool: &str, args: &Value) -> PermissionResult {
        match self.mode {
            PermissionMode::Yolo => PermissionResult::Allowed,
            PermissionMode::Allow => PermissionResult::Allowed,
            PermissionMode::Ask => {
                for rule in &self.rules {
                    if rule.matches(tool, args) {
                        return match rule.action {
                            RuleAction::Allow => PermissionResult::Allowed,
                            RuleAction::Deny => PermissionResult::Denied,
                            RuleAction::Ask => PermissionResult::Ask,
                        };
                    }
                }
                PermissionResult::Ask
            }
        }
    }
}
```

### 17.10 Progress Tracking

```rust
// src/storage/progress.rs
pub struct ProgressTracker {
    project: ProjectPath,
}

impl ProgressTracker {
    pub fn load(&self) -> Progress {
        let path = self.project.join("PROGRESS.md");
        if path.exists() {
            serde_json::from_file(&path).unwrap_or_default()
        } else {
            Progress::default()
        }
    }
    
    pub fn update(&self, feature: &str, task: &str, status: TaskStatus) {
        let mut progress = self.load();
        
        if let Some(feat) = progress.features.iter_mut().find(|f| f.name == feature) {
            if let Some(t) = feat.tasks.iter_mut().find(|t| t.name == task) {
                t.status = status;
                t.updated_at = Utc::now();
            }
        }
        
        self.save(&progress);
    }
    
    pub fn next_task(&self) -> Option<(String, String)> {
        let progress = self.load();
        
        for feature in &progress.features {
            for task in &feature.tasks {
                if task.status == TaskStatus::Pending {
                    return Some((feature.name.clone(), task.name.clone()));
                }
            }
        }
        
        None
    }
}
```

---

## Part XVIII: User Experience Details

### 18.1 First Launch Experience

```
$ everthink

Welcome to Everthink IDE!
─────────────────────────────────────

First time setup:
1. Choose your LLM provider:
   - anthropic (Claude)
   - openai (GPT)
   - google (Gemini)
   - [other providers]

2. Enter API key: [hidden input]

3. Choose default model: [model list]

Done! Type "init [your project idea]" to begin.

─────────────────────────────────────
```

### 18.2 Init Experience

```
$ everthink init "Habit tracker without guilt"

Creating project: Habit Tracker
─────────────────────────────────────

Let me understand your vision:

Q1: What problem does this solve?
> People quit habit apps because of gamification

Q2: Who is this for?
> People who've tried every app and quit

Q3: What's the core idea?
> Quiet accountability, no rewards, no streaks

Q4: Platform?
> iOS first, maybe Android

Q5: Tech stack?
> SwiftUI, SQLite

─────────────────────────────────────

Project created: habit-tracker/
├── PROJECT.md
├── STATUS.md
├── DECISIONS.md
├── sessions/
└── features/

Ready to add features. Try: add "login"
```

### 18.3 Adding First Feature

```
$ everthink add login

Running AUDIT for login feature...
─────────────────────────────────────

Based on your project, here are my questions:

Q1: Authentication method?
> Email only (no username)

Q2: Social login?
> Not for V1, maybe V2

Q3: Password reset?
> Email link flow

Q4: Session duration?
> 30 days

─────────────────────────────────────

💡 AI Suggestions:
• Add biometric (Face ID/Touch ID) for mobile?
> Yes, add it

• Passwordless magic links as alternative?
> Interesting, consider for V2

• Remember device for 30 days?
> Yes

─────────────────────────────────────

AUDIT.md saved.

Ready to create SPEC.md? (yes/no)
> yes
```

### 18.4 Autonomous Build

```
$ everthink build all --yolo

╔════════════════════════════════════════╗
║     AUTONOMOUS BUILD MODE            ║
╚════════════════════════════════════════╝

Loading PROGRESS.md...
Found: 5 pending tasks

─────────────────────────────────────

[1/5] Building: Login email input
→ Fresh context: 45k tokens
→ Building...
→ Lint: ✓
→ Build: ✓
→ Tests: ✓ 3/3
→ Updated: PROGRESS.md
✓ DONE

[2/5] Building: JWT validation
→ Fresh context: 42k tokens
→ Building...
→ Lint: ✓
→ Build: ✓
→ Tests: ✓ 5/5
✓ DONE

[3/5] Building: Biometric authentication
→ Fresh context: 48k tokens
→ Building...
→ Lint: ✓
→ Build: ✓
→ Tests: ✓ 4/4
✓ DONE

[4/5] Building: Database schema
→ Fresh context: 52k tokens
→ Building...
→ Lint: ✓
→ Build: ✓
→ Tests: ✓ 2/2
✓ DONE

[5/5] Building: Session management
→ Fresh context: 38k tokens
→ Building...
→ Lint: ✓
→ Build: ✓
→ Tests: ✓ 3/3
✓ DONE

─────────────────────────────────────

🎉 ALL TASKS COMPLETED

Total: 5 features built
Verification: All passed
Time: 4m 32s

Blocked: None!
```

### 18.5 Continue After Break

```
$ everthink continue

Loading context...
─────────────────────────────────────

Last session: Apr 15, 2025
Feature: Login
Progress: 3/5 tasks done

Remaining:
[ ] JWT validation
[ ] Biometric auth
[ ] Session management

─────────────────────────────────────

→ Fresh context loaded: 35k tokens
→ Recent sessions: Apr 15 conversation loaded
→ Decisions: Email-only, JWT, Biometric added

Continue building JWT validation? (yes/no)
> yes
```

### 18.6 Manual Permission

```
$ everthink build

Building: Login feature

Tool call: Bash("rm -rf node_modules")
─────────────────────────────────────
⚠️  PERMISSION REQUIRED

Tool: Bash
Command: rm -rf node_modules

Allow once? (a) / Deny (d) / Allow always (A)
> a

Executing...

─────────────────────────────────────
```

---

## Part XIX: Configuration Files

### 19.1 Project Config

```toml
# everthink.toml
[project]
name = "habit-tracker"
version = "0.1.0"

[llm]
provider = "anthropic"
model = "claude-sonnet-4-5"

[autonomous]
fresh_context = true
auto_compact = true
yolo_mode = false

[skills]
auto_discover = true
favorites = ["pdf-generator", "email-sender"]

[permissions]
mode = "ask"
rules = [
    { tool = "Bash", pattern = "git *", action = "allow" },
    { tool = "Bash", pattern = "rm *", action = "ask" },
    { tool = "Edit", pattern = "src/*", action = "allow" },
]
```

### 19.2 Global Config

```toml
# ~/.config/everthink/config.toml
[providers]
anthropic.key = "$ANTHROPIC_API_KEY"
openai.key = "$OPENAI_API_KEY"

[defaults]
provider = "anthropic"
model = "claude-sonnet-4-5"

[ui]
theme = "dark"
editor = "vim"

[keys]
commit = "Ctrl+s"
clear = "Ctrl+l"
```

### 19.3 Skills Config

```yaml
# .skills/index.yaml
version: 1

skills:
  - name: pdf-generator
    source: skills.sh
    purpose: Generate PDF documents
    tags: [pdf, document, export, invoice]
    example: "Generate invoice PDF"
    
  - name: email-sender
    source: awesome-claude-skills
    purpose: Send emails
    tags: [email, notification, smtp]
    example: "Send welcome email"
    
  - name: webhook-handler
    source: skills.sh
    purpose: Handle webhooks
    tags: [api, webhook, http]
    example: "Process Stripe webhook"

providers:
  - name: skills.sh
    url: https://api.skills.sh/v1
    count: 200
    
  - name: awesome-claude-skills
    url: https://github.com/ComposioHQ/awesome-claude-skills
    count: 1000
```

---

## Part XX: Error Handling

### 20.1 Error Types

```rust
#[derive(Error, Debug)]
pub enum EverthinkError {
    #[error("Project not found: {0}")]
    ProjectNotFound(String),
    
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    #[error("Feature not found: {0}")]
    FeatureNotFound(String),
    
    #[error("Skill not found: {0}")]
    SkillNotFound(String),
    
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    #[error("LLM error: {0}")]
    LLMError(String),
    
    #[error("Build error: {0}")]
    BuildError(String),
    
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Context too large: {0}")]
    ContextTooLarge(u32),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
```

### 20.2 Error Recovery

```rust
impl Error {
    pub fn retry(&self) -> bool {
        match self {
            EverthinkError::LLMError(_) => true,
            EverthinkError::BuildError(_) => true,
            EverthinkError::VerificationFailed(_) => true,
            _ => false,
        }
    }
    
    pub fn auto_fix(&self) -> Option<Action> {
        match self {
            EverthinkError::VerificationFailed(msg) => {
                if msg.contains("lint") {
                    Some(Action::RunLinterAutoFix)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
```

---

## Part XXI: Performance Considerations

### 21.1 Token Budget

| Operation | Budget | Notes |
|-----------|--------|-------|
| Fresh context | 200,000 | GSD-2 default |
| Compressed | 108,000 | 46% reduction |
| Warning threshold | 150,000 | Start compression |
| Hard limit | 200,000 | Force fresh context |

### 21.2 Caching Strategy

| Cache | TTL | Size |
|-------|-----|------|
| Skills index | 24h | ~1MB |
| Session summary | Session | ~10KB |
| File listings | 1h | ~100KB |
| LLM responses | 24h | ~10MB |

### 21.3 Parallel Operations

```rust
pub async fn parallel_verify(&self, project: &Project) -> Vec<VerificationResult> {
    let tasks = vec![
        self.lint(project),
        self.build(project),
        self.test(project),
    ];
    
    // Run all verifications in parallel
    let results = join_all(tasks).await;
    
    results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect()
}
```

---

## Part XXII: Security

### 22.1 API Key Storage

API keys stored in OS keychain:
- macOS: Keychain
- Linux: Secret Service / pass
- Windows: Credential Manager

### 22.2 Dangerous Command Blocking

```rust
fn is_dangerous(command: &str) -> bool {
    let dangerous = [
        "rm -rf /",
        "rm -rf /*",
        "dd if=",
        ":(){ :|:& };:",
        "curl | bash",
        "wget | bash",
        "mkfs",
        "shutdown",
        "reboot",
        "halt",
        "init 0",
        "init 6",
    ];
    
    dangerous.iter().any(|d| command.contains(d))
}
```

### 22.3 Sandbox

```rust
pub struct Sandbox {
    allowed_paths: Vec<PathBuf>,
    max_memory: u64,
    max_cpu: u32,
}

impl Sandbox {
    pub fn execute(&self, cmd: &str) -> Result<Output> {
        // Restrict to project directory
        if !cmd.starts_with(&self.allowed_paths) {
            return Err(Error::PathNotAllowed);
        }
        
        // Apply limits and execute
        // ...
    }
}
```

---

## Part XXIII: Testing

### 23.1 Test Coverage

| Area | Tests | Priority |
|------|-------|----------|
| Tool registry | Unit | High |
| Session management | Unit + Integration | High |
| Verification pipeline | Integration | High |
| Skills discovery | Unit | Medium |
| Context compression | Unit | Medium |
| Permission system | Integration | High |
| Autonomous build | E2E | Critical |

### 23.2 Benchmarking

| Operation | Target | Current |
|-----------|--------|---------|
| Cold start | <1s | TBD |
| Tool execution | <100ms | TBD |
| Context load | <500ms | TBD |
| Verification | <30s | TBD |
| Token compression | <1s | TBD |

---

## Part XXIV: Future Considerations

### 24.1 Post-MVP Features

| Feature | Description | Complexity |
|---------|-------------|------------|
| Image upload | Via sixel/iterm2 | Medium |
| Voice input | Speech to text | Medium |
| Plugin SDK | Third-party tools | High |
| Team collaboration | Shared sessions | High |
| Cloud sync | Cross-device | High |
| Web UI | Browser interface | High |

### 24.2 Advanced Agents

| Agent | Purpose | Status |
|-------|---------|--------|
| Security | Security audit | Planned |
| Performance | Performance profiling | Planned |
| Accessibility | A11y audit | Planned |
| TestGen | Auto-generate tests | Planned |
| Refactor | Code refactoring | Planned |

---

## Summary

V6 is the ultimate AI coding IDE combining:

| Feature | Implementation |
|--------|----------------|
| Identity | CORE prompt + AGENTS.md |
| AUDIT | Fresh questions per feature |
| Skills | Library → Installed → Favorites (3-tier) |
| Providers | 22+ LLMs + Skills sources |
| Memory | sessions + PROGRESS + DECISIONS |
| Autonomous | Fresh + Compact + YOLO |
| TUI | All slash commands |
| Agents | Build / Plan / Subagents |
| Verification | Auto lint/build/test |
| GM/G | Simplified |
| Technical | Full Rust implementation |
| UX | Complete flow examples |
| Config | YAML + TOML |
| Security | API keys + sandbox |

---

**Document Version:** 6.0
**Created:** April 2026
**Status:** Complete specification ready for implementation
**Lines:** 1500+