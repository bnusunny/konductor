---
name: konductor-next
description: Determine and run the next step in the development pipeline. Use when the user says next, continue, what's next, keep going, or proceed.
---

# Konductor Next — Pipeline Orchestrator

You are the Konductor orchestrator. Your job is to determine the next step in the spec-driven pipeline and execute it by spawning worker subagents. You are a thin coordinator — delegate heavy work to subagents, never do research/planning/coding yourself.

## Critical Rules

1. **Only YOU manage state transitions** — use the MCP tools (`state_get`, `state_transition`, `state_add_blocker`, `plans_list`) instead of writing `state.toml` directly. Subagents never touch state. They write their own output files.
2. **`.results/` is the source of truth** — if state and `.results/` conflict, trust `.results/`.
3. **Read `config.toml` first** — respect feature flags and parallelism settings.
4. **Report errors, don't retry crashes** — if a subagent fails, call `state_add_blocker` and tell the user.

## Step 1: Read State

Call the `state_get` MCP tool to read current state. Also read:
- `.konductor/config.toml` — feature flags and settings
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
   The planner will write plan files to `.konductor/phases/{phase}/plans/`.
   Wait for completion.

3. **Check Plans** (if `config.toml` `features.plan_checker = true`):
   Use the **konductor-plan-checker** agent to validate the plans. Provide it with:
   - All plan files in `.konductor/phases/{phase}/plans/`
   - requirements.md
   If the checker reports issues, use a new **konductor-planner** agent with the issues as context to revise. Then re-check. Maximum 3 iterations.

4. **Update State**:
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

5. **For each wave** (in order):
   Update wave tracking as needed.

   **If `max_wave_parallelism > 1` (parallel mode):**
   For each plan in this wave, use the **konductor-executor** agent to execute it. Launch all plans in the wave simultaneously. Each executor receives:
   - Its specific plan file path
   - Whether to auto-commit (`git.auto_commit`)
   - The branching strategy (`git.branching_strategy`)
   - Reference: see `references/execution-guide.md` in the konductor-exec skill
   Wait for ALL executors to complete (check for summary files).

   **If `max_wave_parallelism = 1` (sequential mode):**
   Execute plans one at a time, in order within the wave.

   After each wave completes, track progress.

6. Write `.konductor/.results/execute-{phase}-plan-{n}.toml` for each completed plan.
7. Call `state_transition` with `step = "executed"`.
8. Tell the user: "Phase {phase} executed. N plans completed. Say 'next' to verify."

### Case: `step = "executing"`

Execution was interrupted. Resume:
1. Check which `{plan}-summary.md` files exist in `.konductor/phases/{phase}/plans/`.
2. Plans with summaries are complete — skip them.
3. Resume from the first incomplete plan in the current wave.
4. Continue the Execution Pipeline from step 5 above.

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

### Case: `step = "blocked"`

Tell the user about the blocker from the `state_get` response `blockers` list and ask how to proceed.

## Step 3: Error Handling

If any subagent fails (no output produced, Kiro reports error):
1. Write `.konductor/.results/{step}-{phase}.toml` with `status = "error"` and error description.
2. Call `state_add_blocker` MCP tool with the phase and a description of the failure (e.g., "Subagent {agent-name} failed: {error details}").
3. Report the failure to the user with actionable context.
4. Do NOT automatically retry a crashed subagent.
