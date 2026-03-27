# Research

Analyzes the ecosystem, codebase, and dependencies before planning begins.

## What Happens

1. The `konductor-researcher` agent receives the phase goal, success criteria, and relevant requirements
2. It analyzes the codebase, maps dependencies, and identifies patterns and conventions
3. If a `context.md` exists from a prior `@k-discuss` session, user decisions are included
4. Outputs a research document that feeds into the planning step

## Output

| File | Description |
|------|-------------|
| `.konductor/phases/{phase}/research.md` | Ecosystem analysis, patterns, dependencies, and recommendations |

## Configuration

Research runs automatically as part of the planning pipeline when enabled:

```toml
[features]
research = true  # default
```

Set `features.research = false` in `config.toml` to skip research and go straight to plan generation.

## Usage

Research is triggered automatically by `@k-plan` or `@k-next`. It is not a standalone command.
