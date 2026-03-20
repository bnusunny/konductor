# How It Works

Konductor is built on four key components.

## Orchestrator Agent

The main `konductor` agent manages pipeline state and delegates work to specialized subagents. It reads `.konductor/state.toml` to track progress and coordinates phase transitions.

## Skills

Each command is a skill (`konductor-init`, `konductor-plan`, `konductor-exec`, etc.) with structured instructions. Skills define:

- When to trigger (keywords like "initialize", "next", "plan")
- Step-by-step execution logic
- Output artifacts and verification steps

## Worker Subagents

Specialized agents handle specific tasks:

| Agent | Role |
|-------|------|
| `konductor-discoverer` | Interviews users to understand project goals |
| `konductor-researcher` | Analyzes codebases and documents patterns |
| `konductor-planner` | Creates detailed phase plans with tasks |
| `konductor-executor` | Implements code following TDD principles |
| `konductor-verifier` | Validates tests and acceptance criteria |

## Hook System

A Rust binary (`konductor-hook`) provides:

- File modification tracking
- Safety guardrails (prevents dangerous operations)
- Pipeline state validation

## Context Rot Prevention

Each subagent starts with a fresh context containing only:

- Relevant spec documents from `.konductor/`
- Current phase artifacts
- Necessary reference materials

This prevents context pollution and ensures decisions are based on documented requirements, not accumulated conversation history.

## File Layout

```
.kiro/                          # Kiro configuration
├── agents/                     # Agent definitions
├── skills/                     # Skill instructions
├── hooks/                      # Hook configuration
└── bin/                        # Hook binary

.konductor/                     # Project state
├── project.md                  # Project vision
├── requirements.md             # Requirements
├── roadmap.md                  # Milestones
├── state.toml                  # Pipeline state
├── phases/                     # Phase plans
└── .results/                   # Execution results
```
