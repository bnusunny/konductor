# How It Works

Konductor is built on five key components.

## Orchestrator Agent

The main `konductor` agent manages pipeline state and delegates work to specialized subagents. It coordinates phase transitions using MCP tools for deterministic state management.

## MCP Server (`konductor mcp`)

A local MCP server provides typed prompts and tools over stdio:

- **9 prompts** with Tab-completable shortcuts (`@k-init`, `@k-plan`, etc.) — with typed arguments where needed
- **State management tools** (`state_get`, `state_transition`, `state_add_blocker`, `state_resolve_blocker`) — eliminates fragile LLM-generated TOML
- **Query tools** (`plans_list`, `status`) — returns structured JSON instead of requiring the LLM to parse files

Built with [rmcp](https://github.com/modelcontextprotocol/rust-sdk) (official Rust MCP SDK).

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

## Hook System (`konductor hook`)

The same `konductor` binary processes hook events:

- File modification tracking (detects changes to tracked files)
- Safety guardrails (prevents dangerous operations like `rm -rf /`)

## Context Rot Prevention

Each subagent starts with a fresh context containing only:

- Relevant spec documents from `.konductor/`
- Current phase artifacts
- Necessary reference materials

This prevents context pollution and ensures decisions are based on documented requirements, not accumulated conversation history.

## Unified Binary

The `konductor` binary serves two roles via subcommands:

```
konductor mcp    # Start MCP server (stdio transport)
konductor hook   # Process hook events from stdin
```

## File Layout

```
.kiro/                          # Kiro configuration
├── agents/                     # Agent definitions (includes hooks and MCP server config)
├── skills/                     # Skill instructions
└── bin/
    └── konductor               # Unified binary (mcp server + hook processor)

.konductor/                     # Project state
├── project.md                  # Project vision
├── requirements.md             # Requirements
├── roadmap.md                  # Milestones
├── state.toml                  # Pipeline state (managed by MCP tools)
├── phases/                     # Phase plans
└── .results/                   # Execution results
```
