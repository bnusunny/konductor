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
curl -fsSL https://konductor.dev/install | bash
```

Installs to `~/.kiro/` (global) or `./.kiro/` (local project).

## Quick Start

Start the Konductor orchestrator:

```bash
kiro-cli --agent konductor
```

Initialize your project:

```
> initialize my project
```

Advance through the pipeline:

```
> next
> next
> next
```

Check current status:

```
> status
```

Execute specific phases:

```
> plan phase 01
> execute
> verify
> ship
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

| Command | Description |
|---------|-------------|
| `initialize` | Start a new project, discover requirements, generate spec docs |
| `next` | Advance to the next pipeline phase automatically |
| `plan` | Generate phase plans with tasks and acceptance criteria |
| `exec` | Execute tasks for current phase (TDD workflow) |
| `verify` | Run tests and validate acceptance criteria |
| `status` | Show current phase, progress, and next steps |
| `ship` | Commit work, create release notes, open PR |
| `discuss` | Ask questions about the project, codebase, or pipeline |
| `map-codebase` | Analyze existing code structure, patterns, and conventions |

## How It Works

Konductor is built on four key components:

**1. Orchestrator Agent**
The main `konductor` agent manages pipeline state and delegates work to specialized subagents. It reads `.konductor/state.toml` to track progress and coordinates phase transitions.

**2. Skills**
Each command is a skill (`konductor-init`, `konductor-plan`, `konductor-exec`, etc.) with structured instructions. Skills define:
- When to trigger (keywords like "initialize", "next", "plan")
- Step-by-step execution logic
- Output artifacts and verification steps
- Reference materials for best practices

**3. Worker Subagents**
Specialized agents handle specific tasks:
- `konductor-discoverer` — Interviews users to understand project goals
- `konductor-researcher` — Analyzes codebases and documents patterns
- `konductor-planner` — Creates detailed phase plans with tasks
- `konductor-executor` — Implements code following TDD principles
- `konductor-verifier` — Validates tests and acceptance criteria

**4. Hook System**
A Rust binary (`konductor-hook`) provides:
- File modification tracking (detects changes to tracked files)
- Safety guardrails (prevents dangerous operations)
- Pipeline state validation (ensures prerequisites are met)

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
│   ├── konductor.json           # Main orchestrator
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
│   └── konductor-hook           # Rust binary
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

# Build the hook binary
cd konductor-hook
cargo build --release

# Install locally
cd ..
./install.sh --local
```

### Running Tests

```bash
# Test the hook binary
cd konductor-hook
cargo test

# Test skills by running the installer
./install.sh --local
kiro-cli --agent konductor
```

## License

MIT License - see [LICENSE](LICENSE) for details.
