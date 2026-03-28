# Pipeline Overview

Konductor transforms high-level project requirements into working software through a structured pipeline with built-in quality gates.

## Phases vs Steps

A **phase** is a unit of work from the roadmap (e.g. "Phase 01: Auth System"). A project has multiple phases. Each phase progresses through the same **steps** — the pipeline stages listed below. Think of phases as *what* you're building, and steps as *how* you build it.

```
┌─────────────┐
│    Spec     │ → project.md, requirements.md, roadmap.md
└──────┬──────┘
       ↓
┌─────────────┐
│  Discover   │ → research.md (optional)
└──────┬──────┘
       ↓
┌─────────────┐
│   Design    │ → design.md (architecture, components, decisions)
└──────┬──────┘
       ↓
┌─────────────┐
│    Plan     │ → plans/*.md (tasks + acceptance criteria)
└──────┬──────┘
       ↓
┌─────────────┐
│   Review    │ → review.md (approve/revise/reject)
└──────┬──────┘
       ↓
┌─────────────┐
│  Execute    │ → per-task dispatch + two-stage review
└──────┬──────┘
       ↓
┌───────────────┐
│Holistic Review│ → cross-task consistency check
└──────┬────────┘
       ↓
┌─────────────┐
│   Verify    │ → test results, quality checks
└──────┬──────┘
       ↓
┌─────────────┐
│    Ship     │ → commits, PRs, release notes
└─────────────┘
```

Each phase outputs artifacts to `.konductor/` that feed into the next step. This prevents context rot — each subagent starts fresh with only the relevant documents it needs.

Discover, review, and code review are enabled by default but can be disabled via feature flags in `config.toml`. See [Configuration](../reference/configuration.md) for details.
