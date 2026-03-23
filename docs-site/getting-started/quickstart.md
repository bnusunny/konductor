# Quick Start

## Start the Orchestrator

```bash
kiro-cli --agent konductor
```

## Initialize Your Project

Type `@k-init` and press Tab, or simply type:

```
> initialize my project
```

Konductor will interview you to understand your project goals, then generate:

- `project.md` — Project vision and scope
- `requirements.md` — Detailed requirements
- `roadmap.md` — High-level milestones

## Advance Through the Pipeline

Use `@k-next` (or type `next`) to move through each phase automatically:

```
> next
```

Or run specific phases using prompt shortcuts:

```
> @k-plan 01
> @k-exec
> @k-verify
> @k-ship
```

## Check Status

```
> @k-status
```

Shows current phase, progress, and next steps.

## All Prompt Shortcuts

Type `@k-` then press Tab to see all available commands:

| Shortcut | Description |
|----------|-------------|
| `@k-init` | Initialize a new project |
| `@k-plan` | Plan a phase (requires phase number) |
| `@k-exec` | Execute the current phase |
| `@k-verify` | Verify the current phase |
| `@k-ship` | Ship and finalize |
| `@k-next` | Advance to next step |
| `@k-status` | Show project status |
| `@k-discuss` | Discuss a phase (requires phase number) |
| `@k-map` | Map the existing codebase |

## Pipeline Flow

```
Initialize → Research → Plan → Design Review → Execute → Code Review → Verify → Ship
```

Each phase outputs artifacts to `.konductor/` that guide the next phase, preventing context rot and ensuring consistency.
