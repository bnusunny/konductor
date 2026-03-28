# Quick Start

## Start the Orchestrator

```bash
kiro-cli --agent konductor
```

## Define Your Project Spec

Type `@k-spec` (or `@k-init`), or simply type:

```
> spec my project
```

Konductor will interview you to understand your project goals, then generate:

- `project.md` — Project vision and scope
- `requirements.md` — Detailed requirements
- `roadmap.md` — High-level milestones

## Advance Through the Pipeline

Use `@k-next` (or type `next`) to move through each step automatically:

```
> next
```

Or run specific steps using prompt shortcuts:

```
> @k-design 01
> @k-plan 01
> @k-review 01
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
| `@k-spec` | Define project requirements (alias: `@k-init`) |
| `@k-design` | Create architecture for a phase |
| `@k-plan` | Break design into execution plans |
| `@k-review` | Review design and plans |
| `@k-exec` | Execute the current phase |
| `@k-verify` | Verify the current phase |
| `@k-ship` | Ship and finalize |
| `@k-next` | Advance to next step |
| `@k-status` | Show project status |
| `@k-discuss` | Discuss a phase (requires phase number) |
| `@k-map` | Map the existing codebase |

## Pipeline Flow

```
Spec → Discover (optional) → Design → Plan → Review → Execute (per-task + two-stage review) → Verify → Ship
```

Each phase outputs artifacts to `.konductor/` that guide the next step, preventing context rot and ensuring consistency.
