# Execute

Implements tasks using a TDD workflow with specialized executor subagents, followed by automated code review.

## What Happens

1. The orchestrator spawns `konductor-executor` subagents for each plan
2. Each executor follows TDD: write tests first, then implement
3. Plans execute wave by wave (parallel within a wave, sequential across waves)
4. After all plans complete, `konductor-code-reviewer` reviews the changes (if `features.code_review = true`)
5. Issues found by the reviewer are automatically fixed and re-reviewed (up to 3 iterations)

## TDD Workflow

For each task:

1. Write failing test based on acceptance criteria
2. Implement minimal code to pass the test
3. Refactor while keeping tests green
4. Commit with clear message

## Code Review

When `features.code_review = true` (the default), a `konductor-code-reviewer` agent evaluates all modified files against:

- **Correctness** — does the code match what the plan describes?
- **Error handling** — are errors handled, not silently swallowed?
- **Security** — no hardcoded secrets, injection risks, or path traversal
- **Duplication** — copy-pasted logic that should be extracted
- **Performance** — unnecessary allocations, N+1 queries, blocking calls in async code
- **Dead code** — unused imports, unreachable branches, commented-out blocks
- **Consistency** — follows patterns established in the codebase
- **TODOs/FIXMEs** — any left behind that should have been resolved

The reviewer runs the project's test suite and linter, reports issues (without fixing them), and writes findings to `.konductor/phases/{phase}/code-review.md`. An executor then fixes the reported issues, and the reviewer re-checks — up to 3 iterations.

## Usage

```
> execute
> exec
```

!!! note "Context Rot Prevention"
    Each executor subagent starts fresh with only the current phase plan and relevant specs. This prevents accumulated conversation history from degrading decision quality.

!!! tip
    Set `features.code_review = false` in `config.toml` to skip automated code review after execution.
