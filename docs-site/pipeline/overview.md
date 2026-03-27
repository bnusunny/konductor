# Pipeline Overview

Konductor transforms high-level project requirements into working software through a structured pipeline with built-in quality gates.

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
│    Plan     │ → phases/*.md (tasks + acceptance criteria)
└──────┬──────┘
       ↓
┌───────────────┐
│Design Review  │ → review.md (approve/revise/reject)
└──────┬────────┘
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

Each phase outputs artifacts to `.konductor/` that feed into the next phase. This prevents context rot — each subagent starts fresh with only the relevant documents it needs.

Design review and code review are enabled by default but can be disabled via feature flags in `config.toml`. See [Configuration](../reference/configuration.md) for details.
