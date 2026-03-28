# Konductor

**Spec-driven development for Kiro CLI** — orchestrates project spec, design, planning, execution, verification, and shipping through a structured pipeline.

| Step | Description |
|------|-------------|
| 🚀 **Spec** | Discover goals, generate requirements |
| 🔍 **Discover** | Analyze codebase, map patterns (optional) |
| 📐 **Design** | Create architecture and key decisions |
| 📋 **Plan** | Break design into tasks with strict granularity |
| 🔎 **Review** | Review design and plans before execution |
| 💻 **Execute** | Per-task dispatch with TDD workflow |
| ✅ **Verify** | Validate tests and quality gates |
| 🚢 **Ship** | Commit, release, move to next phase |

## What's New in v0.13.0

- **Per-task executor dispatch** — fresh executor agent per task for better isolation and context
- **Two-stage review pipeline** — spec compliance review + code quality review after each task
- **Spec reviewer agent** — new `konductor-spec-reviewer` validates implementation matches task spec
- **Implementer status protocol** — structured status reporting (DONE, DONE_WITH_CONCERNS, NEEDS_CONTEXT, BLOCKED)
- **Circuit breaker** — execution stops automatically after 3+ blocked tasks
- **Stricter planning** — no placeholders, explicit task granularity rules

## Quick Install

```bash
npm install -g konductor
```

## Quick Start

```bash
kiro-cli --agent konductor
```

```
> spec my project
> next
> status
```

[Get Started →](getting-started/installation.md){ .md-button .md-button--primary }
