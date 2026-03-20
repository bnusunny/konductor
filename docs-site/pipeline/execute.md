# Execute

Implements tasks using a TDD workflow with specialized executor subagents.

## What Happens

1. The orchestrator spawns `konductor-executor` subagents for each task
2. Each executor follows TDD: write tests first, then implement
3. Subagents start with fresh context containing only relevant specs
4. Progress is tracked in `.konductor/state.toml`

## TDD Workflow

For each task:

1. Write failing test based on acceptance criteria
2. Implement minimal code to pass the test
3. Refactor while keeping tests green
4. Commit with clear message

## Usage

```
> execute
> exec
```

!!! note "Context Rot Prevention"
    Each executor subagent starts fresh with only the current phase plan and relevant specs. This prevents accumulated conversation history from degrading decision quality.
