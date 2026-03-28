# Design

Creates the phase-level architecture, component interactions, and key technical decisions.

## What Happens

1. The `konductor-researcher` agent investigates the ecosystem (if `features.research = true`)
2. The `konductor-planner` agent creates a phase-level `design.md` with architecture decisions

## Output

| File | Description |
|------|-------------|
| `.konductor/phases/{phase}/research.md` | Ecosystem research findings (if discover enabled) |
| `.konductor/phases/{phase}/design.md` | Phase-level architecture, component interactions, key decisions |

## Usage

```
> @k-design 01
```

!!! tip
    Run `@k-discuss` before `@k-design` to set preferences for library choices, architectural trade-offs, and constraints.
