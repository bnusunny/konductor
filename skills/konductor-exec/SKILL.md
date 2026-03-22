---
name: konductor-exec
description: Execute the plans for a phase by spawning executor subagents. Use when the user says execute phase, exec, run phase, build phase, or implement.
---

# Konductor Exec — Phase Execution Pipeline

You are the Konductor orchestrator. Execute the plans for a phase by spawning executor subagents to implement each plan.

## Critical Rules

1. **Only YOU manage state transitions** — use the MCP tools (`state_get`, `state_transition`, `state_add_blocker`) instead of writing `state.toml` directly. Subagents write their own output files (summary files, result files).
2. **Read `config.toml` first** — respect parallelism settings and git configuration.
3. **Report errors, don't retry crashes** — if an executor fails, write an error result for that plan, continue with remaining plans, and report all failures at the end.
4. **Resume support** — scan for existing summary files to skip completed plans.

## Step 1: Read State and Config

Call the `state_get` MCP tool to read current state, and read `.konductor/config.toml` for execution settings (parallelism, git config).

Validate that `[current].step` is either:
- `"planned"` — ready to start execution
- `"executing"` — resuming interrupted execution

If the phase is not in one of these states, tell the user:
> "Phase {phase} is not ready for execution. Current step: {step}. Run 'plan phase {phase}' first."
Then stop.

## Step 2: Set Executing State

Call `state_transition` with `step = "executing"` to mark the start of execution.

## Step 3: Load and Group Plans by Wave

Read all plan files from `.konductor/phases/{phase}/plans/`.

For each plan file:
1. Parse the TOML frontmatter (delimited by `+++` markers at start and end)
2. Extract the `wave` field (required)
3. Extract the `plan` field (plan number)
4. Group plans by wave number

**Wave ordering:** Plans execute in wave order (wave 1, then wave 2, etc.). Plans within a wave can execute in parallel if `max_wave_parallelism > 1`.

## Step 4: Resume Check

Scan `.konductor/phases/{phase}/plans/` for existing summary files.

**Summary file naming:** `{plan-number}-summary.md` (e.g., `001-summary.md`, `002-summary.md`)

For each plan:
- If `{plan-number}-summary.md` exists, the plan is complete — skip it
- If summary does not exist, the plan needs execution

Resume from the current wave with incomplete plans.

## Step 5: Wave Execution Loop

For each wave (in ascending order):

### 5.1: Update Wave State

Track the current wave number for reporting purposes.

### 5.2: Execute Plans in Wave

Read `config.toml` field `execution.max_wave_parallelism`:

**If `max_wave_parallelism > 1` (parallel mode):**
- For each plan in this wave, spawn a **konductor-executor** agent simultaneously
- Each executor receives:
  - Its specific plan file path (absolute path)
  - The git configuration: `git.auto_commit` and `git.branching_strategy`
  - Reference to `references/execution-guide.md` (deviation rules, commit protocol, analysis paralysis guard)
  - Reference to `references/tdd.md` if plan frontmatter `type = "tdd"`
- Wait for ALL executors in the wave to complete (check for summary files)

**If `max_wave_parallelism = 1` (sequential mode):**
- Execute plans one at a time within the wave
- Spawn one executor, wait for completion, then spawn the next

**Executor completion check:**
- A plan is complete when `{plan-number}-summary.md` exists
- If an executor crashes or produces no summary, treat it as a failure (see Step 5.4)

### 5.3: Write Result Files

After each plan completes (successfully or with errors), write `.konductor/.results/execute-{phase}-plan-{n}.toml`:

```toml
step = "execute"
phase = "{phase}"
plan = {plan_number}
wave = {wave_number}
status = "ok"  # or "error" if executor failed
timestamp = {current ISO timestamp}
```

### 5.4: Error Handling

If an executor fails (crashes, times out, or reports errors):
1. Write `.konductor/.results/execute-{phase}-plan-{n}.toml` with `status = "error"` and error details
2. **Continue** with remaining plans in the wave (do not stop)
3. Track failed plan numbers
4. At the end of the wave, report which plans failed

**Do NOT retry failed executors automatically.** Let the user decide how to proceed.

### 5.5: Update Progress Counters

After each wave completes, track progress (completed plans count and percentage) for reporting.

## Step 6: Code Review

After all waves complete, perform a code review of the changes made during execution.

### 6.1: Gather Changes

Read the modified files log at `.konductor/.tracking/modified-files.log` to get the list of all files changed during execution. Also read the `*-summary.md` files from each completed plan to understand what was implemented.

### 6.2: Review Each Changed File

For each modified source file (skip generated files, lock files, and binary artifacts):

1. Read the file
2. Check for:
   - **Correctness**: Does the code match what the plan and summary describe?
   - **Error handling**: Are errors handled, not silently swallowed?
   - **Security**: No hardcoded secrets, no SQL injection, no path traversal, proper input validation
   - **Duplication**: Is there copy-pasted or near-identical logic that should be extracted into a shared function/module?
   - **Performance**: Unnecessary allocations in hot paths, N+1 queries, missing pagination, unbounded loops, blocking calls in async code
   - **Dead code**: No unused imports, unreachable branches, or commented-out blocks left behind
   - **Consistency**: Does the code follow the patterns already established in the codebase?
   - **TODOs/FIXMEs**: Any left behind that should have been resolved?

### 6.3: Run Tests and Linting

Run the project's test suite and linter to catch issues the executors may have missed:

- Run tests (e.g., `cargo test`, `npm test`, `pytest`, `go test ./...`)
- Run linter if available (e.g., `cargo clippy`, `ruff check`, `eslint`, `golangci-lint run`)
- Note any failures or warnings

### 6.4: Fix or Report Issues

**For minor issues** (unused imports, missing error handling, formatting):
- Fix them directly
- Commit with message: `fix({phase}): code review cleanup`

**For significant issues** (logic bugs, security problems, architectural concerns):
- Do NOT fix them silently
- Report them to the user with the file, line, and description
- Call `state_add_blocker` if the issues are severe enough to block the phase

## Step 7: Set Executed State

After code review completes (with no blocking issues), call `state_transition` with `step = "executed"` to advance the pipeline.

Tell the user:
- Total plans executed
- Plans succeeded vs. failed (if any)
- Code review findings (issues fixed, warnings reported)
- Next step suggestion: "Say 'next' to verify the phase."

If any plans failed, list them and suggest:
> "Review the errors in `.results/execute-{phase}-plan-{n}.toml` files. You can re-run individual plans or fix issues manually."

## Error Handling

**Executor crashes:**
If an executor subagent crashes:
1. Write error result file for that plan
2. Continue with remaining plans
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
