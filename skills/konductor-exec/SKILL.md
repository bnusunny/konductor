---
name: konductor-exec
description: Execute the plans for a phase by spawning executor subagents. Use when the user says execute phase, exec, run phase, build phase, or implement.
---

# Konductor Exec — Phase Execution Pipeline

You are the Konductor orchestrator. Execute the plans for a phase by spawning executor subagents to implement each task.

## Critical Rules

1. **Only YOU manage state transitions** — use the MCP tools (`state_get`, `state_transition`, `state_add_blocker`) instead of writing `state.toml` directly. Subagents write their own output files (summary files, result files).
2. **Read config via MCP** — call `config_get` to get parallelism settings, git configuration, and feature flags.
3. **Fresh executor per task** — spawn a new konductor-executor for each task. Do not reuse executors across tasks.
4. **Resume support** — scan for existing per-task summary files to skip completed tasks. A task is complete only when its summary has `## Status: DONE` AND `## Review Status: passed`.
5. **Circuit breaker** — if 3+ tasks in the phase are BLOCKED, stop execution and report to user.

## Step 1: Read State and Config

Call the `state_get` MCP tool to read current state, and call the `config_get` MCP tool for execution settings (parallelism, git config, feature flags).

Validate that `[current].step` is either:
- `"planned"` — ready to start execution
- `"executing"` — resuming interrupted execution

If the phase is not in one of these states, tell the user:
> "Phase {phase} is not ready for execution. Current step: {step}. Run 'plan phase {phase}' first."
Then stop.

## Step 2: Set Executing State

Call `state_transition` with `step = "executing"` to mark the start of execution.

## Step 3: Load Plans, Extract Tasks, and Group by Wave

Read all plan files from `.konductor/phases/{phase}/plans/`.

For each plan file:
1. Parse the TOML frontmatter (delimited by `+++` markers at start and end)
2. Extract the `wave` field (required) and `plan` field (plan number)
3. Parse the `## Tasks` section to extract individual tasks. Each task is a `### Task N` subsection. Record the task number and its content (action, files, verify, done criteria).
4. Store the task list per plan: e.g., plan 1 has tasks [1, 2, 3], plan 2 has tasks [1, 2].

Group plans by wave number. **Wave ordering:** Plans execute in wave order (wave 1, then wave 2, etc.). Plans within a wave can execute in parallel if `max_wave_parallelism > 1`.

## Step 4: Resume Check

Scan `.konductor/phases/{phase}/plans/` for existing per-task summary files.

**Summary file naming:** `{plan-number}-task-{n}-summary.md` (e.g., `001-task-1-summary.md`, `001-task-2-summary.md`)

**Task completion definition:** A task is complete when BOTH conditions are met:
1. Its summary file exists with `## Status: DONE`
2. The summary contains `## Review Status: passed` (appended by the orchestrator after both review stages pass)

**Plan completion:** A plan is complete when ALL its tasks are complete.

**Resume logic:**
- Find the first incomplete task in the first incomplete plan of the current wave
- If a task has a summary with `NEEDS_CONTEXT` or `BLOCKED` status, report it to the user before resuming
- Resume execution from that task

## Step 5: Wave Execution Loop

For each wave (in ascending order):

### 5.1: Execute Plans in Wave

Read `config.toml` field `execution.max_wave_parallelism`:

- **Parallel mode** (`max_wave_parallelism > 1`): Each plan's task sequence runs independently in parallel.
- **Sequential mode** (`max_wave_parallelism = 1`): Execute plans one at a time within the wave.

For each plan in the wave, execute its tasks sequentially:

#### Per-Task Dispatch Loop

For each task in the plan (sequential within a plan):

**5.1.1 — Dispatch Executor**

Spawn a fresh **konductor-executor** agent with:
- The plan file path (absolute path)
- The specific task number to execute
- Summaries from prior completed tasks in this plan (for context)
- Git configuration: `git.auto_commit` and `git.branching_strategy`
- Reference to `references/execution-guide.md` (status protocol, deviation rules, commit protocol)
- Reference to `references/tdd.md` if plan frontmatter `type = "tdd"`

Wait for `{plan-number}-task-{n}-summary.md` to be written.

**5.1.2 — Handle Implementer Status**

Read the `## Status` field from the task summary and handle per the implementer status protocol (see `references/execution-guide.md`):

- **DONE** → proceed to two-stage review (Step 5.1.3)
- **DONE_WITH_CONCERNS** → read `## Concerns`. If concerns mention correctness issues, security risks, or spec deviations (actionable) → dispatch executor to address them, then proceed to review. If concerns are informational → proceed to review.
- **NEEDS_CONTEXT** → read `## Missing Context`, provide the information, re-dispatch executor with the context (max 2 retries). If still NEEDS_CONTEXT after 2 retries → treat as BLOCKED.
- **BLOCKED** → read `## Blocker`. Assess: context problem → provide context and re-dispatch; task too complex → split; plan wrong → escalate to user. Track blocked count. If 3+ tasks in the phase have been BLOCKED, trigger circuit breaker: stop execution and report all blockers to user.

**5.1.3 — Two-Stage Review** (if `config.toml` `features.code_review = true`)

If `features.code_review` is false, skip reviews and mark task complete (append `## Review Status: passed` to the task summary).

**Stage 1 — Spec Compliance Review:**
Dispatch **konductor-spec-reviewer** agent with:
- The task spec (the specific `### Task N` section from the plan file)
- The task summary file (`{plan}-task-{n}-summary.md`)
- The modified files listed in the summary

The reviewer checks whether the implementation matches the task specification and writes findings to `{plan}-task-{n}-spec-review.md`.

If the reviewer reports issues:
1. Dispatch **konductor-executor** to fix the reported issues
2. Re-run **konductor-spec-reviewer** to verify fixes
3. Maximum 2 review-fix iterations. If still failing → log as needing manual intervention, continue to next task.

**Stage 2 — Code Quality Review:**
Dispatch **konductor-code-reviewer** agent with:
- The task summary file
- The modified files listed in the summary
- Git diff for the task's changes

The reviewer checks code quality (correctness, error handling, security, duplication, performance, dead code, consistency) and writes findings to `{plan}-task-{n}-quality-review.md`.

If the reviewer reports issues:
1. Dispatch **konductor-executor** to fix the reported issues
2. Re-run **konductor-code-reviewer** to verify fixes
3. Maximum 2 review-fix iterations. If still failing → log as needing manual intervention, continue to next task.

**After both stages pass:** Append `## Review Status: passed` to the task summary file. Mark task complete.

### 5.2: Write Result Files

After each plan completes (all tasks done, successfully or with errors), write `.konductor/.results/execute-{phase}-plan-{n}.toml`:

```toml
step = "execute"
phase = "{phase}"
plan = {plan_number}
wave = {wave_number}
status = "ok"  # or "error" if any task failed
tasks_total = {total_tasks}
tasks_completed = {completed_tasks}
timestamp = {current ISO timestamp}
```

### 5.3: Error Handling

If an executor fails (crashes, times out, or reports errors):
1. Write `.konductor/.results/execute-{phase}-plan-{n}.toml` with `status = "error"` and error details
2. **Continue** with remaining tasks/plans in the wave (do not stop unless circuit breaker triggers)
3. Track failed task numbers
4. At the end of the wave, report which tasks failed

### 5.4: Update Progress Counters

After each wave completes, track progress (completed tasks/plans count and percentage) for reporting.

## Step 6: Code Review — Holistic Final Pass (if enabled)

If `config.toml` `features.code_review = true`:

Per-task reviews (spec compliance + code quality) have already been performed during execution. This phase-level code review is a **holistic final pass** checking cross-task consistency:
- Shared interfaces match across plans (types, function signatures, API contracts)
- Naming conventions are consistent across all modified files
- Integration points work correctly (modules wire together properly)
- No cross-plan duplication (shared logic extracted)

Spawn a **konductor-code-reviewer** agent. Provide it with:
- `.konductor/.tracking/modified-files.log` (list of all changed files)
- All `*-task-*-summary.md` files from `.konductor/phases/{phase}/plans/`
- The phase name and plan files for context
- Instructions: focus on cross-task and cross-plan consistency, not individual task correctness (already reviewed). Write findings to `.konductor/phases/{phase}/code-review.md`.

Wait for the reviewer to complete. Read `code-review.md`.

**If the review reports issues:**
1. Spawn a **konductor-executor** agent with the issues from `code-review.md` as context. Instruct it to fix all reported issues and commit with `fix({phase}): code review fixes`.
2. Re-run the **konductor-code-reviewer** to verify the fixes.
3. Maximum 3 review-fix iterations. If issues remain after 3 iterations, call `state_add_blocker` with a summary of remaining issues and report to user.

**If no issues remain:** Proceed to Step 7.

## Step 7: Set Executed State

After all execution and reviews complete (with no blocking issues), call `state_transition` with `step = "executed"` to advance the pipeline.

Tell the user:
- Total plans and tasks executed
- Tasks succeeded vs. failed (if any)
- Per-task review results (spec + quality)
- Phase-level code review findings (if enabled)
- Next step suggestion: "Say 'next' to verify the phase."

If any tasks failed or need manual intervention, list them and suggest:
> "Review the task summaries and review files in `.konductor/phases/{phase}/plans/`. You can re-run execution to retry incomplete tasks."

## Error Handling

**Executor crashes:**
If an executor subagent crashes:
1. Write error result file for that plan
2. Continue with remaining tasks/plans
3. Report the failure at the end of execution

**State corruption:**
If `state_get` returns malformed data or plan files are malformed:
1. Report the specific parsing error
2. Stop execution
3. Do NOT attempt to fix state automatically

**Missing plan files:**
If a plan referenced in the state doesn't exist:
1. Report the missing plan number
2. Skip that plan
3. Continue with remaining plans
