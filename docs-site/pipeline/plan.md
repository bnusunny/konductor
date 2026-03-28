# Plan

Breaks the phase design into execution plans with tasks, acceptance criteria, and wave ordering.

## What Happens

1. The `konductor-planner` agent creates detailed execution plans from the design
2. The `konductor-plan-checker` validates plans for completeness (if `features.plan_checker = true`)

## Output

```
phases/{phase}/plans/
├── 001.md      # Execution plans with TOML frontmatter
├── 002.md
└── ...
```

Each plan file contains:

- TOML frontmatter (wave, dependencies, requirements)
- A `## Design` section with technical approach
- Ordered task list with acceptance criteria

## Usage

```
> /k-plan 01
```

!!! tip
    After planning, run `/k-review` to review the design and plans before execution.
