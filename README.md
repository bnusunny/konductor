# Konductor

Spec-driven development for Kiro CLI — orchestrates project planning, execution, verification, and shipping through a structured pipeline.

## What It Does

Konductor transforms high-level project requirements into working software through a systematic six-phase pipeline:

1. **Initialize** — Discovers project goals, generates spec documents (project.md, requirements.md, roadmap.md)
2. **Research** — Analyzes codebase, maps dependencies, identifies patterns and conventions
3. **Plan** — Breaks work into phases with specific tasks, acceptance criteria, and cross-phase regression gates
4. **Execute** — Implements tasks with TDD, creates tests before code, maintains context freshness via subagents
5. **Verify** — Runs tests, validates acceptance criteria, ensures quality gates pass
6. **Ship** — Creates release notes, commits work, opens pull requests, moves to next phase

Each phase outputs artifacts to `.konductor/` that guide the next phase, preventing context rot and ensuring consistency.

## Quick Install

Clone and run the installer:

```bash
git clone https://github.com/bnusunny/konductor.git
cd konductor
./install.sh
```

Or use the one-line installer:

```bash
curl -fsSL https://konductor.cloud/install | bash
```

Installs to `~/.kiro/` (global) or `./.kiro/` (local project). Use `--force` to overwrite existing files.

## Quick Start

Start the Konductor orchestrator:

```bash
kiro-cli --agent konductor
```

Initialize your project:

```
> @k-init
```

Advance through the pipeline:

```
> @k-next
> @k-next
> @k-next
```

Check current status:

```
> @k-status
```

Execute specific phases:

```
> @k-plan 01
> @k-exec
> @k-verify
> @k-ship
```

## Pipeline Diagram

```
┌─────────────┐
│ Initialize  │ → project.md, requirements.md, roadmap.md
└──────┬──────┘
       ↓
┌─────────────┐
│  Research   │ → structure.md, tech.md, patterns.md
└──────┬──────┘
       ↓
┌─────────────┐
│    Plan     │ → phases/*.md (with tasks + acceptance criteria)
└──────┬──────┘
       ↓
┌─────────────┐
│  Execute    │ → working code + tests
└──────┬──────┘
       ↓
┌─────────────┐
│   Verify    │ → test results, quality checks
└──────┬──────┘
       ↓
┌─────────────┐
│    Ship     │ → commits, PRs, release notes
└─────────────┘
```

## Commands

| Command | Prompt Shortcut | Description |
|---------|----------------|-------------|
| `initialize` | `@k-init` | Start a new project, discover requirements, generate spec docs |
| `next` | `@k-next` | Advance to the next pipeline phase automatically |
| `plan` | `@k-plan` | Generate phase plans with tasks and acceptance criteria |
| `exec` | `@k-exec` | Execute tasks for current phase (TDD workflow) |
| `verify` | `@k-verify` | Run tests and validate acceptance criteria |
| `status` | `@k-status` | Show current phase, progress, and next steps |
| `ship` | `@k-ship` | Commit work, create release notes, open PR |
| `discuss` | `@k-discuss` | Ask questions about the project, codebase, or pipeline |
| `map-codebase` | `@k-map` | Analyze existing code structure, patterns, and conventions |

> **Tip:** Type `@k-` then press Tab to autocomplete any Konductor command.

## MCP Tools

The orchestrator agent uses these tools via the built-in MCP server for deterministic state management:

| Tool | Description |
|------|-------------|
| `state_get` | Read current pipeline state from `state.toml` |
| `state_transition` | Advance to a new step (with validation) |
| `state_add_blocker` | Add a blocker to the current phase |
| `state_resolve_blocker` | Resolve a blocker for a phase |
| `plans_list` | List plans for a phase with wave and completion status |
| `status` | Get a structured status report of the entire project |

## How It Works

Konductor is built on five key components:

**1. Orchestrator Agent**
The main `konductor` agent manages pipeline state and delegates work to specialized subagents. It coordinates phase transitions using MCP tools for deterministic state management.

**2. MCP Server (`konductor mcp`)**
A local MCP server provides:
- 9 prompts with Tab-completable shortcuts (`@k-init`, `@k-plan`, etc.) — with typed arguments where needed
- State management tools (`state_get`, `state_transition`, `state_add_blocker`, `state_resolve_blocker`) — eliminates fragile LLM-generated TOML
- Query tools (`plans_list`, `status`) — returns structured JSON instead of requiring the LLM to parse files

**3. Skills**
Each command is a skill (`konductor-init`, `konductor-plan`, `konductor-exec`, etc.) with structured instructions. Skills define:
- When to trigger (keywords like "initialize", "next", "plan")
- Step-by-step execution logic
- Output artifacts and verification steps
- Reference materials for best practices

**4. Worker Subagents**
Specialized agents handle specific tasks:
- `konductor-discoverer` — Interviews users to understand project goals
- `konductor-researcher` — Analyzes codebases and documents patterns
- `konductor-planner` — Creates detailed phase plans with tasks
- `konductor-executor` — Implements code following TDD principles
- `konductor-verifier` — Validates tests and acceptance criteria

**5. Hook System (`konductor hook`)**
The same `konductor` binary processes hook events:
- File modification tracking (detects changes to tracked files)
- Safety guardrails (prevents dangerous operations like `rm -rf /`)

### Context Rot Prevention

Each subagent starts with a fresh context containing only:
- Relevant spec documents from `.konductor/`
- Current phase artifacts
- Necessary reference materials

This prevents context pollution and ensures decisions are based on documented requirements, not accumulated conversation history.

## Customization

### Edit Skills

Modify skill instructions to match your workflow:

```bash
vim ~/.kiro/skills/konductor-exec/SKILL.md
```

### Edit Agents

Customize agent behavior and resources:

```bash
vim ~/.kiro/agents/konductor.json
```

### Create Custom Skills

Add new commands by creating a skill directory:

```bash
mkdir -p ~/.kiro/skills/konductor-deploy
cat > ~/.kiro/skills/konductor-deploy/SKILL.md <<EOF
---
name: konductor-deploy
description: Deploy the current phase to staging or production
---

# Your deployment workflow here
EOF
```

## File Layout

### Kiro Configuration (`.kiro/`)

```
.kiro/
├── agents/
│   ├── konductor.json           # Main orchestrator (includes MCP server config)
│   ├── konductor-discoverer.json
│   ├── konductor-planner.json
│   └── ...
├── skills/
│   ├── konductor-init/
│   ├── konductor-next/
│   ├── konductor-plan/
│   ├── konductor-exec/
│   ├── konductor-verify/
│   └── ...
├── hooks/
│   └── konductor-hooks.json      # Hook configuration
├── bin/
│   └── konductor                # Unified binary (mcp server + hook processor)
└── steering/
    ├── structure.md             # Codebase analysis
    └── tech.md                  # Tech stack details
```

### Project State (`.konductor/`)

```
.konductor/
├── project.md                   # Project vision
├── requirements.md              # User requirements
├── roadmap.md                   # High-level milestones
├── state.toml                   # Pipeline state tracker
├── config.toml                  # Project-specific config
├── phases/
│   ├── phase-01-foundation.md
│   ├── phase-02-core-features.md
│   └── ...
├── milestones/
│   └── v1.0-launch.md
├── .results/
│   ├── phase-01.json            # Execution results
│   └── tests/
└── .tracking/
    └── modified-files.json      # File change tracking
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes with tests
4. Commit with clear messages (`git commit -m 'Add feature X'`)
5. Push to your fork (`git push origin feature/my-feature`)
6. Open a Pull Request

### Development Setup

```bash
# Clone the repository
git clone https://github.com/bnusunny/konductor.git
cd konductor

# Build the konductor binary
cd konductor-cli
cargo build --release

# Install locally
cd ..
./install.sh --local
```

### Running Tests

```bash
# Test the konductor binary
cd konductor-cli
cargo test

# Test skills by running the installer
./install.sh --local
kiro-cli --agent konductor
```

## Acknowledgments

Konductor is inspired by [Get Shit Done (GSD)](https://github.com/gsd-build/get-shit-done), a spec-driven development framework for AI-assisted coding.

## License

MIT License - see [LICENSE](LICENSE) for details.
