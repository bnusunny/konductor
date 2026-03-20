# Pipeline Overview

Konductor transforms high-level project requirements into working software through a six-phase pipeline.

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

Each phase outputs artifacts to `.konductor/` that feed into the next phase. This prevents context rot — each subagent starts fresh with only the relevant documents it needs.
