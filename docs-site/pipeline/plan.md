# Plan

Breaks work into phases with specific tasks, acceptance criteria, and a technical design that gets reviewed before execution.

## What Happens

1. The `konductor-researcher` agent investigates the ecosystem (if `features.research = true`)
2. The `konductor-planner` agent creates a phase-level `design.md` and detailed execution plans
3. The `konductor-plan-checker` validates plans for completeness (if `features.plan_checker = true`)
4. The `konductor-design-reviewer` reviews the architecture (if `features.design_review = true`)
5. The user is asked to approve or reject the plans before execution proceeds

## Outputs

Phase files in `.konductor/phases/{phase}/`:

```
phases/{phase}/
├── research.md          # Ecosystem research findings
├── design.md            # Phase-level architecture
├── review.md            # Design review verdict and findings
└── plans/
    ├── 001-plan.md      # Execution plans with Design sections
    ├── 002-plan.md
    └── ...
```

Each plan file contains:

- TOML frontmatter (wave, dependencies, requirements)
- Phase goals and scope
- A `## Design` section with technical approach
- Ordered task list with acceptance criteria

## Design Review

When `features.design_review = true` (the default), a `konductor-design-reviewer` agent evaluates:

- Architectural soundness of `design.md`
- Feasibility of per-plan Design sections
- Cross-plan consistency of shared interfaces
- Requirement coverage
- Risk identification (security, error handling, dependencies)

The review produces a verdict:

- **pass** — plans are approved, proceed to execution
- **revise** — issues found, planner revises and reviewer re-checks (up to 2 iterations)
- **reject** — fundamental problems, user must intervene

After the review, the user is asked to approve or reject the plans.

## Usage

```
> plan
> plan phase 01
```

!!! tip
    Set `features.design_review = false` in `config.toml` to skip the review and approval gate.
