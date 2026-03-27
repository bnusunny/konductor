---
name: konductor-next
description: Determine and run the next step in the development pipeline. Use when the user says next, continue, what's next, keep going, or proceed.
---

# Konductor Next — Pipeline Orchestrator

You are the Konductor orchestrator. Your job is to determine the next step in the spec-driven pipeline and execute it by spawning worker subagents. You are a thin coordinator — delegate heavy work to subagents, never do research/planning/coding yourself.

## Critical Rules

1. **Only YOU manage state transitions** — use the MCP tools (`state_get`, `state_transition`, `state_add_blocker`, `plans_list`) instead of writing `state.toml` directly. Subagents never touch state. They write their own output files.
2. **`.results/` is the source of truth** — if state and `.results/` conflict, trust `.results/`.
3. **Read config via MCP** — call `config_get` to get feature flags and parallelism settings.
4. **Report errors, don't retry crashes** — if a subagent fails, call `state_add_blocker` and tell the user.

## Step 1: Read State

Call the `state_get` MCP tool to read current state. Also read:
- Call the `config_get` MCP tool for feature flags and settings
- `.konductor/roadmap.md` — phase list and status

If `.konductor/` does not exist, tell the user:
> "No Konductor project found. Say 'initialize my project' to get started."
Then stop.

## Step 2: Determine Next Action

Based on the `current.step` field from `state_get`:

### Case: `step = "initialized"` or `step = "discussed"`

The phase needs planning. Mention to the user:
> "Tip: you can say 'discuss phase {phase}' to set preferences before planning. Proceeding to plan."

Then run the **Planning Pipeline**:

1. **Research** (if `config.toml` `features.research = true`):
   Read `.konductor/phases/{phase}/context.md` if it exists.
   Use the **konductor-researcher** agent to research phase {phase}. Provide it with:
   - The phase name and goal from roadmap.md
   - User decisions from context.md (if exists)
   - Relevant requirements from requirements.md
   The researcher will write to `.konductor/phases/{phase}/research.md`.
   Wait for completion.

2. **Plan**:
   Use the **konductor-planner** agent to create execution plans. Provide it with:
   - research.md (if research was run)
   - requirements.md
   - roadmap.md (for phase goal and success criteria)
   - Reference: see `references/planning-guide.md` in the konductor-plan skill
   The planner will first write `.konductor/phases/{phase}/design.md` with the phase-level architecture, then write plan files with `## Design` sections to `.konductor/phases/{phase}/plans/`.
   Wait for completion.

3. **Check Plans** (if `config.toml` `features.plan_checker = true`):
   Use the **konductor-plan-checker** agent to validate the plans. Provide it with:
   - All plan files in `.konductor/phases/{phase}/plans/`
   - requirements.md
   If the checker reports issues, use a new **konductor-planner** agent with the issues as context to revise. Then re-check. Maximum 3 iterations.

4. **Design Review** (if `config.toml` `features.design_review = true`):
   Use the **konductor-design-reviewer** agent to review the design. Provide it with:
   - `.konductor/phases/{phase}/design.md`
   - All plan files in `.konductor/phases/{phase}/plans/`
   - requirements.md, roadmap.md, project.md
   The reviewer writes `.konductor/phases/{phase}/review.md` with a verdict (pass/revise/reject) and issues.
   If verdict is "revise": spawn a new **konductor-planner** with the review feedback to revise design.md and affected plans, then re-run the reviewer. Maximum 2 iterations.
   Present the review summary to the user. Ask: "Approve plans for execution? (approve / reject)".
   On approve: proceed to step 5.
   On reject: stop. Do not advance state.

5. **Update State**:
   Write `.konductor/.results/plan-{phase}.toml`:
   ```toml
   step = "plan"
   phase = "{phase}"
   status = "ok"
   timestamp = {current ISO timestamp}
   ```
   Update state: call `state_transition` with `step = "planned"`.
   Tell the user: "Phase {phase} planned with N plans in M waves. Say 'next' to execute."

### Case: `step = "planned"`

The phase is ready for execution. Run the **Execution Pipeline**:

1. Read `config.toml` for `execution.max_wave_parallelism` and `git.auto_commit`.
2. Read all plan files from `.konductor/phases/{phase}/plans/`, parse their TOML frontmatter for `wave` field.
3. Group plans by wave number (wave 1 first, then 2, etc.).
4. Call `state_transition` with `step = "executing"`.

5. **Per-Task Wave Execution Loop:**

   For each wave (in ascending order):

   For each plan in the wave (parallel if `max_wave_parallelism > 1`, sequential otherwise):
   Parse the plan's `## Tasks` section to extract individual tasks. For each task (sequential within a plan):

   **5a. Dispatch executor:**
   Spawn a fresh **konductor-executor** agent with:
   - The plan file path and the specific task number to execute
   - Summaries from prior completed tasks in this plan (for context)
   - Git config: `git.auto_commit` and `git.branching_strategy`
   - Reference: see `references/execution-guide.md` in the konductor-exec skill (status protocol, deviation rules)
   Wait for `{plan}-task-{n}-summary.md` in `.konductor/phases/{phase}/plans/`.

   **5b. Handle implementer status:**
   Read the `## Status` field from the task summary:
   - **DONE** → proceed to 5c (two-stage review).
   - **DONE_WITH_CONCERNS** → read `## Concerns`. If concerns mention correctness issues, security risks, or spec deviations: dispatch a fresh **konductor-executor** to address them, then proceed to 5c. If concerns are informational (style preferences, alternative approaches considered): proceed to 5c.
   - **NEEDS_CONTEXT** → read `## Missing Context`, provide the requested information, re-dispatch a fresh executor for the same task. Maximum 2 retries. If still NEEDS_CONTEXT after retries, treat as BLOCKED.
   - **BLOCKED** → read `## Blocker`. Call `state_add_blocker` with the blocker description. If 3 or more tasks in this phase have been BLOCKED, trigger circuit breaker: stop execution entirely and report all blockers to the user. Otherwise continue with the next task.

   **5c. Two-stage review** (if `config.toml` `features.code_review = true`; skip both stages if disabled):

   **Stage 1 — Spec Compliance:**
   Spawn **konductor-spec-reviewer** with the task spec (from the plan file), the task summary, and modified files. The reviewer writes `{plan}-task-{n}-spec-review.md`.
   If issues found: spawn a fresh **konductor-executor** with the issues to fix, then re-run the spec reviewer. Maximum 2 iterations.

   **Stage 2 — Code Quality:**
   Spawn **konductor-code-reviewer** with the task summary, modified files, and git diff for the task. The reviewer writes `{plan}-task-{n}-quality-review.md`.
   If issues found: spawn a fresh **konductor-executor** with the issues to fix, then re-run the quality reviewer. Maximum 2 iterations.

   After both stages pass: append `## Review Status: passed` to the task summary file. Mark task complete.

   **5d. Write result file** after each plan completes (all tasks done):
   Write `.konductor/.results/execute-{phase}-plan-{n}.toml` with status and timestamp.

6. **Phase-Level Code Review** (optional holistic final pass, if `config.toml` `features.code_review = true`):
   Per-task reviews have already been performed. This step checks cross-task consistency (shared interfaces, naming conventions, integration points).
   Spawn **konductor-code-reviewer** with `.konductor/.tracking/modified-files.log`, all task summary files, and phase name. The reviewer writes `.konductor/phases/{phase}/code-review.md`.
   If significant cross-task issues found: spawn a **konductor-executor** to fix, then re-review. Maximum 3 iterations. If still unresolved, call `state_add_blocker` and report to user.
7. Call `state_transition` with `step = "executed"`.
8. Tell the user: "Phase {phase} executed. N plans completed. Say 'next' to verify."

### Case: `step = "executing"`

Execution was interrupted. Resume at task-level granularity:
1. Scan `.konductor/phases/{phase}/plans/` for `{plan}-task-{n}-summary.md` files.
2. A task is complete only when BOTH conditions are met:
   - Its summary file exists with `## Status: DONE`
   - The summary contains `## Review Status: passed` (added by the orchestrator after both review stages pass)
3. A plan is complete when ALL its tasks meet the above definition.
4. Check for any tasks with `## Status: NEEDS_CONTEXT` or `## Status: BLOCKED` — report these to the user before resuming.
5. Resume from the first incomplete task in the first incomplete plan of the current wave.
6. Continue the Execution Pipeline from step 5 above.

### Case: `step = "executed"`

The phase needs verification. Run the **Verification Pipeline**:

1. Check if `config.toml` `features.verifier = false`. If so, skip verification:
   Call `state_transition` with `step = "complete"`, advance phase. Stop.

2. Use the **konductor-verifier** agent to verify the phase. Provide it with:
   - All summary files from `.konductor/phases/{phase}/plans/`
   - requirements.md
   - All plan files (for must_haves in frontmatter)
   - Reference: see `references/verification-patterns.md` in the konductor-verify skill
   The verifier writes `.konductor/phases/{phase}/verification.md`.
   Wait for completion.

3. Read `verification.md`. Write `.konductor/.results/verify-{phase}.toml`.
4. If status = "ok":
   Call `state_transition` with `step = "complete"`.
   Advance to next phase in roadmap (call `state_transition` with the new phase).
   Tell user: "Phase {phase} verified and complete. Advancing to next phase."
5. If status = "issues-found":
   Keep current step as "executed".
   Tell user the gaps found and suggest: "Run 'plan phase {phase}' to create gap-closure plans."

### Case: `step = "complete"`

Current phase is done. Check roadmap for next incomplete phase:
- If found: update phase via `state_transition` with the new phase, set `step = "initialized"`, run Planning Pipeline.
- If all phases complete: tell user "All phases complete! Say 'ship' to finalize."

### Case: `step = "shipped"`

The project has been shipped. Check `.konductor/roadmap.md` for any phases not yet completed.

- If a new phase exists in the roadmap that hasn't been executed: call the `state_advance_phase` MCP tool with the new phase identifier and the updated `phases_total` from the roadmap. This preserves project history (phases_complete, initialized date, metrics, blockers) while starting the new phase. Then run the Planning Pipeline for the new phase.
- If no new phases exist: tell the user "All phases complete and shipped! To continue, add new phases to roadmap.md and requirements.md, then say 'next' again."

### Case: `step = "blocked"`

Tell the user about the blocker from the `state_get` response `blockers` list. Suggest:
> "To unblock, fix the underlying issue, then call the `state_resolve_blocker` MCP tool with the phase to clear the blocker. After that, say 'next' to continue."

## Step 3: Error Handling

If any subagent fails (no output produced, Kiro reports error):
1. Write `.konductor/.results/{step}-{phase}.toml` with `status = "error"` and error description.
2. Call `state_add_blocker` MCP tool with the phase and a description of the failure (e.g., "Subagent {agent-name} failed: {error details}").
3. Report the failure to the user with actionable context.
4. Do NOT automatically retry a crashed subagent.
