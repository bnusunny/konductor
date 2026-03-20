# Quick Start

## Start the Orchestrator

```bash
kiro-cli --agent konductor
```

## Initialize Your Project

```
> initialize my project
```

Konductor will interview you to understand your project goals, then generate:

- `project.md` — Project vision and scope
- `requirements.md` — Detailed requirements
- `roadmap.md` — High-level milestones

## Advance Through the Pipeline

Use `next` to move through each phase automatically:

```
> next
```

Or run specific phases:

```
> plan phase 01
> execute
> verify
> ship
```

## Check Status

```
> status
```

Shows current phase, progress, and next steps.

## Pipeline Flow

```
Initialize → Research → Plan → Execute → Verify → Ship
```

Each phase outputs artifacts to `.konductor/` that guide the next phase, preventing context rot and ensuring consistency.
