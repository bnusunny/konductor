# Discover

Analyzes the ecosystem, codebase, and dependencies before design begins.

## What Happens

1. The `konductor-researcher` agent receives the phase goal, success criteria, and relevant requirements
2. It analyzes the codebase, maps dependencies, and identifies patterns and conventions
3. If a `context.md` exists from a prior `/k-discuss` session, user decisions are included
4. Outputs a research document that feeds into the design step

## Output

| File | Description |
|------|-------------|
| `.konductor/phases/{phase}/research.md` | Ecosystem analysis, patterns, dependencies, and recommendations |

## Configuration

Discover runs automatically as part of the design pipeline when enabled:

```toml
[features]
research = true  # default
```

Set `features.research = false` in `config.toml` to skip discovery and go straight to design.

## Usage

Discover is triggered automatically by `/k-design` or `/k-next`. It is not a standalone command.
