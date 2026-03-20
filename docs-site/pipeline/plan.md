# Plan

Breaks work into phases with specific tasks, acceptance criteria, and cross-phase regression gates.

## What Happens

1. The `konductor-planner` agent reads your specs and roadmap
2. Creates detailed phase plans with numbered tasks
3. Each task includes acceptance criteria and test requirements
4. A `konductor-plan-checker` validates the plan for completeness

## Outputs

Phase files in `.konductor/phases/`:

```
phases/
├── phase-01-foundation.md
├── phase-02-core-features.md
└── ...
```

Each phase file contains:

- Phase goals and scope
- Ordered task list with acceptance criteria
- Cross-phase regression gates
- Dependencies between tasks

## Usage

```
> plan
> plan phase 01
```
