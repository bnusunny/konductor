# Execute

Implements tasks using per-task executor dispatch with TDD workflow, followed by a two-stage review pipeline per task and a holistic final pass.

## What Happens

1. The orchestrator dispatches a fresh `konductor-executor` for each task (not per plan)
2. Each executor follows TDD: write tests first, then implement
3. Tasks execute sequentially within a plan, plans execute wave by wave
4. After each task, a two-stage review runs: spec compliance then code quality
5. After all tasks complete, a holistic cross-task code review checks consistency

## Per-Task Dispatch

Each task gets a fresh executor agent with:

- The plan file and specific task number
- Summaries from prior completed tasks (for context)
- Git and TDD configuration

This prevents context rot — no accumulated conversation history across tasks.

## Implementer Status Protocol

After completing a task, the executor writes a summary with one of these statuses:

| Status | Meaning | Orchestrator Action |
|--------|---------|-------------------|
| `DONE` | Task complete | Proceed to review |
| `DONE_WITH_CONCERNS` | Complete but has concerns | Assess concerns, fix if actionable, then review |
| `NEEDS_CONTEXT` | Missing information | Provide context and re-dispatch (max 2 retries) |
| `BLOCKED` | Cannot proceed | Assess blocker, attempt resolution, or add to state |

## Two-Stage Review Pipeline

When `features.code_review = true` (the default), each completed task goes through two review stages:

### Stage 1 — Spec Compliance

A `konductor-spec-reviewer` agent checks whether the implementation matches the task specification:

- Do actual file changes match the task's action description?
- Are all files listed in the task created or modified?
- Does the verify step pass?
- Are there extra changes not specified in the task?

### Stage 2 — Code Quality

A `konductor-code-reviewer` agent checks code quality:

- **Correctness** — does the code match what the plan describes?
- **Error handling** — are errors handled, not silently swallowed?
- **Security** — no hardcoded secrets, injection risks, or path traversal
- **Duplication** — copy-pasted logic that should be extracted
- **Performance** — unnecessary allocations, N+1 queries, blocking calls in async code
- **Dead code** — unused imports, unreachable branches, commented-out blocks
- **Consistency** — follows patterns established in the codebase

Each stage allows up to 2 review-fix iterations. Both stages must pass before a task is marked complete.

## Circuit Breaker

If 3 or more tasks in a phase are BLOCKED, execution stops automatically and all blockers are reported to the user.

## Holistic Final Pass

After all tasks complete, a phase-level code review checks cross-task consistency:

- Shared interfaces match across plans
- Naming conventions are consistent
- Integration points work correctly
- No cross-plan duplication

## Usage

```
> execute
> exec
```

!!! tip
    Set `features.code_review = false` in `config.toml` to skip the two-stage review and holistic final pass.
